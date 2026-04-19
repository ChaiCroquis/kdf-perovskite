//! KDF 実データ検証
//!
//! KDFの本質に基づく検証実験
//!
//! KDFとは:
//!   - 情報の整理整頓フレームワーク
//!   - 「ゴミに見えても確信がないなら捨てない」ポリシー
//!   - 発見アルゴリズムではない
//!
//! 4層の意味:
//!   - CORE: 知識が飽和している領域（追加情報の価値が低い）
//!   - EDGE: まだ知識に余地がある領域
//!   - RARE: 判断材料が少なすぎる領域（ゴミか宝か不明）
//!   - GARBAGE: 十分な証拠でゴミと確定した領域

use kdf::Kdf;

/// Iris データセットの統計的特性
/// 出典: Fisher, R.A. (1936). UCI Machine Learning Repository
const IRIS_STATS: [(f64, f64, f64, f64, f64, f64, f64, f64); 3] = [
    // Setosa
    (5.006, 0.352, 3.428, 0.379, 1.462, 0.174, 0.246, 0.105),
    // Versicolor
    (5.936, 0.516, 2.770, 0.314, 4.260, 0.470, 1.326, 0.198),
    // Virginica
    (6.588, 0.636, 2.974, 0.322, 5.552, 0.552, 2.026, 0.275),
];

fn main() {
    println!("# KDF 実データ検証\n");
    println!("KDFの本質: 情報整理 + 早まった廃棄の防止\n");
    println!("{}", "=".repeat(60));

    // 検証1: 冗長情報の整理
    validation_redundancy_reduction();

    // 検証2: 判断保留の効果
    validation_deferred_judgment();

    // 検証3: 層分類の一貫性
    validation_layer_consistency();

    println!("\n{}", "=".repeat(60));
    println!("\n検証完了");
}

/// 検証1: 冗長情報の整理（Knowledge Decay）
fn validation_redundancy_reduction() {
    println!("\n## 検証1: 冗長情報の整理\n");
    println!("KDFの核心:");
    println!("  同じことを言っている情報が100個ある");
    println!("  → 知識としての価値は1個分");
    println!("  → 残り99個の「追加価値」は減衰（Decay）する\n");

    let (data, _) = generate_iris_data(150, 42);

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.85, euclidean_similarity);

    let core_items = result.core_items();
    let edge_items = result.edge_items();
    let rare_items = result.rare_items();

    let total = data.len();
    let core_count = core_items.len();
    let edge_count = edge_items.len();
    let rare_count = rare_items.len();

    println!("Iris風データ (n={}) の整理結果:\n", total);
    println!("  {:>10} {:>10} {:>15}", "層", "件数", "解釈");
    println!("  {}", "-".repeat(40));
    println!("  {:>10} {:>10} {:>15}", "Core",
             core_count, "知識飽和（冗長）");
    println!("  {:>10} {:>10} {:>15}", "Edge",
             edge_count, "知識に余地あり");
    println!("  {:>10} {:>10} {:>15}", "Rare",
             rare_count, "判断材料不足");

    let compression = 100.0 * (1.0 - (edge_count + rare_count) as f64 / total as f64);
    println!("\n  情報圧縮率: {:.1}%", compression);
    println!("  （Core層は「同じような情報」として集約可能）");

    println!("\n意味:");
    println!("  - Core層のデータは互いに類似 → 代表1件で知識として十分");
    println!("  - Edge/Rare層は追加情報の価値がある可能性");
}

/// 検証2: 判断保留の効果
fn validation_deferred_judgment() {
    println!("\n## 検証2: 判断保留の効果\n");
    println!("KDFの方針:");
    println!("  「宝だから保存する」← 間違い");
    println!("  「ゴミかどうか確信がないから保存する」← 正しい\n");

    // シナリオ: 主流研究(Pb系)と少数研究(Sn系)
    let (data, labels) = generate_research_scenario(100, 42);

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.85, euclidean_similarity);

    let rare_items = result.rare_items();

    // Rare層に含まれるラベル別の件数
    let mut rare_by_label = [0usize; 3];
    for &idx in rare_items.iter() {
        if idx < labels.len() {
            rare_by_label[labels[idx]] += 1;
        }
    }

    // 元データのラベル分布
    let mut total_by_label = [0usize; 3];
    for &l in &labels {
        total_by_label[l] += 1;
    }

    println!("研究シナリオ:");
    println!("  - 主流研究 (Pb系風): {} 件", total_by_label[0]);
    println!("  - 少数研究A (Sn系風): {} 件", total_by_label[1]);
    println!("  - 少数研究B (孤立): {} 件", total_by_label[2]);

    println!("\nRare層（判断保留）の内訳:");
    println!("  {:>15} {:>10} {:>15}", "研究タイプ", "Rare件数", "保留理由");
    println!("  {}", "-".repeat(45));
    println!("  {:>15} {:>10} {:>15}", "主流 (Pb系)",
             rare_by_label[0], "（通常Core）");
    println!("  {:>15} {:>10} {:>15}", "少数A (Sn系)",
             rare_by_label[1], "データ不足");
    println!("  {:>15} {:>10} {:>15}", "少数B (孤立)",
             rare_by_label[2], "判断材料なし");

    println!("\n解釈:");
    println!("  - Rare層 ≠ 「宝がある場所」");
    println!("  - Rare層 = 「判断できないから捨てない場所」");
    println!("  - 少数研究がRareに入るのは「価値があるから」ではなく");
    println!("    「価値を判断する材料が足りないから」");
}

/// 検証3: 層分類の一貫性
fn validation_layer_consistency() {
    println!("\n## 検証3: 層分類の一貫性\n");
    println!("確認事項: 同じデータに対して同じ分類が得られるか\n");

    let (data, _) = generate_iris_data(100, 42);
    let kdf = Kdf::with_defaults();

    // 複数回実行
    let mut results = Vec::new();
    for _ in 0..5 {
        let result = kdf.process(&data, 0.85, euclidean_similarity);
        results.push(extract_layers(&result, data.len()));
    }

    // 一貫性チェック
    let base = &results[0];
    let all_consistent = results.iter().all(|r| r == base);

    println!("再現性テスト (5回実行):");
    println!("  結果: {}", if all_consistent { "100% 一致" } else { "不一致あり" });

    // 層別の件数
    let core_count = base.iter().filter(|l| *l == "Core").count();
    let edge_count = base.iter().filter(|l| *l == "Edge").count();
    let rare_count = base.iter().filter(|l| *l == "Rare").count();

    println!("\n分類結果:");
    println!("  Core: {} 件 (知識飽和)", core_count);
    println!("  Edge: {} 件 (余地あり)", edge_count);
    println!("  Rare: {} 件 (判断保留)", rare_count);

    println!("\n意味:");
    println!("  - KDFは決定論的（同じ入力→同じ出力）");
    println!("  - 分類基準は「類似度に基づく接続性」");
    println!("  - 「宝の発見」ではなく「情報の整理」");
}

// ============================================================================
// データ生成
// ============================================================================

/// Iris風データ生成
fn generate_iris_data(n: usize, seed: u64) -> (Vec<Vec<f64>>, Vec<usize>) {
    let mut rng = SimpleRng::new(seed);
    let mut data = Vec::new();
    let mut labels = Vec::new();

    let per_class = n / 3;
    for class_idx in 0..3 {
        let stats = IRIS_STATS[class_idx];
        for _ in 0..per_class {
            data.push(vec![
                stats.0 + rng.normal() * stats.1,
                stats.2 + rng.normal() * stats.3,
                stats.4 + rng.normal() * stats.5,
                stats.6 + rng.normal() * stats.7,
            ]);
            labels.push(class_idx);
        }
    }

    (data, labels)
}

/// 研究シナリオデータ生成
/// - 主流研究: 密集したクラスタ（多数）
/// - 少数研究A: 小さなクラスタ（少数）
/// - 少数研究B: 孤立点（極少数）
fn generate_research_scenario(n: usize, seed: u64) -> (Vec<Vec<f64>>, Vec<usize>) {
    let mut rng = SimpleRng::new(seed);
    let mut data = Vec::new();
    let mut labels = Vec::new();

    // 主流研究 (80%): 密集クラスタ
    let n_mainstream = (n as f64 * 0.80) as usize;
    for _ in 0..n_mainstream {
        data.push(vec![
            rng.normal() * 0.5 + 0.0,
            rng.normal() * 0.5 + 0.0,
        ]);
        labels.push(0);
    }

    // 少数研究A (15%): 別の場所に小クラスタ
    let n_minor_a = (n as f64 * 0.15) as usize;
    for _ in 0..n_minor_a {
        data.push(vec![
            rng.normal() * 0.3 + 3.0,
            rng.normal() * 0.3 + 0.0,
        ]);
        labels.push(1);
    }

    // 少数研究B (5%): 孤立点
    let n_minor_b = n - n_mainstream - n_minor_a;
    for i in 0..n_minor_b {
        // 各点を離れた位置に配置
        let angle = (i as f64 / n_minor_b.max(1) as f64) * 2.0 * std::f64::consts::PI;
        let radius = 5.0 + rng.uniform();
        data.push(vec![
            radius * angle.cos(),
            radius * angle.sin(),
        ]);
        labels.push(2);
    }

    (data, labels)
}

// ============================================================================
// ユーティリティ
// ============================================================================

/// 層を抽出
fn extract_layers(result: &kdf::KdfResult, n: usize) -> Vec<String> {
    let rare_items = result.rare_items();
    let edge_items = result.edge_items();
    let core_items = result.core_items();

    (0..n).map(|i| {
        if rare_items.contains(&i) { "Rare".to_string() }
        else if edge_items.contains(&i) { "Edge".to_string() }
        else if core_items.contains(&i) { "Core".to_string() }
        else { "Unknown".to_string() }
    }).collect()
}

/// 簡易乱数生成器
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.state
    }

    fn uniform(&mut self) -> f64 {
        self.next() as f64 / u64::MAX as f64
    }

    fn normal(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-10);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
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
