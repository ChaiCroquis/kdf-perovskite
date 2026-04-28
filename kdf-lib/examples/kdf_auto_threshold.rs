//! Auto-threshold Example for KDF
//!
//! Demonstrates automatic optimal threshold selection.
//!
//! Run: cargo run --example kdf_auto_threshold

use kdf::{cosine_similarity, Kdf};

fn main() {
    println!("=== KDF Auto-Threshold Demo ===\n");

    // Create sample data with different cluster densities
    let items = vec![
        // Dense cluster A (very similar)
        vec![1.0, 0.0, 0.0],
        vec![1.0, 0.05, 0.0],
        vec![1.0, 0.0, 0.05],
        vec![1.0, 0.03, 0.03],
        vec![0.98, 0.02, 0.0],
        // Dense cluster B (very similar)
        vec![0.0, 1.0, 0.0],
        vec![0.0, 1.0, 0.05],
        vec![0.0, 0.98, 0.02],
        vec![0.0, 1.0, 0.03],
        // Sparse cluster C (moderately similar)
        vec![0.0, 0.0, 1.0],
        vec![0.0, 0.2, 0.8],
        vec![0.1, 0.1, 0.9],
        // Rare items (isolated)
        vec![0.5, 0.5, 0.0], // Between A and B
        vec![0.3, 0.3, 0.4], // Mixed
        vec![5.0, 5.0, 5.0], // Outlier
    ];

    let kdf = Kdf::with_defaults();

    println!("Total items: {}", items.len());
    println!();

    // =========================================================================
    // 1. Auto-threshold (full analysis)
    // =========================================================================
    println!("--- Auto-threshold (Full Analysis) ---");

    let auto_result = kdf.process_auto(&items, |a, b| cosine_similarity(a, b));

    println!("Optimal threshold: {:.2}", auto_result.threshold);
    println!("Selected items: {:?}", auto_result.result.selected);
    println!("Selection count: {}", auto_result.result.selected.len());
    println!(
        "Compression: {:.1}%",
        (1.0 - auto_result.result.selected.len() as f64 / items.len() as f64) * 100.0
    );
    println!("Rare items: {:?}", auto_result.result.rare_items());
    println!();

    // Show score distribution
    println!("Threshold evaluation scores:");
    for (i, &threshold) in auto_result.thresholds_evaluated.iter().enumerate() {
        let score = auto_result.scores[i];
        let marker = if (threshold - auto_result.threshold).abs() < 0.001 {
            " <-- optimal"
        } else {
            ""
        };
        println!("  {:.2}: score = {:.4}{}", threshold, score, marker);
    }
    println!();

    // =========================================================================
    // 2. Auto-threshold Quick (fast approximation)
    // =========================================================================
    println!("--- Auto-threshold Quick ---");

    let (quick_threshold, quick_result) =
        kdf.process_auto_quick(&items, |a, b| cosine_similarity(a, b));

    println!("Quick threshold: {:.2}", quick_threshold);
    println!("Selected items: {:?}", quick_result.selected);
    println!("Rare items: {:?}", quick_result.rare_items());
    println!();

    // =========================================================================
    // 3. Compare with manual thresholds
    // =========================================================================
    println!("--- Manual Threshold Comparison ---");

    for threshold in [0.70, 0.80, 0.90, 0.95] {
        let result = kdf.process(&items, threshold, |a, b| cosine_similarity(a, b));
        println!(
            "  {:.2}: {} selected, {} rare",
            threshold,
            result.selected.len(),
            result.rare_items().len()
        );
    }
    println!();

    // =========================================================================
    // 4. Text data example
    // =========================================================================
    println!("--- Text Data Example (with Levenshtein) ---");

    use kdf::levenshtein_similarity;

    let texts = vec![
        "error: connection failed",
        "error: connection refused",
        "error: connection timeout",
        "error: connection failed", // duplicate
        "info: server started",
        "info: server started successfully",
        "warning: unusual activity detected", // isolated
        "critical: security breach",          // isolated
    ];

    let text_auto = kdf.process_auto(&texts, |a, b| levenshtein_similarity(a, b));

    println!("Optimal threshold: {:.2}", text_auto.threshold);
    println!("Selected texts:");
    for &idx in &text_auto.result.selected {
        let layer = &text_auto.result.layers[idx];
        println!("  [{}] {:?}: \"{}\"", idx, layer, texts[idx]);
    }
    println!();

    println!("=== Summary ===");
    println!("Auto-threshold automatically finds the optimal balance between");
    println!("compression (removing redundancy) and preservation (keeping rare items).");
    println!();
    println!("Use process_auto() for thorough analysis (16 threshold evaluations)");
    println!("Use process_auto_quick() for fast approximation (3 threshold evaluations)");
}
