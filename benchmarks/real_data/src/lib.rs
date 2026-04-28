//! Phase 6 real-data benchmark framework.
//!
//! Abstracts:
//! - Dataset loading (any source producing `Dataset`)
//! - KDF and baseline methods (unified `Selector` trait)
//! - Metrics: Rare Recall, Precision@Rare, F1@Rare, Compression, Time
//! - Statistical tests: Wilcoxon signed-rank

pub mod metrics;
pub mod obsidian;
pub mod public_datasets;
pub mod selectors;
pub mod wilcoxon;

use serde::Serialize;
use std::collections::HashSet;

/// A dataset with optional rare ground truth.
#[derive(Clone, Debug)]
pub struct Dataset {
    pub name: String,
    pub n_nodes: usize,
    pub edges: Vec<(u32, u32, f64)>,
    /// Ground-truth rare items (empty if unknown).
    pub rare_ground_truth: HashSet<u32>,
    /// Free-form description, e.g. "FB15K-237 test split relations <= freq 5"
    pub description: String,
}

impl Dataset {
    pub fn n_edges(&self) -> usize {
        self.edges.len()
    }
    pub fn n_rare(&self) -> usize {
        self.rare_ground_truth.len()
    }
}

/// Trial-level result for a single (dataset, method, seed) run.
#[derive(Serialize, Debug, Clone)]
pub struct TrialResult {
    pub dataset: String,
    pub method: String,
    pub seed: u64,
    pub trial: usize,
    pub n_nodes: usize,
    pub n_edges: usize,
    pub n_selected: usize,
    pub n_rare_total: usize,
    pub n_rare_selected: usize,
    pub rare_recall: f64,
    pub precision_at_rare: f64,
    pub f1_at_rare: f64,
    pub compression_rate: f64,
    pub elapsed_ms: f64,
}
