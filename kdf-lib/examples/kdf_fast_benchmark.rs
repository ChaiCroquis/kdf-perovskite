//! Benchmark: process() vs process_fast() vs process_fast_verified()
//!
//! Run: cargo run --release --example kdf_fast_benchmark

use kdf::{cosine_similarity, Kdf, Layer};
use std::time::Instant;

fn generate_redundant_data(n: usize, dim: usize, redundancy: f64) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);
    let n_redundant = (n as f64 * redundancy) as usize;
    let n_rare = n - n_redundant;

    // Redundant cluster (all similar)
    let base: Vec<f64> = (0..dim).map(|d| (d as f64) * 0.1).collect();
    for i in 0..n_redundant {
        let mut point = base.clone();
        // Add tiny noise
        point[0] += (i as f64 * 0.001).sin() * 0.01;
        data.push(point);
    }

    // Rare items (orthogonal to each other)
    for i in 0..n_rare {
        let mut rare = vec![0.0; dim];
        rare[i % dim] = 1.0;
        data.push(rare);
    }

    data
}

fn count_layers(layers: &[Layer]) -> (usize, usize, usize) {
    let core = layers.iter().filter(|&&l| l == Layer::Core).count();
    let edge = layers.iter().filter(|&&l| l == Layer::Edge).count();
    let rare = layers.iter().filter(|&&l| l == Layer::Rare).count();
    (core, edge, rare)
}

fn rare_recall(true_layers: &[Layer], predicted_layers: &[Layer]) -> f64 {
    let true_rare: std::collections::HashSet<usize> = true_layers
        .iter()
        .enumerate()
        .filter(|(_, &l)| l == Layer::Rare)
        .map(|(i, _)| i)
        .collect();

    if true_rare.is_empty() {
        return 1.0;
    }

    let predicted_rare: std::collections::HashSet<usize> = predicted_layers
        .iter()
        .enumerate()
        .filter(|(_, &l)| l == Layer::Rare)
        .map(|(i, _)| i)
        .collect();

    true_rare.intersection(&predicted_rare).count() as f64 / true_rare.len() as f64
}

fn main() {
    println!("=== process_fast() Benchmark ===\n");

    let kdf = Kdf::with_defaults();
    let threshold = 0.95;
    let dim = 20;

    println!("## 1. Speed Comparison (90% redundancy)\n");
    println!("| n | Standard | Fast | Fast+Verify | Fast Speedup | Verify Speedup |");
    println!("|---|----------|------|-------------|--------------|----------------|");

    for &n in &[500, 1000, 2000, 5000, 10000] {
        let data = generate_redundant_data(n, dim, 0.9);

        // Standard
        let start = Instant::now();
        let _std_result = kdf.process(&data, threshold, |a, b| cosine_similarity(a, b));
        let std_time = start.elapsed().as_secs_f64() * 1000.0;

        // Fast
        let start = Instant::now();
        let _fast_result = kdf.process_fast(&data, threshold, |a, b| cosine_similarity(a, b));
        let fast_time = start.elapsed().as_secs_f64() * 1000.0;

        // Fast + Verify
        let start = Instant::now();
        let _verify_result =
            kdf.process_fast_verified(&data, threshold, |a, b| cosine_similarity(a, b), true);
        let verify_time = start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "| {} | {:.1}ms | {:.2}ms | {:.2}ms | {:.0}x | {:.0}x |",
            n,
            std_time,
            fast_time,
            verify_time,
            std_time / fast_time,
            std_time / verify_time
        );
    }

    println!("\n## 2. Accuracy Comparison (n=2000)\n");
    println!("| Method | Selected | Core | Edge | Rare | Rare Recall |");
    println!("|--------|----------|------|------|------|-------------|");

    let n = 2000;
    for &redundancy in &[0.7, 0.9, 0.95] {
        let data = generate_redundant_data(n, dim, redundancy);
        let n_rare_expected = n - (n as f64 * redundancy) as usize;

        // Standard (ground truth)
        let std_result = kdf.process(&data, threshold, |a, b| cosine_similarity(a, b));
        let (std_core, std_edge, std_rare) = count_layers(&std_result.layers);

        // Fast
        let fast_result = kdf.process_fast(&data, threshold, |a, b| cosine_similarity(a, b));
        let (fast_core, fast_edge, fast_rare) = count_layers(&fast_result.layers);
        let fast_recall = rare_recall(&std_result.layers, &fast_result.layers);

        // Fast + Verify
        let verify_result =
            kdf.process_fast_verified(&data, threshold, |a, b| cosine_similarity(a, b), true);
        let (ver_core, ver_edge, ver_rare) = count_layers(&verify_result.layers);
        let ver_recall = rare_recall(&std_result.layers, &verify_result.layers);

        println!(
            "\n**Redundancy: {:.0}%** (Expected rare: {})",
            redundancy * 100.0,
            n_rare_expected
        );
        println!(
            "| Standard | {} | {} | {} | {} | 100% |",
            std_result.selected.len(),
            std_core,
            std_edge,
            std_rare
        );
        println!(
            "| Fast | {} | {} | {} | {} | {:.1}% |",
            fast_result.selected.len(),
            fast_core,
            fast_edge,
            fast_rare,
            fast_recall * 100.0
        );
        println!(
            "| Fast+Verify | {} | {} | {} | {} | {:.1}% |",
            verify_result.selected.len(),
            ver_core,
            ver_edge,
            ver_rare,
            ver_recall * 100.0
        );
    }

    println!("\n## 3. Summary\n");
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ process_fast():                                           │");
    println!("│   - 100-1000x speedup for redundant data                  │");
    println!("│   - O(n × k) complexity (k = cluster count)               │");
    println!("│   - ⚠️ 0% Rare Recall - cannot detect rare items          │");
    println!("│   - Use for: redundancy reduction only                    │");
    println!("│                                                            │");
    println!("│ process_fast_verified():                                   │");
    println!("│   - Still fast for mostly-redundant data                  │");
    println!("│   - Better Rare detection with verify_rare=true           │");
    println!("│   - Use for: speed + some Rare detection accuracy         │");
    println!("│                                                            │");
    println!("│ process() (standard):                                      │");
    println!("│   - O(n²) but accurate                                    │");
    println!("│   - 100% Rare Recall guaranteed                           │");
    println!("│   - Use for: anomaly detection, rare item preservation    │");
    println!("└────────────────────────────────────────────────────────────┘");
}
