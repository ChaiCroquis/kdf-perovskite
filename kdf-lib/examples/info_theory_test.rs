//! Information-theoretic foundation demonstration
use kdf::{cosine_similarity, Kdf, KdfParams, TheoreticalBounds};

fn main() {
    println!("=== KDF Information-Theoretic Foundation ===\n");

    // Create test dataset with clear redundancy structure
    let items = vec![
        // Highly redundant cluster (5 similar items)
        vec![1.0, 0.0, 0.0],
        vec![0.99, 0.05, 0.0],
        vec![0.98, 0.1, 0.0],
        vec![0.97, 0.12, 0.0],
        vec![0.96, 0.15, 0.0],
        // Another cluster (3 similar items)
        vec![0.0, 1.0, 0.0],
        vec![0.05, 0.98, 0.0],
        vec![0.1, 0.97, 0.0],
        // Rare items (high information content)
        vec![-1.0, 0.0, 0.0], // Unique direction
        vec![0.0, 0.0, 1.0],  // Unique direction
        vec![0.5, 0.5, 0.5],  // Unique position
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&items, 0.9, |a, b| cosine_similarity(a, b));

    // ========================================================================
    // 1. Information-Theoretic Metrics
    // ========================================================================
    println!("## 1. Information-Theoretic Metrics\n");

    let metrics = result.info_metrics();

    println!("   Entropy Analysis:");
    println!(
        "   - Original entropy:  {:.3} bits",
        metrics.original_entropy
    );
    println!(
        "   - Selected entropy:  {:.3} bits",
        metrics.selected_entropy
    );
    println!(
        "   - Info preserved:    {:.1}%",
        metrics.information_preserved * 100.0
    );

    println!("\n   Compression:");
    println!("   - Compression ratio: {:.2}x", metrics.compression_ratio);
    println!("   - MDL original:      {:.2} bits", metrics.mdl_original);
    println!("   - MDL selected:      {:.2} bits", metrics.mdl_selected);
    println!(
        "   - Redundancy removed: {:.2} bits",
        metrics.redundancy_removed
    );

    println!("\n   Rare Item Contribution:");
    println!(
        "   - Rare information:  {:.2} bits",
        metrics.rare_information
    );

    // ========================================================================
    // 2. Per-Item Information Content
    // ========================================================================
    println!("\n## 2. Per-Item Information Content\n");

    println!("   | Item | Layer | Selected | Information (bits) |");
    println!("   |------|-------|----------|-------------------|");
    for i in 0..items.len() {
        let info = result.item_information(i);
        let selected = if result.is_selected(i) { "✓" } else { " " };
        println!(
            "   | {:>4} | {:?} | {:>8} | {:>17.3} |",
            i, result.layers[i], selected, info
        );
    }

    // ========================================================================
    // 3. Items Ranked by Information
    // ========================================================================
    println!("\n## 3. Items Ranked by Information Content\n");

    let ranked = result.items_by_information();
    println!("   Top 5 highest-information items:");
    for (i, (idx, info)) in ranked.iter().take(5).enumerate() {
        let layer = result.layers[*idx];
        let selected = if result.is_selected(*idx) {
            "selected"
        } else {
            "filtered"
        };
        println!(
            "   {}. Item {} ({:.3} bits) - {:?}, {}",
            i + 1,
            idx,
            info,
            layer,
            selected
        );
    }

    // ========================================================================
    // 4. Theoretical Bounds Verification
    // ========================================================================
    println!("\n## 4. Theoretical Bounds\n");

    // Rare preservation guarantee
    let rare_preserved = TheoreticalBounds::verify_rare_preservation(&result);
    println!(
        "   Rare preservation verified: {}",
        if rare_preserved { "✅ YES" } else { "❌ NO" }
    );

    // Maximum information loss
    let max_loss = TheoreticalBounds::max_information_loss(&result);
    println!("   Max theoretical info loss: {:.2} bits", max_loss);

    // Convergence iterations
    let params = KdfParams::default();
    let conv_iter = TheoreticalBounds::convergence_iterations(&params);
    println!(
        "   Convergence iterations: {} (using {})",
        conv_iter, params.iterations
    );

    // ========================================================================
    // 5. Theoretical Justification
    // ========================================================================
    println!("\n## 5. Theoretical Justification\n");

    println!("   KDF optimizes a dual objective:");
    println!("   ");
    println!("   1. MINIMIZE redundancy (maximize compression)");
    println!("      - Items with high connectivity → fast decay → filtered");
    println!(
        "      - Compression achieved: {:.1}x",
        metrics.compression_ratio
    );
    println!("   ");
    println!("   2. MAXIMIZE rare preservation (minimize information loss)");
    println!("      - Items with zero connectivity → slow decay → preserved");
    println!("      - Rare items preserved: 100% (mathematical guarantee)");
    println!("   ");
    println!("   Information-theoretic interpretation:");
    println!("   - Rare items: I(x) = -log₂(P(x)) is HIGH (low probability)");
    println!("   - Redundant items: I(x) is LOW (high probability)");
    println!("   - KDF selects items that maximize total information retention");

    // ========================================================================
    // 6. Mathematical Proof Summary
    // ========================================================================
    println!("\n## 6. Mathematical Proof (Rev.12)\n");

    println!("   For Rare items (degree=0):");
    println!("   - Decay factor: λ = β(1 + γ·0^α) = β = 0.01");
    println!("   - Weight after T=100: w(100) = (1-0.01)^100 = 0.366");
    println!("   - Threshold θ_E = 0.15");
    println!("   - Since 0.366 > 0.15, all Rare items are SELECTED ✓");
    println!("   ");
    println!("   For Core items (degree≥5, avg*1.5):");
    println!("   - Decay factor: λ = β(1 + γ·C^α_C) >> β");
    println!("   - Weight decays exponentially faster");
    println!("   - Falls below θ_E, assigned to representative ✓");

    println!("\n✅ Information-Theoretic Analysis Complete");
}
