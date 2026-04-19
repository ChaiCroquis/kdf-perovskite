//! KDF Performance Benchmark
//!
//! Detailed measurements:
//! 1. Time per operation
//! 2. Throughput (items/sec)
//! 3. Memory usage estimation
//! 4. Phase-by-phase breakdown

use rand::Rng;
use std::time::Instant;

#[derive(Clone)]
struct DataItem {
    features: Vec<f64>,
    is_rare: bool,
}

impl DataItem {
    fn new(features: Vec<f64>, is_rare: bool) -> Self {
        Self { features, is_rare }
    }

    fn similarity(&self, other: &DataItem) -> f64 {
        let dot: f64 = self.features.iter().zip(&other.features).map(|(a, b)| a * b).sum();
        let mag1: f64 = self.features.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2: f64 = other.features.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag1 == 0.0 || mag2 == 0.0 { return 0.0; }
        dot / (mag1 * mag2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layer { Core, Edge, Rare }

struct KdfParams {
    alpha_edge: f64,
    alpha_rare: f64,
    alpha_core: f64,
    theta_edge: f64,
    beta: f64,
    gamma: f64,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            alpha_edge: 1.5,
            alpha_rare: 0.3,
            alpha_core: 2.0,
            theta_edge: 0.15,
            beta: 0.01,
            gamma: 0.1,
        }
    }
}

#[derive(Default)]
struct BenchmarkResult {
    n: usize,
    dim: usize,
    // Time measurements (microseconds)
    graph_build_us: u128,
    layer_classify_us: u128,
    decay_iter_us: u128,
    selection_us: u128,
    total_us: u128,
    // Counts
    edge_count: usize,
    similarity_calls: usize,
    // Throughput
    items_per_sec: f64,
    comparisons_per_sec: f64,
    // Memory (bytes)
    data_memory: usize,
    degree_memory: usize,
    weight_memory: usize,
    total_memory: usize,
}

fn benchmark_kdf(items: &[DataItem], sim_threshold: f64) -> BenchmarkResult {
    let params = KdfParams::default();
    let n = items.len();
    let dim = if n > 0 { items[0].features.len() } else { 0 };

    let total_start = Instant::now();
    let mut result = BenchmarkResult::default();
    result.n = n;
    result.dim = dim;

    // Phase 1: Graph construction
    let graph_start = Instant::now();
    let mut degrees = vec![0usize; n];
    let mut edge_count = 0usize;
    let mut similarity_calls = 0usize;

    for i in 0..n {
        for j in (i + 1)..n {
            similarity_calls += 1;
            if items[i].similarity(&items[j]) >= sim_threshold {
                degrees[i] += 1;
                degrees[j] += 1;
                edge_count += 1;
            }
        }
    }
    result.graph_build_us = graph_start.elapsed().as_micros();
    result.edge_count = edge_count;
    result.similarity_calls = similarity_calls;

    // Phase 2: Layer classification
    let classify_start = Instant::now();
    let avg_degree: f64 = degrees.iter().sum::<usize>() as f64 / n.max(1) as f64;
    let mut layers = vec![Layer::Edge; n];
    for i in 0..n {
        let deg = degrees[i];
        if deg == 0 {
            layers[i] = Layer::Rare;
        } else if (deg as f64) > avg_degree * 1.5 {
            layers[i] = Layer::Core;
        } else if (deg as f64) < avg_degree * 0.3 {
            layers[i] = Layer::Rare;
        }
    }
    result.layer_classify_us = classify_start.elapsed().as_micros();

    // Phase 3: Decay iteration
    let decay_start = Instant::now();
    let mut weights = vec![1.0f64; n];
    for _ in 0..100 {
        for i in 0..n {
            let c = degrees[i] as f64;
            let alpha = match layers[i] {
                Layer::Core => params.alpha_core,
                Layer::Edge => params.alpha_edge,
                Layer::Rare => params.alpha_rare,
            };
            let decay_rate = params.beta * (1.0 + params.gamma * c.powf(alpha));
            weights[i] *= (1.0 - decay_rate).max(0.0);
        }
    }
    result.decay_iter_us = decay_start.elapsed().as_micros();

    // Phase 4: Selection
    let select_start = Instant::now();
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|a, b| weights[*b].partial_cmp(&weights[*a]).unwrap());

    let mut selected: Vec<usize> = Vec::new();
    for &i in &indices {
        if layers[i] == Layer::Rare {
            selected.push(i);
        } else if weights[i] >= params.theta_edge {
            let has_similar = selected.iter()
                .any(|&s| items[i].similarity(&items[s]) >= 0.75);
            if !has_similar {
                selected.push(i);
            }
        }
    }
    if selected.is_empty() && !indices.is_empty() {
        selected.push(indices[0]);
    }
    result.selection_us = select_start.elapsed().as_micros();

    result.total_us = total_start.elapsed().as_micros();

    // Calculate throughput
    let total_secs = result.total_us as f64 / 1_000_000.0;
    result.items_per_sec = n as f64 / total_secs;
    result.comparisons_per_sec = similarity_calls as f64 / total_secs;

    // Memory estimation
    // DataItem: Vec<f64> (24 + dim*8) + bool (1) ≈ 25 + dim*8 bytes
    result.data_memory = n * (25 + dim * 8);
    result.degree_memory = n * 8; // usize
    result.weight_memory = n * 8; // f64
    result.total_memory = result.data_memory + result.degree_memory + result.weight_memory + n; // +n for layers

    result
}

fn generate_dataset(cluster_count: usize, items_per_cluster: usize, rare_count: usize, dim: usize) -> Vec<DataItem> {
    let mut rng = rand::thread_rng();
    let mut items = Vec::new();

    for _c in 0..cluster_count {
        let center: Vec<f64> = (0..dim).map(|_| rng.gen::<f64>() + 0.5).collect();
        for _ in 0..items_per_cluster {
            let features: Vec<f64> = center.iter()
                .map(|&v| v + (rng.gen::<f64>() * 0.1 - 0.05))
                .collect();
            items.push(DataItem::new(features, false));
        }
    }

    for r in 0..rare_count {
        let mut features = vec![0.0; dim];
        let dominant_dim = r % dim;
        features[dominant_dim] = -1.0 - (r as f64 * 0.01);
        for d in 0..dim {
            if d != dominant_dim {
                features[d] = rng.gen::<f64>() * 0.1;
            }
        }
        items.push(DataItem::new(features, true));
    }

    items
}

fn format_time(us: u128) -> String {
    if us >= 1_000_000 {
        format!("{:.2}s", us as f64 / 1_000_000.0)
    } else if us >= 1_000 {
        format!("{:.2}ms", us as f64 / 1_000.0)
    } else {
        format!("{}μs", us)
    }
}

fn format_memory(bytes: usize) -> String {
    if bytes >= 1_073_741_824 {
        format!("{:.2}GB", bytes as f64 / 1_073_741_824.0)
    } else if bytes >= 1_048_576 {
        format!("{:.2}MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.2}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

fn format_rate(rate: f64) -> String {
    if rate >= 1_000_000.0 {
        format!("{:.2}M/s", rate / 1_000_000.0)
    } else if rate >= 1_000.0 {
        format!("{:.2}K/s", rate / 1_000.0)
    } else {
        format!("{:.2}/s", rate)
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║             KDF パフォーマンスベンチマーク                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================
    // Test 1: Phase timing breakdown
    // ========================================
    println!("══════════════════════════════════════════════════════════════");
    println!("【検証1】処理フェーズ別時間計測");
    println!("══════════════════════════════════════════════════════════════\n");

    let test_sizes = [
        (10, 100, 10, "1K"),
        (50, 100, 50, "5K"),
        (100, 100, 100, "10K"),
    ];

    for (clusters, per_cluster, rare, label) in test_sizes {
        let data = generate_dataset(clusters, per_cluster, rare, 8);
        let r = benchmark_kdf(&data, 0.95);

        println!("【{}件 (dim={})】", label, r.dim);
        println!("  フェーズ別時間:");
        println!("    ・グラフ構築:   {:>10} ({:.1}%)",
            format_time(r.graph_build_us),
            r.graph_build_us as f64 / r.total_us as f64 * 100.0);
        println!("    ・レイヤー分類: {:>10} ({:.1}%)",
            format_time(r.layer_classify_us),
            r.layer_classify_us as f64 / r.total_us as f64 * 100.0);
        println!("    ・減衰計算:     {:>10} ({:.1}%)",
            format_time(r.decay_iter_us),
            r.decay_iter_us as f64 / r.total_us as f64 * 100.0);
        println!("    ・選択処理:     {:>10} ({:.1}%)",
            format_time(r.selection_us),
            r.selection_us as f64 / r.total_us as f64 * 100.0);
        println!("    ────────────────────────────");
        println!("    ・合計:         {:>10}", format_time(r.total_us));
        println!();
    }

    // ========================================
    // Test 2: Throughput measurement
    // ========================================
    println!("══════════════════════════════════════════════════════════════");
    println!("【検証2】スループット計測");
    println!("══════════════════════════════════════════════════════════════\n");

    println!("{:<10} {:>12} {:>15} {:>15}",
        "規模", "処理時間", "アイテム/秒", "比較/秒");
    println!("{}", "─".repeat(55));

    let throughput_sizes = [
        (10, 100, 10),
        (20, 100, 20),
        (50, 100, 50),
        (100, 100, 100),
    ];

    for (clusters, per_cluster, rare) in throughput_sizes {
        let data = generate_dataset(clusters, per_cluster, rare, 8);
        let r = benchmark_kdf(&data, 0.95);

        println!("{:<10} {:>12} {:>15} {:>15}",
            format!("{}件", r.n),
            format_time(r.total_us),
            format_rate(r.items_per_sec),
            format_rate(r.comparisons_per_sec));
    }

    // ========================================
    // Test 3: Memory usage
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証3】メモリ使用量計測");
    println!("══════════════════════════════════════════════════════════════\n");

    println!("{:<10} {:>12} {:>12} {:>12} {:>12}",
        "規模", "データ", "度数配列", "重み配列", "合計");
    println!("{}", "─".repeat(60));

    for (clusters, per_cluster, rare) in throughput_sizes {
        let data = generate_dataset(clusters, per_cluster, rare, 8);
        let r = benchmark_kdf(&data, 0.95);

        println!("{:<10} {:>12} {:>12} {:>12} {:>12}",
            format!("{}件", r.n),
            format_memory(r.data_memory),
            format_memory(r.degree_memory),
            format_memory(r.weight_memory),
            format_memory(r.total_memory));
    }

    // ========================================
    // Test 4: Dimension impact
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証4】次元数の影響");
    println!("══════════════════════════════════════════════════════════════\n");

    let dimensions = [4, 8, 16, 32, 64, 128];

    println!("{:<8} {:>12} {:>15} {:>12}",
        "次元", "処理時間", "類似度計算/秒", "メモリ");
    println!("{}", "─".repeat(50));

    for dim in dimensions {
        let data = generate_dataset(10, 100, 10, dim); // 1010 items
        let r = benchmark_kdf(&data, 0.95);

        let sim_per_sec = r.similarity_calls as f64 / (r.graph_build_us as f64 / 1_000_000.0);

        println!("{:<8} {:>12} {:>15} {:>12}",
            dim,
            format_time(r.total_us),
            format_rate(sim_per_sec),
            format_memory(r.total_memory));
    }

    // ========================================
    // Test 5: Complexity verification
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証5】計算複雑度の確認");
    println!("══════════════════════════════════════════════════════════════\n");

    let complexity_sizes = [500, 1000, 2000, 4000];
    let mut timings: Vec<(usize, u128)> = Vec::new();

    println!("{:<10} {:>12} {:>15} {:>12}",
        "n", "時間", "n²", "ns/比較");
    println!("{}", "─".repeat(50));

    for n in complexity_sizes {
        let clusters = n / 10;
        let per_cluster = 10;
        let rare = n / 100;

        let data = generate_dataset(clusters, per_cluster, rare, 8);
        let r = benchmark_kdf(&data, 0.95);

        let n_squared = (r.n as u128) * (r.n as u128);
        let ns_per_comparison = (r.graph_build_us * 1000) as f64 / r.similarity_calls as f64;

        println!("{:<10} {:>12} {:>15} {:>10.2}ns",
            r.n,
            format_time(r.total_us),
            n_squared,
            ns_per_comparison);

        timings.push((r.n, r.total_us));
    }

    // Verify O(n²)
    if timings.len() >= 2 {
        let (n1, t1) = timings[0];
        let (n2, t2) = timings[timings.len() - 1];
        let n_ratio = n2 as f64 / n1 as f64;
        let t_ratio = t2 as f64 / t1 as f64;
        let expected = n_ratio.powi(2);

        println!();
        println!("O(n²)検証:");
        println!("  n倍率: {:.1}x ({} → {})", n_ratio, n1, n2);
        println!("  時間倍率: {:.1}x", t_ratio);
        println!("  理論倍率: {:.1}x", expected);
        println!("  実測/理論: {:.2}", t_ratio / expected);
    }

    // ========================================
    // Summary
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【性能サマリ】");
    println!("══════════════════════════════════════════════════════════════\n");

    // Get 10K benchmark
    let data_10k = generate_dataset(100, 100, 100, 8);
    let r_10k = benchmark_kdf(&data_10k, 0.95);

    println!("┌─────────────────────────────────────────────────────────────┐");
    println!("│ KDF Rev.12 性能特性（10K件基準）                            │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ 処理時間:        {:>12}                              │", format_time(r_10k.total_us));
    println!("│ スループット:    {:>12}                              │", format_rate(r_10k.items_per_sec));
    println!("│ 比較速度:        {:>12}                              │", format_rate(r_10k.comparisons_per_sec));
    println!("│ メモリ使用:      {:>12}                              │", format_memory(r_10k.total_memory));
    println!("│ 計算量:          O(n²)                                     │");
    println!("├─────────────────────────────────────────────────────────────┤");
    println!("│ ボトルネック:    グラフ構築 ({:.0}%)                          │",
        r_10k.graph_build_us as f64 / r_10k.total_us as f64 * 100.0);
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n【証明された事項】");
    println!("  47. 10K件処理時間: {}", format_time(r_10k.total_us));
    println!("  48. スループット: {}", format_rate(r_10k.items_per_sec));
    println!("  49. メモリ効率: {}（10K件）", format_memory(r_10k.total_memory));
    println!("  50. ボトルネック: グラフ構築フェーズ（O(n²)類似度計算）");
    println!();
}
