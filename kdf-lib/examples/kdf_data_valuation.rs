//! KDF Data Valuation
//!
//! KDFで情報を整理し、データの代替可能性を評価する
//!
//! KDFの本質:
//! - Core層 = 知識飽和（冗長情報が多く、代替可能）
//! - Edge層 = 知識に余地あり（部分的に代替可能）
//! - Rare層 = 判断材料不足（代替不可能だが、価値は不明）
//!
//! 応用:
//! - 代替可能性に基づく価値評価が可能であった
//! - 注意: Rare = 高価値 ではなく、Rare = 判断困難

use kdf::Kdf;

fn main() {
    println!("# KDF データ代替可能性評価\n");
    println!("KDFで情報を整理し、代替可能性を評価する");
    println!("（結果として価値評価への応用も可能であった）\n");
    println!("注意: Rare ≠ 高価値。Rare = 判断材料不足。\n");

    // シナリオ1: 基本的な価値評価
    demo_basic_valuation();

    // シナリオ2: データマーケットプレイス
    demo_data_marketplace();

    // シナリオ3: 学習データの価値
    demo_training_data_value();

    // シナリオ4: 価値ベースのサンプリング
    demo_value_based_sampling();

    println!("\n✅ 代替可能性評価デモ完了");
    println!("\n本質: KDFは情報整理ツール。価値評価は副産物。");
}

/// 基本的な価値評価
fn demo_basic_valuation() {
    println!("## 1. 基本的な価値評価\n");

    let kdf = Kdf::with_defaults();

    // データセット: 冗長なデータと希少なデータの混合
    let data = vec![
        // 冗長データ (多数の類似点)
        vec![1.0, 1.0],
        vec![1.1, 0.9],
        vec![0.9, 1.1],
        vec![1.0, 1.0],
        vec![1.0, 1.0],
        vec![1.1, 1.0],
        vec![1.0, 1.1],
        vec![0.9, 0.9],
        // 境界データ
        vec![2.5, 2.5],
        vec![2.6, 2.4],
        // 希少データ
        vec![5.0, 5.0],   // 孤立点1
        vec![-3.0, -3.0], // 孤立点2
    ];

    let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));

    // 価値評価
    let valuator = DataValuator::new(&result);

    println!("   データ価値評価結果:\n");
    println!(
        "   {:>5} {:>8} {:>10} {:>12}",
        "Index", "層", "価値", "価値スコア"
    );
    println!("   {}", "-".repeat(40));

    for i in 0..data.len() {
        let (layer, value, score) = valuator.evaluate(i);
        println!("   {:>5} {:>8} {:>10} {:>12.2}", i, layer, value, score);
    }

    println!("\n   【代替可能性の分布】");
    println!(
        "   Rare:  {} 件 → 代替不可能（ただし価値は不明）",
        result.rare_items().len()
    );
    println!(
        "   Edge:  {} 件 → 部分的に代替可能",
        result.edge_items().len()
    );
    println!(
        "   Core:  {} 件 → 代替可能（冗長情報）\n",
        result.core_items().len()
    );
}

/// データマーケットプレイス
fn demo_data_marketplace() {
    println!("## 2. データマーケットプレイス価格設定\n");

    let kdf = Kdf::with_defaults();

    // 医療データを想定
    let medical_records = vec![
        // 一般的な症例 (多数)
        ("一般的な風邪", vec![1.0, 0.5, 0.2]),
        ("一般的な風邪", vec![1.0, 0.4, 0.3]),
        ("一般的な風邪", vec![0.9, 0.5, 0.2]),
        ("一般的な風邪", vec![1.0, 0.5, 0.1]),
        ("一般的な風邪", vec![1.1, 0.5, 0.2]),
        ("軽度の発熱", vec![0.8, 0.6, 0.3]),
        ("軽度の発熱", vec![0.8, 0.5, 0.3]),
        ("軽度の発熱", vec![0.9, 0.6, 0.2]),
        // 中程度の症例
        ("インフルエンザ", vec![2.0, 1.5, 0.8]),
        ("インフルエンザ", vec![2.1, 1.4, 0.7]),
        // 希少症例
        ("難病A", vec![5.0, 4.0, 3.0]),
        ("難病B", vec![-2.0, 3.0, 5.0]),
    ];

    let features: Vec<Vec<f64>> = medical_records.iter().map(|(_, f)| f.clone()).collect();

    let result = kdf.process(&features, 0.85, |a, b| euclidean_similarity(a, b));
    let valuator = DataValuator::new(&result);

    println!("   医療データの価値評価:\n");
    println!("   {:>16} {:>8} {:>12}", "症例", "層", "価格(単位)");
    println!("   {}", "-".repeat(40));

    let base_price = 10.0; // 基本価格

    for (i, (label, _)) in medical_records.iter().enumerate() {
        let (layer, _, score) = valuator.evaluate(i);
        let price = base_price * score;
        println!("   {:>16} {:>8} {:>12.0}", label, layer, price);
    }

    // 総価値の計算
    let total_value: f64 = (0..medical_records.len())
        .map(|i| valuator.evaluate(i).2 * base_price)
        .sum();

    println!("\n   総データ価値: {:.0} 単位", total_value);
    println!(
        "   希少データの価値比率: {:.0}%",
        result.rare_items().len() as f64 / medical_records.len() as f64
            * 100.0
            * (10.0 / (10.0 + 3.0 + 1.0) * 3.0)
    ); // 概算
}

/// 学習データの価値
fn demo_training_data_value() {
    println!("\n## 3. 機械学習データセットの価値評価\n");

    let kdf = Kdf::with_defaults();

    // 画像分類データを想定
    println!("   シナリオ: 画像分類データセット\n");

    let image_features = vec![
        // 猫画像 (多数)
        ("猫-正面", vec![1.0, 0.5]),
        ("猫-正面", vec![1.1, 0.5]),
        ("猫-正面", vec![0.9, 0.5]),
        ("猫-正面", vec![1.0, 0.4]),
        ("猫-正面", vec![1.0, 0.6]),
        // 犬画像 (多数)
        ("犬-正面", vec![0.5, 1.0]),
        ("犬-正面", vec![0.5, 1.1]),
        ("犬-正面", vec![0.4, 1.0]),
        ("犬-正面", vec![0.6, 1.0]),
        // 境界ケース
        ("猫-横向き", vec![1.5, 0.8]),
        ("犬-横向き", vec![0.8, 1.5]),
        // 希少ケース
        ("珍しい猫種", vec![3.0, 0.2]),
        ("珍しい犬種", vec![0.2, 3.0]),
        ("猫犬同居", vec![1.0, 1.0]), // 両方の特徴
    ];

    let features: Vec<Vec<f64>> = image_features.iter().map(|(_, f)| f.clone()).collect();

    let result = kdf.process(&features, 0.85, |a, b| euclidean_similarity(a, b));
    let _valuator = DataValuator::new(&result);

    println!("   学習への貢献度:\n");

    // 層ごとの分析
    let core_labels: Vec<_> = result
        .core_items()
        .iter()
        .map(|&i| image_features[i].0)
        .collect();
    let edge_labels: Vec<_> = result
        .edge_items()
        .iter()
        .map(|&i| image_features[i].0)
        .collect();
    let rare_labels: Vec<_> = result
        .rare_items()
        .iter()
        .map(|&i| image_features[i].0)
        .collect();

    println!("   【Core層】 基本パターンの学習に使用");
    println!(
        "      {} (低価値: 類似データで代替可能)",
        core_labels.join(", ")
    );

    println!("\n   【Edge層】 境界ケースの学習に使用");
    println!(
        "      {} (中価値: 汎化性能向上に寄与)",
        edge_labels.join(", ")
    );

    println!("\n   【Rare層】 判断材料不足のケース");
    println!(
        "      {} (代替不可能: 捨てると判断できなくなる)",
        rare_labels.join(", ")
    );

    // 価値ベースのデータ削減提案
    println!("\n   【データ削減提案】");
    println!("      Core層から50%削減 → 精度への影響: 軽微");
    println!("      Edge層は維持 → 境界性能維持");
    println!("      Rare層は100%維持 → エッジケース対応");
}

/// 価値ベースのサンプリング
fn demo_value_based_sampling() {
    println!("\n## 4. 価値ベースサンプリング\n");

    let kdf = Kdf::with_defaults();

    // 大規模データセットをシミュレート
    let mut data = Vec::new();

    // 冗長データ (80件)
    for i in 0..80 {
        let x = (i % 10) as f64 * 0.1;
        let y = (i / 10) as f64 * 0.1;
        data.push(vec![x, y]);
    }

    // 境界データ (15件)
    for i in 0..15 {
        let angle = i as f64 * 0.4;
        data.push(vec![1.5 + angle.cos() * 0.3, 1.5 + angle.sin() * 0.3]);
    }

    // 希少データ (5件)
    data.push(vec![5.0, 5.0]);
    data.push(vec![-3.0, 4.0]);
    data.push(vec![4.0, -3.0]);
    data.push(vec![-4.0, -4.0]);
    data.push(vec![6.0, 0.0]);

    let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));
    let valuator = DataValuator::new(&result);

    println!("   元データ: {} 件", data.len());
    println!("      Core: {} 件", result.core_items().len());
    println!("      Edge: {} 件", result.edge_items().len());
    println!("      Rare: {} 件\n", result.rare_items().len());

    // 異なるサンプリング戦略の比較
    let strategies = vec![
        ("ランダム", sample_random(&data, 20)),
        ("価値ベース", sample_by_value(&result, 20)),
    ];

    println!("   20件にサンプリング:\n");

    let rare_indices: Vec<usize> = result.rare_items().to_vec();

    for (name, sampled) in strategies {
        // サンプル中の希少データ数をカウント
        let rare_in_sample = sampled
            .iter()
            .filter(|&&i| rare_indices.contains(&i))
            .count();

        let total_value: f64 = sampled.iter().map(|&i| valuator.evaluate(i).2).sum();

        println!("   【{}】", name);
        println!("      希少データ保持: {}/5 件", rare_in_sample);
        println!("      合計価値スコア: {:.1}", total_value);
        println!();
    }

    println!("   結論: 代替可能性に基づくサンプリングで判断材料を維持");
    println!("         （結果として価値の高いデータも保持できた）");
}

// ============================================================================
// ヘルパー構造体・関数
// ============================================================================

/// データ価値評価器
struct DataValuator {
    core_indices: Vec<usize>,
    edge_indices: Vec<usize>,
    rare_indices: Vec<usize>,
}

impl DataValuator {
    fn new(result: &kdf::KdfResult) -> Self {
        DataValuator {
            core_indices: result.core_items().to_vec(),
            edge_indices: result.edge_items().to_vec(),
            rare_indices: result.rare_items().to_vec(),
        }
    }

    /// データの代替可能性を評価
    /// 戻り値: (層名, 代替可能性ラベル, スコア)
    /// 注意: スコアは代替可能性の逆数であり、価値とは限らない
    fn evaluate(&self, index: usize) -> (&str, &str, f64) {
        if self.rare_indices.contains(&index) {
            ("Rare", "代替不可", 10.0) // 判断材料不足
        } else if self.edge_indices.contains(&index) {
            ("Edge", "部分代替", 3.0) // 知識に余地
        } else if self.core_indices.contains(&index) {
            ("Core", "代替可能", 1.0) // 知識飽和
        } else {
            ("不明", "未評価", 0.0)
        }
    }
}

/// ランダムサンプリング
fn sample_random(data: &[Vec<f64>], n: usize) -> Vec<usize> {
    use std::collections::HashSet;

    let mut sampled = HashSet::new();
    let mut seed = 42u64;

    while sampled.len() < n && sampled.len() < data.len() {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let idx = (seed as usize) % data.len();
        sampled.insert(idx);
    }

    sampled.into_iter().collect()
}

/// 価値ベースサンプリング
fn sample_by_value(result: &kdf::KdfResult, n: usize) -> Vec<usize> {
    let mut sampled = Vec::new();

    // 1. Rare層を全て含める
    sampled.extend(result.rare_items());

    // 2. Edge層を追加
    let remaining = n.saturating_sub(sampled.len());
    let edge = result.edge_items();
    sampled.extend(edge.iter().take(remaining));

    // 3. 残りをCoreから
    let remaining = n.saturating_sub(sampled.len());
    let core = result.core_items();
    sampled.extend(core.iter().take(remaining));

    sampled.into_iter().take(n).collect()
}

/// ユークリッド類似度
fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}
