//! KDF Complexity Verification
//!
//! Verify the actual computational behavior:
//! 1. Standard KDF is O(n²) regardless of data redundancy
//! 2. Optimized versions can benefit from redundancy
//!
//! Run: cargo run --release --example kdf_complexity_verification

use kdf::{Kdf, Layer, cosine_similarity};
use std::time::Instant;

/// Count similarity comparisons (instrumented version)
fn count_comparisons(data: &[Vec<f64>], threshold: f64) -> (usize, usize, usize) {
    let n = data.len();
    let mut total_comparisons = 0usize;
    let mut above_threshold = 0usize;

    for i in 0..n {
        for j in (i + 1)..n {
            total_comparisons += 1;
            if cosine_similarity(&data[i], &data[j]) >= threshold {
                above_threshold += 1;
            }
        }
    }

    let theoretical_max = n * (n - 1) / 2;
    (total_comparisons, above_threshold, theoretical_max)
}

/// Generate highly redundant data (90% identical, 10% rare)
fn generate_redundant_data(n: usize, dim: usize) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);

    // 90% identical cluster
    let base: Vec<f64> = (0..dim).map(|d| (d as f64) * 0.1).collect();
    for _ in 0..(n * 9 / 10) {
        data.push(base.clone());
    }

    // 10% rare (orthogonal)
    for i in 0..(n / 10) {
        let mut rare = vec![0.0; dim];
        rare[i % dim] = 1.0; // Each rare point is unique
        data.push(rare);
    }

    data
}

/// Generate diverse data (all different)
fn generate_diverse_data(n: usize, dim: usize) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);

    for i in 0..n {
        let angle = (i as f64) * std::f64::consts::PI / n as f64;
        let point: Vec<f64> = (0..dim).map(|d| (angle + d as f64 * 0.1).sin()).collect();
        data.push(point);
    }

    data
}

fn main() {
    println!("=== KDF Complexity Verification ===\n");

    let dim = 10;
    let threshold = 0.95;

    println!("## 1. Theoretical Analysis\n");
    println!("Current implementation (lib.rs lines 1112-1120):");
    println!("```");
    println!("for i in 0..n {{");
    println!("    for j in (i + 1)..n {{");
    println!("        if similarity(&items[i], &items[j]) >= sim_threshold {{");
    println!("            degrees[i] += 1;");
    println!("            degrees[j] += 1;");
    println!("        }}");
    println!("    }}");
    println!("}}");
    println!("```");
    println!("→ ALL pairs are compared: O(n²)\n");

    println!("## 2. Empirical Verification\n");

    let sizes = [100, 200, 500, 1000];

    println!("### 2.1 Comparison Count (Standard KDF)\n");
    println!("| n | Theoretical n(n-1)/2 | Actual Comparisons | Match? |");
    println!("|---|---------------------|-------------------|--------|");

    for &n in &sizes {
        let data = generate_redundant_data(n, dim);
        let (actual, _, theoretical) = count_comparisons(&data, threshold);
        let matches = actual == theoretical;
        println!(
            "| {} | {} | {} | {} |",
            n,
            theoretical,
            actual,
            if matches { "✓" } else { "✗" }
        );
    }

    println!("\n### 2.2 Processing Time: Redundant vs Diverse\n");
    println!("| n | Redundant Time | Diverse Time | Ratio |");
    println!("|---|---------------|--------------|-------|");

    for &n in &sizes {
        let redundant = generate_redundant_data(n, dim);
        let diverse = generate_diverse_data(n, dim);
        let kdf = Kdf::with_defaults();

        // Warm up
        let _ = kdf.process(&redundant, threshold, |a, b| cosine_similarity(a, b));
        let _ = kdf.process(&diverse, threshold, |a, b| cosine_similarity(a, b));

        // Measure redundant
        let start = Instant::now();
        for _ in 0..3 {
            let _ = kdf.process(&redundant, threshold, |a, b| cosine_similarity(a, b));
        }
        let redundant_time = start.elapsed().as_secs_f64() / 3.0 * 1000.0;

        // Measure diverse
        let start = Instant::now();
        for _ in 0..3 {
            let _ = kdf.process(&diverse, threshold, |a, b| cosine_similarity(a, b));
        }
        let diverse_time = start.elapsed().as_secs_f64() / 3.0 * 1000.0;

        let ratio = redundant_time / diverse_time;
        println!(
            "| {} | {:.2}ms | {:.2}ms | {:.2}x |",
            n, redundant_time, diverse_time, ratio
        );
    }

    println!("\n### 2.3 Layer Distribution (n=500)\n");

    let n = 500;
    let redundant = generate_redundant_data(n, dim);
    let diverse = generate_diverse_data(n, dim);
    let kdf = Kdf::with_defaults();

    let result_r = kdf.process(&redundant, threshold, |a, b| cosine_similarity(a, b));
    let result_d = kdf.process(&diverse, threshold, |a, b| cosine_similarity(a, b));

    let count_layers = |layers: &[Layer]| -> (usize, usize, usize) {
        let core = layers.iter().filter(|&&l| l == Layer::Core).count();
        let edge = layers.iter().filter(|&&l| l == Layer::Edge).count();
        let rare = layers.iter().filter(|&&l| l == Layer::Rare).count();
        (core, edge, rare)
    };

    let (rc, re, rr) = count_layers(&result_r.layers);
    let (dc, de, dr) = count_layers(&result_d.layers);

    println!("| Dataset | Core | Edge | Rare | Selected |");
    println!("|---------|------|------|------|----------|");
    println!(
        "| Redundant (90% same) | {} | {} | {} | {} |",
        rc,
        re,
        rr,
        result_r.selected.len()
    );
    println!(
        "| Diverse (all diff) | {} | {} | {} | {} |",
        dc,
        de,
        dr,
        result_d.selected.len()
    );

    println!("\n## 3. Conclusion\n");
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ 現状の実装:                                                │");
    println!("│   - 標準KDFは O(n²) - 全ペア比較を実行                     │");
    println!("│   - データの冗長性に関わらず比較回数は同じ                 │");
    println!("│   - 処理時間もほぼ同程度                                   │");
    println!("│                                                            │");
    println!("│ 違いが出るのは:                                            │");
    println!("│   - 層分類結果（Core/Edge/Rare の比率）                    │");
    println!("│   - 選択されるアイテム数                                   │");
    println!("│   - Phase 4 の選択処理時間（O(n × |selected|)）            │");
    println!("│                                                            │");
    println!("│ 「冗長データほど軽い」を実現するには:                      │");
    println!("│   - LSH による近似（examples/kdf_optimization_strategies） │");
    println!("│   - 早期終了による枝刈り                                   │");
    println!("│   - これらはコアライブラリ未統合                           │");
    println!("└────────────────────────────────────────────────────────────┘");

    println!("\n## 4. Future Optimization Path\n");
    println!("To achieve sub-O(n²) for redundant data:");
    println!("1. Integrate LSH into core process() function");
    println!("2. Early termination when Core threshold is reached");
    println!("3. Incremental processing with representative caching");
}
