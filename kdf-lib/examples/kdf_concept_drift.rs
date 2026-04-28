//! KDF Concept Drift Detection
//!
//! KDFで情報を整理し、時系列での層構成の変化を追跡する
//!
//! KDFの本質:
//! - Core層 = 知識が飽和（冗長情報が多い）
//! - Rare層 = 判断材料が不足（捨てるべきか不明）
//!
//! 応用:
//! - 層構成の変化を追跡することで、分布変化の検出も可能であった
//! - Core層の移動 = 主流パターンの変化として解釈可能
//! - Rare層の変化 = 新規の判断困難データの出現として解釈可能

use kdf::Kdf;
use std::collections::HashSet;

fn main() {
    println!("# KDF 時系列層構成の追跡\n");
    println!("KDFで情報を整理し、層構成の変化を追跡する");
    println!("（結果として分布変化の検出も可能であった）\n");

    // シナリオ1: 段階的ドリフト
    demo_gradual_drift();

    // シナリオ2: 突発的ドリフト
    demo_sudden_drift();

    // シナリオ3: 周期的パターン
    demo_seasonal_pattern();

    // シナリオ4: 異常出現の検出
    demo_anomaly_emergence();

    println!("\n✅ 層構成追跡デモ完了");
    println!("\n本質: KDFは情報整理ツール。ドリフト検出は副産物。");
}

/// 段階的ドリフト: 徐々に分布が変化
fn demo_gradual_drift() {
    println!("## 1. 段階的ドリフト検出\n");

    let kdf = Kdf::with_defaults();
    let mut drift_detector = DriftDetector::new();

    // 5つの時間窓をシミュレート
    for t in 0..5 {
        // 時間とともにセンター位置がシフト
        let center = t as f64 * 0.5;
        let data = generate_cluster_data(100, center, 0.3);

        let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));
        let metrics = LayerMetrics::from_result(&result, &data);

        let drift = drift_detector.update(metrics.clone());

        println!("   時刻 t={}: センター={:.1}", t, center);
        println!(
            "      Core: {} 件, 重心=({:.2}, {:.2})",
            metrics.core_count, metrics.core_centroid.0, metrics.core_centroid.1
        );
        println!("      Rare: {} 件", metrics.rare_count);

        if let Some(d) = drift {
            println!("      ⚠️  ドリフト検出: {}", d.description);
            println!("         Core移動距離: {:.3}", d.core_shift);
        }
        println!();
    }

    println!("   結論: Core層（知識飽和領域）の重心移動を追跡");
    println!("         → 結果としてドリフト検出も可能であった\n");
}

/// 突発的ドリフト: 急激な分布変化
fn demo_sudden_drift() {
    println!("## 2. 突発的ドリフト検出\n");

    let kdf = Kdf::with_defaults();
    let mut drift_detector = DriftDetector::new();

    let scenarios = vec![
        ("正常期", 0.0, 0.3),
        ("正常期", 0.0, 0.3),
        ("突発変化", 3.0, 0.3), // 突然センターが移動
        ("新正常", 3.0, 0.3),
        ("新正常", 3.0, 0.3),
    ];

    for (t, (label, center, _std)) in scenarios.iter().enumerate() {
        let data = generate_cluster_data(100, *center, 0.3);
        let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));
        let metrics = LayerMetrics::from_result(&result, &data);

        let drift = drift_detector.update(metrics.clone());

        print!("   時刻 t={} [{}]: ", t, label);

        if let Some(d) = drift {
            if d.core_shift > 1.0 {
                println!("🚨 突発ドリフト! (移動={:.2})", d.core_shift);
            } else {
                println!("軽微な変化 (移動={:.2})", d.core_shift);
            }
        } else {
            println!("安定");
        }
    }

    println!("\n   結論: Core層の急激な移動を追跡");
    println!("         → 結果として突発ドリフトの識別も可能であった\n");
}

/// 周期的パターン: 季節性などの検出
fn demo_seasonal_pattern() {
    println!("## 3. 周期的パターン検出\n");

    let kdf = Kdf::with_defaults();
    let mut rare_history: Vec<HashSet<usize>> = Vec::new();

    // 周期的なパターン (季節性をシミュレート)
    let patterns = vec![
        ("春", vec![(0.0, 0.0), (1.0, 0.0)]),
        ("夏", vec![(0.0, 1.0), (1.0, 1.0)]),
        ("秋", vec![(0.0, 0.0), (1.0, 0.0)]),
        ("冬", vec![(0.0, -1.0), (1.0, -1.0)]),
        ("春", vec![(0.0, 0.0), (1.0, 0.0)]), // 周期の繰り返し
        ("夏", vec![(0.0, 1.0), (1.0, 1.0)]),
    ];

    for (season, centers) in &patterns {
        let mut data = Vec::new();
        for (cx, cy) in centers {
            for _ in 0..30 {
                let x = cx + (rand_f64() - 0.5) * 0.4;
                let y = cy + (rand_f64() - 0.5) * 0.4;
                data.push(vec![x, y]);
            }
        }
        // 少数の異常を追加
        data.push(vec![5.0, 5.0]);

        let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));
        let rare_set: HashSet<usize> = result.rare_items().iter().copied().collect();

        // 過去との類似性を計算
        let similarity = if rare_history.len() >= 2 {
            // 2つ前（同じ季節）との比較
            let prev_same = &rare_history[rare_history.len() - 2];
            jaccard_similarity(&rare_set, prev_same)
        } else {
            0.0
        };

        print!("   {}: Rare={} 件", season, rare_set.len());
        if similarity > 0.5 {
            println!(" → 周期パターン検出 (類似度={:.0}%)", similarity * 100.0);
        } else {
            println!();
        }

        rare_history.push(rare_set);
    }

    println!("\n   結論: Rare層（判断保留領域）の周期的類似性を追跡");
    println!("         → 結果として季節パターンの検出も可能であった\n");
}

/// 異常出現の検出
fn demo_anomaly_emergence() {
    println!("## 4. 新規異常の出現検出\n");

    let kdf = Kdf::with_defaults();
    let mut prev_rare: Option<HashSet<usize>> = None;
    let mut prev_data: Option<Vec<Vec<f64>>> = None;

    let scenarios = vec![
        ("通常", false),
        ("通常", false),
        ("異常出現", true), // 新しいタイプの異常が出現
        ("異常継続", true),
        ("通常復帰", false),
    ];

    for (t, (label, has_anomaly)) in scenarios.iter().enumerate() {
        // 通常データ
        let mut data = generate_cluster_data(80, 0.0, 0.3);

        // 異常データの追加
        if *has_anomaly {
            // 新しいタイプの異常（遠い位置）
            data.push(vec![5.0, 5.0]);
            data.push(vec![5.1, 4.9]);
            data.push(vec![-5.0, -5.0]);
        }

        let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));
        let rare_set: HashSet<usize> = result.rare_items().iter().copied().collect();

        // 新規Rare項目を検出
        let new_rares = if let Some(ref prev) = prev_rare {
            // 現在のRareデータの位置が、以前のRareと異なるか確認
            let mut new_count = 0;
            for &idx in &rare_set {
                let current_pos = &data[idx];
                let is_new = prev_data.as_ref().is_none_or(|pd| {
                    // 以前のRare位置と比較
                    !prev.iter().any(|&pi| {
                        if pi < pd.len() {
                            euclidean_distance(current_pos, &pd[pi]) < 0.5
                        } else {
                            false
                        }
                    })
                });
                if is_new {
                    new_count += 1;
                }
            }
            new_count
        } else {
            0
        };

        print!("   時刻 t={} [{}]: Rare={} 件", t, label, rare_set.len());

        if new_rares > 0 {
            println!(" → 🆕 新規異常 {} 件検出!", new_rares);
        } else {
            println!();
        }

        prev_rare = Some(rare_set);
        prev_data = Some(data);
    }

    println!("\n   結論: Rare層（判断保留）の新規メンバーを追跡");
    println!("         → 結果として異常出現の検出も可能であった\n");
}

// ============================================================================
// ヘルパー構造体・関数
// ============================================================================

/// 層のメトリクス
#[derive(Clone)]
#[allow(dead_code)]
struct LayerMetrics {
    core_count: usize,
    edge_count: usize,
    rare_count: usize,
    core_centroid: (f64, f64),
    rare_indices: Vec<usize>,
}

impl LayerMetrics {
    fn from_result(result: &kdf::KdfResult, data: &[Vec<f64>]) -> Self {
        let core = result.core_items();
        let edge = result.edge_items();
        let rare = result.rare_items();

        // Core層の重心を計算
        let core_centroid = if core.is_empty() {
            (0.0, 0.0)
        } else {
            let sum: (f64, f64) = core
                .iter()
                .map(|&i| (data[i][0], data[i][1]))
                .fold((0.0, 0.0), |acc, (x, y)| (acc.0 + x, acc.1 + y));
            (sum.0 / core.len() as f64, sum.1 / core.len() as f64)
        };

        LayerMetrics {
            core_count: core.len(),
            edge_count: edge.len(),
            rare_count: rare.len(),
            core_centroid,
            rare_indices: rare.to_vec(),
        }
    }
}

/// ドリフト検出器
struct DriftDetector {
    prev_metrics: Option<LayerMetrics>,
}

impl DriftDetector {
    fn new() -> Self {
        DriftDetector { prev_metrics: None }
    }

    fn update(&mut self, current: LayerMetrics) -> Option<DriftInfo> {
        let result = if let Some(ref prev) = self.prev_metrics {
            let core_shift = euclidean_distance(
                &[prev.core_centroid.0, prev.core_centroid.1],
                &[current.core_centroid.0, current.core_centroid.1],
            );

            let rare_change = (current.rare_count as i32 - prev.rare_count as i32).abs();

            if core_shift > 0.1 || rare_change > 2 {
                Some(DriftInfo {
                    core_shift,
                    rare_change: rare_change as usize,
                    description: if core_shift > 1.0 {
                        "大規模ドリフト".to_string()
                    } else if core_shift > 0.3 {
                        "中程度ドリフト".to_string()
                    } else {
                        "軽微な変動".to_string()
                    },
                })
            } else {
                None
            }
        } else {
            None
        };

        self.prev_metrics = Some(current);
        result
    }
}

/// ドリフト情報
#[allow(dead_code)]
struct DriftInfo {
    core_shift: f64,
    rare_change: usize,
    description: String,
}

/// クラスタデータ生成
fn generate_cluster_data(n: usize, center: f64, std: f64) -> Vec<Vec<f64>> {
    (0..n)
        .map(|_| {
            vec![
                center + (rand_f64() - 0.5) * std * 2.0,
                center + (rand_f64() - 0.5) * std * 2.0,
            ]
        })
        .collect()
}

/// 簡易乱数生成
fn rand_f64() -> f64 {
    use std::time::SystemTime;
    static mut SEED: u64 = 0;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        (SEED as f64) / (u64::MAX as f64)
    }
}

/// ユークリッド類似度
fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist = euclidean_distance(a, b);
    1.0 / (1.0 + dist)
}

/// ユークリッド距離
fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt()
}

/// Jaccard類似度
fn jaccard_similarity(a: &HashSet<usize>, b: &HashSet<usize>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(b).count();
    let union = a.union(b).count();
    intersection as f64 / union as f64
}
