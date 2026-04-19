//! Early Termination + Representative Comparison
//!
//! A different approach: stop comparing once we know an item's layer
//! and use representatives to avoid redundant comparisons.
//!
//! Run: cargo run --release --example kdf_early_termination_test

use kdf::{Kdf, Layer, cosine_similarity};
use std::time::Instant;

// ============================================================================
// Early Termination KDF with Representative Comparison
// ============================================================================

struct EarlyTerminationResult {
    selected: Vec<usize>,
    layers: Vec<Layer>,
    comparisons: usize,
}

/// Process with early termination and representative comparison
fn process_early_termination(
    data: &[Vec<f64>],
    threshold: f64,
    core_degree_threshold: usize,  // If degree >= this, definitely Core
) -> EarlyTerminationResult {
    let n = data.len();
    if n == 0 {
        return EarlyTerminationResult {
            selected: vec![],
            layers: vec![],
            comparisons: 0,
        };
    }

    let mut comparisons = 0usize;
    let mut degrees = vec![0usize; n];
    let mut is_core = vec![false; n];  // Early determination

    // Representatives for each cluster (first item found as representative)
    let mut representatives: Vec<usize> = Vec::new();
    let mut cluster_assignments: Vec<Option<usize>> = vec![None; n];

    // Phase 1: Process items one by one
    for i in 0..n {
        if is_core[i] {
            // Already determined as Core, skip detailed comparison
            // Just assign to nearest representative
            let mut best_rep = None;
            let mut best_sim = 0.0f64;

            for &rep in &representatives {
                comparisons += 1;
                let sim = cosine_similarity(&data[i], &data[rep]);
                if sim > best_sim && sim >= threshold {
                    best_sim = sim;
                    best_rep = Some(rep);
                }
            }

            cluster_assignments[i] = best_rep;
            continue;
        }

        // Compare with existing representatives first
        let mut found_similar_rep = false;
        for &rep in &representatives {
            comparisons += 1;
            let sim = cosine_similarity(&data[i], &data[rep]);
            if sim >= threshold {
                degrees[i] += 1;
                degrees[rep] += 1;

                // Check if rep becomes Core
                if degrees[rep] >= core_degree_threshold && !is_core[rep] {
                    is_core[rep] = true;
                }

                cluster_assignments[i] = Some(rep);
                found_similar_rep = true;

                // Check if this item becomes Core
                if degrees[i] >= core_degree_threshold {
                    is_core[i] = true;
                    break;  // Don't need more comparisons
                }
            }
        }

        // If not similar to any representative, it might be a new cluster
        if !found_similar_rep {
            representatives.push(i);
        }

        // If not yet Core, compare with non-representative items
        // But only if we need more information
        if !is_core[i] && degrees[i] < core_degree_threshold {
            for j in 0..i {
                if is_core[j] {
                    continue;  // Skip items already determined as Core
                }
                if cluster_assignments[j].is_some() && cluster_assignments[j] != cluster_assignments[i] {
                    continue;  // Different clusters, unlikely to be similar
                }

                comparisons += 1;
                let sim = cosine_similarity(&data[i], &data[j]);
                if sim >= threshold {
                    degrees[i] += 1;
                    degrees[j] += 1;

                    if degrees[i] >= core_degree_threshold {
                        is_core[i] = true;
                        break;
                    }
                    if degrees[j] >= core_degree_threshold && !is_core[j] {
                        is_core[j] = true;
                    }
                }
            }
        }
    }

    // Phase 2: Classify remaining items
    let avg_degree = if n > 0 {
        degrees.iter().sum::<usize>() as f64 / n as f64
    } else {
        0.0
    };

    let layers: Vec<Layer> = degrees.iter().enumerate().map(|(i, &deg)| {
        if is_core[i] || deg as f64 > avg_degree * 1.5 {
            Layer::Core
        } else if deg == 0 {
            Layer::Rare
        } else if (deg as f64) < avg_degree * 0.3 {
            Layer::Rare
        } else {
            Layer::Edge
        }
    }).collect();

    // Phase 3: Select representatives
    let mut selected = Vec::new();

    for (i, &layer) in layers.iter().enumerate() {
        match layer {
            Layer::Rare => selected.push(i),
            Layer::Edge => selected.push(i),
            Layer::Core => {
                // Only representatives from Core
                if representatives.contains(&i) {
                    selected.push(i);
                }
            }
        }
    }

    EarlyTerminationResult {
        selected,
        layers,
        comparisons,
    }
}

/// Simpler approach: Grid-based early termination
fn process_grid_early_termination(
    data: &[Vec<f64>],
    threshold: f64,
    grid_size: f64,
) -> EarlyTerminationResult {
    let n = data.len();
    if n == 0 {
        return EarlyTerminationResult {
            selected: vec![],
            layers: vec![],
            comparisons: 0,
        };
    }

    let dim = data[0].len();
    let mut comparisons = 0usize;

    // Assign items to grid cells (only first 3 dimensions for efficiency)
    let dims_to_use = dim.min(3);
    let mut grid: std::collections::HashMap<Vec<i32>, Vec<usize>> = std::collections::HashMap::new();

    for (i, point) in data.iter().enumerate() {
        let cell: Vec<i32> = point.iter()
            .take(dims_to_use)
            .map(|&x| (x / grid_size).floor() as i32)
            .collect();
        grid.entry(cell).or_default().push(i);
    }

    // Compute degrees only within same cell and adjacent cells
    let mut degrees = vec![0usize; n];

    for (cell, members) in &grid {
        // Within cell
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                comparisons += 1;
                if cosine_similarity(&data[members[i]], &data[members[j]]) >= threshold {
                    degrees[members[i]] += 1;
                    degrees[members[j]] += 1;
                }
            }
        }

        // Adjacent cells (26 neighbors in 3D)
        for d0 in -1i32..=1 {
            for d1 in -1i32..=1 {
                for d2 in -1i32..=1 {
                    if d0 == 0 && d1 == 0 && d2 == 0 {
                        continue;
                    }

                    let mut neighbor = cell.clone();
                    if neighbor.len() > 0 { neighbor[0] += d0; }
                    if neighbor.len() > 1 { neighbor[1] += d1; }
                    if neighbor.len() > 2 { neighbor[2] += d2; }

                    if let Some(neighbor_members) = grid.get(&neighbor) {
                        for &i in members {
                            for &j in neighbor_members {
                                if i < j {
                                    comparisons += 1;
                                    if cosine_similarity(&data[i], &data[j]) >= threshold {
                                        degrees[i] += 1;
                                        degrees[j] += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Classify
    let avg_degree = degrees.iter().sum::<usize>() as f64 / n.max(1) as f64;
    let layers: Vec<Layer> = degrees.iter().map(|&deg| {
        if deg == 0 {
            Layer::Rare
        } else if deg as f64 > avg_degree * 1.5 {
            Layer::Core
        } else if (deg as f64) < avg_degree * 0.3 {
            Layer::Rare
        } else {
            Layer::Edge
        }
    }).collect();

    let mut selected = Vec::new();
    let mut core_count = 0;
    for (i, &layer) in layers.iter().enumerate() {
        match layer {
            Layer::Rare => selected.push(i),
            Layer::Edge => selected.push(i),
            Layer::Core => {
                if core_count % 3 == 0 {
                    selected.push(i);
                }
                core_count += 1;
            }
        }
    }

    EarlyTerminationResult {
        selected,
        layers,
        comparisons,
    }
}

// ============================================================================
// Standard for comparison
// ============================================================================

fn process_standard(data: &[Vec<f64>], threshold: f64) -> (Vec<usize>, Vec<Layer>, usize) {
    let n = data.len();
    let comparisons = n * (n - 1) / 2;
    let kdf = Kdf::with_defaults();
    let result = kdf.process(data, threshold, |a, b| cosine_similarity(a, b));
    (result.selected.clone(), result.layers.clone(), comparisons)
}

// ============================================================================
// Data generators
// ============================================================================

fn generate_redundant(n: usize, dim: usize) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);

    // 80% cluster 1
    for i in 0..(n * 8 / 10) {
        let noise = (i as f64 * 0.001).sin() * 0.02;
        let point: Vec<f64> = (0..dim).map(|d| 0.5 + (d as f64 * 0.01) + noise).collect();
        data.push(point);
    }

    // 10% cluster 2
    for i in 0..(n / 10) {
        let noise = (i as f64 * 0.002).cos() * 0.02;
        let point: Vec<f64> = (0..dim).map(|d| -0.5 + (d as f64 * 0.01) + noise).collect();
        data.push(point);
    }

    // 10% rare
    for i in 0..(n / 10) {
        let mut point = vec![0.0; dim];
        let angle = (i as f64) * 0.3;
        point[0] = angle.cos() * 2.0;
        point[1] = angle.sin() * 2.0;
        data.push(point);
    }

    data
}

fn evaluate_rare_recall(true_layers: &[Layer], predicted_layers: &[Layer]) -> f64 {
    let true_rare: std::collections::HashSet<usize> = true_layers.iter()
        .enumerate()
        .filter(|(_, &l)| l == Layer::Rare)
        .map(|(i, _)| i)
        .collect();

    if true_rare.is_empty() {
        return 1.0;
    }

    let predicted_rare: std::collections::HashSet<usize> = predicted_layers.iter()
        .enumerate()
        .filter(|(_, &l)| l == Layer::Rare)
        .map(|(i, _)| i)
        .collect();

    true_rare.intersection(&predicted_rare).count() as f64 / true_rare.len() as f64
}

fn main() {
    println!("=== Early Termination + Representative Comparison ===\n");

    let threshold = 0.95;
    let dim = 20;

    println!("## 1. Benchmark\n");
    println!("| n | Method | Time | Comparisons | Reduction | Rare Recall |");
    println!("|---|--------|------|-------------|-----------|-------------|");

    let sizes = [500, 1000, 2000, 5000];

    for &n in &sizes {
        let data = generate_redundant(n, dim);
        let full_comp = n * (n - 1) / 2;

        // Standard
        let start = Instant::now();
        let (_, std_layers, std_comp) = process_standard(&data, threshold);
        let std_time = start.elapsed().as_secs_f64() * 1000.0;

        // Early termination
        let start = Instant::now();
        let et_result = process_early_termination(&data, threshold, 5);
        let et_time = start.elapsed().as_secs_f64() * 1000.0;
        let et_recall = evaluate_rare_recall(&std_layers, &et_result.layers);

        // Grid-based
        let start = Instant::now();
        let grid_result = process_grid_early_termination(&data, threshold, 0.3);
        let grid_time = start.elapsed().as_secs_f64() * 1000.0;
        let grid_recall = evaluate_rare_recall(&std_layers, &grid_result.layers);

        println!("| {} | Standard | {:.1}ms | {} | - | 100% |",
                 n, std_time, std_comp);
        println!("| {} | EarlyTerm | {:.1}ms | {} | {:.1}% | {:.1}% |",
                 n, et_time, et_result.comparisons,
                 (1.0 - et_result.comparisons as f64 / full_comp as f64) * 100.0,
                 et_recall * 100.0);
        println!("| {} | Grid | {:.1}ms | {} | {:.1}% | {:.1}% |",
                 n, grid_time, grid_result.comparisons,
                 (1.0 - grid_result.comparisons as f64 / full_comp as f64) * 100.0,
                 grid_recall * 100.0);
    }

    println!("\n## 2. Core Threshold Sensitivity (n=2000)\n");

    let n = 2000;
    let data = generate_redundant(n, dim);
    let (_, std_layers, _) = process_standard(&data, threshold);
    let full_comp = n * (n - 1) / 2;

    println!("| Core Threshold | Comparisons | Reduction | Rare Recall |");
    println!("|----------------|-------------|-----------|-------------|");

    for core_thresh in [3, 5, 10, 20, 50] {
        let result = process_early_termination(&data, threshold, core_thresh);
        let recall = evaluate_rare_recall(&std_layers, &result.layers);
        println!("| {} | {} | {:.1}% | {:.1}% |",
                 core_thresh, result.comparisons,
                 (1.0 - result.comparisons as f64 / full_comp as f64) * 100.0,
                 recall * 100.0);
    }

    println!("\n## 3. Conclusion\n");
    println!("早期終了の効果を確認...");
}
