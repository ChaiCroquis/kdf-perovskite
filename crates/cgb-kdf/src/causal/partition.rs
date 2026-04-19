//! Causal Partition Builder

use std::collections::{HashMap, HashSet};
use super::types::{TeStrategy, CausalLink};
use super::engine::CausalEngine;

/// Causal Cluster
#[derive(Clone, Debug)]
pub struct CausalCluster {
    /// Cluster ID
    pub cluster_id: u32,
    /// Nodes in this cluster
    pub nodes: HashSet<String>,
    /// Internal causal links
    pub internal_links: Vec<CausalLink>,
    /// Hub node (most connections)
    pub hub_node: Option<String>,
}

/// Causal Partition Builder
///
/// Builds partitions from causal structure (TE links).
/// Groups causally connected nodes into the same module.
///
/// Integration Pattern #3: TE → KDF Partitions
pub struct CausalPartitionBuilder {
    /// TE threshold for causal links
    pub te_threshold: f64,
    /// Minimum cluster size
    pub min_cluster_size: usize,
    /// Maximum cluster size
    pub max_cluster_size: usize,
}

impl CausalPartitionBuilder {
    /// Create a new partition builder
    pub fn new(te_threshold: f64, min_cluster_size: usize, max_cluster_size: usize) -> Self {
        Self {
            te_threshold,
            min_cluster_size,
            max_cluster_size,
        }
    }
}

impl Default for CausalPartitionBuilder {
    fn default() -> Self {
        Self::new(0.01, 2, 50)
    }
}

impl CausalPartitionBuilder {
    /// Build partition from causal links
    pub fn build_partition_from_links(
        &self,
        causal_links: &[CausalLink],
        all_nodes: Option<&[String]>,
    ) -> HashMap<String, u32> {
        // Filter significant links
        let significant_links: Vec<&CausalLink> = causal_links
            .iter()
            .filter(|link| link.te >= self.te_threshold && link.is_significant)
            .collect();

        // Build adjacency list
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        for link in &significant_links {
            adjacency
                .entry(link.source.clone())
                .or_default()
                .insert(link.target.clone());
            adjacency
                .entry(link.target.clone())
                .or_default()
                .insert(link.source.clone());
        }

        // Find connected components using BFS
        let mut visited: HashSet<String> = HashSet::new();
        let mut clusters: Vec<HashSet<String>> = Vec::new();

        for node in adjacency.keys() {
            if !visited.contains(node) {
                let mut component: HashSet<String> = HashSet::new();
                let mut queue = vec![node.clone()];

                while let Some(current) = queue.pop() {
                    if visited.contains(&current) {
                        continue;
                    }
                    visited.insert(current.clone());
                    component.insert(current.clone());

                    if let Some(neighbors) = adjacency.get(&current) {
                        for neighbor in neighbors {
                            if !visited.contains(neighbor) {
                                queue.push(neighbor.clone());
                            }
                        }
                    }
                }

                if component.len() >= self.min_cluster_size {
                    // Split large clusters
                    if component.len() > self.max_cluster_size {
                        let splits = self.split_cluster(&component);
                        clusters.extend(splits);
                    } else {
                        clusters.push(component);
                    }
                }
            }
        }

        // Build partition
        let mut partition: HashMap<String, u32> = HashMap::new();
        for (cluster_id, cluster) in clusters.iter().enumerate() {
            for node in cluster {
                partition.insert(node.clone(), cluster_id as u32);
            }
        }

        // Add unassigned nodes
        if let Some(all) = all_nodes {
            let mut next_id = partition.values().max().copied().unwrap_or(0) + 1;
            for node in all {
                if !partition.contains_key(node) {
                    partition.insert(node.clone(), next_id);
                    next_id += 1;
                }
            }
        }

        partition
    }

    /// Split large cluster into smaller ones
    fn split_cluster(&self, nodes: &HashSet<String>) -> Vec<HashSet<String>> {
        let node_list: Vec<String> = nodes.iter().cloned().collect();
        let n_splits = (node_list.len() / self.max_cluster_size) + 1;
        let split_size = node_list.len() / n_splits;

        let mut splits: Vec<HashSet<String>> = Vec::new();
        for i in 0..n_splits {
            let start = i * split_size;
            let end = if i < n_splits - 1 {
                start + split_size
            } else {
                node_list.len()
            };
            splits.push(node_list[start..end].iter().cloned().collect());
        }

        splits
    }

    /// Build partition from time series data
    pub fn build_partition_from_time_series(
        &self,
        time_series: &HashMap<String, Vec<f64>>,
        strategy: TeStrategy,
    ) -> (HashMap<String, u32>, Vec<CausalLink>) {
        // Compute all-pairs TE
        let nodes: Vec<String> = time_series.keys().cloned().collect();
        let mut causal_links = Vec::new();
        let mut engine = CausalEngine::default();

        for (i, source) in nodes.iter().enumerate() {
            for (j, target) in nodes.iter().enumerate() {
                if i == j {
                    continue;
                }

                let source_data = &time_series[source];
                let target_data = &time_series[target];

                if let Some(link) = engine.compute_pair(source_data, target_data, strategy, source, target) {
                    if link.te >= self.te_threshold {
                        causal_links.push(link);
                    }
                }
            }
        }

        let partition = self.build_partition_from_links(&causal_links, Some(&nodes));

        (partition, causal_links)
    }
}
