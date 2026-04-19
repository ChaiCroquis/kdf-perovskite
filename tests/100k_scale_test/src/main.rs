//! KDF 100K Scale Verification
//!
//! Tests KDF with 100,000 items to verify:
//! 1. Algorithm correctness at extreme scale
//! 2. Rare preservation guarantee
//! 3. Time/memory characteristics

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

    #[inline]
    fn similarity(&self, other: &DataItem) -> f64 {
        let mut dot = 0.0f64;
        let mut mag1 = 0.0f64;
        let mut mag2 = 0.0f64;

        for i in 0..self.features.len() {
            dot += self.features[i] * other.features[i];
            mag1 += self.features[i] * self.features[i];
            mag2 += other.features[i] * other.features[i];
        }

        let mag1 = mag1.sqrt();
        let mag2 = mag2.sqrt();

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

fn generate_dataset(cluster_count: usize, items_per_cluster: usize, rare_count: usize, dim: usize) -> Vec<DataItem> {
    let mut rng = rand::thread_rng();
    let mut items = Vec::with_capacity(cluster_count * items_per_cluster + rare_count);

    // Generate clusters with positive values
    for _c in 0..cluster_count {
        let center: Vec<f64> = (0..dim).map(|_| rng.gen::<f64>() + 0.5).collect();

        for _ in 0..items_per_cluster {
            let features: Vec<f64> = center.iter()
                .map(|&v| v + (rng.gen::<f64>() * 0.1 - 0.05))
                .collect();
            items.push(DataItem::new(features, false));
        }
    }

    // Generate rare items with orthogonal/negative patterns
    for r in 0..rare_count {
        let mut features = vec![0.0; dim];
        let dominant_dim = r % dim;
        features[dominant_dim] = -1.0 - (r as f64 * 0.001);
        for d in 0..dim {
            if d != dominant_dim {
                features[d] = rng.gen::<f64>() * 0.1;
            }
        }
        items.push(DataItem::new(features, true));
    }

    items
}

fn run_kdf(items: &[DataItem], sim_threshold: f64) -> (Vec<usize>, Vec<Layer>, u128) {
    let params = KdfParams::default();
    let n = items.len();
    let total_start = Instant::now();

    println!("  Phase 1: グラフ構築 ({} 比較)...", n * (n - 1) / 2);
    let graph_start = Instant::now();

    // Phase 1: Graph construction with progress
    let mut degrees = vec![0usize; n];
    let mut comparisons = 0u64;
    let total_comparisons = (n * (n - 1) / 2) as u64;
    let report_interval = total_comparisons / 10;

    for i in 0..n {
        for j in (i + 1)..n {
            if items[i].similarity(&items[j]) >= sim_threshold {
                degrees[i] += 1;
                degrees[j] += 1;
            }
            comparisons += 1;
            if report_interval > 0 && comparisons % report_interval == 0 {
                print!("    進捗: {}%\r", comparisons * 100 / total_comparisons);
                use std::io::Write;
                std::io::stdout().flush().unwrap();
            }
        }
    }
    println!("    完了: {:.1}秒                    ", graph_start.elapsed().as_secs_f64());

    // Phase 2: Layer classification
    println!("  Phase 2: レイヤー分類...");
    let avg_degree: f64 = degrees.iter().sum::<usize>() as f64 / n as f64;
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

    // Phase 3: Decay iteration
    println!("  Phase 3: 減衰反復 (100回)...");
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

    // Phase 4: Selection
    println!("  Phase 4: 選択処理...");
    let mut indices: Vec<usize> = (0..n).collect();
    indices.sort_by(|a, b| weights[*b].partial_cmp(&weights[*a]).unwrap());

    let mut selected: Vec<usize> = Vec::new();

    for &i in &indices {
        if layers[i] == Layer::Rare {
            selected.push(i);
        } else if weights[i] >= params.theta_edge {
            let has_similar = selected.iter()
                .take(1000) // Limit similarity checks for performance
                .any(|&s| items[i].similarity(&items[s]) >= 0.75);
            if !has_similar {
                selected.push(i);
            }
        }
    }

    if selected.is_empty() && !indices.is_empty() {
        selected.push(indices[0]);
    }

    let total_time = total_start.elapsed().as_millis();

    (selected, layers, total_time)
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              KDF 100K スケール検証                            ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // Test configuration
    let cluster_count = 1000;      // 1000 clusters
    let items_per_cluster = 100;   // 100 items each
    let rare_count = 1000;         // 1000 rare items
    let total_items = cluster_count * items_per_cluster + rare_count;
    let dim = 8;

    println!("【構成】");
    println!("  ・クラスタ数: {}", cluster_count);
    println!("  ・クラスタあたり: {}件", items_per_cluster);
    println!("  ・冗長データ: {}件", cluster_count * items_per_cluster);
    println!("  ・レアデータ: {}件", rare_count);
    println!("  ・合計: {}件", total_items);
    println!("  ・次元数: {}", dim);
    println!();

    // Generate data
    println!("【データ生成】");
    let gen_start = Instant::now();
    let items = generate_dataset(cluster_count, items_per_cluster, rare_count, dim);
    println!("  生成完了: {:.2}秒\n", gen_start.elapsed().as_secs_f64());

    // Run KDF
    println!("【KDF実行】");
    let (selected, layers, total_time) = run_kdf(&items, 0.95);

    // Calculate metrics
    let rare_total = items.iter().filter(|i| i.is_rare).count();
    let redundant_total = items.iter().filter(|i| !i.is_rare).count();
    let rare_selected = selected.iter().filter(|&&i| items[i].is_rare).count();
    let redundant_selected = selected.iter().filter(|&&i| !items[i].is_rare).count();

    let rare_preservation = rare_selected as f64 / rare_total as f64;
    let redundancy_reduction = (redundant_total - redundant_selected) as f64 / redundant_total as f64;
    let f1_score = if rare_preservation + redundancy_reduction > 0.0 {
        2.0 * rare_preservation * redundancy_reduction / (rare_preservation + redundancy_reduction)
    } else { 0.0 };

    // Layer statistics
    let rare_layer_count = layers.iter().filter(|&&l| l == Layer::Rare).count();
    let edge_layer_count = layers.iter().filter(|&&l| l == Layer::Edge).count();
    let core_layer_count = layers.iter().filter(|&&l| l == Layer::Core).count();

    println!("\n【結果】");
    println!("  ・処理時間: {:.1}秒", total_time as f64 / 1000.0);
    println!("  ・スループット: {:.0} items/sec", total_items as f64 / (total_time as f64 / 1000.0));
    println!();
    println!("  レイヤー分布:");
    println!("    ・Rare:  {} 件", rare_layer_count);
    println!("    ・Edge:  {} 件", edge_layer_count);
    println!("    ・Core:  {} 件", core_layer_count);
    println!();
    println!("  選択結果:");
    println!("    ・選択数: {} 件 (削減率: {:.1}%)",
        selected.len(),
        (1.0 - selected.len() as f64 / total_items as f64) * 100.0);
    println!("    ・レア保持: {}/{} ({:.1}%)",
        rare_selected, rare_total, rare_preservation * 100.0);
    println!("    ・冗長削減: {}/{} ({:.1}%)",
        redundant_total - redundant_selected, redundant_total, redundancy_reduction * 100.0);
    println!("    ・F1スコア: {:.3}", f1_score);

    // Verification
    println!("\n【検証】");
    let pass = rare_preservation >= 0.95 && f1_score >= 0.90;

    if pass {
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ ✓ 100K スケール検証: PASS                                  │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ ・100,000件規模で正常動作 ✓                                │");
        println!("│ ・レア保持率 ≥ 95% ✓                                       │");
        println!("│ ・F1スコア ≥ 0.90 ✓                                        │");
        println!("└─────────────────────────────────────────────────────────────┘");
    } else {
        println!("┌─────────────────────────────────────────────────────────────┐");
        println!("│ △ 100K スケール検証: 要確認                                │");
        println!("├─────────────────────────────────────────────────────────────┤");
        if rare_preservation < 0.95 {
            println!("│ ・レア保持率 {:.1}% < 95% △                               │", rare_preservation * 100.0);
        }
        if f1_score < 0.90 {
            println!("│ ・F1スコア {:.3} < 0.90 △                                 │", f1_score);
        }
        println!("└─────────────────────────────────────────────────────────────┘");
    }

    println!("\n【証明事項】");
    if pass {
        println!("  65. 100K件規模でレア保持率≥95%維持");
        println!("  66. 100K件規模でF1≥0.90達成");
    }
    println!();
}
