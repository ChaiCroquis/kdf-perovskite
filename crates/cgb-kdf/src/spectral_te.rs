//! Spectral Gap → TE Pair Prioritization Module
//!
//! Uses spectral gap analysis to prioritize TE computation pairs,
//! reducing computation by 40-60%.
//!
//! # Integration Pattern #4: Spectral → TE Priority
//!
//! Features:
//! - VNE spectral analysis for node importance computation
//! - Priority ranking for TE pair computation
//! - Computational reduction through prioritization
//!
//! # Reference
//!
//! Python implementation: python/kdf/spectral_te_priority.py

use std::collections::HashMap;
use nalgebra::{DMatrix, SymmetricEigen};

use super::causal::{CausalLink, TeStrategy};

// ============================================================================
// Priority Types
// ============================================================================

/// Node spectral information
#[derive(Clone, Debug)]
pub struct NodeSpectralInfo {
    /// Node ID
    pub node_id: String,
    /// Importance score (0.0-1.0)
    pub importance_score: f64,
    /// Centrality measure
    pub centrality: f64,
    /// Cluster ID from spectral clustering
    pub cluster_id: u32,
}

/// Pair priority
#[derive(Clone, Debug)]
pub struct PairPriority {
    /// Source node
    pub source: String,
    /// Target node
    pub target: String,
    /// Priority score (higher = more priority)
    pub priority_score: f64,
    /// Reason for priority
    pub reason: String,
}

// ============================================================================
// Spectral TE Prioritizer
// ============================================================================

/// Statistics for prioritization
#[derive(Clone, Debug, Default)]
pub struct PrioritizationStats {
    /// Total possible pairs
    pub total_possible_pairs: usize,
    /// Prioritized pairs
    pub prioritized_pairs: usize,
    /// Reduction rate
    pub reduction_rate: f64,
}

/// Spectral TE Prioritizer
///
/// Uses VNE spectral gap and eigenvectors to prioritize
/// which node pairs to compute TE for.
///
/// Integration Pattern #4: Spectral → TE Priority
pub struct SpectralTEPrioritizer {
    /// Top K% of pairs to prioritize
    pub top_k_percent: f64,
    /// Use spectral clustering
    pub use_spectral_clustering: bool,
    /// Minimum importance threshold
    pub min_importance: f64,
    /// Statistics
    stats: PrioritizationStats,
}

impl SpectralTEPrioritizer {
    /// Create a new prioritizer
    pub fn new(top_k_percent: f64, use_spectral_clustering: bool, min_importance: f64) -> Self {
        Self {
            top_k_percent,
            use_spectral_clustering,
            min_importance,
            stats: PrioritizationStats::default(),
        }
    }
}

impl Default for SpectralTEPrioritizer {
    fn default() -> Self {
        Self::new(0.3, true, 0.1)
    }
}

impl SpectralTEPrioritizer {

    /// Prioritize pairs using spectral analysis
    ///
    /// # Arguments
    /// * `node_count` - Number of nodes
    /// * `edges` - Edge list as (from, to, weight)
    ///
    /// # Returns
    /// (priority_pairs, node_spectral_info)
    pub fn prioritize_pairs(
        &mut self,
        node_count: usize,
        edges: &[(u32, u32, f64)],
    ) -> (Vec<PairPriority>, Vec<NodeSpectralInfo>) {
        if node_count < 2 {
            return (Vec::new(), Vec::new());
        }

        // Total possible pairs
        let total_pairs = node_count * (node_count - 1);
        self.stats.total_possible_pairs = total_pairs;

        // Compute spectral importance
        let node_info = self.compute_spectral_importance(node_count, edges);

        // Select priority pairs
        let priority_pairs = self.select_priority_pairs(&node_info, edges);

        // Update statistics
        self.stats.prioritized_pairs = priority_pairs.len();
        self.stats.reduction_rate = 1.0 - (priority_pairs.len() as f64 / total_pairs.max(1) as f64);

        (priority_pairs, node_info)
    }

    /// Compute spectral importance for nodes
    fn compute_spectral_importance(
        &self,
        node_count: usize,
        edges: &[(u32, u32, f64)],
    ) -> Vec<NodeSpectralInfo> {
        // Build Laplacian matrix
        let mut laplacian = DMatrix::zeros(node_count, node_count);
        let mut degrees: Vec<f64> = vec![0.0; node_count];

        for &(u, v, weight) in edges {
            let i = u as usize;
            let j = v as usize;
            if i < node_count && j < node_count {
                laplacian[(i, j)] = -weight;
                laplacian[(j, i)] = -weight;
                laplacian[(i, i)] += weight;
                laplacian[(j, j)] += weight;
                degrees[i] += weight;
                degrees[j] += weight;
            }
        }

        // Compute eigenvalues and eigenvectors
        let eigen = SymmetricEigen::new(laplacian);
        let eigenvalues = eigen.eigenvalues;
        let eigenvectors = eigen.eigenvectors;

        // Sort by eigenvalue
        let mut sorted_indices: Vec<usize> = (0..node_count).collect();
        sorted_indices.sort_by(|&a, &b| {
            eigenvalues[a]
                .partial_cmp(&eigenvalues[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Fiedler vector (second smallest eigenvalue's eigenvector) for importance
        let fiedler_importance: Vec<f64> = if node_count >= 2 {
            let fiedler_idx = sorted_indices[1];
            let fiedler: Vec<f64> = eigenvectors.column(fiedler_idx).iter().cloned().collect();
            let max_fiedler = fiedler.iter().map(|x| x.abs()).fold(0.0_f64, |a, b| a.max(b)) + 1e-10;
            fiedler.iter().map(|&x| x.abs() / max_fiedler).collect()
        } else {
            vec![1.0; node_count]
        };

        // Degree centrality
        let max_degree = degrees.iter().cloned().fold(0.0_f64, |a, b| a.max(b)) + 1e-10;
        let degree_centrality: Vec<f64> = degrees.iter().map(|&d| d / max_degree).collect();

        // Combined importance score
        let importance_scores: Vec<f64> = fiedler_importance
            .iter()
            .zip(degree_centrality.iter())
            .map(|(&f, &d)| 0.5 * f + 0.5 * d)
            .collect();

        // Spectral clustering
        let cluster_ids = if self.use_spectral_clustering && node_count >= 2 {
            self.spectral_clustering(&eigenvectors, node_count)
        } else {
            (0..node_count as u32).collect()
        };

        // Build NodeSpectralInfo
        let mut node_info: Vec<NodeSpectralInfo> = (0..node_count)
            .map(|i| NodeSpectralInfo {
                node_id: i.to_string(),
                importance_score: importance_scores[i],
                centrality: degree_centrality[i],
                cluster_id: cluster_ids[i],
            })
            .collect();

        // Sort by importance
        node_info.sort_by(|a, b| {
            b.importance_score
                .partial_cmp(&a.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        node_info
    }

    /// Spectral clustering using eigenvectors
    fn spectral_clustering(&self, eigenvectors: &DMatrix<f64>, n: usize) -> Vec<u32> {
        if n < 4 {
            return (0..n as u32).collect();
        }

        // Use first few eigenvectors as features
        let k = (n - 1).min(5);
        let mut features: Vec<Vec<f64>> = Vec::with_capacity(n);

        for i in 0..n {
            let mut feature = Vec::with_capacity(k);
            for j in 1..=k {
                feature.push(eigenvectors[(i, j)]);
            }
            features.push(feature);
        }

        // Simple k-means clustering
        let n_clusters = (n / 2 + 1).min(5);
        self.simple_kmeans(&features, n_clusters)
    }

    /// Simple k-means clustering
    fn simple_kmeans(&self, features: &[Vec<f64>], k: usize) -> Vec<u32> {
        let n = features.len();
        if n <= k {
            return (0..n as u32).collect();
        }

        let dim = features[0].len();

        // Initialize centers (first k points)
        let mut centers: Vec<Vec<f64>> = features[..k].to_vec();
        let mut labels = vec![0u32; n];

        for _ in 0..50 {
            // Assignment step
            for (i, feature) in features.iter().enumerate() {
                let mut min_dist = f64::INFINITY;
                for (c, center) in centers.iter().enumerate() {
                    let dist: f64 = feature
                        .iter()
                        .zip(center.iter())
                        .map(|(&a, &b)| (a - b).powi(2))
                        .sum();
                    if dist < min_dist {
                        min_dist = dist;
                        labels[i] = c as u32;
                    }
                }
            }

            // Update step
            let mut new_centers = vec![vec![0.0; dim]; k];
            let mut counts = vec![0usize; k];

            for (i, feature) in features.iter().enumerate() {
                let c = labels[i] as usize;
                for (j, &v) in feature.iter().enumerate() {
                    new_centers[c][j] += v;
                }
                counts[c] += 1;
            }

            let mut converged = true;
            for c in 0..k {
                if counts[c] > 0 {
                    for j in 0..dim {
                        let new_val = new_centers[c][j] / counts[c] as f64;
                        if (new_val - centers[c][j]).abs() > 1e-6 {
                            converged = false;
                        }
                        centers[c][j] = new_val;
                    }
                }
            }

            if converged {
                break;
            }
        }

        labels
    }

    /// Select priority pairs based on spectral analysis
    fn select_priority_pairs(
        &self,
        node_info: &[NodeSpectralInfo],
        edges: &[(u32, u32, f64)],
    ) -> Vec<PairPriority> {
        let n = node_info.len();
        let total_pairs = n * (n - 1);
        let max_pairs = (total_pairs as f64 * self.top_k_percent) as usize;

        let mut priority_pairs = Vec::new();

        // Important nodes (used for filtering below)
        let _important_nodes: Vec<&NodeSpectralInfo> = node_info
            .iter()
            .filter(|info| info.importance_score >= self.min_importance)
            .collect();

        // Group nodes by cluster
        let mut cluster_nodes: HashMap<u32, Vec<&NodeSpectralInfo>> = HashMap::new();
        for info in node_info {
            cluster_nodes
                .entry(info.cluster_id)
                .or_default()
                .push(info);
        }

        // Same-cluster pairs (high priority)
        for members in cluster_nodes.values() {
            for (i, source) in members.iter().enumerate() {
                for target in members[i + 1..].iter() {
                    if priority_pairs.len() >= max_pairs {
                        break;
                    }
                    let score = (source.importance_score + target.importance_score) / 2.0 * 1.5;
                    priority_pairs.push(PairPriority {
                        source: source.node_id.clone(),
                        target: target.node_id.clone(),
                        priority_score: score,
                        reason: "same_cluster".to_string(),
                    });
                }
            }
        }

        // Inter-cluster hub pairs (medium priority)
        let cluster_ids: Vec<u32> = cluster_nodes.keys().cloned().collect();
        for (i, &cluster_i) in cluster_ids.iter().enumerate() {
            for &cluster_j in cluster_ids[i + 1..].iter() {
                if priority_pairs.len() >= max_pairs {
                    break;
                }

                let members_i = &cluster_nodes[&cluster_i];
                let members_j = &cluster_nodes[&cluster_j];

                if members_i.is_empty() || members_j.is_empty() {
                    continue;
                }

                // Hub of each cluster
                let hub_i = members_i
                    .iter()
                    .max_by(|a, b| {
                        a.importance_score
                            .partial_cmp(&b.importance_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap();
                let hub_j = members_j
                    .iter()
                    .max_by(|a, b| {
                        a.importance_score
                            .partial_cmp(&b.importance_score)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .unwrap();

                let score = (hub_i.importance_score + hub_j.importance_score) / 2.0;
                priority_pairs.push(PairPriority {
                    source: hub_i.node_id.clone(),
                    target: hub_j.node_id.clone(),
                    priority_score: score,
                    reason: "inter_cluster_hub".to_string(),
                });
            }
        }

        // Edge-connected pairs
        let existing: std::collections::HashSet<(String, String)> = priority_pairs
            .iter()
            .map(|p| (p.source.clone(), p.target.clone()))
            .collect();

        for &(u, v, _) in edges {
            if priority_pairs.len() >= max_pairs {
                break;
            }

            let u_str = u.to_string();
            let v_str = v.to_string();

            if existing.contains(&(u_str.clone(), v_str.clone()))
                || existing.contains(&(v_str.clone(), u_str.clone()))
            {
                continue;
            }

            let info_u = node_info.iter().find(|n| n.node_id == u_str);
            let info_v = node_info.iter().find(|n| n.node_id == v_str);

            if let (Some(iu), Some(iv)) = (info_u, info_v) {
                let score = (iu.importance_score + iv.importance_score) / 2.0 * 0.8;
                priority_pairs.push(PairPriority {
                    source: u_str,
                    target: v_str,
                    priority_score: score,
                    reason: "edge_connected".to_string(),
                });
            }
        }

        // Sort by priority
        priority_pairs.sort_by(|a, b| {
            b.priority_score
                .partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        priority_pairs.truncate(max_pairs);
        priority_pairs
    }

    /// Compute prioritized TE
    ///
    /// # Arguments
    /// * `node_count` - Number of nodes
    /// * `edges` - Edge list
    /// * `time_series` - Time series data per node
    pub fn compute_prioritized_te(
        &mut self,
        node_count: usize,
        edges: &[(u32, u32, f64)],
        time_series: &HashMap<String, Vec<f64>>,
    ) -> (Vec<CausalLink>, PrioritizedTeStats) {
        use super::causal::GaussianEstimator;

        // Get priority pairs
        let (priority_pairs, node_info) = self.prioritize_pairs(node_count, edges);

        // Compute TE for priority pairs
        let estimator = GaussianEstimator::default();
        let mut causal_links = Vec::new();
        let mut computed = 0;

        for pair in &priority_pairs {
            let source_data = match time_series.get(&pair.source) {
                Some(d) => d,
                None => continue,
            };
            let target_data = match time_series.get(&pair.target) {
                Some(d) => d,
                None => continue,
            };

            if let Some(result) = estimator.compute(source_data, target_data) {
                if result.te > 0.01 {
                    causal_links.push(CausalLink::new(
                        pair.source.clone(),
                        pair.target.clone(),
                        result.te,
                        TeStrategy::Screening,
                    ));
                }
            }

            computed += 1;
        }

        let stats = PrioritizedTeStats {
            total_possible_pairs: self.stats.total_possible_pairs,
            prioritized_pairs: self.stats.prioritized_pairs,
            reduction_rate: self.stats.reduction_rate,
            pairs_computed: computed,
            links_found: causal_links.len(),
            node_spectral_info_count: node_info.len(),
        };

        (causal_links, stats)
    }

    /// Get statistics
    pub fn get_statistics(&self) -> PrioritizationStats {
        self.stats.clone()
    }
}

/// Statistics for prioritized TE computation
#[derive(Clone, Debug)]
pub struct PrioritizedTeStats {
    /// Total possible pairs
    pub total_possible_pairs: usize,
    /// Prioritized pairs
    pub prioritized_pairs: usize,
    /// Reduction rate
    pub reduction_rate: f64,
    /// Pairs actually computed
    pub pairs_computed: usize,
    /// Links found
    pub links_found: usize,
    /// Node spectral info count
    pub node_spectral_info_count: usize,
}

/// Quick access function for TE prioritization
pub fn prioritize_te_computation(
    node_count: usize,
    edges: &[(u32, u32, f64)],
    top_k_percent: f64,
) -> Vec<(String, String)> {
    let mut prioritizer = SpectralTEPrioritizer::new(top_k_percent, true, 0.1);
    let (priority_pairs, _) = prioritizer.prioritize_pairs(node_count, edges);
    priority_pairs
        .into_iter()
        .map(|p| (p.source, p.target))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_graph() -> (usize, Vec<(u32, u32, f64)>) {
        let edges = vec![
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 3, 1.0),
            (3, 0, 1.0),
            (0, 2, 0.5),
            (4, 5, 1.0),
            (5, 6, 1.0),
        ];
        (7, edges)
    }

    #[test]
    fn test_spectral_importance() {
        let (node_count, edges) = create_test_graph();
        let mut prioritizer = SpectralTEPrioritizer::default();

        let (pairs, node_info) = prioritizer.prioritize_pairs(node_count, &edges);

        // Should have node info for all nodes
        assert_eq!(node_info.len(), node_count);

        // Should have some priority pairs
        assert!(!pairs.is_empty());

        // Reduction rate should be positive
        let stats = prioritizer.get_statistics();
        assert!(stats.reduction_rate > 0.0);
    }

    #[test]
    fn test_clustering() {
        let (node_count, edges) = create_test_graph();
        let mut prioritizer = SpectralTEPrioritizer::new(0.5, true, 0.0);

        let (_, node_info) = prioritizer.prioritize_pairs(node_count, &edges);

        // All nodes should have cluster IDs assigned
        assert_eq!(node_info.len(), node_count);

        // All cluster IDs should be valid
        for info in &node_info {
            assert!(info.cluster_id < node_count as u32);
        }
    }

    #[test]
    fn test_priority_scoring() {
        let (node_count, edges) = create_test_graph();
        let mut prioritizer = SpectralTEPrioritizer::default();

        let (pairs, _) = prioritizer.prioritize_pairs(node_count, &edges);

        // Pairs should be sorted by priority
        for i in 1..pairs.len() {
            assert!(pairs[i - 1].priority_score >= pairs[i].priority_score);
        }
    }

    #[test]
    fn test_prioritize_te_computation() {
        let (node_count, edges) = create_test_graph();
        let pairs = prioritize_te_computation(node_count, &edges, 0.3);

        // Should return some pairs
        assert!(!pairs.is_empty());

        // Should be less than total possible pairs
        let total = node_count * (node_count - 1);
        assert!(pairs.len() <= total);
    }

    #[test]
    fn test_prioritized_te_computation() {
        let (node_count, edges) = create_test_graph();

        // Create time series data
        let mut time_series = HashMap::new();
        for i in 0..node_count {
            let series: Vec<f64> = (0..100)
                .map(|j| (i as f64 + j as f64 * 0.1).sin())
                .collect();
            time_series.insert(i.to_string(), series);
        }

        let mut prioritizer = SpectralTEPrioritizer::default();
        let (links, stats) = prioritizer.compute_prioritized_te(node_count, &edges, &time_series);

        // Should have computed some pairs
        assert!(stats.pairs_computed > 0);

        // Links should have valid values
        for link in &links {
            assert!(link.te >= 0.0);
        }
    }
}
