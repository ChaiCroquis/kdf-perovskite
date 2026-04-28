//! Hybrid Optimization: Representative-based with Rare preservation
//!
//! Key insight:
//! - Core detection: Only need enough neighbors → can early terminate
//! - Rare detection: Need to confirm NO neighbors → requires more checks
//!
//! Strategy:
//! 1. Build representative set incrementally
//! 2. Items similar to representatives → assign to cluster (skip further comparisons)
//! 3. Items NOT similar to any representative → potential Rare, check more carefully
//!
//! Run: cargo run --release --example kdf_hybrid_optimization

use kdf::{cosine_similarity, Kdf, Layer};
use std::time::Instant;

// ============================================================================
// Hybrid Optimization
// ============================================================================

#[allow(dead_code)]
struct HybridResult {
    layers: Vec<Layer>,
    comparisons: usize,
    representatives: Vec<usize>,
}

fn process_hybrid(
    data: &[Vec<f64>],
    threshold: f64,
    max_reps: usize, // Maximum number of representatives
) -> HybridResult {
    let n = data.len();
    if n == 0 {
        return HybridResult {
            layers: vec![],
            comparisons: 0,
            representatives: vec![],
        };
    }

    let mut comparisons = 0usize;
    let mut degrees = vec![0usize; n];

    // Representatives and their cluster members
    let mut representatives: Vec<usize> = Vec::new();
    let mut cluster_assignments: Vec<Option<usize>> = vec![None; n];
    let mut potential_rare: Vec<usize> = Vec::new();

    // Phase 1: Build representative set and identify potential rare items
    for i in 0..n {
        let mut found_similar = false;
        let mut best_sim = 0.0f64;
        let mut best_rep = None;

        // Compare with existing representatives
        for &rep in &representatives {
            comparisons += 1;
            let sim = cosine_similarity(&data[i], &data[rep]);

            if sim >= threshold {
                found_similar = true;
                if sim > best_sim {
                    best_sim = sim;
                    best_rep = Some(rep);
                }
            }
        }

        if found_similar {
            // Assign to best matching representative
            cluster_assignments[i] = best_rep;
            if let Some(rep) = best_rep {
                degrees[i] += 1;
                degrees[rep] += 1;
            }
        } else {
            // Not similar to any representative
            if representatives.len() < max_reps {
                // Become a new representative
                representatives.push(i);
            } else {
                // Too many reps, mark as potential rare
                potential_rare.push(i);
            }
        }
    }

    // Phase 2: Verify potential rare items (need to check they're truly isolated)
    for &i in &potential_rare {
        // Check against ALL other items (not just representatives)
        // But we can use early termination once we find ANY neighbor
        let mut found_neighbor = false;

        // First check cluster members (likely to be similar)
        for &rep in &representatives {
            comparisons += 1;
            if cosine_similarity(&data[i], &data[rep]) >= threshold {
                degrees[i] += 1;
                degrees[rep] += 1;
                cluster_assignments[i] = Some(rep);
                found_neighbor = true;
                break;
            }
        }

        if !found_neighbor {
            // Check other potential rare items
            for &j in &potential_rare {
                if i >= j {
                    continue;
                }
                comparisons += 1;
                if cosine_similarity(&data[i], &data[j]) >= threshold {
                    degrees[i] += 1;
                    degrees[j] += 1;
                    break;
                }
            }
        }
    }

    // Phase 3: Refine clusters - compare within clusters for accurate degrees
    for &rep in &representatives {
        let members: Vec<usize> = (0..n)
            .filter(|&i| cluster_assignments[i] == Some(rep))
            .collect();

        // Sample within-cluster comparisons (not all pairs)
        let sample_size = members.len().min(10); // Compare up to 10 pairs
        for i in 0..sample_size.min(members.len()) {
            for j in (i + 1)..sample_size.min(members.len()) {
                if members[i] < n && members[j] < n {
                    comparisons += 1;
                    if cosine_similarity(&data[members[i]], &data[members[j]]) >= threshold {
                        degrees[members[i]] += 1;
                        degrees[members[j]] += 1;
                    }
                }
            }
        }
    }

    // Phase 4: Classify layers
    let avg_degree = if n > 0 {
        degrees.iter().sum::<usize>() as f64 / n as f64
    } else {
        0.0
    };

    let layers: Vec<Layer> = degrees
        .iter()
        .map(|&deg| {
            if deg == 0 {
                Layer::Rare
            } else if (deg as f64) > avg_degree * 1.5 {
                Layer::Core
            } else if (deg as f64) < avg_degree * 0.3 {
                Layer::Rare
            } else {
                Layer::Edge
            }
        })
        .collect();

    HybridResult {
        layers,
        comparisons,
        representatives,
    }
}

// ============================================================================
// Incremental Clustering Approach
// ============================================================================

fn process_incremental_clustering(data: &[Vec<f64>], threshold: f64) -> HybridResult {
    let n = data.len();
    if n == 0 {
        return HybridResult {
            layers: vec![],
            comparisons: 0,
            representatives: vec![],
        };
    }

    let mut comparisons = 0usize;
    let mut degrees = vec![0usize; n];

    // Clusters: representative -> members
    let mut clusters: Vec<(usize, Vec<usize>)> = Vec::new();

    for i in 0..n {
        let mut best_cluster = None;
        let mut best_sim = threshold;

        // Find best matching cluster
        for (cluster_idx, (rep, _)) in clusters.iter().enumerate() {
            comparisons += 1;
            let sim = cosine_similarity(&data[i], &data[*rep]);
            if sim >= best_sim {
                best_sim = sim;
                best_cluster = Some(cluster_idx);
            }
        }

        if let Some(cluster_idx) = best_cluster {
            // Add to existing cluster
            clusters[cluster_idx].1.push(i);
            degrees[i] += 1;
            degrees[clusters[cluster_idx].0] += 1;

            // Compare with other members in cluster (sample)
            let members = &clusters[cluster_idx].1;
            let sample = members.len().min(5);
            for &member in members.iter().rev().take(sample) {
                if member != i {
                    comparisons += 1;
                    if cosine_similarity(&data[i], &data[member]) >= threshold {
                        degrees[i] += 1;
                        degrees[member] += 1;
                    }
                }
            }
        } else {
            // Start new cluster
            clusters.push((i, vec![i]));
        }
    }

    // Classify
    let avg_degree = degrees.iter().sum::<usize>() as f64 / n.max(1) as f64;
    let layers: Vec<Layer> = degrees
        .iter()
        .map(|&deg| {
            if deg == 0 {
                Layer::Rare
            } else if (deg as f64) > avg_degree * 1.5 {
                Layer::Core
            } else if (deg as f64) < avg_degree * 0.3 {
                Layer::Rare
            } else {
                Layer::Edge
            }
        })
        .collect();

    let representatives: Vec<usize> = clusters.iter().map(|(rep, _)| *rep).collect();

    HybridResult {
        layers,
        comparisons,
        representatives,
    }
}

// ============================================================================
// Standard
// ============================================================================

fn process_standard(data: &[Vec<f64>], threshold: f64) -> (Vec<Layer>, usize) {
    let n = data.len();
    let comparisons = n * (n - 1) / 2;
    let kdf = Kdf::with_defaults();
    let result = kdf.process(data, threshold, |a, b| cosine_similarity(a, b));
    (result.layers.clone(), comparisons)
}

// ============================================================================
// Data generators
// ============================================================================

fn generate_data(n: usize, dim: usize, redundancy: f64) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);
    let n_redundant = (n as f64 * redundancy) as usize;
    let n_rare = n - n_redundant;

    // Redundant cluster
    for i in 0..n_redundant {
        let noise = (i as f64 * 0.001).sin() * 0.02;
        let point: Vec<f64> = (0..dim).map(|d| 0.5 + (d as f64 * 0.01) + noise).collect();
        data.push(point);
    }

    // Rare items (spread out)
    for i in 0..n_rare {
        let angle = (i as f64) * std::f64::consts::PI * 2.0 / n_rare as f64;
        let mut point = vec![0.0; dim];
        point[0] = angle.cos() * 5.0;
        point[1] = angle.sin() * 5.0;
        point[2] = (i as f64 * 0.1).sin();
        data.push(point);
    }

    data
}

fn evaluate_rare_recall(true_layers: &[Layer], predicted_layers: &[Layer]) -> f64 {
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
    println!("=== Hybrid Optimization Benchmark ===\n");

    let threshold = 0.95;
    let dim = 20;

    println!("## 1. Different Redundancy Levels (n=2000)\n");
    println!("| Redundancy | Method | Time | Comparisons | Reduction | Rare Recall |");
    println!("|------------|--------|------|-------------|-----------|-------------|");

    let n = 2000;

    for &redundancy in &[0.5, 0.7, 0.9, 0.95] {
        let data = generate_data(n, dim, redundancy);
        let full_comp = n * (n - 1) / 2;

        // Standard
        let start = Instant::now();
        let (std_layers, _) = process_standard(&data, threshold);
        let std_time = start.elapsed().as_secs_f64() * 1000.0;

        // Hybrid
        let start = Instant::now();
        let hybrid = process_hybrid(&data, threshold, 100);
        let hybrid_time = start.elapsed().as_secs_f64() * 1000.0;
        let hybrid_recall = evaluate_rare_recall(&std_layers, &hybrid.layers);

        // Incremental clustering
        let start = Instant::now();
        let incr = process_incremental_clustering(&data, threshold);
        let incr_time = start.elapsed().as_secs_f64() * 1000.0;
        let incr_recall = evaluate_rare_recall(&std_layers, &incr.layers);

        println!(
            "| {:.0}% | Standard | {:.1}ms | {} | - | 100% |",
            redundancy * 100.0,
            std_time,
            full_comp
        );
        println!(
            "| {:.0}% | Hybrid | {:.1}ms | {} | {:.1}% | {:.1}% |",
            redundancy * 100.0,
            hybrid_time,
            hybrid.comparisons,
            (1.0 - hybrid.comparisons as f64 / full_comp as f64) * 100.0,
            hybrid_recall * 100.0
        );
        println!(
            "| {:.0}% | IncrCluster | {:.1}ms | {} | {:.1}% | {:.1}% |",
            redundancy * 100.0,
            incr_time,
            incr.comparisons,
            (1.0 - incr.comparisons as f64 / full_comp as f64) * 100.0,
            incr_recall * 100.0
        );
    }

    println!("\n## 2. Scaling Test (90% redundancy)\n");
    println!("| n | Standard | Hybrid | IncrCluster | Hybrid Speedup | Incr Speedup |");
    println!("|---|----------|--------|-------------|----------------|--------------|");

    for &n in &[500, 1000, 2000, 5000] {
        let data = generate_data(n, dim, 0.9);

        let start = Instant::now();
        let _ = process_standard(&data, threshold);
        let std_time = start.elapsed().as_secs_f64() * 1000.0;

        let start = Instant::now();
        let _ = process_hybrid(&data, threshold, 100);
        let hybrid_time = start.elapsed().as_secs_f64() * 1000.0;

        let start = Instant::now();
        let _ = process_incremental_clustering(&data, threshold);
        let incr_time = start.elapsed().as_secs_f64() * 1000.0;

        println!(
            "| {} | {:.1}ms | {:.1}ms | {:.1}ms | {:.1}x | {:.1}x |",
            n,
            std_time,
            hybrid_time,
            incr_time,
            std_time / hybrid_time,
            std_time / incr_time
        );
    }

    println!("\n## 3. Conclusion\n");
    println!("┌────────────────────────────────────────────────────────────┐");
    println!("│ 最適化手法の比較:                                          │");
    println!("│                                                            │");
    println!("│ Incremental Clustering:                                    │");
    println!("│   - 代表点との比較のみ → O(n × k) where k = cluster count  │");
    println!("│   - 冗長データで効果的                                     │");
    println!("│   - Rare検出精度が課題                                     │");
    println!("│                                                            │");
    println!("│ 実装推奨:                                                  │");
    println!("│   process_fast() - n > threshold でクラスタリングベース    │");
    println!("│   Rare候補は追加検証で精度向上                             │");
    println!("└────────────────────────────────────────────────────────────┘");
}
