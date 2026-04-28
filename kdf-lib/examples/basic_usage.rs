//! Basic usage example for KDF

use kdf::{cosine_similarity, Kdf};

fn main() {
    println!("=== KDF Basic Usage Example ===\n");

    // Sample data: 10 redundant + 2 rare
    let items = vec![
        // Cluster A (redundant)
        vec![1.0, 0.9, 0.1, 0.0],
        vec![1.0, 0.9, 0.1, 0.0],
        vec![1.0, 0.9, 0.1, 0.0],
        vec![1.0, 0.9, 0.1, 0.0],
        vec![1.0, 0.9, 0.1, 0.0],
        // Cluster B (redundant)
        vec![0.0, 0.1, 0.9, 1.0],
        vec![0.0, 0.1, 0.9, 1.0],
        vec![0.0, 0.1, 0.9, 1.0],
        vec![0.0, 0.1, 0.9, 1.0],
        vec![0.0, 0.1, 0.9, 1.0],
        // Rare items
        vec![-1.0, 0.0, 0.0, 0.0],
        vec![0.0, -1.0, 0.0, 0.0],
    ];

    // Create KDF with default parameters
    let kdf = Kdf::with_defaults();

    // Process items
    let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));

    // Print results
    println!("Input: {} items", items.len());
    println!("Selected: {} items", result.selected_count());
    println!("\nSelected indices: {:?}", result.selected_indices());

    println!("\nLayer classification:");
    for (i, layer) in result.layers.iter().enumerate() {
        println!(
            "  Item {}: {:?} (score: {:.4})",
            i, layer, result.selection_scores[i]
        );
    }

    // Calculate metrics
    let is_rare = |i: &usize| *i >= 10; // Items 10, 11 are rare
    let rr = result.redundancy_reduction(&(0..12).collect::<Vec<_>>(), |i| is_rare(i));
    let rp = result.rare_preservation(&(0..12).collect::<Vec<_>>(), |i| is_rare(i));

    println!("\nMetrics:");
    println!("  Redundancy reduction: {:.1}%", rr * 100.0);
    println!("  Rare preservation: {:.1}%", rp * 100.0);
}
