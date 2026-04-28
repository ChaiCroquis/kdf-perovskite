//! Evaluation metrics for Phase 6.

use super::{Dataset, TrialResult};
use std::collections::HashSet;

pub fn evaluate(
    dataset_name: &str,
    method: &str,
    seed: u64,
    trial: usize,
    ds: &Dataset,
    selected: &HashSet<u32>,
    elapsed_ms: f64,
) -> TrialResult {
    let n_rare_total = ds.rare_ground_truth.len().max(1);
    let n_rare_selected = selected.intersection(&ds.rare_ground_truth).count();
    let rare_recall = n_rare_selected as f64 / n_rare_total as f64;

    // Precision@Rare = TP / (TP + FP_among_selected_rare_predictions)
    // In this framing, "predicted rare" = selected ∩ rare_ground_truth ratio within selected
    // but more informative: how many selected are in the rare set vs selected.
    let precision_at_rare = if !selected.is_empty() {
        n_rare_selected as f64 / selected.len() as f64
    } else {
        0.0
    };

    let f1_at_rare = if rare_recall + precision_at_rare > 0.0 {
        2.0 * rare_recall * precision_at_rare / (rare_recall + precision_at_rare)
    } else {
        0.0
    };

    let compression_rate = 1.0 - selected.len() as f64 / ds.n_nodes.max(1) as f64;

    TrialResult {
        dataset: dataset_name.to_string(),
        method: method.to_string(),
        seed,
        trial,
        n_nodes: ds.n_nodes,
        n_edges: ds.edges.len(),
        n_selected: selected.len(),
        n_rare_total,
        n_rare_selected,
        rare_recall,
        precision_at_rare,
        f1_at_rare,
        compression_rate,
        elapsed_ms,
    }
}
