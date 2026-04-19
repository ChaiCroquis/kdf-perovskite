//! KDF Model Debugging
//!
//! KDFで情報を整理し、モデル分析に応用する
//!
//! KDFの本質:
//! - Core層 = 知識飽和（典型的なケース）
//! - Rare層 = 判断材料不足（判断が難しいケース）
//!
//! 応用:
//! - Rare層での精度を別途計測
//! - 結果として失敗モードの特定も可能であった
//! - 注意: Rare = 失敗 ではなく、Rare = 典型から外れたケース

use kdf::Kdf;
use std::collections::HashMap;

fn main() {
    println!("# KDF モデル分析\n");
    println!("KDFで情報を整理し、層別のモデル挙動を分析する");
    println!("（結果として失敗モードの特定も可能であった）\n");

    // シナリオ1: 層別精度分析
    demo_layer_accuracy();

    // シナリオ2: 失敗パターンの特定
    demo_failure_patterns();

    // シナリオ3: テストケース生成
    demo_test_case_generation();

    // シナリオ4: モデル改善提案
    demo_improvement_suggestions();

    println!("\n✅ モデルデバッグデモ完了");
}

/// 層別精度分析
fn demo_layer_accuracy() {
    println!("## 1. 層別精度分析\n");

    let kdf = Kdf::with_defaults();

    // 画像分類モデルの予測結果をシミュレート
    let test_data = vec![
        // (特徴量, 正解ラベル, 予測ラベル, 確信度)
        (vec![1.0, 1.0], "猫", "猫", 0.95),
        (vec![1.1, 0.9], "猫", "猫", 0.92),
        (vec![0.9, 1.1], "猫", "猫", 0.90),
        (vec![1.0, 1.0], "猫", "猫", 0.88),
        (vec![0.5, 0.5], "犬", "犬", 0.93),
        (vec![0.6, 0.4], "犬", "犬", 0.91),
        (vec![0.4, 0.6], "犬", "犬", 0.89),
        (vec![0.5, 0.5], "犬", "犬", 0.87),
        // エッジケース (失敗しやすい)
        (vec![0.8, 0.8], "猫", "犬", 0.52),  // 境界で誤分類
        (vec![3.0, 3.0], "猫", "鳥", 0.45),  // 外れ値で誤分類
        (vec![-1.0, 2.0], "犬", "猫", 0.48), // 異常値で誤分類
    ];

    let features: Vec<Vec<f64>> = test_data.iter()
        .map(|(f, _, _, _)| f.clone())
        .collect();

    let result = kdf.process(&features, 0.85, euclidean_similarity);

    // 層別の精度を計算
    let mut layer_stats: HashMap<&str, (usize, usize)> = HashMap::new(); // (正解数, 総数)

    for (i, (_, true_label, pred_label, _)) in test_data.iter().enumerate() {
        let layer = get_layer(&result, i);
        let is_correct = true_label == pred_label;

        let entry = layer_stats.entry(layer).or_insert((0, 0));
        if is_correct {
            entry.0 += 1;
        }
        entry.1 += 1;
    }

    println!("   層別精度:\n");
    println!("   {:>8} {:>10} {:>10} {:>10}", "層", "正解数", "総数", "精度");
    println!("   {}", "-".repeat(45));

    for layer in &["Core", "Edge", "Rare"] {
        if let Some(&(correct, total)) = layer_stats.get(layer) {
            let accuracy = if total > 0 { correct as f64 / total as f64 * 100.0 } else { 0.0 };
            let indicator = if accuracy < 70.0 { "⚠️" } else { "✅" };
            println!("   {:>8} {:>10} {:>10} {:>9.1}% {}", layer, correct, total, accuracy, indicator);
        }
    }

    println!("\n   【発見】");
    println!("   Rare層の精度が低い → エッジケースへの対応が必要");
}

/// 失敗パターンの特定
fn demo_failure_patterns() {
    println!("\n## 2. 失敗パターンの特定\n");

    let kdf = Kdf::with_defaults();

    // より詳細な失敗分析
    let predictions = vec![
        // (特徴量, 正解, 予測, 失敗理由)
        (vec![1.0, 1.0], true, true, None),
        (vec![1.1, 0.9], true, true, None),
        (vec![0.9, 1.1], true, true, None),
        (vec![0.5, 0.5], true, true, None),
        (vec![0.6, 0.4], true, true, None),
        // 失敗ケース
        (vec![0.75, 0.75], true, false, Some("境界ケース: 両クラスの特徴が混在")),
        (vec![5.0, 0.0], true, false, Some("外れ値: 学習データ範囲外")),
        (vec![0.0, 5.0], true, false, Some("外れ値: 学習データ範囲外")),
        (vec![-2.0, -2.0], true, false, Some("異常値: 負の特徴量")),
    ];

    let features: Vec<Vec<f64>> = predictions.iter()
        .map(|(f, _, _, _)| f.clone())
        .collect();

    let result = kdf.process(&features, 0.85, euclidean_similarity);

    println!("   失敗パターンの分析:\n");

    let failures: Vec<_> = predictions.iter()
        .enumerate()
        .filter(|(_, (_, _, pred, _))| !pred)
        .collect();

    println!("   {:>6} {:>15} {:>8} {:>30}", "Index", "特徴量", "層", "失敗理由");
    println!("   {}", "-".repeat(65));

    for (i, (feat, _, _, reason)) in &failures {
        let layer = get_layer(&result, *i);
        let reason_str = reason.unwrap_or("不明");
        println!("   {:>6} {:>15} {:>8} {:>30}",
                 i,
                 format!("[{:.1}, {:.1}]", feat[0], feat[1]),
                 layer,
                 reason_str);
    }

    // パターン集計
    let rare_failures = failures.iter()
        .filter(|(i, _)| get_layer(&result, *i) == "Rare")
        .count();

    println!("\n   【失敗パターンの傾向】");
    println!("   Rare層での失敗: {}/{} ({:.0}%)",
             rare_failures, failures.len(),
             rare_failures as f64 / failures.len() as f64 * 100.0);
    println!("   → Rare層に集中する失敗 = エッジケースハンドリングの問題");
}

/// テストケース生成
fn demo_test_case_generation() {
    println!("\n## 3. Rare層ベースのテストケース生成\n");

    let kdf = Kdf::with_defaults();

    // 学習データ
    let training_data = vec![
        vec![1.0, 1.0], vec![1.1, 0.9], vec![0.9, 1.1], vec![1.0, 1.0],
        vec![0.5, 0.5], vec![0.6, 0.4], vec![0.4, 0.6], vec![0.5, 0.5],
        vec![1.5, 0.8], vec![0.8, 1.5],
        // エッジケース
        vec![3.0, 0.5],
        vec![0.5, 3.0],
        vec![-0.5, 1.0],
    ];

    let result = kdf.process(&training_data, 0.85, euclidean_similarity);

    println!("   学習データからのテストケース優先度:\n");

    // テストケースの優先度を決定
    let mut test_priority: Vec<(usize, &str, &str)> = Vec::new();

    for i in 0..training_data.len() {
        let layer = get_layer(&result, i);
        let priority = match layer {
            "Rare" => "最優先 - 必ずテスト",
            "Edge" => "高 - 優先的にテスト",
            "Core" => "通常 - 代表サンプル",
            _ => "不明",
        };
        test_priority.push((i, layer, priority));
    }

    // Rare層を表示
    println!("   【必須テストケース (Rare層)】");
    for (i, layer, priority) in &test_priority {
        if *layer == "Rare" {
            println!("   Index {}: {:?} → {}", i, training_data[*i], priority);
        }
    }

    // Edge層を表示
    println!("\n   【推奨テストケース (Edge層)】");
    for (i, layer, priority) in &test_priority {
        if *layer == "Edge" {
            println!("   Index {}: {:?} → {}", i, training_data[*i], priority);
        }
    }

    println!("\n   【テスト戦略】");
    println!("   1. Rare層: 100% カバレッジ必須");
    println!("   2. Edge層: 高優先度でテスト");
    println!("   3. Core層: 代表サンプルのみ");
}

/// モデル改善提案
fn demo_improvement_suggestions() {
    println!("\n## 4. モデル改善提案\n");

    let kdf = Kdf::with_defaults();

    // モデルの予測結果と信頼度
    let model_output = vec![
        (vec![1.0, 1.0], 0.95, true),   // 高信頼度, 正解
        (vec![1.1, 0.9], 0.92, true),
        (vec![0.9, 1.1], 0.88, true),
        (vec![0.5, 0.5], 0.91, true),
        (vec![0.6, 0.4], 0.89, true),
        // 問題のあるケース
        (vec![0.75, 0.75], 0.52, false), // 低信頼度, 誤り
        (vec![3.0, 0.5], 0.48, false),   // 低信頼度, 誤り
        (vec![0.5, 3.0], 0.45, false),   // 低信頼度, 誤り
        (vec![-1.0, 1.0], 0.55, true),   // 低信頼度, 正解 (たまたま)
    ];

    let features: Vec<Vec<f64>> = model_output.iter()
        .map(|(f, _, _)| f.clone())
        .collect();

    let result = kdf.process(&features, 0.85, euclidean_similarity);

    // 改善提案を生成
    println!("   モデル診断結果:\n");

    let mut issues = Vec::new();

    for (i, (_feat, conf, correct)) in model_output.iter().enumerate() {
        let layer = get_layer(&result, i);

        if layer == "Rare" {
            if !correct {
                issues.push(format!(
                    "🔴 Index {}: Rare層で誤分類 (信頼度: {:.0}%) → データ拡張を検討",
                    i, conf * 100.0
                ));
            } else if *conf < 0.7 {
                issues.push(format!(
                    "🟡 Index {}: Rare層で低信頼度 ({:.0}%) → 追加学習データが必要",
                    i, conf * 100.0
                ));
            }
        } else if layer == "Edge" && *conf < 0.7 {
            issues.push(format!(
                "🟡 Index {}: Edge層で低信頼度 ({:.0}%) → 境界ケースの強化学習",
                i, conf * 100.0
            ));
        }
    }

    for issue in &issues {
        println!("   {}", issue);
    }

    // 改善提案のサマリー
    println!("\n   【改善提案サマリー】");

    let rare_errors = model_output.iter()
        .enumerate()
        .filter(|(i, (_, _, c))| get_layer(&result, *i) == "Rare" && !c)
        .count();

    let low_conf_count = model_output.iter()
        .filter(|(_, conf, _)| *conf < 0.7)
        .count();

    if rare_errors > 0 {
        println!("   1. Rare層でのエラー: {} 件", rare_errors);
        println!("      → エッジケース用のデータ拡張を推奨");
    }

    if low_conf_count > 0 {
        println!("   2. 低信頼度予測: {} 件", low_conf_count);
        println!("      → モデルのキャリブレーションまたは追加学習を推奨");
    }

    println!("\n   【推奨アクション】");
    println!("   ┌─────────────────────────────────────────────────┐");
    println!("   │ 1. Rare層のサンプルでデータ拡張                │");
    println!("   │ 2. Edge層を中心にハードネガティブマイニング   │");
    println!("   │ 3. 低信頼度サンプルのアクティブラーニング     │");
    println!("   └─────────────────────────────────────────────────┘");
}

// ============================================================================
// ヘルパー関数
// ============================================================================

/// 層を取得
fn get_layer(result: &kdf::KdfResult, index: usize) -> &'static str {
    if result.rare_items().contains(&index) {
        "Rare"
    } else if result.edge_items().contains(&index) {
        "Edge"
    } else if result.core_items().contains(&index) {
        "Core"
    } else {
        "Unknown"
    }
}

/// ユークリッド類似度
fn euclidean_similarity(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let dist: f64 = a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}
