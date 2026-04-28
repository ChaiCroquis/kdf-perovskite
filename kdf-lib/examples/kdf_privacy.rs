//! KDF Privacy-Sensitive Data Detection
//!
//! KDFで情報を整理し、プライバシー分析に応用する
//!
//! KDFの本質:
//! - Core層 = 知識飽和（多数の類似データ、匿名性高い）
//! - Rare層 = 判断材料不足（孤立データ、匿名性低い可能性）
//!
//! 応用:
//! - Rare層を匿名化優先対象として検討
//! - 結果としてプライバシーリスクの評価も可能であった
//! - 注意: Rare = 危険 ではなく、Rare = 類似データなし

use kdf::Kdf;

fn main() {
    println!("# KDF プライバシー分析\n");
    println!("KDFで情報を整理し、匿名性の分析に応用する");
    println!("（結果としてプライバシーリスク評価も可能であった）\n");

    // シナリオ1: 基本的なプライバシーリスク評価
    demo_basic_privacy_risk();

    // シナリオ2: 匿名化優先度の決定
    demo_anonymization_priority();

    // シナリオ3: k-匿名性との関連
    demo_k_anonymity_relation();

    // シナリオ4: 差分プライバシーへの応用
    demo_differential_privacy();

    println!("\n✅ プライバシー検出デモ完了");
}

/// 基本的なプライバシーリスク評価
fn demo_basic_privacy_risk() {
    println!("## 1. 基本的なプライバシーリスク評価\n");

    let kdf = Kdf::with_defaults();

    // 人口統計データ (年齢, 収入, 教育年数)
    let population_data = vec![
        // 一般的なプロファイル (多数派)
        ("一般A", vec![35.0, 50000.0, 12.0]),
        ("一般B", vec![36.0, 52000.0, 12.0]),
        ("一般C", vec![34.0, 48000.0, 13.0]),
        ("一般D", vec![35.0, 51000.0, 12.0]),
        ("一般E", vec![37.0, 49000.0, 12.0]),
        ("一般F", vec![33.0, 53000.0, 14.0]),
        ("一般G", vec![38.0, 47000.0, 12.0]),
        ("一般H", vec![34.0, 50500.0, 13.0]),
        // ユニークなプロファイル (特定されやすい)
        ("特異A", vec![85.0, 200000.0, 20.0]), // 高齢・高収入・高学歴
        ("特異B", vec![18.0, 15000.0, 8.0]),   // 若年・低収入・低学歴
        ("特異C", vec![50.0, 500000.0, 22.0]), // 超高収入・博士
    ];

    // 正規化
    let features: Vec<Vec<f64>> = population_data
        .iter()
        .map(|(_, f)| normalize_features(f))
        .collect();

    let result = kdf.process(&features, 0.85, |a, b| euclidean_similarity(a, b));

    println!("   人口統計データのプライバシーリスク評価:\n");
    println!("   {:>8} {:>8} {:>15}", "ID", "KDF層", "リスクレベル");
    println!("   {}", "-".repeat(35));

    for (i, (label, _)) in population_data.iter().enumerate() {
        let layer = get_layer(&result, i);
        let risk = match layer {
            "Rare" => "🔴 高 (特定容易)",
            "Edge" => "🟡 中 (注意必要)",
            "Core" => "🟢 低 (群衆に紛れる)",
            _ => "不明",
        };
        println!("   {:>8} {:>8} {:>15}", label, layer, risk);
    }

    println!("\n   【解釈】");
    println!("   Rare層: 属性の組み合わせがユニーク → 個人特定リスク高");
    println!("   Core層: 多くの人と類似 → 群衆に紛れてプライバシー保護");
}

/// 匿名化優先度の決定
fn demo_anonymization_priority() {
    println!("\n## 2. 匿名化優先度の決定\n");

    let kdf = Kdf::with_defaults();

    // 医療データ
    let medical_records = vec![
        // 一般的な症例
        ("患者001", vec![45.0, 120.0, 80.0, 25.0]), // 年齢, 血圧上, 血圧下, BMI
        ("患者002", vec![46.0, 118.0, 78.0, 24.0]),
        ("患者003", vec![44.0, 122.0, 82.0, 26.0]),
        ("患者004", vec![47.0, 119.0, 79.0, 25.0]),
        ("患者005", vec![43.0, 121.0, 81.0, 24.0]),
        ("患者006", vec![48.0, 117.0, 77.0, 27.0]),
        // 特異な症例
        ("患者007", vec![95.0, 180.0, 110.0, 18.0]), // 超高齢・高血圧・低体重
        ("患者008", vec![12.0, 90.0, 60.0, 35.0]),   // 小児・低血圧・肥満
    ];

    let features: Vec<Vec<f64>> = medical_records
        .iter()
        .map(|(_, f)| normalize_features(f))
        .collect();

    let result = kdf.process(&features, 0.85, |a, b| euclidean_similarity(a, b));

    println!("   匿名化処理の優先度:\n");
    println!(
        "   {:>10} {:>8} {:>12} {:>20}",
        "患者ID", "層", "優先度", "推奨処理"
    );
    println!("   {}", "-".repeat(55));

    for (i, (id, _)) in medical_records.iter().enumerate() {
        let layer = get_layer(&result, i);
        let (priority, action) = match layer {
            "Rare" => ("最優先", "一般化 + ノイズ追加"),
            "Edge" => ("高", "一般化"),
            "Core" => ("通常", "標準匿名化"),
            _ => ("不明", "要確認"),
        };
        println!("   {:>10} {:>8} {:>12} {:>20}", id, layer, priority, action);
    }

    println!("\n   【匿名化戦略】");
    println!("   Rare層: 属性の一般化 (年齢→年代、住所→地域) + ノイズ注入");
    println!("   Edge層: 軽度の一般化");
    println!("   Core層: 標準的な匿名化で十分");
}

/// k-匿名性との関連
fn demo_k_anonymity_relation() {
    println!("\n## 3. k-匿名性との関連\n");

    let kdf = Kdf::with_defaults();

    println!("   k-匿名性: 各レコードが少なくともk-1個の他のレコードと区別不可能\n");

    // データセット生成
    let mut data = Vec::new();
    let mut labels = Vec::new();

    // k=5 を満たすグループ
    for i in 0..5 {
        data.push(vec![30.0 + i as f64 * 0.1, 50000.0 + i as f64 * 100.0]);
        labels.push("グループA");
    }

    // k=3 を満たすグループ
    for i in 0..3 {
        data.push(vec![50.0 + i as f64 * 0.1, 80000.0 + i as f64 * 100.0]);
        labels.push("グループB");
    }

    // k=1 (ユニーク - k-匿名性違反)
    data.push(vec![90.0, 200000.0]);
    labels.push("ユニーク");

    let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));

    println!(
        "   {:>12} {:>10} {:>8} {:>15}",
        "データ", "グループ", "KDF層", "k-匿名性"
    );
    println!("   {}", "-".repeat(50));

    for (i, label) in labels.iter().enumerate() {
        let layer = get_layer(&result, i);
        let k_status = if layer == "Rare" {
            "❌ 違反 (k=1)"
        } else if layer == "Edge" {
            "⚠️ 要注意"
        } else {
            "✅ 満足"
        };
        println!(
            "   {:>12} {:>10} {:>8} {:>15}",
            format!("record_{}", i),
            label,
            layer,
            k_status
        );
    }

    println!("\n   【発見】");
    println!("   KDFのRare層 ≈ k-匿名性を満たさないレコード");
    println!("   → KDFを k-匿名性違反の検出に活用可能");
}

/// 差分プライバシーへの応用
fn demo_differential_privacy() {
    println!("\n## 4. 差分プライバシーへの応用\n");

    let kdf = Kdf::with_defaults();

    println!("   差分プライバシー: 個人の参加/不参加が結果に与える影響を制限\n");

    // センシティブなデータ
    let salary_data: Vec<Vec<f64>> = vec![
        vec![50000.0],
        vec![52000.0],
        vec![48000.0],
        vec![51000.0],
        vec![49000.0],
        vec![53000.0],
        vec![47000.0],
        vec![50500.0],
        vec![51500.0],
        vec![48500.0],
        // 外れ値 (高い感度を持つ)
        vec![500000.0], // CEO級の給与
        vec![10000.0],  // 極端に低い給与
    ];

    let result = kdf.process(&salary_data, 0.85, |a, b| euclidean_similarity(a, b));

    // 各レコードの感度を推定
    println!("   給与データの感度分析:\n");
    println!(
        "   {:>10} {:>12} {:>8} {:>15}",
        "Index", "給与", "層", "感度"
    );
    println!("   {}", "-".repeat(50));

    for (i, s) in salary_data.iter().enumerate() {
        let layer = get_layer(&result, i);
        let sensitivity = match layer {
            "Rare" => "高 (大きなノイズ必要)",
            "Edge" => "中 (中程度のノイズ)",
            "Core" => "低 (小さなノイズで十分)",
            _ => "不明",
        };
        println!(
            "   {:>10} {:>12.0} {:>8} {:>15}",
            i, s[0], layer, sensitivity
        );
    }

    // 平均値計算のシミュレーション
    println!("\n   【差分プライバシーへの応用】");

    let mean_all: f64 = salary_data.iter().map(|s| s[0]).sum::<f64>() / salary_data.len() as f64;

    // Rare層を除いた平均
    let core_edge: Vec<_> = (0..salary_data.len())
        .filter(|&i| get_layer(&result, i) != "Rare")
        .collect();
    let mean_without_rare: f64 =
        core_edge.iter().map(|&i| salary_data[i][0]).sum::<f64>() / core_edge.len() as f64;

    println!("\n   全データの平均: ${:.0}", mean_all);
    println!("   Rare除外の平均: ${:.0}", mean_without_rare);
    println!(
        "   差異: ${:.0} ({:.1}%)",
        (mean_all - mean_without_rare).abs(),
        (mean_all - mean_without_rare).abs() / mean_all * 100.0
    );

    println!("\n   → Rare層が統計値に大きな影響 = 高感度");
    println!("   → Rare層には追加のノイズ注入が必要");
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

/// 特徴量の正規化
fn normalize_features(features: &[f64]) -> Vec<f64> {
    // Min-Max正規化の簡易版 (固定スケール)
    features
        .iter()
        .enumerate()
        .map(|(i, &v)| {
            match i {
                0 => v / 100.0,    // 年齢: 0-100
                1 => v / 500000.0, // 収入/血圧: 適当なスケール
                2 => v / 100.0,    // 教育年数/血圧下
                3 => v / 50.0,     // BMI
                _ => v / 100.0,
            }
        })
        .collect()
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
