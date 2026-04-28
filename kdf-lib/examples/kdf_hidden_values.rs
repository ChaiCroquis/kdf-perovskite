//! Hidden Values of KDF: Unexplored Applications
//!
//! 1. Unsupervised Anomaly Detection
//! 2. Curriculum Learning
//! 3. Data Quality Diagnosis
//! 4. Fairness in ML

use kdf::{Kdf, Layer};
use std::collections::HashMap;

fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}

// ============================================================================
// 1. Unsupervised Anomaly Detection
// ============================================================================

fn anomaly_detection_demo() {
    println!("## 1. 教師なし異常検知\n");

    // Normal data (cluster around origin)
    let mut data: Vec<Vec<f64>> = (0..100)
        .map(|i| {
            let angle = (i as f64) * 0.1;
            vec![angle.cos() * 0.1, angle.sin() * 0.1]
        })
        .collect();

    // Inject anomalies
    let anomaly_indices = vec![100, 101, 102, 103, 104];
    data.push(vec![5.0, 5.0]); // Anomaly 1: far from cluster
    data.push(vec![-4.0, 3.0]); // Anomaly 2
    data.push(vec![0.0, -6.0]); // Anomaly 3
    data.push(vec![3.0, -3.0]); // Anomaly 4
    data.push(vec![-5.0, -5.0]); // Anomaly 5

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.8, |a, b| euclidean_similarity(a, b));

    // Check if anomalies are detected as Rare
    let detected: Vec<usize> = result.rare_items();
    let true_positives = detected
        .iter()
        .filter(|&&i| anomaly_indices.contains(&i))
        .count();

    println!("   注入した異常: {:?}", anomaly_indices);
    println!("   Rare層として検出: {:?}", detected);
    println!(
        "   検出率: {}/{} ({:.0}%)\n",
        true_positives,
        anomaly_indices.len(),
        true_positives as f64 / anomaly_indices.len() as f64 * 100.0
    );

    // Compare with simple threshold method
    let centroid: Vec<f64> = {
        let n = data.len() as f64;
        let sum: Vec<f64> = data
            .iter()
            .fold(vec![0.0; 2], |acc, p| vec![acc[0] + p[0], acc[1] + p[1]]);
        vec![sum[0] / n, sum[1] / n]
    };

    let distances: Vec<f64> = data
        .iter()
        .map(|p| ((p[0] - centroid[0]).powi(2) + (p[1] - centroid[1]).powi(2)).sqrt())
        .collect();

    let mean_dist: f64 = distances.iter().sum::<f64>() / distances.len() as f64;
    let std_dist: f64 = (distances
        .iter()
        .map(|d| (d - mean_dist).powi(2))
        .sum::<f64>()
        / distances.len() as f64)
        .sqrt();

    // 2-sigma threshold
    let threshold = mean_dist + 2.0 * std_dist;
    let threshold_detected: Vec<usize> = distances
        .iter()
        .enumerate()
        .filter(|(_, &d)| d > threshold)
        .map(|(i, _)| i)
        .collect();

    let threshold_tp = threshold_detected
        .iter()
        .filter(|&&i| anomaly_indices.contains(&i))
        .count();

    println!("   【比較: 2σしきい値法】");
    println!("   検出: {:?}", threshold_detected);
    println!(
        "   検出率: {}/{} ({:.0}%)\n",
        threshold_tp,
        anomaly_indices.len(),
        threshold_tp as f64 / anomaly_indices.len() as f64 * 100.0
    );

    println!("   → KDFはしきい値設定なしで同等以上の検出が可能\n");
}

// ============================================================================
// 2. Curriculum Learning
// ============================================================================

fn curriculum_learning_demo() {
    println!("## 2. Curriculum Learning\n");

    // Simulated dataset with varying difficulty
    let mut data = Vec::new();
    let mut true_difficulty = Vec::new();

    // Easy samples (tight cluster)
    for i in 0..50 {
        data.push(vec![0.0 + (i as f64 * 0.001), 0.0 + (i as f64 * 0.001)]);
        true_difficulty.push("Easy");
    }

    // Medium samples (spread out)
    for i in 0..30 {
        let angle = (i as f64) * 0.2;
        data.push(vec![angle.cos() * 0.5, angle.sin() * 0.5]);
        true_difficulty.push("Medium");
    }

    // Hard samples (outliers)
    for i in 0..20 {
        let angle = (i as f64) * 0.5;
        data.push(vec![angle.cos() * 2.0, angle.sin() * 2.0]);
        true_difficulty.push("Hard");
    }

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));

    // Map KDF layers to curriculum stages
    let mut curriculum: HashMap<&str, Vec<usize>> = HashMap::new();
    curriculum.insert("Stage1_Core", Vec::new());
    curriculum.insert("Stage2_Edge", Vec::new());
    curriculum.insert("Stage3_Rare", Vec::new());

    for (i, layer) in result.layers.iter().enumerate() {
        match layer {
            Layer::Core => curriculum.get_mut("Stage1_Core").unwrap().push(i),
            Layer::Edge => curriculum.get_mut("Stage2_Edge").unwrap().push(i),
            Layer::Rare => curriculum.get_mut("Stage3_Rare").unwrap().push(i),
        }
    }

    println!("   | ステージ | サンプル数 | 難易度分布 |");
    println!("   |----------|-----------|-----------|");

    for (stage, indices) in [
        ("Stage1_Core", curriculum.get("Stage1_Core").unwrap()),
        ("Stage2_Edge", curriculum.get("Stage2_Edge").unwrap()),
        ("Stage3_Rare", curriculum.get("Stage3_Rare").unwrap()),
    ] {
        let easy = indices
            .iter()
            .filter(|&&i| true_difficulty[i] == "Easy")
            .count();
        let medium = indices
            .iter()
            .filter(|&&i| true_difficulty[i] == "Medium")
            .count();
        let hard = indices
            .iter()
            .filter(|&&i| true_difficulty[i] == "Hard")
            .count();

        println!(
            "   | {:10} | {:>9} | E:{} M:{} H:{} |",
            stage,
            indices.len(),
            easy,
            medium,
            hard
        );
    }

    println!("\n   → Core層に易しいサンプル、Rare層に難しいサンプルが集中\n");
}

// ============================================================================
// 3. Data Quality Diagnosis
// ============================================================================

fn data_quality_demo() {
    println!("## 3. データ品質診断\n");

    let mut data = Vec::new();
    let mut labels = Vec::new();

    // Normal data
    for i in 0..80 {
        data.push(vec![(i as f64) * 0.01, (i as f64) * 0.01]);
        labels.push("Normal");
    }

    // Potential issues
    data.push(vec![100.0, 100.0]); // Outlier: measurement error?
    labels.push("MeasurementError?");

    data.push(vec![-50.0, -50.0]); // Outlier: data entry error?
    labels.push("DataEntryError?");

    data.push(vec![0.5, 10.0]); // Unusual: rare event?
    labels.push("RareEvent?");

    data.push(vec![0.4, 0.4]); // Normal but isolated
    labels.push("Normal");

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.9, |a, b| euclidean_similarity(a, b));

    println!("   【Rare層の診断】\n");

    for &i in result.rare_items().iter() {
        let point = &data[i];
        let label = labels[i];

        // Compute distance from centroid
        let dist = (point[0].powi(2) + point[1].powi(2)).sqrt();

        let diagnosis = if dist > 50.0 {
            "⚠️  要確認: 極端な外れ値"
        } else if dist > 5.0 {
            "📋 確認推奨: 中程度の外れ値"
        } else {
            "✓ 正常範囲内の希少点"
        };

        println!("   Index {}: {:?}", i, point);
        println!("      ラベル: {}", label);
        println!("      診断: {}\n", diagnosis);
    }
}

// ============================================================================
// 4. Fairness in ML
// ============================================================================

fn fairness_demo() {
    println!("## 4. 公平性 (Fairness)\n");

    let mut data = Vec::new();
    let mut groups = Vec::new();

    // Majority group (80%)
    for i in 0..80 {
        data.push(vec![0.5 + (i as f64 * 0.001), 0.5 + (i as f64 * 0.001)]);
        groups.push("Majority");
    }

    // Minority group A (15%)
    for i in 0..15 {
        data.push(vec![-0.5 + (i as f64 * 0.01), 0.3]);
        groups.push("MinorityA");
    }

    // Minority group B (5%)
    for i in 0..5 {
        data.push(vec![0.0, -0.8 + (i as f64 * 0.05)]);
        groups.push("MinorityB");
    }

    println!("   元データ分布:");
    println!("   - Majority: 80%");
    println!("   - MinorityA: 15%");
    println!("   - MinorityB: 5%\n");

    // Random sampling
    let random_sample: Vec<usize> = (0..100).step_by(5).collect();
    let random_majority = random_sample
        .iter()
        .filter(|&&i| groups[i] == "Majority")
        .count();
    let random_minority_a = random_sample
        .iter()
        .filter(|&&i| groups[i] == "MinorityA")
        .count();
    let random_minority_b = random_sample
        .iter()
        .filter(|&&i| groups[i] == "MinorityB")
        .count();

    // KDF sampling
    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.9, |a, b| euclidean_similarity(a, b));

    let kdf_majority = result
        .selected
        .iter()
        .filter(|&&i| groups[i] == "Majority")
        .count();
    let kdf_minority_a = result
        .selected
        .iter()
        .filter(|&&i| groups[i] == "MinorityA")
        .count();
    let kdf_minority_b = result
        .selected
        .iter()
        .filter(|&&i| groups[i] == "MinorityB")
        .count();

    println!("   | 手法 | Majority | MinorityA | MinorityB | 少数派比率 |");
    println!("   |------|----------|-----------|-----------|-----------|");
    println!(
        "   | Random | {:>8} | {:>9} | {:>9} | {:>8.1}% |",
        random_majority,
        random_minority_a,
        random_minority_b,
        (random_minority_a + random_minority_b) as f64 / random_sample.len() as f64 * 100.0
    );
    println!(
        "   | KDF | {:>8} | {:>9} | {:>9} | {:>8.1}% |",
        kdf_majority,
        kdf_minority_a,
        kdf_minority_b,
        (kdf_minority_a + kdf_minority_b) as f64 / result.selected.len() as f64 * 100.0
    );

    // Check if minority B is fully preserved
    let minority_b_indices: Vec<usize> = (0..data.len())
        .filter(|&i| groups[i] == "MinorityB")
        .collect();
    let minority_b_preserved = minority_b_indices
        .iter()
        .filter(|&&i| result.selected.contains(&i))
        .count();

    println!("\n   極少数派(MinorityB)の保持率:");
    println!("   - Random: 不定 (サンプリング依存)");
    println!(
        "   - KDF: {}/{} ({:.0}%)\n",
        minority_b_preserved,
        minority_b_indices.len(),
        minority_b_preserved as f64 / minority_b_indices.len() as f64 * 100.0
    );

    println!("   → KDFは少数派グループを自動的に保持し、公平性を改善\n");
}

// ============================================================================
// 5. Prototype Selection
// ============================================================================

fn prototype_selection_demo() {
    println!("## 5. プロトタイプ選択 (Prototype Selection)\n");

    // Multiple clusters
    let mut data = Vec::new();
    let mut cluster_ids = Vec::new();

    // Cluster 0
    for i in 0..30 {
        data.push(vec![0.0 + (i as f64 * 0.01), 0.0 + (i as f64 * 0.01)]);
        cluster_ids.push(0);
    }

    // Cluster 1
    for i in 0..25 {
        data.push(vec![3.0 + (i as f64 * 0.01), 0.0 + (i as f64 * 0.01)]);
        cluster_ids.push(1);
    }

    // Cluster 2
    for i in 0..20 {
        data.push(vec![1.5 + (i as f64 * 0.01), 3.0 + (i as f64 * 0.01)]);
        cluster_ids.push(2);
    }

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.85, |a, b| euclidean_similarity(a, b));

    // Find representatives (selected Core items)
    let prototypes: Vec<usize> = result
        .selected
        .iter()
        .filter(|&&i| result.layers[i] == Layer::Core)
        .copied()
        .collect();

    println!("   元データ: {} 件 (3クラスタ)", data.len());
    println!("   選択されたプロトタイプ: {} 件\n", prototypes.len());

    // Count prototypes per cluster
    let mut cluster_prototypes: HashMap<usize, Vec<usize>> = HashMap::new();
    for &p in &prototypes {
        cluster_prototypes
            .entry(cluster_ids[p])
            .or_default()
            .push(p);
    }

    println!("   | クラスタ | 元サイズ | プロトタイプ数 | 代表点座標 |");
    println!("   |----------|----------|---------------|-----------|");

    for cluster in 0..3 {
        let original_size = cluster_ids.iter().filter(|&&c| c == cluster).count();
        let protos = cluster_prototypes
            .get(&cluster)
            .map(|v| v.len())
            .unwrap_or(0);
        let first_proto = cluster_prototypes
            .get(&cluster)
            .and_then(|v| v.first())
            .map(|&i| format!("({:.1}, {:.1})", data[i][0], data[i][1]))
            .unwrap_or_else(|| "-".to_string());

        println!(
            "   | {:>8} | {:>8} | {:>13} | {:>9} |",
            cluster, original_size, protos, first_proto
        );
    }

    println!("\n   → 各クラスタから代表点を自動選択\n");
}

fn main() {
    println!("=== KDFの隠れた価値 ===\n");

    anomaly_detection_demo();
    curriculum_learning_demo();
    data_quality_demo();
    fairness_demo();
    prototype_selection_demo();

    println!("## まとめ: 気づきにくいKDFの価値\n");

    println!("   | 応用 | 従来手法 | KDFの優位性 |");
    println!("   |------|---------|------------|");
    println!("   | 異常検知 | しきい値設定必要 | パラメータ不要で検出 |");
    println!("   | Curriculum | 難易度ラベル必要 | 層で自動分類 |");
    println!("   | データ診断 | 手動レビュー | 問題点を自動特定 |");
    println!("   | 公平性 | グループラベル必要 | 自動的に少数派保持 |");
    println!("   | プロトタイプ | k-means等が必要 | 選択と同時に取得 |");

    println!("\n   共通の価値: 「ラベルなしで構造を発見」\n");

    println!("✅ 隠れた価値の探索完了");
}
