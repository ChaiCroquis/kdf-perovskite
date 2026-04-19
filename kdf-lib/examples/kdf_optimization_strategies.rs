//! KDF Optimization Strategies: Alternative approaches
//!
//! Different perspectives to improve KDF efficiency:
//! 1. Early stopping: Stop when layer is determined
//! 2. k-NN graph: Only compute k nearest neighbors
//! 3. Dimension reduction: PCA before KDF
//! 4. LSH approximation: Hash-based neighbor finding
//! 5. Threshold pruning: Skip obviously similar/dissimilar pairs

use kdf::{Kdf, Layer};
use std::collections::{HashMap, HashSet};
use std::time::Instant;

/// Euclidean distance
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

/// Euclidean similarity
fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    1.0 / (1.0 + euclidean_distance(a, b))
}

// ============================================================================
// Strategy 1: Early Stopping KDF
// ============================================================================

/// Early stopping: Once we know an item's layer, skip remaining comparisons
fn early_stopping_kdf(
    data: &[Vec<f64>],
    sim_threshold: f64,
    core_degree_threshold: usize,  // degree >= this → Core
) -> (Vec<usize>, Vec<Layer>, f64, usize) {
    let start = Instant::now();
    let n = data.len();

    let mut degrees = vec![0usize; n];
    let mut layers = vec![Layer::Rare; n];  // Default to Rare
    let mut determined = vec![false; n];
    let mut comparisons = 0usize;

    // For each pair, check if we need to compare
    for i in 0..n {
        if determined[i] && layers[i] == Layer::Core {
            continue;  // Already determined as Core, skip
        }

        for j in (i + 1)..n {
            // Skip if both are already determined as Core
            if determined[i] && determined[j] &&
               layers[i] == Layer::Core && layers[j] == Layer::Core {
                continue;
            }

            comparisons += 1;
            let sim = euclidean_similarity(&data[i], &data[j]);

            if sim >= sim_threshold {
                degrees[i] += 1;
                degrees[j] += 1;

                // Check if we can determine layers early
                if degrees[i] >= core_degree_threshold && !determined[i] {
                    layers[i] = Layer::Core;
                    determined[i] = true;
                }
                if degrees[j] >= core_degree_threshold && !determined[j] {
                    layers[j] = Layer::Core;
                    determined[j] = true;
                }
            }
        }
    }

    // Finalize layers based on degrees
    let max_degree = degrees.iter().max().copied().unwrap_or(1).max(1);
    for i in 0..n {
        if !determined[i] {
            if degrees[i] == 0 {
                layers[i] = Layer::Rare;
            } else if degrees[i] as f64 / max_degree as f64 > 0.3 {
                layers[i] = Layer::Core;
            } else {
                layers[i] = Layer::Edge;
            }
        }
    }

    // Select: all Rare + representatives from Core/Edge
    let mut selected = Vec::new();
    let mut core_count = 0;

    for i in 0..n {
        match layers[i] {
            Layer::Rare => selected.push(i),
            Layer::Edge => selected.push(i),
            Layer::Core => {
                if core_count % 3 == 0 {  // Sample 1/3 of Core
                    selected.push(i);
                }
                core_count += 1;
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (selected, layers, elapsed, comparisons)
}

// ============================================================================
// Strategy 2: k-NN Graph KDF
// ============================================================================

/// k-NN based KDF: Only consider k nearest neighbors
fn knn_graph_kdf(
    data: &[Vec<f64>],
    k: usize,
    sim_threshold: f64,
) -> (Vec<usize>, Vec<Layer>, f64, usize) {
    let start = Instant::now();
    let n = data.len();
    let mut comparisons = 0usize;

    // Build k-NN for each point (still O(n²) but could be optimized with KD-tree)
    let mut knn: Vec<Vec<(usize, f64)>> = vec![Vec::new(); n];

    for i in 0..n {
        let mut distances: Vec<(usize, f64)> = Vec::new();
        for j in 0..n {
            if i != j {
                comparisons += 1;
                let dist = euclidean_distance(&data[i], &data[j]);
                distances.push((j, dist));
            }
        }
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        knn[i] = distances.into_iter().take(k).collect();
    }

    // Compute degrees based on k-NN graph
    let mut degrees = vec![0usize; n];
    for i in 0..n {
        for &(_j, dist) in &knn[i] {
            let sim = 1.0 / (1.0 + dist);
            if sim >= sim_threshold {
                degrees[i] += 1;
            }
        }
    }

    // Classify layers
    let max_degree = degrees.iter().max().copied().unwrap_or(1).max(1);
    let mut layers = vec![Layer::Rare; n];

    for i in 0..n {
        if degrees[i] == 0 {
            layers[i] = Layer::Rare;
        } else if degrees[i] as f64 / max_degree as f64 > 0.5 {
            layers[i] = Layer::Core;
        } else {
            layers[i] = Layer::Edge;
        }
    }

    // Select
    let mut selected = Vec::new();
    let mut core_sample = 0;

    for i in 0..n {
        match layers[i] {
            Layer::Rare => selected.push(i),
            Layer::Edge => selected.push(i),
            Layer::Core => {
                if core_sample % 3 == 0 {
                    selected.push(i);
                }
                core_sample += 1;
            }
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (selected, layers, elapsed, comparisons)
}

// ============================================================================
// Strategy 3: Dimension Reduction + KDF
// ============================================================================

/// Simple PCA-like dimension reduction (power iteration for top components)
fn reduce_dimensions(data: &[Vec<f64>], target_dim: usize) -> Vec<Vec<f64>> {
    let n = data.len();
    if n == 0 || data[0].len() <= target_dim {
        return data.to_vec();
    }

    let orig_dim = data[0].len();

    // Center data
    let mut means = vec![0.0; orig_dim];
    for point in data {
        for (d, &val) in point.iter().enumerate() {
            means[d] += val;
        }
    }
    for m in &mut means {
        *m /= n as f64;
    }

    let centered: Vec<Vec<f64>> = data.iter()
        .map(|p| p.iter().zip(&means).map(|(v, m)| v - m).collect())
        .collect();

    // Random projection (faster than true PCA, similar effect)
    let mut projection = vec![vec![0.0; orig_dim]; target_dim];
    for i in 0..target_dim {
        for j in 0..orig_dim {
            projection[i][j] = ((i * 7 + j * 13) % 100) as f64 / 100.0 - 0.5;
        }
        // Normalize
        let norm: f64 = projection[i].iter().map(|x| x * x).sum::<f64>().sqrt();
        for x in &mut projection[i] {
            *x /= norm;
        }
    }

    // Project data
    centered.iter()
        .map(|point| {
            projection.iter()
                .map(|proj| point.iter().zip(proj).map(|(a, b)| a * b).sum())
                .collect()
        })
        .collect()
}

fn dim_reduction_kdf(
    data: &[Vec<f64>],
    target_dim: usize,
    sim_threshold: f64,
) -> (Vec<usize>, f64) {
    let start = Instant::now();

    // Reduce dimensions
    let reduced = reduce_dimensions(data, target_dim);

    // Run standard KDF on reduced data
    let kdf = Kdf::with_defaults();
    let result = kdf.process(&reduced, sim_threshold, |a, b| euclidean_similarity(a, b));

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (result.selected.clone(), elapsed)
}

// ============================================================================
// Strategy 4: LSH-based Approximation
// ============================================================================

/// Simple LSH using random hyperplanes
fn lsh_hash(point: &[f64], hyperplanes: &[Vec<f64>]) -> u64 {
    let mut hash = 0u64;
    for (i, plane) in hyperplanes.iter().enumerate() {
        let dot: f64 = point.iter().zip(plane).map(|(a, b)| a * b).sum();
        if dot >= 0.0 {
            hash |= 1 << i;
        }
    }
    hash
}

fn lsh_kdf(
    data: &[Vec<f64>],
    n_hyperplanes: usize,
    sim_threshold: f64,
) -> (Vec<usize>, f64, usize) {
    let start = Instant::now();
    let n = data.len();
    if n == 0 {
        return (vec![], 0.0, 0);
    }

    let dim = data[0].len();
    let mut comparisons = 0usize;

    // Generate random hyperplanes
    let hyperplanes: Vec<Vec<f64>> = (0..n_hyperplanes)
        .map(|i| {
            (0..dim)
                .map(|j| ((i * 17 + j * 31) % 1000) as f64 / 1000.0 - 0.5)
                .collect()
        })
        .collect();

    // Hash all points
    let mut buckets: HashMap<u64, Vec<usize>> = HashMap::new();
    for (i, point) in data.iter().enumerate() {
        let hash = lsh_hash(point, &hyperplanes);
        buckets.entry(hash).or_default().push(i);
    }

    // Compute degrees only within same bucket + neighboring buckets
    let mut degrees = vec![0usize; n];

    for (hash, members) in &buckets {
        // Within bucket
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                comparisons += 1;
                let sim = euclidean_similarity(&data[members[i]], &data[members[j]]);
                if sim >= sim_threshold {
                    degrees[members[i]] += 1;
                    degrees[members[j]] += 1;
                }
            }
        }

        // Check neighboring buckets (1-bit difference)
        for bit in 0..n_hyperplanes {
            let neighbor_hash = hash ^ (1 << bit);
            if let Some(neighbors) = buckets.get(&neighbor_hash) {
                for &i in members {
                    for &j in neighbors {
                        if i < j {
                            comparisons += 1;
                            let sim = euclidean_similarity(&data[i], &data[j]);
                            if sim >= sim_threshold {
                                degrees[i] += 1;
                                degrees[j] += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    // Classify and select
    let max_degree = degrees.iter().max().copied().unwrap_or(1).max(1);
    let mut selected = Vec::new();
    let mut core_count = 0;

    for i in 0..n {
        if degrees[i] == 0 {
            selected.push(i);  // Rare
        } else if degrees[i] as f64 / max_degree as f64 <= 0.3 {
            selected.push(i);  // Edge
        } else {
            if core_count % 3 == 0 {
                selected.push(i);  // Sample Core
            }
            core_count += 1;
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (selected, elapsed, comparisons)
}

// ============================================================================
// Strategy 5: Threshold Pruning
// ============================================================================

/// Pruning: Skip pairs that are obviously too far apart
fn pruning_kdf(
    data: &[Vec<f64>],
    sim_threshold: f64,
) -> (Vec<usize>, f64, usize) {
    let start = Instant::now();
    let n = data.len();
    let mut comparisons = 0usize;

    // Precompute norms for triangle inequality pruning
    let norms: Vec<f64> = data.iter()
        .map(|p| p.iter().map(|x| x * x).sum::<f64>().sqrt())
        .collect();

    // Distance threshold (from similarity threshold)
    let dist_threshold = 1.0 / sim_threshold - 1.0;

    let mut degrees = vec![0usize; n];

    for i in 0..n {
        for j in (i + 1)..n {
            // Triangle inequality pruning
            let norm_diff = (norms[i] - norms[j]).abs();
            if norm_diff > dist_threshold {
                continue;  // Cannot be similar enough
            }

            comparisons += 1;
            let sim = euclidean_similarity(&data[i], &data[j]);

            if sim >= sim_threshold {
                degrees[i] += 1;
                degrees[j] += 1;
            }
        }
    }

    // Classify and select
    let max_degree = degrees.iter().max().copied().unwrap_or(1).max(1);
    let mut selected = Vec::new();
    let mut core_count = 0;

    for i in 0..n {
        if degrees[i] == 0 {
            selected.push(i);
        } else if degrees[i] as f64 / max_degree as f64 <= 0.3 {
            selected.push(i);
        } else {
            if core_count % 3 == 0 {
                selected.push(i);
            }
            core_count += 1;
        }
    }

    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (selected, elapsed, comparisons)
}

// ============================================================================
// Standard KDF for comparison
// ============================================================================

fn standard_kdf(data: &[Vec<f64>], sim_threshold: f64) -> (Vec<usize>, Vec<Layer>, f64) {
    let start = Instant::now();
    let kdf = Kdf::with_defaults();
    let result = kdf.process(data, sim_threshold, |a, b| euclidean_similarity(a, b));
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    (result.selected.clone(), result.layers.clone(), elapsed)
}

// ============================================================================
// Dataset
// ============================================================================

fn generate_dataset(n: usize, dim: usize) -> Vec<Vec<f64>> {
    let mut data = Vec::with_capacity(n);

    // 70% dense cluster
    for i in 0..(n * 7 / 10) {
        let mut point = vec![0.0; dim];
        for d in 0..dim {
            point[d] = 0.5 + ((i * (d + 1)) as f64 * 0.001).sin() * 0.1;
        }
        data.push(point);
    }

    // 20% secondary
    for i in 0..(n * 2 / 10) {
        let mut point = vec![0.0; dim];
        for d in 0..dim {
            point[d] = -0.5 + ((i * (d + 2)) as f64 * 0.002).cos() * 0.15;
        }
        data.push(point);
    }

    // 10% rare
    for i in 0..(n / 10) {
        let mut point = vec![0.0; dim];
        let angle = (i as f64) * 0.5;
        point[0] = angle.cos() * 0.9;
        point[1] = angle.sin() * 0.9;
        for d in 2..dim {
            point[d] = (i as f64) * 0.1 * ((d % 3) as f64 - 1.0);
        }
        data.push(point);
    }

    data
}

fn check_rare_preservation(layers_true: &[Layer], selected: &[usize]) -> f64 {
    let rare_indices: HashSet<usize> = layers_true.iter()
        .enumerate()
        .filter(|(_, &l)| l == Layer::Rare)
        .map(|(i, _)| i)
        .collect();

    if rare_indices.is_empty() {
        return 1.0;
    }

    let preserved = selected.iter().filter(|&&i| rare_indices.contains(&i)).count();
    preserved as f64 / rare_indices.len() as f64
}

fn main() {
    println!("=== KDF最適化戦略: 別の観点からの改善 ===\n");

    let dim = 10;
    let sim_threshold = 0.85;

    println!("## 1. 最適化戦略一覧\n");
    println!("   | # | 戦略 | アイデア | 期待効果 |");
    println!("   |---|------|----------|----------|");
    println!("   | 1 | 早期終了 | 層が決まったら計算スキップ | 比較回数削減 |");
    println!("   | 2 | k-NNグラフ | k近傍のみ考慮 | O(nk)に削減 |");
    println!("   | 3 | 次元削減 | 低次元で距離計算 | 距離計算高速化 |");
    println!("   | 4 | LSH | ハッシュで近傍探索 | O(n)期待 |");
    println!("   | 5 | 枝刈り | 三角不等式でスキップ | 比較回数削減 |");

    println!("\n## 2. ベンチマーク\n");

    let sizes = [500, 1000, 2000];

    for &n in &sizes {
        println!("### n = {}\n", n);

        let data = generate_dataset(n, dim);
        let full_comparisons = n * (n - 1) / 2;

        // Standard
        let (std_sel, std_layers, std_time) = standard_kdf(&data, sim_threshold);

        // Early stopping
        let (early_sel, _early_layers, early_time, early_comp) =
            early_stopping_kdf(&data, sim_threshold, 5);
        let early_rare = check_rare_preservation(&std_layers, &early_sel);

        // k-NN
        let k = (n as f64).sqrt() as usize;
        let (knn_sel, _knn_layers, knn_time, knn_comp) =
            knn_graph_kdf(&data, k, sim_threshold);
        let knn_rare = check_rare_preservation(&std_layers, &knn_sel);

        // Dimension reduction
        let (dim_sel, dim_time) = dim_reduction_kdf(&data, 3, sim_threshold);
        let dim_rare = check_rare_preservation(&std_layers, &dim_sel);

        // LSH
        let (lsh_sel, lsh_time, lsh_comp) = lsh_kdf(&data, 8, sim_threshold);
        let lsh_rare = check_rare_preservation(&std_layers, &lsh_sel);

        // Pruning
        let (prune_sel, prune_time, prune_comp) = pruning_kdf(&data, sim_threshold);
        let prune_rare = check_rare_preservation(&std_layers, &prune_sel);

        println!("   | 戦略 | 時間 | 比較回数 | 削減率 | 希少保持 | 選択数 |");
        println!("   |------|------|----------|--------|----------|--------|");
        println!("   | Standard | {:>5.1}ms | {:>8} | 100.0% | 100.0% | {:>6} |",
                 std_time, full_comparisons, std_sel.len());
        println!("   | 早期終了 | {:>5.1}ms | {:>8} | {:>5.1}% | {:>5.1}% | {:>6} |",
                 early_time, early_comp, early_comp as f64 / full_comparisons as f64 * 100.0,
                 early_rare * 100.0, early_sel.len());
        println!("   | k-NN | {:>5.1}ms | {:>8} | {:>5.1}% | {:>5.1}% | {:>6} |",
                 knn_time, knn_comp, knn_comp as f64 / full_comparisons as f64 * 100.0,
                 knn_rare * 100.0, knn_sel.len());
        println!("   | 次元削減 | {:>5.1}ms | {:>8} | - | {:>5.1}% | {:>6} |",
                 dim_time, full_comparisons, dim_rare * 100.0, dim_sel.len());
        println!("   | LSH | {:>5.1}ms | {:>8} | {:>5.1}% | {:>5.1}% | {:>6} |",
                 lsh_time, lsh_comp, lsh_comp as f64 / full_comparisons as f64 * 100.0,
                 lsh_rare * 100.0, lsh_sel.len());
        println!("   | 枝刈り | {:>5.1}ms | {:>8} | {:>5.1}% | {:>5.1}% | {:>6} |",
                 prune_time, prune_comp, prune_comp as f64 / full_comparisons as f64 * 100.0,
                 prune_rare * 100.0, prune_sel.len());
        println!();
    }

    // Best combination analysis
    println!("## 3. 最適組み合わせ分析 (n=2000)\n");

    let n = 2000;
    let data = generate_dataset(n, dim);
    let full_comp = n * (n - 1) / 2;

    let (_std_sel, std_layers, std_time) = standard_kdf(&data, sim_threshold);

    // Combination: Dimension Reduction + LSH
    let start = Instant::now();
    let reduced_data = reduce_dimensions(&data, 3);
    let (combo_sel, _, combo_lsh_comp) = lsh_kdf(&reduced_data, 10, sim_threshold);
    let combo_time = start.elapsed().as_secs_f64() * 1000.0;
    let combo_rare = check_rare_preservation(&std_layers, &combo_sel);

    println!("   組み合わせ: 次元削減(10→3) + LSH(10 hyperplanes)\n");
    println!("   | 指標 | Standard | 組み合わせ | 改善率 |");
    println!("   |------|----------|------------|--------|");
    println!("   | 時間 | {:>5.1}ms | {:>9.1}ms | {:>5.1}x |",
             std_time, combo_time, std_time / combo_time);
    println!("   | 比較回数 | {:>8} | {:>10} | {:>5.1}% |",
             full_comp, combo_lsh_comp, combo_lsh_comp as f64 / full_comp as f64 * 100.0);
    println!("   | 希少保持 | 100.0% | {:>9.1}% | - |", combo_rare * 100.0);

    println!("\n## 4. 各戦略の特性分析\n");

    println!("   | 戦略 | 長所 | 短所 | 推奨場面 |");
    println!("   |------|------|------|----------|");
    println!("   | 早期終了 | 実装簡単 | 効果限定的 | 密なデータ |");
    println!("   | k-NN | 理論的基盤 | k選択が難しい | 局所構造重視 |");
    println!("   | 次元削減 | 大幅高速化 | 情報損失 | 高次元データ |");
    println!("   | LSH | O(n)期待 | パラメータ依存 | 大規模データ |");
    println!("   | 枝刈り | 品質維持 | 効果はデータ依存 | 疎なデータ |");

    println!("\n## 5. 重要な発見\n");

    println!("   【比較回数削減の効果】");
    println!("   - LSH: 比較回数を大幅削減可能");
    println!("   - 枝刈り: データ構造により効果が変動");
    println!();
    println!("   【希少保持との両立】");
    println!("   - 枝刈り: 100%に近い希少保持を維持");
    println!("   - LSH: バケット設計で改善可能");
    println!();
    println!("   【実用的な推奨】");
    println!("   - 高次元: 次元削減 → KDF");
    println!("   - 大規模: LSH → クラスタ内KDF");
    println!("   - 品質重視: 枝刈り + Standard");

    println!("\n## 6. 理論と実測の差を埋める戦略\n");

    println!("   問題: 理論 k²倍 vs 実測 ~10倍");
    println!();
    println!("   解決策:");
    println!("   ┌─────────────────────────────────────────┐");
    println!("   │ 1. 前処理を高速化 (LSH/次元削減)         │");
    println!("   │ 2. 後続処理で希少を補完                  │");
    println!("   │ 3. 並列化で定数倍改善                    │");
    println!("   └─────────────────────────────────────────┘");
    println!();
    println!("   期待される総合効果:");
    println!("   - 前処理: O(n²) → O(n log n) [LSH]");
    println!("   - 後続処理: O(m²) → O(m²) [変わらず]");
    println!("   - 合計: k²倍に近づく");

    println!("\n✅ 最適化戦略実験完了");
}
