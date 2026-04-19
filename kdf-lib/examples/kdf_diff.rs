//! Differential KDF - Analyzing Changes Between Two Time Points
//!
//! This example demonstrates how to detect and analyze changes in data
//! distribution between two snapshots using KDF.
//!
//! Key capabilities:
//! - Detect new items (emerged)
//! - Detect removed items
//! - Track layer transitions (e.g., Rare → Core)
//! - Measure distribution drift
//!
//! Run: cargo run --example kdf_diff

use kdf::{cosine_similarity, Kdf, KdfResult, Layer};
use std::collections::HashSet;

/// Result of differential analysis between two KDF snapshots
#[derive(Debug)]
struct KdfDiffResult {
    /// Items that are new in the second snapshot
    pub emerged: Vec<usize>,
    /// Items that were removed (present in first, not in second)
    pub removed: Vec<usize>,
    /// Items that transitioned from Rare to Core (became common)
    pub rare_to_core: Vec<usize>,
    /// Items that transitioned from Core to Rare (became isolated)
    pub core_to_rare: Vec<usize>,
    /// Items with any layer change
    pub layer_changes: Vec<(usize, Layer, Layer)>,
    /// Jaccard similarity between selected sets (0-1)
    pub selection_stability: f64,
    /// Layer distribution change (Chi-squared-like)
    pub distribution_drift: f64,
}

impl KdfDiffResult {
    fn summary(&self) -> String {
        format!(
            "KdfDiff Summary:\n\
             - Emerged: {} items\n\
             - Removed: {} items\n\
             - Rare→Core: {} items (became common)\n\
             - Core→Rare: {} items (became isolated)\n\
             - Total layer changes: {}\n\
             - Selection stability: {:.1}%\n\
             - Distribution drift: {:.4}",
            self.emerged.len(),
            self.removed.len(),
            self.rare_to_core.len(),
            self.core_to_rare.len(),
            self.layer_changes.len(),
            self.selection_stability * 100.0,
            self.distribution_drift
        )
    }
}

/// Compare two KDF results to detect changes
fn kdf_diff(
    result_before: &KdfResult,
    result_after: &KdfResult,
    n_before: usize,
    n_after: usize,
) -> KdfDiffResult {
    // Track emerged and removed
    let emerged: Vec<usize> = (n_before..n_after).collect();
    let removed: Vec<usize> = if n_after < n_before {
        (n_after..n_before).collect()
    } else {
        vec![]
    };

    // Track layer transitions for items present in both
    let common_count = n_before.min(n_after);
    let mut rare_to_core = Vec::new();
    let mut core_to_rare = Vec::new();
    let mut layer_changes = Vec::new();

    for i in 0..common_count {
        let before_layer = result_before.layers[i];
        let after_layer = result_after.layers[i];

        if before_layer != after_layer {
            layer_changes.push((i, before_layer, after_layer));

            match (before_layer, after_layer) {
                (Layer::Rare, Layer::Core) => rare_to_core.push(i),
                (Layer::Core, Layer::Rare) => core_to_rare.push(i),
                _ => {}
            }
        }
    }

    // Calculate selection stability (Jaccard similarity)
    let selected_before: HashSet<_> = result_before.selected.iter().cloned().collect();
    let selected_after: HashSet<_> = result_after.selected.iter()
        .filter(|&&i| i < common_count)
        .cloned()
        .collect();

    let intersection = selected_before.intersection(&selected_after).count();
    let union = selected_before.union(&selected_after).count();
    let selection_stability = if union > 0 {
        intersection as f64 / union as f64
    } else {
        1.0
    };

    // Calculate distribution drift
    let count_layer = |result: &KdfResult, n: usize| -> (usize, usize, usize) {
        let mut core = 0;
        let mut edge = 0;
        let mut rare = 0;
        for i in 0..n.min(result.layers.len()) {
            match result.layers[i] {
                Layer::Core => core += 1,
                Layer::Edge => edge += 1,
                Layer::Rare => rare += 1,
            }
        }
        (core, edge, rare)
    };

    let (c1, e1, r1) = count_layer(result_before, common_count);
    let (c2, e2, r2) = count_layer(result_after, common_count);

    let total1 = (c1 + e1 + r1) as f64;
    let total2 = (c2 + e2 + r2) as f64;

    let distribution_drift = if total1 > 0.0 && total2 > 0.0 {
        let p1 = [c1 as f64 / total1, e1 as f64 / total1, r1 as f64 / total1];
        let p2 = [c2 as f64 / total2, e2 as f64 / total2, r2 as f64 / total2];

        // Jensen-Shannon divergence (bounded 0-1)
        let mut js_div = 0.0;
        for i in 0..3 {
            let m = (p1[i] + p2[i]) / 2.0;
            if p1[i] > 0.0 && m > 0.0 {
                js_div += p1[i] * (p1[i] / m).ln();
            }
            if p2[i] > 0.0 && m > 0.0 {
                js_div += p2[i] * (p2[i] / m).ln();
            }
        }
        js_div / 2.0 / 2.0_f64.ln() // Normalize to 0-1
    } else {
        0.0
    };

    KdfDiffResult {
        emerged,
        removed,
        rare_to_core,
        core_to_rare,
        layer_changes,
        selection_stability,
        distribution_drift,
    }
}

fn main() {
    println!("=== Differential KDF Demo ===\n");

    let kdf = Kdf::with_defaults();
    let threshold = 0.85;

    // =========================================================================
    // Scenario 1: Stable Distribution (minor changes)
    // =========================================================================
    println!("--- Scenario 1: Stable Distribution ---\n");

    // Time T1: Initial data
    let items_t1 = vec![
        vec![1.0, 0.0, 0.0], // Cluster A
        vec![1.0, 0.1, 0.0], // Cluster A
        vec![0.0, 1.0, 0.0], // Cluster B
        vec![0.0, 1.0, 0.1], // Cluster B
        vec![5.0, 5.0, 0.0], // Rare
    ];

    // Time T2: Same structure, slightly modified
    let items_t2 = vec![
        vec![1.0, 0.0, 0.0], // Cluster A (same)
        vec![1.0, 0.15, 0.0], // Cluster A (slightly different)
        vec![0.0, 1.0, 0.0], // Cluster B (same)
        vec![0.0, 1.0, 0.15], // Cluster B (slightly different)
        vec![5.0, 5.0, 0.0], // Rare (same)
    ];

    let result_t1 = kdf.process(&items_t1, threshold, |a, b| cosine_similarity(a, b));
    let result_t2 = kdf.process(&items_t2, threshold, |a, b| cosine_similarity(a, b));

    let diff_stable = kdf_diff(&result_t1, &result_t2, items_t1.len(), items_t2.len());

    println!("T1: {} items, T2: {} items", items_t1.len(), items_t2.len());
    println!("{}\n", diff_stable.summary());

    // =========================================================================
    // Scenario 2: Distribution Shift (Rare → Core)
    // =========================================================================
    println!("--- Scenario 2: Rare Becoming Common ---\n");

    // Time T1: One rare item
    let items_shift_t1 = vec![
        vec![1.0, 0.0], // Cluster A
        vec![1.0, 0.1], // Cluster A
        vec![5.0, 5.0], // Rare (isolated)
    ];

    // Time T2: More similar items appear, rare becomes core
    let items_shift_t2 = vec![
        vec![1.0, 0.0], // Cluster A
        vec![1.0, 0.1], // Cluster A
        vec![5.0, 5.0], // Now has friends
        vec![5.0, 5.1], // New similar
        vec![5.1, 5.0], // New similar
    ];

    let result_shift_t1 = kdf.process(&items_shift_t1, threshold, |a, b| cosine_similarity(a, b));
    let result_shift_t2 = kdf.process(&items_shift_t2, threshold, |a, b| cosine_similarity(a, b));

    println!("T1 layers: {:?}", result_shift_t1.layers);
    println!("T2 layers: {:?}", result_shift_t2.layers);

    let diff_shift = kdf_diff(&result_shift_t1, &result_shift_t2, items_shift_t1.len(), items_shift_t2.len());
    println!("{}\n", diff_shift.summary());

    if !diff_shift.rare_to_core.is_empty() {
        println!("Items that transitioned Rare → Core:");
        for idx in &diff_shift.rare_to_core {
            println!("  Index {}: {:?}", idx, items_shift_t1[*idx]);
        }
        println!();
    }

    // =========================================================================
    // Scenario 3: Concept Drift Detection
    // =========================================================================
    println!("--- Scenario 3: Concept Drift (Data Distribution Change) ---\n");

    // T1: Two balanced clusters
    let items_drift_t1 = vec![
        vec![1.0, 0.0], vec![1.0, 0.1], vec![1.0, 0.2], // Cluster A (3)
        vec![0.0, 1.0], vec![0.1, 1.0], vec![0.2, 1.0], // Cluster B (3)
    ];

    // T2: Cluster A dominates, B shrinks
    let items_drift_t2 = vec![
        vec![1.0, 0.0], vec![1.0, 0.1], vec![1.0, 0.2], vec![1.0, 0.3], vec![1.0, 0.4], // Cluster A (5)
        vec![0.0, 1.0], // Cluster B (1) - now rare!
    ];

    let result_drift_t1 = kdf.process(&items_drift_t1, threshold, |a, b| cosine_similarity(a, b));
    let result_drift_t2 = kdf.process(&items_drift_t2, threshold, |a, b| cosine_similarity(a, b));

    println!("T1: Balanced clusters");
    println!("  Layers: {:?}", result_drift_t1.layers);
    println!("  Rare: {:?}", result_drift_t1.rare_items());

    println!("T2: Imbalanced (Cluster A dominates)");
    println!("  Layers: {:?}", result_drift_t2.layers);
    println!("  Rare: {:?}", result_drift_t2.rare_items());

    let common = items_drift_t1.len().min(items_drift_t2.len());
    let diff_drift = kdf_diff(&result_drift_t1, &result_drift_t2, common, common);
    println!("\n{}", diff_drift.summary());

    if diff_drift.distribution_drift > 0.1 {
        println!("\n** DRIFT DETECTED! Distribution drift = {:.4} **", diff_drift.distribution_drift);
        println!("This indicates significant change in data distribution.");
    }

    // =========================================================================
    // Scenario 4: Streaming with New Items
    // =========================================================================
    println!("\n--- Scenario 4: Streaming (New Items Emerging) ---\n");

    let items_stream_t1: Vec<Vec<f64>> = vec![
        vec![1.0, 0.0],
        vec![1.0, 0.1],
        vec![0.0, 1.0],
    ];

    // New items arrive
    let items_stream_t2: Vec<Vec<f64>> = vec![
        vec![1.0, 0.0],
        vec![1.0, 0.1],
        vec![0.0, 1.0],
        // New items
        vec![0.5, 0.5], // New: between clusters
        vec![9.0, 9.0], // New: outlier
    ];

    let result_stream_t1 = kdf.process(&items_stream_t1, threshold, |a, b| cosine_similarity(a, b));
    let result_stream_t2 = kdf.process(&items_stream_t2, threshold, |a, b| cosine_similarity(a, b));

    let diff_stream = kdf_diff(&result_stream_t1, &result_stream_t2, items_stream_t1.len(), items_stream_t2.len());

    println!("Existing items: {}", items_stream_t1.len());
    println!("New items: {}", diff_stream.emerged.len());
    println!("\n{}", diff_stream.summary());

    if !diff_stream.emerged.is_empty() {
        println!("\nNew items emerged at indices: {:?}", diff_stream.emerged);
        for idx in &diff_stream.emerged {
            let layer = &result_stream_t2.layers[*idx];
            println!("  {} -> {:?}", idx, layer);
        }
    }

    // =========================================================================
    // Summary
    // =========================================================================
    println!("\n=== Key Insights ===");
    println!("1. Selection stability measures how consistent the selections are");
    println!("2. Distribution drift detects when data patterns change");
    println!("3. Rare→Core transitions show previously isolated items gaining neighbors");
    println!("4. Core→Rare transitions can indicate data loss or cluster shrinking");
    println!("5. Emerged items in streaming scenarios can be monitored for anomalies");
}
