//! Simple graph structure for internal use by engines

use std::collections::{HashMap, HashSet};
use super::super::framework::Layer;

/// Simple graph structure for engine use
#[derive(Clone, Debug, Default)]
pub(crate) struct SimpleGraph {
    /// Nodes with their layers
    pub(crate) nodes: HashMap<String, Layer>,
    /// Edges with weights
    pub(crate) edges: Vec<(String, String, f64)>,
    /// Simulation step count
    pub(crate) step_count: u64,
}

impl SimpleGraph {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn add_node(&mut self, id: &str, layer: Layer) {
        self.nodes.insert(id.to_string(), layer);
    }

    pub(crate) fn add_edge(&mut self, from: &str, to: &str, weight: f64) {
        self.edges.push((from.to_string(), to.to_string(), weight));
    }

    pub(crate) fn get_nodes_by_layer(&self, layer: Layer) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(_, &l)| l == layer)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub(crate) fn extract_clusters(&self, layer: Layer) -> Vec<Vec<String>> {
        // Simple clustering: group connected nodes
        let layer_nodes: HashSet<String> = self.get_nodes_by_layer(layer).into_iter().collect();
        if layer_nodes.is_empty() {
            return Vec::new();
        }

        // Build adjacency for layer nodes
        let mut adjacency: HashMap<String, HashSet<String>> = HashMap::new();
        for node in &layer_nodes {
            adjacency.insert(node.clone(), HashSet::new());
        }
        for (from, to, _) in &self.edges {
            if layer_nodes.contains(from) && layer_nodes.contains(to) {
                adjacency.get_mut(from).map(|s| s.insert(to.clone()));
                adjacency.get_mut(to).map(|s| s.insert(from.clone()));
            }
        }

        // Find connected components
        let mut visited: HashSet<String> = HashSet::new();
        let mut clusters = Vec::new();

        for node in &layer_nodes {
            if visited.contains(node) {
                continue;
            }

            let mut component = Vec::new();
            let mut queue = vec![node.clone()];

            while let Some(current) = queue.pop() {
                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current.clone());
                component.push(current.clone());

                if let Some(neighbors) = adjacency.get(&current) {
                    for neighbor in neighbors {
                        if !visited.contains(neighbor) {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }

            if !component.is_empty() {
                clusters.push(component);
            }
        }

        clusters
    }
}
