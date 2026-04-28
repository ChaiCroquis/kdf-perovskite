//! Node Classifier - Classifies nodes into KDF layers

use super::{ClassificationStats, Layer, NodeClassification};
use crate::fingerprint::{Fingerprint, NodeLabel, StructuralFingerprintEngine};
use std::collections::{HashMap, HashSet};

/// Node Classifier - Classifies nodes into KDF layers
///
/// Classification criteria:
/// - **CORE**: degree > mean + std, high centrality
/// - **EDGE**: degree within [mean - std, mean + std]
/// - **RARE**: degree == 1, unique connection pattern
/// - **GARBAGE**: degree == 0 or very low connectivity with noise pattern
pub struct NodeClassifier {
    /// Threshold multiplier for CORE classification
    pub core_threshold: f64,
    /// Threshold multiplier for GARBAGE classification
    pub garbage_threshold: f64,
    /// Minimum degree for RARE (vs GARBAGE)
    pub rare_min_degree: usize,
    /// Fingerprint engine for RARE node preservation
    fp_engine: StructuralFingerprintEngine,
}

impl Default for NodeClassifier {
    fn default() -> Self {
        Self {
            core_threshold: 1.0,    // mean + 1*std
            garbage_threshold: 0.5, // below mean - 0.5*std with low clustering
            rare_min_degree: 1,
            fp_engine: StructuralFingerprintEngine::default(),
        }
    }
}

impl NodeClassifier {
    /// Create a new classifier with custom thresholds
    pub fn new(core_threshold: f64, garbage_threshold: f64) -> Self {
        Self {
            core_threshold,
            garbage_threshold,
            ..Default::default()
        }
    }

    /// Classify nodes based on graph structure
    ///
    /// # Arguments
    /// * `node_count` - Number of nodes
    /// * `edges` - Edge list as (from, to, weight)
    ///
    /// # Returns
    /// NodeClassification with layer assignments and RARE fingerprints
    pub fn classify(&mut self, node_count: usize, edges: &[(u32, u32, f64)]) -> NodeClassification {
        // Compute degree for each node
        let mut degrees: Vec<f64> = vec![0.0; node_count];
        let mut neighbors: Vec<HashSet<u32>> = vec![HashSet::new(); node_count];

        for &(from, to, weight) in edges {
            if (from as usize) < node_count && (to as usize) < node_count {
                degrees[from as usize] += weight;
                degrees[to as usize] += weight;
                neighbors[from as usize].insert(to);
                neighbors[to as usize].insert(from);
            }
        }

        // Compute statistics
        let sum: f64 = degrees.iter().sum();
        let mean = if node_count > 0 {
            sum / node_count as f64
        } else {
            0.0
        };

        let variance: f64 =
            degrees.iter().map(|&d| (d - mean).powi(2)).sum::<f64>() / node_count.max(1) as f64;
        let std_dev = variance.sqrt();

        // Thresholds
        let core_min = mean + self.core_threshold * std_dev;
        let garbage_max = (mean - self.garbage_threshold * std_dev).max(0.0);

        // Classify nodes
        let mut layers = HashMap::new();
        let mut rare_fingerprints = HashMap::new();
        let mut stats = ClassificationStats::default();

        for node in 0..node_count {
            let node_id = node as u32;
            let degree = degrees[node];
            let neighbor_count = neighbors[node].len();

            let layer = if degree >= core_min && neighbor_count >= 2 {
                // High degree, well connected -> CORE
                Layer::Core
            } else if neighbor_count == 0 {
                // Isolated node -> GARBAGE
                Layer::Garbage
            } else if neighbor_count >= 1 && neighbor_count <= self.rare_min_degree && degree > 0.0
            {
                // Low-degree connection(s) within rare bandwidth -> might be RARE
                // rare_min_degree=1 (default, historical) => exact singleton
                // rare_min_degree=k => neighbor_count in [1..=k] with meaningful connection
                if self.is_meaningful_rare(node_id, &neighbors[node], &degrees) {
                    Layer::Rare
                } else {
                    Layer::Garbage
                }
            } else if degree <= garbage_max
                && self.looks_like_noise_fast(degree, neighbors[node].len())
            {
                // Low degree + noise pattern -> GARBAGE
                Layer::Garbage
            } else {
                // Everything else -> EDGE
                Layer::Edge
            };

            // Generate fingerprint for RARE nodes
            if layer == Layer::Rare {
                let fp = self.generate_rare_fingerprint(node_id, &neighbors[node], edges);
                rare_fingerprints.insert(node_id, fp);
            }

            // Update stats
            match layer {
                Layer::Core => stats.core_count += 1,
                Layer::Edge => stats.edge_count += 1,
                Layer::Rare => stats.rare_count += 1,
                Layer::Garbage => stats.garbage_count += 1,
            }

            layers.insert(node_id, layer);
        }

        NodeClassification {
            layers,
            rare_fingerprints,
            stats,
        }
    }

    /// Check if a single-connection node is a meaningful RARE node
    fn is_meaningful_rare(&self, _node: u32, neighbors: &HashSet<u32>, degrees: &[f64]) -> bool {
        // A RARE node should be connected to a well-connected node (CORE/EDGE)
        // If it's connected to another low-degree node, it's probably noise
        for &neighbor in neighbors {
            if (neighbor as usize) < degrees.len() && degrees[neighbor as usize] >= 2.0 {
                return true;
            }
        }
        false
    }

    /// Check if a node looks like noise (fast O(1) version using precomputed data)
    ///
    /// Uses degree sum and neighbor count already computed in classify()
    /// instead of iterating all edges (O(E) -> O(1) improvement)
    #[inline]
    fn looks_like_noise_fast(&self, degree: f64, neighbor_count: usize) -> bool {
        if neighbor_count == 0 {
            return true;
        }

        let avg_weight = degree / neighbor_count as f64;

        // If average weight is very low and few connections, likely noise
        avg_weight < 0.1 && neighbor_count <= 2
    }

    /// Check if a node looks like noise (legacy O(E) version - kept for reference)
    #[allow(dead_code)]
    fn looks_like_noise(
        &self,
        node: u32,
        neighbors: &HashSet<u32>,
        edges: &[(u32, u32, f64)],
    ) -> bool {
        // Check if edges have very small weights (noise-like)
        let mut total_weight = 0.0;
        let mut edge_count = 0;

        for &(from, to, weight) in edges {
            if from == node || to == node {
                total_weight += weight;
                edge_count += 1;
            }
        }

        if edge_count == 0 {
            return true;
        }

        let avg_weight = total_weight / edge_count as f64;

        // If average weight is very low and few connections, likely noise
        avg_weight < 0.1 && neighbors.len() <= 2
    }

    /// Generate fingerprint for RARE node preservation
    fn generate_rare_fingerprint(
        &mut self,
        node: u32,
        _neighbors: &HashSet<u32>,
        _edges: &[(u32, u32, f64)],
    ) -> Fingerprint {
        // Generate fingerprint using IsolatedTruth pattern
        // The fingerprint engine will create a structured pattern suitable for RARE nodes
        self.fp_engine.compute_fingerprint(
            &node.to_string(),
            &NodeLabel::IsolatedTruth,
            None, // Use default pattern generation
        )
    }

    /// Get processing order (CORE first, then EDGE, skip GARBAGE)
    pub fn processing_order(&self, classification: &NodeClassification) -> Vec<u32> {
        let mut nodes: Vec<(u32, Layer)> = classification
            .layers
            .iter()
            .filter(|(_, layer)| layer.should_process())
            .map(|(&id, &layer)| (id, layer))
            .collect();

        // Sort by priority (CORE > EDGE > RARE), then by node ID for determinism
        nodes.sort_by(|a, b| {
            b.1.priority()
                .cmp(&a.1.priority())
                .then_with(|| a.0.cmp(&b.0))
        });

        nodes.into_iter().map(|(id, _)| id).collect()
    }
}
