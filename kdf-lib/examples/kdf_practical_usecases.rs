//! KDF 実用ユースケース
//!
//! 実際の業務で使えるKDFの適用例
//!
//! ユースケース:
//!   1. ログ分析 - 異常イベントの検出と保持
//!   2. テキスト重複排除 - 類似文書の集約
//!   3. 不均衡データ前処理 - 少数クラスの保護
//!   4. キャッシュ管理 - 希少アクセスパターンの保持

use kdf::{Kdf, levenshtein_similarity};

fn main() {
    println!("# KDF 実用ユースケース\n");
    println!("{}", "=".repeat(70));

    // ユースケース1: ログ分析
    usecase_log_analysis();

    // ユースケース2: テキスト重複排除
    usecase_text_dedup();

    // ユースケース3: 不均衡データ前処理
    usecase_imbalanced_data();

    // ユースケース4: キャッシュ管理
    usecase_cache_management();

    println!("\n{}", "=".repeat(70));
    practical_guidelines();
}

/// ユースケース1: ログ分析
fn usecase_log_analysis() {
    println!("\n## ユースケース1: ログ分析\n");
    println!("問題: 大量のログから異常イベントを見逃さずに抽出したい");
    println!("KDFの役割: 繰り返しログを圧縮し、珍しいログを保持\n");

    // シミュレートされたログデータ
    let logs = vec![
        // 正常ログ（大量に繰り返される）
        "INFO: User login successful",
        "INFO: User login successful",
        "INFO: User login successful",
        "INFO: Request processed in 50ms",
        "INFO: Request processed in 52ms",
        "INFO: Request processed in 48ms",
        "INFO: Request processed in 51ms",
        "INFO: Database query completed",
        "INFO: Database query completed",
        "INFO: Database query completed",
        // 警告ログ（少数）
        "WARN: High memory usage detected",
        "WARN: Slow query detected (500ms)",
        // エラーログ（希少）
        "ERROR: Connection timeout to external API",
        "ERROR: Database connection failed",
        // 重大エラー（極めて希少）
        "CRITICAL: Security breach attempt detected",
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&logs, 0.6, |a, b| levenshtein_similarity(a, b));

    println!("入力ログ: {} 件", logs.len());
    println!("選択ログ: {} 件 (削減率: {:.1}%)\n",
        result.selected.len(),
        100.0 * (1.0 - result.selected.len() as f64 / logs.len() as f64)
    );

    println!("選択されたログ:");
    for &i in &result.selected {
        let layer = &result.layers[i];
        let prefix = match layer {
            kdf::Layer::Core => "  [Core]",
            kdf::Layer::Edge => "  [Edge]",
            kdf::Layer::Rare => "→ [Rare]",
        };
        println!("{} {}", prefix, logs[i]);
    }

    // 異常ログの保持率
    let critical_keywords = ["ERROR", "CRITICAL", "WARN"];
    let critical_indices: Vec<_> = logs.iter().enumerate()
        .filter(|(_, log)| critical_keywords.iter().any(|k| log.contains(k)))
        .map(|(i, _)| i)
        .collect();

    let selected_set: std::collections::HashSet<_> = result.selected.iter().cloned().collect();
    let preserved = critical_indices.iter().filter(|i| selected_set.contains(*i)).count();

    println!("\n異常ログ保持率: {}/{} ({:.0}%)",
        preserved, critical_indices.len(),
        100.0 * preserved as f64 / critical_indices.len() as f64
    );

    println!("\n結論: 繰り返しログは圧縮され、異常ログは全て保持される");
}

/// ユースケース2: テキスト重複排除
fn usecase_text_dedup() {
    println!("\n## ユースケース2: テキスト重複排除\n");
    println!("問題: 類似した文書を集約して、ユニークな情報を抽出したい");
    println!("KDFの役割: 類似文書をグループ化し、代表を選択\n");

    let documents = vec![
        // グループ1: 製品紹介（類似）
        "Our product helps you manage tasks efficiently",
        "Our product helps you manage projects efficiently",
        "Our product helps you manage work efficiently",
        // グループ2: 価格情報（類似）
        "Starting at $9.99 per month",
        "Starting at $19.99 per month",
        "Starting at $29.99 per month",
        // グループ3: 技術仕様（類似）
        "Supports Windows, Mac, and Linux",
        "Supports Windows and Mac platforms",
        // ユニークな文書
        "Contact us at support@example.com",
        "Founded in 2020 in San Francisco",
        "We are hiring engineers!",
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&documents, 0.5, |a, b| levenshtein_similarity(a, b));

    println!("入力文書: {} 件", documents.len());
    println!("選択文書: {} 件\n", result.selected.len());

    println!("層別の文書分類:");
    println!("\nCore層（冗長 - 類似文書あり）:");
    for &i in result.core_items().iter() {
        println!("  - {}", documents[i]);
    }

    println!("\nEdge層（部分的に類似）:");
    for &i in result.edge_items().iter() {
        println!("  - {}", documents[i]);
    }

    println!("\nRare層（ユニーク）:");
    for &i in result.rare_items().iter() {
        println!("  → {}", documents[i]);
    }

    println!("\n結論: 類似文書は代表に集約され、ユニークな情報は保持される");
}

/// ユースケース3: 不均衡データ前処理
fn usecase_imbalanced_data() {
    println!("\n## ユースケース3: 不均衡データ前処理\n");
    println!("問題: 多数クラス(正常)と少数クラス(異常)の不均衡");
    println!("KDFの役割: ラベルなしで少数クラスを保護\n");

    // シミュレートされた不均衡データ
    // 特徴量: [feature1, feature2]
    let mut data = Vec::new();
    let mut labels = Vec::new();
    let mut rng = SimpleRng::new(42);

    // 多数クラス（正常）: 90%
    for _ in 0..180 {
        data.push(vec![
            rng.normal() * 0.5 + 0.0,
            rng.normal() * 0.5 + 0.0,
        ]);
        labels.push(0);
    }

    // 少数クラス（異常）: 10%
    for _ in 0..20 {
        data.push(vec![
            rng.normal() * 0.3 + 3.0,
            rng.normal() * 0.3 + 3.0,
        ]);
        labels.push(1);
    }

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&data, 0.7, euclidean_similarity);

    // ラベル別の保持率
    let selected_set: std::collections::HashSet<_> = result.selected.iter().cloned().collect();

    let majority_total = labels.iter().filter(|&&l| l == 0).count();
    let minority_total = labels.iter().filter(|&&l| l == 1).count();

    let majority_selected = labels.iter().enumerate()
        .filter(|(i, &l)| l == 0 && selected_set.contains(i))
        .count();
    let minority_selected = labels.iter().enumerate()
        .filter(|(i, &l)| l == 1 && selected_set.contains(i))
        .count();

    println!("データ分布:");
    println!("  多数クラス（正常）: {} 件", majority_total);
    println!("  少数クラス（異常）: {} 件\n", minority_total);

    println!("KDF処理後:");
    println!("  多数クラス選択: {}/{} ({:.1}%)",
        majority_selected, majority_total,
        100.0 * majority_selected as f64 / majority_total as f64
    );
    println!("  少数クラス選択: {}/{} ({:.1}%)",
        minority_selected, minority_total,
        100.0 * minority_selected as f64 / minority_total as f64
    );

    // Rare層に含まれる少数クラスの割合
    let rare_items = result.rare_items();
    let minority_in_rare = rare_items.iter()
        .filter(|&&i| i < labels.len() && labels[i] == 1)
        .count();

    println!("\nRare層の少数クラス: {}/{} ({:.1}%)",
        minority_in_rare, minority_total,
        100.0 * minority_in_rare as f64 / minority_total as f64
    );

    println!("\n結論:");
    println!("  - ラベルなしで少数クラスを高確率で保持");
    println!("  - 多数クラスは冗長として圧縮");
    println!("  - 不均衡データの前処理に有効");
}

/// ユースケース4: キャッシュ管理
fn usecase_cache_management() {
    println!("\n## ユースケース4: キャッシュ管理\n");
    println!("問題: キャッシュ容量に制限がある中、希少なアクセスパターンを保持したい");
    println!("KDFの役割: 頻出パターンを代表に集約し、希少パターンを保護\n");

    // シミュレートされたアクセスパターン
    // 各パターン: [hour, day_of_week, request_type, user_segment]
    let mut patterns = Vec::new();
    let mut rng = SimpleRng::new(42);

    // 頻出パターン: 平日の日中アクセス
    for _ in 0..150 {
        patterns.push(vec![
            9.0 + rng.uniform() * 9.0,  // 9-18時
            (rng.uniform() * 5.0).floor(),  // 月-金
            0.0,  // 通常リクエスト
            0.0,  // 一般ユーザー
        ]);
    }

    // 中程度: 週末アクセス
    for _ in 0..30 {
        patterns.push(vec![
            10.0 + rng.uniform() * 8.0,  // 10-18時
            5.0 + rng.uniform() * 2.0,  // 土-日
            0.0,
            0.0,
        ]);
    }

    // 希少: 深夜の管理者アクセス
    for _ in 0..10 {
        patterns.push(vec![
            rng.uniform() * 5.0,  // 0-5時
            rng.uniform() * 7.0,  // 任意の曜日
            1.0,  // 管理リクエスト
            1.0,  // 管理者
        ]);
    }

    // 極めて希少: 異常パターン
    patterns.push(vec![2.0, 3.0, 2.0, 2.0]);  // 深夜の未知リクエスト
    patterns.push(vec![3.0, 6.0, 2.0, 0.0]);  // 一般ユーザーの異常リクエスト

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&patterns, 0.8, euclidean_similarity);

    println!("アクセスパターン: {} 件", patterns.len());
    println!("キャッシュ対象: {} 件 (削減率: {:.1}%)\n",
        result.selected.len(),
        100.0 * (1.0 - result.selected.len() as f64 / patterns.len() as f64)
    );

    println!("層別パターン数:");
    println!("  Core（頻出）: {} 件", result.core_items().len());
    println!("  Edge（中程度）: {} 件", result.edge_items().len());
    println!("  Rare（希少）: {} 件", result.rare_items().len());

    // 希少パターンの保持確認
    let rare_indices: Vec<usize> = (patterns.len() - 12..patterns.len()).collect();
    let selected_set: std::collections::HashSet<_> = result.selected.iter().cloned().collect();
    let rare_preserved = rare_indices.iter().filter(|i| selected_set.contains(*i)).count();

    println!("\n希少パターン保持: {}/{} ({:.1}%)",
        rare_preserved, rare_indices.len(),
        100.0 * rare_preserved as f64 / rare_indices.len() as f64
    );

    println!("\n結論:");
    println!("  - 頻出パターンは代表に集約（キャッシュ容量節約）");
    println!("  - 希少パターンは保持（異常検知に活用可能）");
}

/// 実践ガイドライン
fn practical_guidelines() {
    println!("\n# 実践ガイドライン\n");
    println!("┌────────────────────────────────────────────────────────────────────┐");
    println!("│  KDF適用の判断基準                                                │");
    println!("├────────────────────────────────────────────────────────────────────┤");
    println!("│                                                                    │");
    println!("│  ✓ KDFが有効な場合:                                               │");
    println!("│    - 冗長なデータを圧縮したい                                      │");
    println!("│    - 希少なデータを見逃したくない                                  │");
    println!("│    - ラベルがない（教師なし）                                      │");
    println!("│    - 類似度が定義できる                                            │");
    println!("│                                                                    │");
    println!("│  ✗ KDFが不向きな場合:                                             │");
    println!("│    - 類似度の定義が困難                                            │");
    println!("│    - ラベルがあり、ラベルに基づく処理が可能                        │");
    println!("│    - 冗長性がほとんどない                                          │");
    println!("│                                                                    │");
    println!("├────────────────────────────────────────────────────────────────────┤");
    println!("│  パラメータ調整                                                    │");
    println!("├────────────────────────────────────────────────────────────────────┤");
    println!("│                                                                    │");
    println!("│  threshold（類似度閾値）:                                          │");
    println!("│    高い(0.8+): 厳密な類似のみグループ化 → 保持数多め              │");
    println!("│    低い(0.5-): 緩い類似もグループ化 → 圧縮率高め                  │");
    println!("│                                                                    │");
    println!("│  類似度関数:                                                       │");
    println!("│    テキスト → levenshtein_similarity                              │");
    println!("│    数値ベクトル → euclidean_similarity, cosine_similarity          │");
    println!("│    時系列 → dtw_similarity                                        │");
    println!("│                                                                    │");
    println!("└────────────────────────────────────────────────────────────────────┘");
}

// ============================================================================
// ユーティリティ
// ============================================================================

fn euclidean_similarity(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    let dist: f64 = a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}

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
