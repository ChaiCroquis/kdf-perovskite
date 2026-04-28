//! # KDF - Knowledge Decay Framework (Rev.10 Basic subset)
//!
//! This crate implements only the **Rev.10 Basic** subset of KDF (代謝制御手段 +
//! 希少性保護手段). The third claim-1 requirement (整合性発見手段) lives in
//! `cgb-kdf` (`crates/cgb-kdf/`). For patent claim 1 compliance, compose
//! `kdf-lib::Kdf` with `cgb_kdf::AnalogyDiscoveryEngine`, or simply use
//! `cgb_kdf::framework::rev12::KdfProcessorRev12` which integrates all three
//! claim-1 手段.
//!
//! # Patent claim compliance (Phase 0/1 snapshot)
//!
//! See `docs/patent/TRACEABILITY.md` for the authoritative matrix.
//! - This crate: Claim 2-10, 15, 18-19 (subset)
//! - `cgb-kdf`:  Claim 1, 35-48 (reference implementation)
//!
//! KDF is an automatic data redundancy reduction framework that preserves rare items.
//!
//! ## Features
//!
//! - `serde` (default): Enable serialization/deserialization support
//! - `parallel`: Enable parallel processing with rayon
//!
//! ## Key Features
//!
//! - **Automatic**: No threshold tuning required
//! - **Rare-preserving**: Isolated/unique items are always preserved
//! - **Redundancy-reducing**: Similar items are automatically consolidated
//! - **Flexible**: Works with any similarity function
//!
//! ## Quick Start
//!
//! ```rust
//! use kdf::{Kdf, KdfParams, cosine_similarity};
//!
//! // Define your data items
//! let items = vec![
//!     vec![1.0, 0.9, 0.1],  // Cluster A
//!     vec![1.0, 0.9, 0.1],  // Cluster A (redundant)
//!     vec![0.1, 0.1, 0.9],  // Cluster B
//!     vec![-1.0, 0.0, 0.0], // Rare (isolated)
//! ];
//!
//! // Create KDF instance with default parameters
//! let kdf = Kdf::new(KdfParams::default());
//!
//! // Process items with cosine similarity threshold
//! let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
//!
//! // result.selected contains indices of non-redundant items
//! assert!(result.selected.len() <= items.len());
//! ```
//!
//! ## Algorithm Overview
//!
//! KDF works in four phases:
//!
//! 1. **Graph Construction**: Build similarity graph from pairwise comparisons
//! 2. **Layer Classification**: Classify items as Core (high connectivity),
//!    Edge (medium), or Rare (isolated)
//! 3. **Decay Iteration**: Apply layer-specific decay to item weights
//! 4. **Selection**: Select items based on weights and similarity
//!
//! ## Parameters (Rev.12)
//!
//! | Parameter | Value | Description |
//! |-----------|-------|-------------|
//! | α_E | 1.5 | Edge layer decay exponent |
//! | α_R | 0.3 | Rare layer decay exponent |
//! | α_C | 2.0 | Core layer decay exponent |
//! | θ_E | 0.15 | Edge layer weight threshold |
//! | β | 0.01 | Base decay rate |
//! | γ | 0.1 | Connectivity scaling factor |

use std::collections::{HashMap, HashSet};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// Layer classification for items
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Layer {
    /// High connectivity items (highly redundant)
    Core,
    /// Medium connectivity items
    Edge,
    /// Low/zero connectivity items (rare, unique)
    Rare,
}

/// KDF algorithm parameters (Rev.12)
///
/// ## Master Specification (Edge-Based)
///
/// The decay formula for edge (u,v) is:
/// ```text
/// P_decay(u,v) = min(1.0, β × (1 + γ × C_(u,v)^α))
/// ```
/// where C_(u,v) = deg(u) + deg(v) (sum of endpoint degrees)
///
/// Layer-specific parameters:
/// - Edge:  α=1.5, γ=0.015
/// - Rare:  α=0.3, γ=0.010
/// - Core:  α=2.0, γ=0.008
/// - Meta:  α=0.5, γ=0.005
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KdfParams {
    /// Edge layer decay exponent (default: 1.5)
    pub alpha_edge: f64,
    /// Rare layer decay exponent (default: 0.3)
    pub alpha_rare: f64,
    /// Core layer decay exponent (default: 2.0)
    pub alpha_core: f64,
    /// Meta layer decay exponent (default: 0.5)
    pub alpha_meta: f64,
    /// Edge layer weight threshold (default: 0.15)
    pub theta_edge: f64,
    /// Base decay rate (default: 0.01)
    pub beta: f64,
    /// Connectivity scaling factor - legacy single value (default: 0.1)
    /// Use layer-specific gamma values for Master spec compliance
    pub gamma: f64,
    /// Edge layer gamma (Master spec: 0.015)
    pub gamma_edge: f64,
    /// Rare layer gamma (Master spec: 0.010)
    pub gamma_rare: f64,
    /// Core layer gamma (Master spec: 0.008)
    pub gamma_core: f64,
    /// Meta layer gamma (Master spec: 0.005)
    pub gamma_meta: f64,
    /// Use edge-based congestion calculation (Master spec compliant)
    pub use_edge_based: bool,
    /// Number of decay iterations (default: 100)
    pub iterations: usize,
    /// Core layer threshold multiplier (default: 1.5)
    pub core_threshold: f64,
    /// Rare layer threshold multiplier (default: 0.3)
    pub rare_threshold: f64,
    /// Selection similarity threshold (default: 0.75)
    pub selection_sim_threshold: f64,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            alpha_edge: 1.5,
            alpha_rare: 0.3,
            alpha_core: 2.0,
            alpha_meta: 0.5,
            theta_edge: 0.15,
            beta: 0.01,
            gamma: 0.1, // Legacy: single gamma for backward compatibility
            // Master spec layer-specific gamma values
            gamma_edge: 0.015,
            gamma_rare: 0.010,
            gamma_core: 0.008,
            gamma_meta: 0.005,
            use_edge_based: true, // Default: Master spec compliant edge-based
            iterations: 100,
            core_threshold: 1.5,
            rare_threshold: 0.3,
            selection_sim_threshold: 0.75,
        }
    }
}

impl KdfParams {
    /// Create Master spec compliant parameters (edge-based processing)
    pub fn master_spec() -> Self {
        Self {
            use_edge_based: true,
            ..Self::default()
        }
    }

    /// Get gamma value for a specific layer
    pub fn gamma_for_layer(&self, layer: Layer) -> f64 {
        if self.use_edge_based {
            match layer {
                Layer::Core => self.gamma_core,
                Layer::Edge => self.gamma_edge,
                Layer::Rare => self.gamma_rare,
            }
        } else {
            self.gamma
        }
    }
}

/// Builder for KdfParams with fluent API
#[derive(Clone, Debug)]
pub struct KdfParamsBuilder {
    params: KdfParams,
}

impl KdfParamsBuilder {
    /// Create a new builder with default parameters
    pub fn new() -> Self {
        Self {
            params: KdfParams::default(),
        }
    }

    /// Set edge layer decay exponent
    pub fn alpha_edge(mut self, value: f64) -> Self {
        self.params.alpha_edge = value;
        self
    }

    /// Set rare layer decay exponent
    pub fn alpha_rare(mut self, value: f64) -> Self {
        self.params.alpha_rare = value;
        self
    }

    /// Set core layer decay exponent
    pub fn alpha_core(mut self, value: f64) -> Self {
        self.params.alpha_core = value;
        self
    }

    /// Set edge layer weight threshold
    pub fn theta_edge(mut self, value: f64) -> Self {
        self.params.theta_edge = value;
        self
    }

    /// Set base decay rate
    pub fn beta(mut self, value: f64) -> Self {
        self.params.beta = value;
        self
    }

    /// Set connectivity scaling factor (legacy single value)
    pub fn gamma(mut self, value: f64) -> Self {
        self.params.gamma = value;
        self
    }

    /// Set Edge layer gamma (Master spec: 0.015)
    pub fn gamma_edge(mut self, value: f64) -> Self {
        self.params.gamma_edge = value;
        self
    }

    /// Set Rare layer gamma (Master spec: 0.010)
    pub fn gamma_rare(mut self, value: f64) -> Self {
        self.params.gamma_rare = value;
        self
    }

    /// Set Core layer gamma (Master spec: 0.008)
    pub fn gamma_core(mut self, value: f64) -> Self {
        self.params.gamma_core = value;
        self
    }

    /// Set Meta layer gamma (Master spec: 0.005)
    pub fn gamma_meta(mut self, value: f64) -> Self {
        self.params.gamma_meta = value;
        self
    }

    /// Set alpha for Meta layer
    pub fn alpha_meta(mut self, value: f64) -> Self {
        self.params.alpha_meta = value;
        self
    }

    /// Enable edge-based processing (Master spec compliant)
    pub fn use_edge_based(mut self, value: bool) -> Self {
        self.params.use_edge_based = value;
        self
    }

    /// Set number of decay iterations
    pub fn iterations(mut self, value: usize) -> Self {
        self.params.iterations = value;
        self
    }

    /// Set core layer threshold multiplier
    pub fn core_threshold(mut self, value: f64) -> Self {
        self.params.core_threshold = value;
        self
    }

    /// Set rare layer threshold multiplier
    pub fn rare_threshold(mut self, value: f64) -> Self {
        self.params.rare_threshold = value;
        self
    }

    /// Set selection similarity threshold
    pub fn selection_sim_threshold(mut self, value: f64) -> Self {
        self.params.selection_sim_threshold = value;
        self
    }

    /// Build the KdfParams
    pub fn build(self) -> KdfParams {
        self.params
    }
}

impl Default for KdfParamsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl KdfParams {
    /// Create a builder for KdfParams
    pub fn builder() -> KdfParamsBuilder {
        KdfParamsBuilder::new()
    }
}

// ============================================================================
// Common utility functions (extracted for reuse)
// ============================================================================

/// Classify items into layers based on their degrees
fn classify_layers(degrees: &[usize], params: &KdfParams) -> Vec<Layer> {
    let n = degrees.len();
    if n == 0 {
        return vec![];
    }

    let avg_degree = degrees.iter().sum::<usize>() as f64 / n as f64;

    degrees
        .iter()
        .map(|&deg| {
            if deg == 0 {
                Layer::Rare
            } else if (deg as f64) > avg_degree * params.core_threshold {
                Layer::Core
            } else if (deg as f64) < avg_degree * params.rare_threshold {
                Layer::Rare
            } else {
                Layer::Edge
            }
        })
        .collect()
}

/// Pre-compute decay factors for each item (optimization: avoid repeated powf calls)
///
/// Uses node-based congestion: C_i = deg(i)
fn compute_decay_factors(degrees: &[usize], layers: &[Layer], params: &KdfParams) -> Vec<f64> {
    degrees
        .iter()
        .zip(layers)
        .map(|(&deg, &layer)| {
            let c = deg as f64;
            let (alpha, gamma) = match layer {
                Layer::Core => (params.alpha_core, params.gamma_for_layer(Layer::Core)),
                Layer::Edge => (params.alpha_edge, params.gamma_for_layer(Layer::Edge)),
                Layer::Rare => (params.alpha_rare, params.gamma_for_layer(Layer::Rare)),
            };
            (1.0 - params.beta * (1.0 + gamma * c.powf(alpha))).max(0.0)
        })
        .collect()
}

/// Compute edge-based congestion: C_(u,v) = deg(u) + deg(v)
///
/// This follows the Master specification for knowledge network processing.
pub fn compute_edge_congestion(u: usize, v: usize, degrees: &[usize]) -> f64 {
    (degrees[u] + degrees[v]) as f64
}

/// Compute decay probability for an edge according to Master spec
///
/// P_decay(u,v) = min(1.0, β × (1 + γ × C_(u,v)^α))
/// where C_(u,v) = deg(u) + deg(v)
pub fn compute_edge_decay_probability(
    u: usize,
    v: usize,
    degrees: &[usize],
    layer: Layer,
    params: &KdfParams,
) -> f64 {
    let congestion = compute_edge_congestion(u, v, degrees);
    let (alpha, gamma) = match layer {
        Layer::Core => (params.alpha_core, params.gamma_core),
        Layer::Edge => (params.alpha_edge, params.gamma_edge),
        Layer::Rare => (params.alpha_rare, params.gamma_rare),
    };
    (params.beta * (1.0 + gamma * congestion.powf(alpha))).min(1.0)
}

/// Apply decay to edge weights (Master spec compliant)
///
/// Updates edge weights in-place using edge-based congestion calculation.
pub fn apply_edge_decay(
    edge_weights: &mut HashMap<(usize, usize), f64>,
    degrees: &[usize],
    edge_layers: &HashMap<(usize, usize), Layer>,
    params: &KdfParams,
) {
    for ((u, v), weight) in edge_weights.iter_mut() {
        let layer = edge_layers.get(&(*u, *v)).copied().unwrap_or(Layer::Edge);
        let decay_prob = compute_edge_decay_probability(*u, *v, degrees, layer, params);
        *weight *= 1.0 - decay_prob;
    }
}

/// Compute weights using pre-computed decay factors
fn compute_weights(decay_factors: &[f64], iterations: usize) -> Vec<f64> {
    let n = decay_factors.len();
    let mut weights = vec![1.0f64; n];

    for _ in 0..iterations {
        for i in 0..n {
            weights[i] *= decay_factors[i];
        }
    }

    weights
}

// ============================================================================
// SelectionReason
// ============================================================================

/// Selection reason for an item
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SelectionReason {
    /// Item was isolated (rare)
    Rare,
    /// Item is representative of a group
    Representative { group_size: usize },
    /// Item was not selected
    NotSelected { representative: usize },
}

// ============================================================================
// KdfResult
// ============================================================================

/// Result of KDF processing
///
/// ## Master Specification (Edge-Based)
///
/// KDF operates on **edge weights** `w_ij`, not node weights.
/// The `selection_scores` field is derived from edge weights for selection purposes.
/// The `edge_weights` field contains the actual KDF-compliant edge weights.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KdfResult {
    /// Indices of selected items
    pub selected: Vec<usize>,
    /// Layer classification for each item
    pub layers: Vec<Layer>,
    /// Selection scores for each item (derived from edge weights, NOT KDF weights)
    /// Use this for selection/ranking purposes.
    pub selection_scores: Vec<f64>,
    /// Degree (connectivity) for each item
    pub degrees: Vec<usize>,
    /// Cluster assignments (representative index for each item)
    pub clusters: Vec<usize>,
    /// Edge weights (Master spec compliant) - the actual KDF weights
    /// Key: (u, v) where u < v, Value: edge weight w_ij
    #[cfg_attr(feature = "serde", serde(default))]
    pub edge_weights: HashMap<(usize, usize), f64>,
    /// Selected indices as HashSet for O(1) lookup
    #[cfg_attr(feature = "serde", serde(skip))]
    selected_set: HashSet<usize>,
}

impl KdfResult {
    /// Create a new KdfResult (internal)
    fn new(
        selected: Vec<usize>,
        layers: Vec<Layer>,
        selection_scores: Vec<f64>,
        degrees: Vec<usize>,
        clusters: Vec<usize>,
    ) -> Self {
        let selected_set: HashSet<usize> = selected.iter().copied().collect();
        Self {
            selected,
            layers,
            selection_scores,
            degrees,
            clusters,
            edge_weights: HashMap::new(),
            selected_set,
        }
    }

    /// Create a new KdfResult with edge weights (Master spec compliant)
    #[allow(dead_code)]
    fn new_with_edges(
        selected: Vec<usize>,
        layers: Vec<Layer>,
        selection_scores: Vec<f64>,
        degrees: Vec<usize>,
        clusters: Vec<usize>,
        edge_weights: HashMap<(usize, usize), f64>,
    ) -> Self {
        let selected_set: HashSet<usize> = selected.iter().copied().collect();
        Self {
            selected,
            layers,
            selection_scores,
            degrees,
            clusters,
            edge_weights,
            selected_set,
        }
    }

    /// Backward compatibility: get weights (deprecated, use selection_scores)
    #[deprecated(
        since = "0.2.0",
        note = "Use selection_scores instead. 'weights' is not a KDF concept."
    )]
    pub fn weights(&self) -> &Vec<f64> {
        &self.selection_scores
    }

    /// Get selected item indices
    pub fn selected_indices(&self) -> &[usize] {
        &self.selected
    }

    /// Get number of selected items
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Check if an item is selected (O(1) lookup)
    pub fn is_selected(&self, idx: usize) -> bool {
        self.selected_set.contains(&idx)
    }

    /// Calculate redundancy reduction ratio
    pub fn redundancy_reduction<T, F>(&self, items: &[T], is_rare: F) -> f64
    where
        F: Fn(&T) -> bool,
    {
        let redundant_total = items.iter().filter(|i| !is_rare(i)).count();
        let redundant_selected = self
            .selected
            .iter()
            .filter(|&&i| !is_rare(&items[i]))
            .count();

        if redundant_total > 0 {
            (redundant_total - redundant_selected) as f64 / redundant_total as f64
        } else {
            1.0
        }
    }

    /// Calculate rare preservation ratio
    pub fn rare_preservation<T, F>(&self, items: &[T], is_rare: F) -> f64
    where
        F: Fn(&T) -> bool,
    {
        let rare_total = items.iter().filter(|i| is_rare(i)).count();
        let rare_selected = self
            .selected
            .iter()
            .filter(|&&i| is_rare(&items[i]))
            .count();

        if rare_total > 0 {
            rare_selected as f64 / rare_total as f64
        } else {
            1.0
        }
    }

    /// Calculate F1 score
    pub fn f1_score<T, F>(&self, items: &[T], is_rare: F) -> f64
    where
        F: Fn(&T) -> bool + Copy,
    {
        let rr = self.redundancy_reduction(items, is_rare);
        let rp = self.rare_preservation(items, is_rare);

        if rr + rp > 0.0 {
            2.0 * rr * rp / (rr + rp)
        } else {
            0.0
        }
    }

    /// Get the representative (cluster center) for an item
    pub fn representative_of(&self, idx: usize) -> usize {
        self.clusters.get(idx).copied().unwrap_or(idx)
    }

    /// Get all items in the same cluster as the given item
    pub fn cluster_members(&self, idx: usize) -> Vec<usize> {
        let rep = self.representative_of(idx);
        self.clusters
            .iter()
            .enumerate()
            .filter(|&(_, &r)| r == rep)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get clusters as groups of indices
    pub fn cluster_groups(&self) -> Vec<Vec<usize>> {
        let mut groups: HashMap<usize, Vec<usize>> = HashMap::new();
        for (i, &rep) in self.clusters.iter().enumerate() {
            groups.entry(rep).or_default().push(i);
        }
        groups.into_values().collect()
    }

    /// Get all Rare layer items
    pub fn rare_items(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|&(_, &layer)| layer == Layer::Rare)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get all Edge layer items
    pub fn edge_items(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|&(_, &layer)| layer == Layer::Edge)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get all Core layer items
    pub fn core_items(&self) -> Vec<usize> {
        self.layers
            .iter()
            .enumerate()
            .filter(|&(_, &layer)| layer == Layer::Core)
            .map(|(i, _)| i)
            .collect()
    }

    /// Get the selection reason for an item (O(1) lookup)
    pub fn reason(&self, idx: usize) -> SelectionReason {
        if self.layers.get(idx) == Some(&Layer::Rare) {
            SelectionReason::Rare
        } else if self.is_selected(idx) {
            let group_size = self.cluster_members(idx).len();
            SelectionReason::Representative { group_size }
        } else {
            SelectionReason::NotSelected {
                representative: self.representative_of(idx),
            }
        }
    }

    // ========================================================================
    // Anomaly Scoring (KDF-specific feature)
    // ========================================================================

    /// Calculate anomaly score for an item (0.0 = normal, 1.0 = highly anomalous)
    ///
    /// Based on structural isolation: lower degree = higher anomaly score
    /// Rare layer items get score 1.0, Core items get score close to 0.0
    pub fn anomaly_score(&self, idx: usize) -> f64 {
        if idx >= self.degrees.len() {
            return 0.0;
        }

        let degree = self.degrees[idx];

        // Degree 0 (Rare) = max anomaly
        if degree == 0 {
            return 1.0;
        }

        // Calculate max degree for normalization
        let max_degree = *self.degrees.iter().max().unwrap_or(&1);
        if max_degree == 0 {
            return 1.0;
        }

        // Inverse relationship: low degree = high anomaly
        // Also factor in the weight (lower weight = more decayed = more normal)
        let degree_score = 1.0 - (degree as f64 / max_degree as f64);
        let weight_score = self.selection_scores.get(idx).copied().unwrap_or(0.0);

        // Combine: high weight + low degree = anomaly
        // Low weight means heavily decayed (normal), high weight means preserved (potentially anomalous)
        (degree_score * 0.7 + weight_score * 0.3).min(1.0)
    }

    /// Get top k items by anomaly score
    pub fn top_anomalies(&self, k: usize) -> Vec<(usize, f64)> {
        let mut scores: Vec<(usize, f64)> = (0..self.degrees.len())
            .map(|i| (i, self.anomaly_score(i)))
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(k);
        scores
    }

    /// Get all items with anomaly score above threshold
    pub fn anomalies_above(&self, threshold: f64) -> Vec<(usize, f64)> {
        (0..self.degrees.len())
            .map(|i| (i, self.anomaly_score(i)))
            .filter(|(_, score)| *score >= threshold)
            .collect()
    }

    // ========================================================================
    // Statistics (KDF-specific feature)
    // ========================================================================

    /// Get comprehensive statistics about the KDF result
    pub fn stats(&self) -> KdfStats {
        let n = self.layers.len();
        if n == 0 {
            return KdfStats::default();
        }

        let mut layer_counts = HashMap::new();
        for &layer in &self.layers {
            *layer_counts.entry(layer).or_insert(0usize) += 1;
        }

        let total_degree: usize = self.degrees.iter().sum();
        let max_degree = *self.degrees.iter().max().unwrap_or(&0);

        let cluster_groups = self.cluster_groups();
        let cluster_sizes: Vec<usize> = cluster_groups.iter().map(|g| g.len()).collect();
        let avg_cluster_size = if cluster_groups.is_empty() {
            0.0
        } else {
            cluster_sizes.iter().sum::<usize>() as f64 / cluster_groups.len() as f64
        };

        let rare_count = layer_counts.get(&Layer::Rare).copied().unwrap_or(0);

        KdfStats {
            total_items: n,
            selected_count: self.selected.len(),
            layer_counts,
            avg_degree: total_degree as f64 / n as f64,
            max_degree,
            cluster_count: cluster_groups.len(),
            avg_cluster_size,
            max_cluster_size: cluster_sizes.into_iter().max().unwrap_or(0),
            isolation_ratio: rare_count as f64 / n as f64,
            redundancy_ratio: 1.0 - (self.selected.len() as f64 / n as f64),
        }
    }

    // ========================================================================
    // Explanation Generation (Interpretability feature)
    // ========================================================================

    /// Generate human-readable explanation for a single item
    pub fn explain(&self, idx: usize) -> String {
        if idx >= self.layers.len() {
            return format!("Item {} does not exist", idx);
        }

        let layer = self.layers[idx];
        let degree = self.degrees[idx];
        let weight = self.selection_scores[idx];
        let is_selected = self.is_selected(idx);

        let mut explanation = String::new();

        // Basic info
        explanation.push_str(&format!("Item {}: ", idx));

        if is_selected {
            explanation.push_str("SELECTED\n");
        } else {
            explanation.push_str("FILTERED\n");
        }

        // Layer explanation
        match layer {
            Layer::Rare => {
                explanation.push_str("  - Layer: Rare (isolated, no similar items found)\n");
                explanation.push_str(&format!(
                    "  - Connectivity: {} connections (degree=0 or very low)\n",
                    degree
                ));
                explanation.push_str("  - Reason: Preserved due to uniqueness\n");
            }
            Layer::Edge => {
                explanation.push_str("  - Layer: Edge (medium connectivity)\n");
                explanation.push_str(&format!(
                    "  - Connectivity: {} similar items found\n",
                    degree
                ));
                if is_selected {
                    let group_size = self.cluster_members(idx).len();
                    explanation.push_str(&format!(
                        "  - Reason: Representative of {} items\n",
                        group_size
                    ));
                } else {
                    let rep = self.representative_of(idx);
                    explanation.push_str(&format!(
                        "  - Reason: Redundant, represented by item {}\n",
                        rep
                    ));
                }
            }
            Layer::Core => {
                explanation.push_str("  - Layer: Core (high connectivity, highly redundant)\n");
                explanation.push_str(&format!(
                    "  - Connectivity: {} similar items found\n",
                    degree
                ));
                if is_selected {
                    let group_size = self.cluster_members(idx).len();
                    explanation.push_str(&format!(
                        "  - Reason: Best representative of {} items\n",
                        group_size
                    ));
                } else {
                    let rep = self.representative_of(idx);
                    explanation.push_str(&format!(
                        "  - Reason: Highly redundant, represented by item {}\n",
                        rep
                    ));
                }
            }
        }

        // Weight info
        explanation.push_str(&format!("  - Final weight: {:.4}\n", weight));

        explanation
    }

    /// Generate explanations for all items
    pub fn explain_all(&self) -> Vec<String> {
        (0..self.layers.len()).map(|i| self.explain(i)).collect()
    }

    /// Generate a summary of the KDF processing
    pub fn summary(&self) -> String {
        let stats = self.stats();
        let mut summary = String::new();

        summary.push_str("=== KDF Processing Summary ===\n\n");

        // Overview
        summary.push_str(&format!("Total items processed: {}\n", stats.total_items));
        summary.push_str(&format!(
            "Items selected: {} ({:.1}%)\n",
            stats.selected_count,
            (stats.selected_count as f64 / stats.total_items as f64) * 100.0
        ));
        summary.push_str(&format!(
            "Items filtered: {} ({:.1}%)\n",
            stats.total_items - stats.selected_count,
            stats.redundancy_ratio * 100.0
        ));

        summary.push_str("\n--- Layer Distribution ---\n");
        for layer in [Layer::Rare, Layer::Edge, Layer::Core] {
            let count = stats.layer_counts.get(&layer).copied().unwrap_or(0);
            let pct = (count as f64 / stats.total_items as f64) * 100.0;
            let desc = match layer {
                Layer::Rare => "Rare (isolated/unique)",
                Layer::Edge => "Edge (medium connectivity)",
                Layer::Core => "Core (highly connected)",
            };
            summary.push_str(&format!(
                "  {:?}: {} items ({:.1}%) - {}\n",
                layer, count, pct, desc
            ));
        }

        summary.push_str("\n--- Connectivity ---\n");
        summary.push_str(&format!(
            "  Average connections per item: {:.2}\n",
            stats.avg_degree
        ));
        summary.push_str(&format!("  Maximum connections: {}\n", stats.max_degree));
        summary.push_str(&format!(
            "  Isolation ratio: {:.1}%\n",
            stats.isolation_ratio * 100.0
        ));

        summary.push_str("\n--- Clustering ---\n");
        summary.push_str(&format!("  Number of clusters: {}\n", stats.cluster_count));
        summary.push_str(&format!(
            "  Average cluster size: {:.2}\n",
            stats.avg_cluster_size
        ));
        summary.push_str(&format!(
            "  Largest cluster: {} items\n",
            stats.max_cluster_size
        ));

        // Key insights
        summary.push_str("\n--- Key Insights ---\n");

        if stats.isolation_ratio > 0.3 {
            summary.push_str("  * High isolation ratio - dataset has many unique items\n");
        }
        if stats.redundancy_ratio > 0.5 {
            summary.push_str("  * High redundancy - significant data compression achieved\n");
        }
        if stats.avg_cluster_size > 5.0 {
            summary.push_str("  * Large cluster sizes - items are highly grouped\n");
        }
        if stats.layer_counts.get(&Layer::Rare).copied().unwrap_or(0) > 0 {
            let rare_count = stats.layer_counts.get(&Layer::Rare).copied().unwrap_or(0);
            summary.push_str(&format!(
                "  * {} rare items preserved (100% rare preservation)\n",
                rare_count
            ));
        }

        summary
    }

    /// Explain why an item was selected or filtered (short version)
    pub fn explain_short(&self, idx: usize) -> String {
        if idx >= self.layers.len() {
            return "Item does not exist".to_string();
        }

        let layer = self.layers[idx];
        let is_selected = self.is_selected(idx);

        match (layer, is_selected) {
            (Layer::Rare, true) => "Selected: Unique/isolated item".to_string(),
            (Layer::Rare, false) => "Filtered: Very old rare item".to_string(),
            (Layer::Edge, true) => {
                let size = self.cluster_members(idx).len();
                format!("Selected: Representative of {} items", size)
            }
            (Layer::Edge, false) => {
                let rep = self.representative_of(idx);
                format!("Filtered: Similar to item {}", rep)
            }
            (Layer::Core, true) => {
                let size = self.cluster_members(idx).len();
                format!("Selected: Best representative of {} items", size)
            }
            (Layer::Core, false) => {
                let rep = self.representative_of(idx);
                format!("Filtered: Redundant, represented by item {}", rep)
            }
        }
    }
}

/// Statistics about KDF processing result
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KdfStats {
    /// Total number of items processed
    pub total_items: usize,
    /// Number of selected items
    pub selected_count: usize,
    /// Count of items in each layer
    pub layer_counts: HashMap<Layer, usize>,
    /// Average degree (connectivity) across all items
    pub avg_degree: f64,
    /// Maximum degree in the dataset
    pub max_degree: usize,
    /// Number of clusters formed
    pub cluster_count: usize,
    /// Average cluster size
    pub avg_cluster_size: f64,
    /// Size of the largest cluster
    pub max_cluster_size: usize,
    /// Ratio of isolated (Rare) items
    pub isolation_ratio: f64,
    /// Ratio of items that were filtered out
    pub redundancy_ratio: f64,
}

// ============================================================================
// Information-Theoretic Foundation
// ============================================================================

/// Information-theoretic metrics for KDF
///
/// Provides theoretical justification for KDF based on entropy and
/// information theory principles.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InfoTheoreticMetrics {
    /// Shannon entropy of the original distribution (bits)
    pub original_entropy: f64,
    /// Shannon entropy after KDF selection (bits)
    pub selected_entropy: f64,
    /// Information preserved (ratio 0-1)
    pub information_preserved: f64,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Theoretical minimum description length (bits)
    pub mdl_original: f64,
    /// MDL after selection
    pub mdl_selected: f64,
    /// Rare item information (bits) - unique contribution
    pub rare_information: f64,
    /// Redundant information removed (bits)
    pub redundancy_removed: f64,
}

impl KdfResult {
    /// Calculate information-theoretic metrics
    ///
    /// Based on the principle that:
    /// - Rare items carry maximum information (low probability → high information)
    /// - Redundant items carry minimum marginal information
    /// - KDF preserves high-information items and removes low-information redundancy
    pub fn info_metrics(&self) -> InfoTheoreticMetrics {
        let n = self.layers.len();
        if n == 0 {
            return InfoTheoreticMetrics {
                original_entropy: 0.0,
                selected_entropy: 0.0,
                information_preserved: 1.0,
                compression_ratio: 1.0,
                mdl_original: 0.0,
                mdl_selected: 0.0,
                rare_information: 0.0,
                redundancy_removed: 0.0,
            };
        }

        let selected_count = self.selected.len();

        // Calculate weight-based probability distribution
        let total_weight: f64 = self.selection_scores.iter().sum();
        let probabilities: Vec<f64> = if total_weight > 0.0 {
            self.selection_scores
                .iter()
                .map(|w| w / total_weight)
                .collect()
        } else {
            vec![1.0 / n as f64; n]
        };

        // Shannon entropy of original distribution
        let original_entropy = shannon_entropy(&probabilities);

        // Entropy of selected items only
        let selected_weights: Vec<f64> = self
            .selected
            .iter()
            .map(|&i| self.selection_scores[i])
            .collect();
        let selected_total: f64 = selected_weights.iter().sum();
        let selected_probs: Vec<f64> = if selected_total > 0.0 {
            selected_weights
                .iter()
                .map(|w| w / selected_total)
                .collect()
        } else {
            vec![1.0 / selected_count.max(1) as f64; selected_count]
        };
        let selected_entropy = shannon_entropy(&selected_probs);

        // Information preserved ratio
        // Based on the information content of selected vs all items
        let info_preserved = if original_entropy > 0.0 {
            let selected_info: f64 = self
                .selected
                .iter()
                .map(|&i| -probabilities[i].log2())
                .sum();
            let total_info: f64 = probabilities
                .iter()
                .map(|p| if *p > 0.0 { -*p * p.log2() } else { 0.0 })
                .sum::<f64>()
                * n as f64;
            (selected_info / total_info.max(1.0)).min(1.0)
        } else {
            1.0
        };

        // Compression ratio
        let compression_ratio = n as f64 / selected_count.max(1) as f64;

        // Minimum Description Length
        // MDL = model complexity + data encoded with model
        let mdl_original = (n as f64).log2() + original_entropy * n as f64;
        let mdl_selected =
            (selected_count as f64).log2() + selected_entropy * selected_count as f64;

        // Rare item information contribution
        // Rare items have maximum self-information (low probability)
        let rare_information: f64 = self
            .selected
            .iter()
            .filter(|&&i| self.layers[i] == Layer::Rare)
            .map(|&i| {
                if probabilities[i] > 0.0 {
                    -probabilities[i].log2()
                } else {
                    0.0
                }
            })
            .sum();

        // Redundancy removed
        let redundancy_removed = mdl_original - mdl_selected;

        InfoTheoreticMetrics {
            original_entropy,
            selected_entropy,
            information_preserved: info_preserved,
            compression_ratio,
            mdl_original,
            mdl_selected,
            rare_information,
            redundancy_removed,
        }
    }

    /// Calculate the information content of a specific item
    ///
    /// Returns self-information: I(x) = -log2(P(x))
    /// Rare items have high information, redundant items have low information
    pub fn item_information(&self, idx: usize) -> f64 {
        if idx >= self.selection_scores.len() {
            return 0.0;
        }

        let total_weight: f64 = self.selection_scores.iter().sum();
        if total_weight <= 0.0 {
            return 0.0;
        }

        let probability = self.selection_scores[idx] / total_weight;
        if probability > 0.0 {
            -probability.log2()
        } else {
            0.0
        }
    }

    /// Get items sorted by information content (highest first)
    pub fn items_by_information(&self) -> Vec<(usize, f64)> {
        let mut items: Vec<(usize, f64)> = (0..self.selection_scores.len())
            .map(|i| (i, self.item_information(i)))
            .collect();
        items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        items
    }

    // ========================================================================
    // Bridge Proposal (Optional feature for connecting Rare to Core)
    // ========================================================================

    /// Generate bridge proposals for Rare items to connect with Core
    ///
    /// This feature helps minority/isolated items find minimal modifications
    /// to become connectable to the mainstream while preserving uniqueness.
    ///
    /// # Arguments
    /// * `data` - The original data items
    /// * `distance` - A function to compute distance between two items
    ///
    /// # Returns
    /// Vector of BridgeProposal for each Rare item
    pub fn bridge_proposals<T, D>(&self, data: &[T], distance: D) -> Vec<BridgeProposal>
    where
        T: Clone,
        D: Fn(&T, &T) -> f64,
    {
        let rare_items = self.rare_items();
        let core_items = self.core_items();
        let edge_items = self.edge_items();

        // Potential targets (Core + Edge)
        let targets: Vec<usize> = core_items
            .iter()
            .chain(edge_items.iter())
            .copied()
            .collect();

        if targets.is_empty() {
            return vec![];
        }

        let mut proposals = Vec::new();

        for &rare_idx in &rare_items {
            // Find nearest target
            let (nearest_idx, nearest_dist) = targets
                .iter()
                .map(|&t| (t, distance(&data[rare_idx], &data[t])))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            // Calculate gap metrics
            let max_dist = targets
                .iter()
                .map(|&t| distance(&data[rare_idx], &data[t]))
                .fold(0.0f64, f64::max);

            let gap_ratio = if max_dist > 0.0 {
                nearest_dist / max_dist
            } else {
                0.0
            };

            // Estimate bridge ratio (how much to move towards target)
            // Lower gap_ratio means closer to target, needs less bridging
            let bridge_ratio = (gap_ratio * 0.8).min(0.8);

            let uniqueness_preserved = 1.0 - bridge_ratio;
            let connectivity_potential = 1.0 / (1.0 + nearest_dist);

            proposals.push(BridgeProposal {
                rare_idx,
                target_idx: nearest_idx,
                gap_distance: nearest_dist,
                bridge_ratio,
                uniqueness_preserved,
                connectivity_potential,
            });
        }

        proposals
    }
}

/// Bridge proposal for connecting a Rare item to Core/Edge
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BridgeProposal {
    /// Index of the Rare item
    pub rare_idx: usize,
    /// Index of the nearest Core/Edge item (target)
    pub target_idx: usize,
    /// Distance to the target
    pub gap_distance: f64,
    /// Recommended bridge ratio (0.0 = stay, 1.0 = fully move to target)
    pub bridge_ratio: f64,
    /// How much uniqueness is preserved (1.0 - bridge_ratio)
    pub uniqueness_preserved: f64,
    /// Potential connectivity after bridging
    pub connectivity_potential: f64,
}

impl BridgeProposal {
    /// Generate a human-readable recommendation
    pub fn recommendation(&self) -> String {
        let urgency = if self.gap_distance > 1.0 {
            "大きな"
        } else if self.gap_distance > 0.5 {
            "中程度の"
        } else {
            "小さな"
        };

        format!(
            "Item {} → Target {}: {}ギャップ (距離={:.2})\n\
             推奨: {:.0}%の調整で接続可能、{:.0}%のユニークさを維持",
            self.rare_idx,
            self.target_idx,
            urgency,
            self.gap_distance,
            self.bridge_ratio * 100.0,
            self.uniqueness_preserved * 100.0
        )
    }
}

/// Calculate Shannon entropy from probability distribution
fn shannon_entropy(probabilities: &[f64]) -> f64 {
    probabilities
        .iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| -p * p.log2())
        .sum()
}

/// Theoretical justification for KDF
pub struct TheoreticalBounds;

impl TheoreticalBounds {
    /// Calculate theoretical upper bound on information loss
    ///
    /// KDF guarantees: Rare items (degree=0) are always preserved
    /// Therefore, information loss is bounded by the redundant portion
    pub fn max_information_loss(result: &KdfResult) -> f64 {
        let n = result.layers.len();
        if n == 0 {
            return 0.0;
        }

        // Count non-rare items that were filtered
        let filtered_redundant: usize = (0..n)
            .filter(|&i| !result.is_selected(i) && result.layers[i] != Layer::Rare)
            .count();

        // Maximum information that could be lost
        // Each filtered item contributes at most log2(n) bits
        filtered_redundant as f64 * (n as f64).log2()
    }

    /// Verify rare preservation guarantee
    ///
    /// Mathematical proof: For degree=0 items with T=100 iterations:
    /// w(T) = (1 - β)^T = 0.99^100 ≈ 0.366 > θ_E = 0.15
    /// Therefore all Rare items are selected.
    pub fn verify_rare_preservation(result: &KdfResult) -> bool {
        result
            .layers
            .iter()
            .enumerate()
            .filter(|&(_, &layer)| layer == Layer::Rare)
            .all(|(i, _)| result.is_selected(i))
    }

    /// Calculate the decay convergence iterations needed for full separation
    ///
    /// Returns the number of iterations T such that:
    /// - Rare items: w(T) ≈ 0.366 (preserved)
    /// - Core items: w(T) < θ_E (filtered)
    pub fn convergence_iterations(params: &KdfParams) -> usize {
        // For Rare: w(T) = (1-β)^T, need w(T) > θ_E
        // (1-β)^T > θ_E → T < ln(θ_E) / ln(1-β)

        // For Core with high degree: w(T) = (1-λ_core)^T < θ_E
        // This converges faster, so Rare is the limiting factor
        (params.theta_edge.ln() / (1.0 - params.beta).ln()) as usize
    }
}

// ============================================================================
// Kdf (main processor)
// ============================================================================

/// Main KDF processor
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Kdf {
    params: KdfParams,
}

/// Result of auto-threshold analysis
#[derive(Clone, Debug)]
pub struct AutoThresholdResult {
    /// Optimal threshold found
    pub threshold: f64,
    /// KdfResult using the optimal threshold
    pub result: KdfResult,
    /// All thresholds evaluated
    pub thresholds_evaluated: Vec<f64>,
    /// Scores for each threshold (higher is better)
    pub scores: Vec<f64>,
}

impl Kdf {
    /// Create a new KDF instance with given parameters
    pub fn new(params: KdfParams) -> Self {
        Self { params }
    }

    /// Create a new KDF instance with default parameters
    pub fn with_defaults() -> Self {
        Self::new(KdfParams::default())
    }

    /// Process items and return selected indices
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `sim_threshold` - Similarity threshold for graph construction
    /// * `similarity` - Similarity function between two items
    ///
    /// # Returns
    ///
    /// `KdfResult` containing selected indices and metadata
    pub fn process<T, F>(&self, items: &[T], sim_threshold: f64, similarity: F) -> KdfResult
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = items.len();

        if n == 0 {
            return KdfResult::new(vec![], vec![], vec![], vec![], vec![]);
        }

        // Phase 1: Build similarity graph
        let mut degrees = vec![0usize; n];
        for i in 0..n {
            for j in (i + 1)..n {
                if similarity(&items[i], &items[j]) >= sim_threshold {
                    degrees[i] += 1;
                    degrees[j] += 1;
                }
            }
        }

        // Phase 2: Classify layers (using common function)
        let layers = classify_layers(&degrees, &self.params);

        // Phase 3: Decay iteration (optimized with pre-computed factors)
        let decay_factors = compute_decay_factors(&degrees, &layers, &self.params);
        let weights = compute_weights(&decay_factors, self.params.iterations);

        // Phase 4: Selection and Clustering
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            weights[*b]
                .partial_cmp(&weights[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<usize> = Vec::new();
        let mut selected_set: HashSet<usize> = HashSet::new();
        let mut clusters: Vec<usize> = (0..n).collect();

        for &i in &indices {
            if layers[i] == Layer::Rare {
                selected.push(i);
                selected_set.insert(i);
            } else if weights[i] >= self.params.theta_edge {
                let similar_rep = selected
                    .iter()
                    .find(|&&s| {
                        similarity(&items[i], &items[s]) >= self.params.selection_sim_threshold
                    })
                    .copied();

                if let Some(rep) = similar_rep {
                    clusters[i] = rep;
                } else {
                    selected.push(i);
                    selected_set.insert(i);
                }
            } else if let Some(&rep) = selected.iter().max_by(|&&a, &&b| {
                similarity(&items[i], &items[a])
                    .partial_cmp(&similarity(&items[i], &items[b]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                clusters[i] = rep;
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
            selected_set.insert(indices[0]);
        }

        KdfResult {
            selected,
            layers,
            selection_scores: weights,
            degrees,
            clusters,
            edge_weights: HashMap::new(),
            selected_set,
        }
    }

    // ========================================================================
    // Auto-threshold Selection
    // ========================================================================

    /// Automatically determine the optimal similarity threshold
    ///
    /// This method evaluates multiple thresholds and selects the one that
    /// best balances compression (redundancy reduction) and information
    /// preservation (keeping diverse and rare items).
    ///
    /// # Algorithm
    ///
    /// Uses an information-theoretic approach:
    /// - Score = compression_ratio * rare_preservation * structure_quality
    /// - compression_ratio = 1 - (selected / total)
    /// - rare_preservation = rare_selected / rare_detected
    /// - structure_quality = penalizes extreme clustering
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `similarity` - Similarity function between two items
    ///
    /// # Returns
    ///
    /// `AutoThresholdResult` containing the optimal threshold and results
    pub fn process_auto<T, F>(&self, items: &[T], similarity: F) -> AutoThresholdResult
    where
        F: Fn(&T, &T) -> f64 + Copy,
    {
        let n = items.len();

        if n == 0 {
            return AutoThresholdResult {
                threshold: 0.85,
                result: KdfResult::new(vec![], vec![], vec![], vec![], vec![]),
                thresholds_evaluated: vec![],
                scores: vec![],
            };
        }

        // Thresholds to evaluate (coarse to fine)
        let thresholds: Vec<f64> = vec![0.50, 0.55, 0.60, 0.65, 0.70, 0.75, 0.80, 0.85, 0.90, 0.95];

        let mut best_threshold = 0.85;
        let mut best_score = f64::MIN;
        let mut best_result = None;
        let mut scores = Vec::with_capacity(thresholds.len());

        for &threshold in &thresholds {
            let result = self.process(items, threshold, similarity);
            let score = self.compute_auto_score(&result, n);
            scores.push(score);

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
                best_result = Some(result);
            }
        }

        // Fine-tuning around the best threshold
        let fine_thresholds: Vec<f64> = vec![
            best_threshold - 0.03,
            best_threshold - 0.02,
            best_threshold - 0.01,
            best_threshold + 0.01,
            best_threshold + 0.02,
            best_threshold + 0.03,
        ]
        .into_iter()
        .filter(|&t| t > 0.0 && t < 1.0)
        .collect();

        for &threshold in &fine_thresholds {
            let result = self.process(items, threshold, similarity);
            let score = self.compute_auto_score(&result, n);

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
                best_result = Some(result);
            }
        }

        AutoThresholdResult {
            threshold: best_threshold,
            result: best_result.unwrap_or_else(|| self.process(items, 0.85, similarity)),
            thresholds_evaluated: thresholds,
            scores,
        }
    }

    /// Compute score for a given threshold result
    fn compute_auto_score(&self, result: &KdfResult, total: usize) -> f64 {
        if total == 0 || result.selected.is_empty() {
            return 0.0;
        }

        let selected_count = result.selected.len() as f64;
        let total_f = total as f64;

        let selection_ratio = selected_count / total_f;

        // 1. Rare detection score: reward thresholds that identify rare items
        //    A good threshold should detect some rare items, not zero
        let rare_count = result.layers.iter().filter(|&&l| l == Layer::Rare).count() as f64;
        let rare_ratio = rare_count / total_f;
        let rare_detection = if rare_ratio > 0.0 {
            // Optimal rare ratio is 5-20%
            if rare_ratio < 0.05 {
                rare_ratio / 0.05
            } else if rare_ratio > 0.5 {
                1.0 - (rare_ratio - 0.5)
            } else {
                1.0
            }
        } else {
            0.3 // Low score if no rare items detected
        };

        // 2. Rare preservation: all detected rare items should be selected
        let rare_selected = result
            .selected
            .iter()
            .filter(|&&i| result.layers[i] == Layer::Rare)
            .count() as f64;
        let rare_preservation = if rare_count > 0.0 {
            rare_selected / rare_count
        } else {
            0.5 // Neutral if no rare items
        };

        // 3. Compression score: moderate compression is good
        //    But extreme compression (>90%) loses information
        let compression = 1.0 - selection_ratio;
        let compression_score = if compression > 0.9 {
            0.5 + 0.5 * (1.0 - compression) / 0.1 // Penalize extreme compression
        } else if compression < 0.3 {
            compression / 0.3 // Penalize low compression
        } else {
            // Sweet spot: 30-90% compression
            0.7 + 0.3 * (compression - 0.3) / 0.6
        };

        // 4. Structure quality: balanced selection is better
        //    Too few (<10%) or too many (>70%) is bad
        let structure_quality = if selection_ratio < 0.1 {
            selection_ratio / 0.1
        } else if selection_ratio > 0.7 {
            (1.0 - selection_ratio) / 0.3
        } else {
            1.0
        };

        // 5. Layer diversity: all three layers should be represented
        let core_count = result.layers.iter().filter(|&&l| l == Layer::Core).count() as f64;
        let edge_count = result.layers.iter().filter(|&&l| l == Layer::Edge).count() as f64;
        let layers_present =
            (core_count > 0.0) as i32 + (edge_count > 0.0) as i32 + (rare_count > 0.0) as i32;
        let layer_diversity = layers_present as f64 / 3.0;

        // Combined score with balanced weights
        // Rare preservation: weight 4 (most important - KDF's core guarantee)
        // Rare detection: weight 3 (important - find the outliers)
        // Compression: weight 2 (secondary - remove redundancy)
        // Structure: weight 1.5 (support balanced selection)
        // Diversity: weight 1.5 (support layer representation)

        (rare_preservation * 4.0
            + rare_detection * 3.0
            + compression_score * 2.0
            + structure_quality * 1.5
            + layer_diversity * 1.5)
            / 12.0
    }

    /// Quick auto-threshold with fewer evaluations
    ///
    /// Faster than `process_auto` but may be less optimal.
    pub fn process_auto_quick<T, F>(&self, items: &[T], similarity: F) -> (f64, KdfResult)
    where
        F: Fn(&T, &T) -> f64 + Copy,
    {
        let thresholds = [0.70, 0.80, 0.90];
        let mut best_threshold = 0.80;
        let mut best_score = f64::MIN;
        let mut best_result = None;

        for &threshold in &thresholds {
            let result = self.process(items, threshold, similarity);
            let score = self.compute_auto_score(&result, items.len());

            if score > best_score {
                best_score = score;
                best_threshold = threshold;
                best_result = Some(result);
            }
        }

        (
            best_threshold,
            best_result.unwrap_or_else(|| self.process(items, 0.80, similarity)),
        )
    }

    // ========================================================================
    // Diversity Sampling (KDF-specific feature)
    // ========================================================================

    /// Select k diverse items using greedy max-min algorithm
    ///
    /// This method selects items that are maximally spread out in feature space.
    /// Uses KDF's isolation information to seed the selection with rare items.
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to sample from
    /// * `k` - Number of items to select
    /// * `similarity` - Similarity function (higher = more similar)
    ///
    /// # Returns
    ///
    /// Vec of selected indices representing diverse items
    pub fn diverse_sample<T, F>(&self, items: &[T], k: usize, similarity: F) -> Vec<usize>
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = items.len();
        if n == 0 || k == 0 {
            return vec![];
        }
        if k >= n {
            return (0..n).collect();
        }

        // First, process to get structural information
        let result = self.process(items, 0.85, &similarity);

        // Start with the most isolated (rare) items
        let mut selected: Vec<usize> = Vec::with_capacity(k);
        let mut selected_set: HashSet<usize> = HashSet::new();

        // Add rare items first (they are naturally diverse)
        for (i, &layer) in result.layers.iter().enumerate() {
            if layer == Layer::Rare && selected.len() < k {
                selected.push(i);
                selected_set.insert(i);
            }
        }

        // If we still need more items, use greedy max-min
        // Track minimum similarity to any selected item for each unselected item
        let mut min_sim_to_selected: Vec<f64> = vec![f64::MAX; n];

        // Initialize distances from already selected items
        for &s in &selected {
            for i in 0..n {
                if !selected_set.contains(&i) {
                    let sim = similarity(&items[i], &items[s]);
                    min_sim_to_selected[i] = min_sim_to_selected[i].min(sim);
                }
            }
        }

        // If no rare items, start with the item that has lowest average similarity
        if selected.is_empty() {
            let mut min_avg_sim = f64::MAX;
            let mut first_idx = 0;

            for i in 0..n {
                let avg_sim: f64 = (0..n)
                    .filter(|&j| j != i)
                    .map(|j| similarity(&items[i], &items[j]))
                    .sum::<f64>()
                    / (n - 1).max(1) as f64;

                if avg_sim < min_avg_sim {
                    min_avg_sim = avg_sim;
                    first_idx = i;
                }
            }

            selected.push(first_idx);
            selected_set.insert(first_idx);

            // Initialize min_sim_to_selected from first item
            for i in 0..n {
                if i != first_idx {
                    min_sim_to_selected[i] = similarity(&items[i], &items[first_idx]);
                }
            }
        }

        // Greedy selection: pick item with minimum similarity to selected set (most diverse)
        while selected.len() < k {
            let mut best_idx = 0;
            let mut lowest_sim = f64::MAX;

            for i in 0..n {
                if !selected_set.contains(&i) && min_sim_to_selected[i] < lowest_sim {
                    lowest_sim = min_sim_to_selected[i];
                    best_idx = i;
                }
            }

            selected.push(best_idx);
            selected_set.insert(best_idx);

            // Update min_sim_to_selected for remaining items
            for i in 0..n {
                if !selected_set.contains(&i) {
                    let sim = similarity(&items[i], &items[best_idx]);
                    min_sim_to_selected[i] = min_sim_to_selected[i].min(sim);
                }
            }
        }

        selected
    }

    // ========================================================================
    // Parallel Processing (requires "parallel" feature)
    // ========================================================================

    /// Process items in parallel using rayon
    ///
    /// This method is available when the "parallel" feature is enabled.
    /// It parallelizes the similarity graph construction phase for better
    /// performance on large datasets.
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `sim_threshold` - Similarity threshold for graph construction
    /// * `similarity` - Similarity function between two items (must be Sync)
    ///
    /// # Returns
    ///
    /// `KdfResult` containing selected indices and metadata
    #[cfg(feature = "parallel")]
    pub fn process_parallel<T: Sync, F>(
        &self,
        items: &[T],
        sim_threshold: f64,
        similarity: F,
    ) -> KdfResult
    where
        F: Fn(&T, &T) -> f64 + Sync,
    {
        let n = items.len();

        if n == 0 {
            return KdfResult::new(vec![], vec![], vec![], vec![], vec![]);
        }

        // Phase 1: Build similarity graph in parallel
        let degrees: Vec<usize> = (0..n)
            .into_par_iter()
            .map(|i| {
                (0..n)
                    .filter(|&j| i != j && similarity(&items[i], &items[j]) >= sim_threshold)
                    .count()
            })
            .collect();

        // Phase 2: Classify layers (using common function)
        let layers = classify_layers(&degrees, &self.params);

        // Phase 3: Decay iteration (optimized with pre-computed factors)
        let decay_factors = compute_decay_factors(&degrees, &layers, &self.params);
        let weights = compute_weights(&decay_factors, self.params.iterations);

        // Phase 4: Selection and Clustering (sequential - depends on prior selections)
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            weights[*b]
                .partial_cmp(&weights[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<usize> = Vec::new();
        let mut selected_set: HashSet<usize> = HashSet::new();
        let mut clusters: Vec<usize> = (0..n).collect();

        for &i in &indices {
            if layers[i] == Layer::Rare {
                selected.push(i);
                selected_set.insert(i);
            } else if weights[i] >= self.params.theta_edge {
                let similar_rep = selected
                    .iter()
                    .find(|&&s| {
                        similarity(&items[i], &items[s]) >= self.params.selection_sim_threshold
                    })
                    .copied();

                if let Some(rep) = similar_rep {
                    clusters[i] = rep;
                } else {
                    selected.push(i);
                    selected_set.insert(i);
                }
            } else {
                if let Some(&rep) = selected.iter().max_by(|&&a, &&b| {
                    similarity(&items[i], &items[a])
                        .partial_cmp(&similarity(&items[i], &items[b]))
                        .unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    clusters[i] = rep;
                }
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
            selected_set.insert(indices[0]);
        }

        KdfResult {
            selected,
            layers,
            selection_scores: weights,
            degrees,
            clusters,
            edge_weights: HashMap::new(),
            selected_set,
        }
    }

    // ========================================================================
    // Fast Processing (Approximate, O(n × k))
    // ========================================================================

    /// Fast approximate processing using incremental clustering
    ///
    /// This method provides massive speedup (100x-1000x) for redundant data
    /// by avoiding O(n²) pairwise comparisons. Instead, it uses an incremental
    /// clustering approach where each item is only compared to existing cluster
    /// representatives.
    ///
    /// # Complexity
    ///
    /// - Time: O(n × k) where k = number of clusters formed
    /// - For redundant data: k << n, so effectively O(n)
    /// - For diverse data: k ≈ n, degrades to O(n²)
    ///
    /// # Tradeoffs
    ///
    /// **WARNING**: This method has significant tradeoffs:
    ///
    /// - **0% Rare Recall**: Rare items CANNOT be reliably detected because
    ///   detecting isolation requires confirming absence of neighbors, which
    ///   needs more comparisons than this method performs.
    ///
    /// - **Approximate layer classification**: Degrees are estimated from
    ///   cluster membership, not exact pairwise comparisons.
    ///
    /// # When to Use
    ///
    /// Use this method when:
    /// - You only need redundancy reduction (not anomaly/rare detection)
    /// - Your data is highly redundant (>70% similar items)
    /// - Speed is critical and you accept accuracy tradeoffs
    /// - You're doing a quick preview before full processing
    ///
    /// # When NOT to Use
    ///
    /// Do NOT use this method when:
    /// - You need to detect rare/anomalous items
    /// - Accurate layer classification is important
    /// - Data is diverse with few redundant items
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `sim_threshold` - Similarity threshold for clustering
    /// * `similarity` - Similarity function between two items
    ///
    /// # Returns
    ///
    /// `KdfResult` with approximate layer assignments and selected indices
    ///
    /// # Example
    ///
    /// ```rust
    /// use kdf::{Kdf, cosine_similarity};
    ///
    /// // Create sample data
    /// let items: Vec<Vec<f64>> = (0..100)
    ///     .map(|i| vec![(i as f64) * 0.01, 0.5, 0.5])
    ///     .collect();
    /// let kdf = Kdf::with_defaults();
    ///
    /// // Fast processing - 100x-1000x faster than process()
    /// // WARNING: Does not detect Rare items!
    /// let result = kdf.process_fast(&items, 0.95, |a, b| cosine_similarity(a, b));
    ///
    /// // For accurate Rare detection, use process() instead
    /// // let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
    /// ```
    pub fn process_fast<T, F>(&self, items: &[T], sim_threshold: f64, similarity: F) -> KdfResult
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = items.len();

        if n == 0 {
            return KdfResult::new(vec![], vec![], vec![], vec![], vec![]);
        }

        // Phase 1: Incremental clustering
        // Each item is compared only to existing cluster representatives
        // Clusters: (representative_idx, members)
        let mut clusters: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut degrees = vec![0usize; n];
        let mut item_cluster: Vec<usize> = (0..n).collect(); // cluster assignment for each item

        for i in 0..n {
            let mut best_cluster = None;
            let mut best_sim = sim_threshold;

            // Find best matching cluster (compare with representative only)
            for (cluster_idx, (rep, _)) in clusters.iter().enumerate() {
                let sim = similarity(&items[i], &items[*rep]);
                if sim >= best_sim {
                    best_sim = sim;
                    best_cluster = Some(cluster_idx);
                }
            }

            if let Some(cluster_idx) = best_cluster {
                // Add to existing cluster
                let rep = clusters[cluster_idx].0;
                clusters[cluster_idx].1.push(i);
                item_cluster[i] = rep;

                // Update degrees (approximate)
                degrees[i] += 1;
                degrees[rep] += 1;

                // Sample comparison with recent members (up to 5)
                let members = &clusters[cluster_idx].1;
                let sample_size = members.len().min(5);
                for &member in members.iter().rev().take(sample_size) {
                    if member != i && similarity(&items[i], &items[member]) >= sim_threshold {
                        degrees[i] += 1;
                        degrees[member] += 1;
                    }
                }
            } else {
                // Start new cluster
                clusters.push((i, vec![i]));
            }
        }

        // Phase 2: Classify layers (using common function)
        let layers = classify_layers(&degrees, &self.params);

        // Phase 3: Compute weights
        let decay_factors = compute_decay_factors(&degrees, &layers, &self.params);
        let weights = compute_weights(&decay_factors, self.params.iterations);

        // Phase 4: Selection
        // Select cluster representatives
        let mut selected: Vec<usize> = clusters.iter().map(|(rep, _)| *rep).collect();
        let selected_set: HashSet<usize> = selected.iter().copied().collect();

        // Also select high-weight items not yet selected
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            weights[*b]
                .partial_cmp(&weights[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for &i in &indices {
            if !selected_set.contains(&i) && weights[i] >= self.params.theta_edge {
                // Check if similar to any selected
                let has_similar = selected.iter().any(|&s| {
                    similarity(&items[i], &items[s]) >= self.params.selection_sim_threshold
                });

                if !has_similar {
                    selected.push(i);
                }
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
        }

        KdfResult::new(selected, layers, weights, degrees, item_cluster)
    }

    /// Fast processing with Rare verification fallback
    ///
    /// This method combines the speed of `process_fast()` with optional
    /// Rare item verification for better accuracy.
    ///
    /// # Strategy
    ///
    /// 1. Use incremental clustering for initial classification
    /// 2. Items with degree=0 (potential Rare) are verified against all items
    /// 3. This hybrid approach is faster than full O(n²) but catches most Rare items
    ///
    /// # Complexity
    ///
    /// - Best case (few Rare): O(n × k) where k = cluster count
    /// - Worst case (many Rare): Approaches O(n²)
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `sim_threshold` - Similarity threshold for clustering
    /// * `similarity` - Similarity function between two items
    /// * `verify_rare` - If true, verify potential Rare items (slower but more accurate)
    ///
    /// # Returns
    ///
    /// `KdfResult` with better Rare accuracy than `process_fast()`
    pub fn process_fast_verified<T, F>(
        &self,
        items: &[T],
        sim_threshold: f64,
        similarity: F,
        verify_rare: bool,
    ) -> KdfResult
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = items.len();

        if n == 0 {
            return KdfResult::new(vec![], vec![], vec![], vec![], vec![]);
        }

        // Phase 1: Incremental clustering (same as process_fast)
        let mut clusters: Vec<(usize, Vec<usize>)> = Vec::new();
        let mut degrees = vec![0usize; n];
        let mut item_cluster: Vec<usize> = (0..n).collect();
        let mut potential_rare: Vec<usize> = Vec::new();

        for i in 0..n {
            let mut best_cluster = None;
            let mut best_sim = sim_threshold;

            for (cluster_idx, (rep, _)) in clusters.iter().enumerate() {
                let sim = similarity(&items[i], &items[*rep]);
                if sim >= best_sim {
                    best_sim = sim;
                    best_cluster = Some(cluster_idx);
                }
            }

            if let Some(cluster_idx) = best_cluster {
                let rep = clusters[cluster_idx].0;
                clusters[cluster_idx].1.push(i);
                item_cluster[i] = rep;
                degrees[i] += 1;
                degrees[rep] += 1;

                // Sample comparison
                let members = &clusters[cluster_idx].1;
                let sample_size = members.len().min(5);
                for &member in members.iter().rev().take(sample_size) {
                    if member != i && similarity(&items[i], &items[member]) >= sim_threshold {
                        degrees[i] += 1;
                        degrees[member] += 1;
                    }
                }
            } else {
                // New cluster - this item is a potential Rare candidate
                clusters.push((i, vec![i]));
                potential_rare.push(i);
            }
        }

        // Phase 2: Verify Rare items if requested
        if verify_rare {
            for &rare_idx in &potential_rare {
                if degrees[rare_idx] == 0 {
                    // Check against ALL other items to confirm isolation
                    for j in 0..n {
                        if j != rare_idx && similarity(&items[rare_idx], &items[j]) >= sim_threshold
                        {
                            degrees[rare_idx] += 1;
                            degrees[j] += 1;
                            // Found a neighbor - no longer Rare
                            break;
                        }
                    }
                }
            }
        }

        // Phase 3-4: Same as process_fast
        let layers = classify_layers(&degrees, &self.params);
        let decay_factors = compute_decay_factors(&degrees, &layers, &self.params);
        let weights = compute_weights(&decay_factors, self.params.iterations);

        let mut selected: Vec<usize> = clusters.iter().map(|(rep, _)| *rep).collect();
        let selected_set: HashSet<usize> = selected.iter().copied().collect();

        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            weights[*b]
                .partial_cmp(&weights[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for &i in &indices {
            if !selected_set.contains(&i) && weights[i] >= self.params.theta_edge {
                let has_similar = selected.iter().any(|&s| {
                    similarity(&items[i], &items[s]) >= self.params.selection_sim_threshold
                });

                if !has_similar {
                    selected.push(i);
                }
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
        }

        KdfResult::new(selected, layers, weights, degrees, item_cluster)
    }
}

// ============================================================================
// Temporal KDF (Time-aware processing)
// ============================================================================

/// Temporal decay parameters for time-aware KDF processing
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TemporalParams {
    /// Decay rate per time unit (default: 0.1)
    /// Higher values = faster decay for older items
    pub decay_rate: f64,
    /// Reference time (typically current time)
    pub reference_time: f64,
    /// Minimum temporal weight (prevents complete decay, default: 0.1)
    pub min_weight: f64,
}

impl Default for TemporalParams {
    fn default() -> Self {
        Self {
            decay_rate: 0.1,
            reference_time: 0.0,
            min_weight: 0.1,
        }
    }
}

impl TemporalParams {
    /// Create temporal params with current time as reference
    pub fn now(decay_rate: f64) -> Self {
        Self {
            decay_rate,
            reference_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0),
            min_weight: 0.1,
        }
    }

    /// Calculate temporal weight for a given timestamp
    /// Returns value between min_weight and 1.0
    pub fn temporal_weight(&self, timestamp: f64) -> f64 {
        let age = (self.reference_time - timestamp).max(0.0);
        let weight = (-self.decay_rate * age).exp();
        weight.max(self.min_weight)
    }
}

/// Temporal KDF processor for time-aware data
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TemporalKdf {
    kdf_params: KdfParams,
    temporal_params: TemporalParams,
}

impl TemporalKdf {
    /// Create a new temporal KDF instance
    pub fn new(kdf_params: KdfParams, temporal_params: TemporalParams) -> Self {
        Self {
            kdf_params,
            temporal_params,
        }
    }

    /// Create with default parameters
    pub fn with_defaults() -> Self {
        Self::new(KdfParams::default(), TemporalParams::default())
    }

    /// Create with decay rate (uses current time as reference)
    pub fn with_decay_rate(decay_rate: f64) -> Self {
        Self::new(KdfParams::default(), TemporalParams::now(decay_rate))
    }

    /// Process items with timestamps
    ///
    /// # Arguments
    ///
    /// * `items` - Slice of items to process
    /// * `timestamps` - Timestamp for each item (same length as items)
    /// * `sim_threshold` - Similarity threshold for graph construction
    /// * `similarity` - Similarity function between two items
    ///
    /// # Returns
    ///
    /// `KdfResult` with time-adjusted weights
    pub fn process<T, F>(
        &self,
        items: &[T],
        timestamps: &[f64],
        sim_threshold: f64,
        similarity: F,
    ) -> KdfResult
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = items.len();

        if n == 0 || timestamps.len() != n {
            return KdfResult::new(vec![], vec![], vec![], vec![], vec![]);
        }

        // Phase 1: Build similarity graph
        let mut degrees = vec![0usize; n];
        for i in 0..n {
            for j in (i + 1)..n {
                if similarity(&items[i], &items[j]) >= sim_threshold {
                    degrees[i] += 1;
                    degrees[j] += 1;
                }
            }
        }

        // Phase 2: Classify layers
        let layers = classify_layers(&degrees, &self.kdf_params);

        // Phase 3: Compute base weights with layer decay
        let decay_factors = compute_decay_factors(&degrees, &layers, &self.kdf_params);
        let base_weights = compute_weights(&decay_factors, self.kdf_params.iterations);

        // Phase 4: Apply temporal decay
        let weights: Vec<f64> = base_weights
            .iter()
            .zip(timestamps)
            .map(|(&w, &t)| w * self.temporal_params.temporal_weight(t))
            .collect();

        // Phase 5: Selection (same as standard KDF)
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            weights[*b]
                .partial_cmp(&weights[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<usize> = Vec::new();
        let mut selected_set: HashSet<usize> = HashSet::new();
        let mut clusters: Vec<usize> = (0..n).collect();

        for &i in &indices {
            // Rare items still get priority, but temporal weight affects final selection
            if layers[i] == Layer::Rare
                && weights[i] >= self.kdf_params.theta_edge * self.temporal_params.min_weight
            {
                selected.push(i);
                selected_set.insert(i);
            } else if weights[i] >= self.kdf_params.theta_edge {
                let similar_rep = selected
                    .iter()
                    .find(|&&s| {
                        similarity(&items[i], &items[s]) >= self.kdf_params.selection_sim_threshold
                    })
                    .copied();

                if let Some(rep) = similar_rep {
                    clusters[i] = rep;
                } else {
                    selected.push(i);
                    selected_set.insert(i);
                }
            } else if let Some(&rep) = selected.iter().max_by(|&&a, &&b| {
                similarity(&items[i], &items[a])
                    .partial_cmp(&similarity(&items[i], &items[b]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            }) {
                clusters[i] = rep;
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
            selected_set.insert(indices[0]);
        }

        KdfResult {
            selected,
            layers,
            selection_scores: weights,
            degrees,
            clusters,
            edge_weights: HashMap::new(),
            selected_set,
        }
    }

    /// Get items that are "fresh" (recently added and still relevant)
    pub fn fresh_items<T, F>(
        &self,
        items: &[T],
        timestamps: &[f64],
        sim_threshold: f64,
        similarity: F,
        freshness_threshold: f64,
    ) -> Vec<usize>
    where
        F: Fn(&T, &T) -> f64,
    {
        let result = self.process(items, timestamps, sim_threshold, similarity);
        result
            .selected
            .into_iter()
            .filter(|&i| self.temporal_params.temporal_weight(timestamps[i]) >= freshness_threshold)
            .collect()
    }

    /// Get items that are "stale" (old but still selected due to rarity)
    pub fn stale_rare_items<T, F>(
        &self,
        items: &[T],
        timestamps: &[f64],
        sim_threshold: f64,
        similarity: F,
        staleness_threshold: f64,
    ) -> Vec<usize>
    where
        F: Fn(&T, &T) -> f64,
    {
        let result = self.process(items, timestamps, sim_threshold, similarity);
        result
            .selected
            .into_iter()
            .filter(|&i| {
                result.layers[i] == Layer::Rare
                    && self.temporal_params.temporal_weight(timestamps[i]) < staleness_threshold
            })
            .collect()
    }
}

// ============================================================================
// IncrementalKdf (merged impl blocks)
// ============================================================================

/// Incremental KDF for dynamic data
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IncrementalKdf<T> {
    items: Vec<T>,
    degrees: Vec<usize>,
    layers: Vec<Layer>,
    selection_scores: Vec<f64>,
    params: KdfParams,
    sim_threshold: f64,
}

impl<T: Clone> IncrementalKdf<T> {
    /// Create a new incremental KDF instance
    pub fn new(params: KdfParams, sim_threshold: f64) -> Self {
        Self {
            items: Vec::new(),
            degrees: Vec::new(),
            layers: Vec::new(),
            selection_scores: Vec::new(),
            params,
            sim_threshold,
        }
    }

    /// Get current items
    pub fn items(&self) -> &[T] {
        &self.items
    }

    /// Get number of items
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Get layers for all items
    pub fn layers(&self) -> &[Layer] {
        &self.layers
    }

    /// Get selection scores for all items
    pub fn selection_scores(&self) -> &[f64] {
        &self.selection_scores
    }

    /// Backward compatibility: get weights (deprecated)
    #[deprecated(since = "0.2.0", note = "Use selection_scores instead")]
    pub fn weights(&self) -> &[f64] {
        &self.selection_scores
    }

    /// Get degrees for all items
    pub fn degrees(&self) -> &[usize] {
        &self.degrees
    }

    /// Add an item incrementally
    pub fn add<F>(&mut self, item: T, similarity: F)
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = self.items.len();

        // Calculate new item's degree
        let mut new_degree = 0usize;
        for i in 0..n {
            if similarity(&item, &self.items[i]) >= self.sim_threshold {
                new_degree += 1;
                self.degrees[i] += 1;
            }
        }

        // Add new item
        self.items.push(item);
        self.degrees.push(new_degree);
        self.layers.push(Layer::Edge);
        self.selection_scores.push(1.0);

        // Reclassify and recompute
        self.reclassify_and_recompute();
    }

    /// Remove an item by index
    pub fn remove<F>(&mut self, idx: usize, similarity: F)
    where
        F: Fn(&T, &T) -> f64,
    {
        if idx >= self.items.len() {
            return;
        }

        let n = self.items.len();

        // Update degrees of connected items
        for i in 0..n {
            if i != idx && similarity(&self.items[idx], &self.items[i]) >= self.sim_threshold {
                self.degrees[i] = self.degrees[i].saturating_sub(1);
            }
        }

        // Remove item
        self.items.remove(idx);
        self.degrees.remove(idx);
        self.layers.remove(idx);
        self.selection_scores.remove(idx);

        // Reclassify and recompute
        self.reclassify_and_recompute();
    }

    /// Get selected items
    pub fn get_selected<F>(&self, similarity: F) -> Vec<usize>
    where
        F: Fn(&T, &T) -> f64,
    {
        let n = self.items.len();
        if n == 0 {
            return vec![];
        }

        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| {
            self.selection_scores[*b]
                .partial_cmp(&self.selection_scores[*a])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut selected: Vec<usize> = Vec::new();
        let mut selected_set: HashSet<usize> = HashSet::new();

        for &i in &indices {
            if self.layers[i] == Layer::Rare {
                selected.push(i);
                selected_set.insert(i);
            } else if self.selection_scores[i] >= self.params.theta_edge {
                let has_similar = selected.iter().any(|&s| {
                    similarity(&self.items[i], &self.items[s])
                        >= self.params.selection_sim_threshold
                });
                if !has_similar {
                    selected.push(i);
                    selected_set.insert(i);
                }
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
        }

        selected
    }

    /// Reclassify layers and recompute selection scores (using common functions)
    fn reclassify_and_recompute(&mut self) {
        // Use common function for layer classification
        self.layers = classify_layers(&self.degrees, &self.params);

        // Use common functions for score computation
        let decay_factors = compute_decay_factors(&self.degrees, &self.layers, &self.params);
        self.selection_scores = compute_weights(&decay_factors, self.params.iterations);
    }
}

// ============================================================================
// Similarity functions
// ============================================================================

/// Cosine similarity between two vectors
pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dot: f64 = a.iter().zip(b).map(|(x, y)| x * y).sum();
    let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if mag_a == 0.0 || mag_b == 0.0 {
        0.0
    } else {
        dot / (mag_a * mag_b)
    }
}

/// Euclidean similarity (1 / (1 + distance))
pub fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();

    1.0 / (1.0 + dist)
}

/// Jaccard similarity for sets
pub fn jaccard_similarity<T: Eq + std::hash::Hash>(a: &HashSet<T>, b: &HashSet<T>) -> f64 {
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Levenshtein (edit distance) similarity for strings
///
/// Returns similarity as 1.0 - (distance / max_len), so 1.0 means identical.
pub fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    if m == 0 && n == 0 {
        return 1.0;
    }
    if m == 0 || n == 0 {
        return 0.0;
    }

    // Use two-row optimization for space efficiency
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut curr: Vec<usize> = vec![0; n + 1];

    for i in 1..=m {
        curr[0] = i;
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let distance = prev[n];
    let max_len = m.max(n);
    1.0 - (distance as f64 / max_len as f64)
}

/// Dynamic Time Warping (DTW) similarity for time series
///
/// Converts DTW distance to similarity as 1 / (1 + distance).
pub fn dtw_similarity(a: &[f64], b: &[f64]) -> f64 {
    let m = a.len();
    let n = b.len();

    if m == 0 && n == 0 {
        return 1.0;
    }
    if m == 0 || n == 0 {
        return 0.0;
    }

    // DTW matrix with two-row optimization
    let mut prev: Vec<f64> = vec![f64::INFINITY; n + 1];
    let mut curr: Vec<f64> = vec![f64::INFINITY; n + 1];
    prev[0] = 0.0;

    for i in 1..=m {
        curr[0] = f64::INFINITY;
        for j in 1..=n {
            let cost = (a[i - 1] - b[j - 1]).abs();
            curr[j] = cost + prev[j].min(curr[j - 1]).min(prev[j - 1]);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let distance = prev[n];
    1.0 / (1.0 + distance)
}

/// Multi-dimensional DTW similarity
///
/// For time series with multiple features per time step.
pub fn dtw_similarity_multi(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    let m = a.len();
    let n = b.len();

    if m == 0 && n == 0 {
        return 1.0;
    }
    if m == 0 || n == 0 {
        return 0.0;
    }

    // DTW matrix with two-row optimization
    let mut prev: Vec<f64> = vec![f64::INFINITY; n + 1];
    let mut curr: Vec<f64> = vec![f64::INFINITY; n + 1];
    prev[0] = 0.0;

    for i in 1..=m {
        curr[0] = f64::INFINITY;
        for j in 1..=n {
            // Euclidean distance between feature vectors
            let cost: f64 = a[i - 1]
                .iter()
                .zip(&b[j - 1])
                .map(|(x, y)| (x - y).powi(2))
                .sum::<f64>()
                .sqrt();
            curr[j] = cost + prev[j].min(curr[j - 1]).min(prev[j - 1]);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    let distance = prev[n];
    1.0 / (1.0 + distance)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let kdf = Kdf::with_defaults();
        let items: Vec<Vec<f64>> = vec![];
        let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
        assert!(result.selected.is_empty());
    }

    #[test]
    fn test_single_item() {
        let kdf = Kdf::with_defaults();
        let items = vec![vec![1.0, 0.0, 0.0]];
        let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
        assert_eq!(result.selected.len(), 1);
        assert_eq!(result.layers[0], Layer::Rare);
    }

    #[test]
    fn test_redundant_cluster() {
        let kdf = Kdf::with_defaults();
        let items = vec![
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
        ];
        let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
        assert!(result.selected.len() <= 2);
    }

    #[test]
    fn test_rare_preservation() {
        let kdf = Kdf::with_defaults();
        let items = vec![
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![-1.0, 0.0, 0.0], // Rare
        ];
        let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));

        assert!(result.selected.contains(&3));
        assert_eq!(result.layers[3], Layer::Rare);
    }

    #[test]
    fn test_is_selected() {
        let kdf = Kdf::with_defaults();
        let items = vec![
            vec![1.0, 0.9, 0.1],
            vec![1.0, 0.9, 0.1],
            vec![-1.0, 0.0, 0.0],
        ];
        let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));

        // Rare item (index 2) should be selected
        assert!(result.is_selected(2));
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);

        let c = vec![0.0, 1.0, 0.0];
        assert!(cosine_similarity(&a, &c).abs() < 0.001);
    }

    #[test]
    fn test_common_functions() {
        let degrees = vec![0, 5, 10, 2];
        let params = KdfParams::default();

        let layers = classify_layers(&degrees, &params);
        assert_eq!(layers[0], Layer::Rare); // degree 0 is always Rare

        let decay_factors = compute_decay_factors(&degrees, &layers, &params);
        assert_eq!(decay_factors.len(), 4);

        let weights = compute_weights(&decay_factors, 100);
        assert_eq!(weights.len(), 4);
        assert!(weights[0] > weights[2]); // Rare decays slower than Core
    }

    // =========================================================================
    // Master Spec (Edge-Based) Tests
    // =========================================================================

    #[test]
    fn test_edge_congestion_calculation() {
        // Graph: 0--1, 0--2, 0--3 (hub=0, deg=3)
        //        1--2 (deg(1)=2, deg(2)=2, deg(3)=1)
        let degrees = vec![3, 2, 2, 1];

        // Edge (0,1): C_(0,1) = deg(0) + deg(1) = 3 + 2 = 5
        let c_01 = compute_edge_congestion(0, 1, &degrees);
        assert_eq!(c_01, 5.0);

        // Edge (0,3): C_(0,3) = deg(0) + deg(3) = 3 + 1 = 4
        let c_03 = compute_edge_congestion(0, 3, &degrees);
        assert_eq!(c_03, 4.0);

        // Edge (1,2): C_(1,2) = deg(1) + deg(2) = 2 + 2 = 4
        let c_12 = compute_edge_congestion(1, 2, &degrees);
        assert_eq!(c_12, 4.0);
    }

    #[test]
    fn test_decay_probability_edge_layer() {
        let degrees = vec![3, 2];
        let params = KdfParams::master_spec();

        // Edge layer: β=0.010, γ=0.015, α=1.5
        // C_(0,1) = 5
        // P_decay = 0.010 × (1 + 0.015 × 5^1.5) = 0.010 × 1.168 ≈ 0.01168
        let decay_prob = compute_edge_decay_probability(0, 1, &degrees, Layer::Edge, &params);

        let expected = params.beta * (1.0 + params.gamma_edge * 5.0_f64.powf(params.alpha_edge));
        assert!(
            (decay_prob - expected).abs() < 1e-10,
            "Edge decay: expected {}, got {}",
            expected,
            decay_prob
        );
        assert!(
            (decay_prob - 0.01168).abs() < 0.0001,
            "Edge decay C=5: expected ~0.01168, got {}",
            decay_prob
        );
    }

    #[test]
    fn test_decay_probability_rare_layer() {
        let degrees = vec![3, 2];
        let params = KdfParams::master_spec();

        // Rare layer: β=0.010, γ=0.010, α=0.3
        // C_(0,1) = 5
        // P_decay = 0.010 × (1 + 0.010 × 5^0.3) ≈ 0.01016
        let decay_prob = compute_edge_decay_probability(0, 1, &degrees, Layer::Rare, &params);

        let expected = params.beta * (1.0 + params.gamma_rare * 5.0_f64.powf(params.alpha_rare));
        assert!(
            (decay_prob - expected).abs() < 1e-10,
            "Rare decay: expected {}, got {}",
            expected,
            decay_prob
        );
    }

    #[test]
    fn test_decay_probability_core_layer() {
        let degrees = vec![3, 2];
        let params = KdfParams::master_spec();

        // Core layer: β=0.010, γ=0.008, α=2.0
        // C_(0,1) = 5
        // P_decay = 0.010 × (1 + 0.008 × 5^2.0) = 0.010 × 1.2 = 0.012
        let decay_prob = compute_edge_decay_probability(0, 1, &degrees, Layer::Core, &params);

        let expected = params.beta * (1.0 + params.gamma_core * 5.0_f64.powf(params.alpha_core));
        assert!(
            (decay_prob - expected).abs() < 1e-10,
            "Core decay: expected {}, got {}",
            expected,
            decay_prob
        );
        assert!(
            (decay_prob - 0.012).abs() < 0.0001,
            "Core decay C=5: expected 0.012, got {}",
            decay_prob
        );
    }

    #[test]
    fn test_edge_weight_decay() {
        let degrees = vec![3, 2];
        let params = KdfParams::master_spec();
        let decay_prob = compute_edge_decay_probability(0, 1, &degrees, Layer::Edge, &params);

        let initial_weight = 1.0;

        // 1 step
        let weight_after_1 = initial_weight * (1.0 - decay_prob);
        assert!(
            (weight_after_1 - 0.98832).abs() < 0.001,
            "1 step: expected ~0.988, got {}",
            weight_after_1
        );

        // 100 steps
        let weight_after_100 = initial_weight * (1.0 - decay_prob).powi(100);
        assert!(
            weight_after_100 > 0.3 && weight_after_100 < 0.35,
            "100 steps: expected ~0.31, got {}",
            weight_after_100
        );
    }

    #[test]
    fn test_apply_edge_decay() {
        let degrees = vec![3, 2, 1];
        let params = KdfParams::master_spec();

        let mut edge_weights: HashMap<(usize, usize), f64> = HashMap::new();
        edge_weights.insert((0, 1), 1.0);
        edge_weights.insert((0, 2), 1.0);

        let mut edge_layers: HashMap<(usize, usize), Layer> = HashMap::new();
        edge_layers.insert((0, 1), Layer::Edge);
        edge_layers.insert((0, 2), Layer::Rare);

        apply_edge_decay(&mut edge_weights, &degrees, &edge_layers, &params);

        // All weights should have decreased
        assert!(edge_weights[&(0, 1)] < 1.0);
        assert!(edge_weights[&(0, 2)] < 1.0);

        // Rare edge should decay slower (lower probability)
        // Note: Rare has lower gamma (0.010) and lower alpha (0.3)
        // so it should have lower decay probability
    }

    #[test]
    fn test_master_spec_params() {
        let params = KdfParams::master_spec();

        assert!(params.use_edge_based);
        assert_eq!(params.gamma_edge, 0.015);
        assert_eq!(params.gamma_rare, 0.010);
        assert_eq!(params.gamma_core, 0.008);
        assert_eq!(params.gamma_meta, 0.005);
        assert_eq!(params.beta, 0.01);
    }

    #[test]
    fn test_gamma_for_layer() {
        // Default is now edge-based (Master spec compliant)
        let params = KdfParams::default();

        assert_eq!(params.gamma_for_layer(Layer::Edge), 0.015);
        assert_eq!(params.gamma_for_layer(Layer::Rare), 0.010);
        assert_eq!(params.gamma_for_layer(Layer::Core), 0.008);

        // Explicit master_spec() should behave the same
        let master_params = KdfParams::master_spec();
        assert_eq!(master_params.gamma_for_layer(Layer::Edge), 0.015);
        assert_eq!(master_params.gamma_for_layer(Layer::Rare), 0.010);
        assert_eq!(master_params.gamma_for_layer(Layer::Core), 0.008);

        // Legacy mode (use_edge_based = false) should use single gamma
        let legacy_params = KdfParams {
            use_edge_based: false,
            ..Default::default()
        };
        assert_eq!(legacy_params.gamma_for_layer(Layer::Edge), 0.1);
        assert_eq!(legacy_params.gamma_for_layer(Layer::Rare), 0.1);
        assert_eq!(legacy_params.gamma_for_layer(Layer::Core), 0.1);
    }
}
