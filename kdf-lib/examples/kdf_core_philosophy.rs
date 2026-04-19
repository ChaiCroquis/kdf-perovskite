//! KDF Core Philosophy - KDFの本質を示すサンプル
//!
//! このサンプルはKDFの「応用」ではなく「本質」を示す
//!
//! KDFの3つの核心:
//! 1. Knowledge Decay - 同じ情報100個の価値は1個分
//! 2. 部屋の主の視点 - 素人にはゴミでも専門家には価値があるかも
//! 3. 判断保留 - 分からないから捨てない（宝だから保存ではない）

use kdf::Kdf;

fn main() {
    println!("# KDF の本質\n");
    println!("KDFは発見アルゴリズムではない。");
    println!("KDFは情報整理 + 早まった廃棄を防ぐポリシー。\n");
    println!("{}", "=".repeat(60));

    // 核心1: Knowledge Decay
    demo_knowledge_decay();

    // 核心2: 部屋の主の視点
    demo_expert_perspective();

    // 核心3: 判断保留
    demo_deferred_judgment();

    // まとめ
    summary();
}

/// 核心1: Knowledge Decay（知識の減衰）
fn demo_knowledge_decay() {
    println!("\n## 1. Knowledge Decay（知識の減衰）\n");
    println!("同じことを言っている情報が100個ある");
    println!("→ 知識としての価値は1個分");
    println!("→ 残り99個の「追加価値」は減衰（Decay）する\n");

    let kdf = Kdf::with_defaults();

    // シナリオ: Pb系ペロブスカイト論文1000本
    println!("シナリオ: ペロブスカイト研究データベース\n");

    // Pb系（多数派）: 類似データが大量
    let mut data = Vec::new();
    let mut labels = Vec::new();

    // Pb系: 96件（非常に密集した似たようなデータ）
    // ほぼ同じ位置にランダムノイズを加える
    for i in 0..96 {
        // ほぼ同じ位置（微小な違い）
        let noise_x = ((i * 7) % 100) as f64 * 0.001 - 0.05;
        let noise_y = ((i * 13) % 100) as f64 * 0.001 - 0.05;
        data.push(vec![1.0 + noise_x, 1.0 + noise_y]);
        labels.push("Pb系");
    }

    // Sn系: 3件（別の位置）
    data.push(vec![3.0, 3.0]);
    data.push(vec![3.1, 2.9]);
    data.push(vec![2.9, 3.1]);
    labels.push("Sn系");
    labels.push("Sn系");
    labels.push("Sn系");

    // 孤立: 1件（完全に別の位置）
    data.push(vec![6.0, 6.0]);
    labels.push("孤立");

    // θ=0.5 で密集データ同士を接続（デモ用）
    let result = kdf.process(&data, 0.5, euclidean_similarity);

    // 層別の分析
    let core_items = result.core_items();
    let edge_items = result.edge_items();
    let rare_items = result.rare_items();

    println!("   入力データ: 100件");
    println!("   - Pb系: 96件（似たようなデータ）");
    println!("   - Sn系: 3件");
    println!("   - 孤立: 1件\n");

    println!("   KDFの整理結果:");
    println!("   - Core/Edge層: {} 件 ← 類似データ（冗長）", core_items.len() + edge_items.len());
    println!("   - Rare層: {} 件 ← 判断材料不足（孤立）\n", rare_items.len());

    // Core/Edge層の内訳
    let connected_pb = core_items.iter().chain(edge_items.iter())
        .filter(|&&i| labels[i] == "Pb系").count();
    println!("   接続データ（Core/Edge）の内訳:");
    println!("   - Pb系: {} 件 → 互いに類似、知識として集約可能", connected_pb);

    println!("\n   【Knowledge Decay】");
    println!("   Pb系論文1本目: 価値 = 100%");
    println!("   Pb系論文2本目: 価値 = 10%（確認）");
    println!("   Pb系論文3本目以降: 価値 ≈ 0%（冗長）");
    println!("\n   → 96件の価値は数件分に「減衰」する");
}

/// 核心2: 部屋の主の視点
fn demo_expert_perspective() {
    println!("\n## 2. 部屋の主（専門家）の視点\n");

    println!("普通の人が見ると:");
    println!("  「このSn系の論文、効率2%しかないからゴミでしょ」\n");

    println!("部屋の主（専門家/Coreの知識）が見ると:");
    println!("  「いや、構造的には筋がいいんだよ。捨てないで」\n");

    let kdf = Kdf::with_defaults();

    // Coreに類似しているが、効率が低いデータ
    let mut data = Vec::new();

    // 主流派（効率20%以上）
    for i in 0..20 {
        let eff = 20.0 + (i as f64 * 0.5);
        let structure = 1.0 + (i as f64 * 0.02);
        data.push(vec![eff, structure]);
    }

    // 効率は低いが構造的に類似（Sn系）
    // 効率2%だが、構造パラメータは主流派に近い
    data.push(vec![2.0, 1.1]);  // 低効率だが構造OK
    data.push(vec![3.0, 1.05]); // 低効率だが構造OK

    // 完全に異なるもの
    data.push(vec![50.0, 5.0]); // 高効率だが構造が異なる

    let result = kdf.process(&data, 0.85, euclidean_similarity);

    let rare_items = result.rare_items();
    let edge_items = result.edge_items();

    println!("   データ構成:");
    println!("   - 主流派（高効率）: 20件");
    println!("   - 低効率だが構造類似: 2件");
    println!("   - 高効率だが構造異質: 1件\n");

    println!("   KDFの判断:");

    // 低効率・構造類似の位置を確認
    let low_eff_structural = [20, 21];
    let high_eff_unusual = 22;

    for &idx in &low_eff_structural {
        let layer = if rare_items.contains(&idx) {
            "Rare（判断保留）"
        } else if edge_items.contains(&idx) {
            "Edge（余地あり）"
        } else {
            "Core（知識内）"
        };
        println!("   - 低効率・構造類似[{}]: {}", idx, layer);
    }

    let unusual_layer = if rare_items.contains(&high_eff_unusual) {
        "Rare（判断保留）"
    } else if edge_items.contains(&high_eff_unusual) {
        "Edge（余地あり）"
    } else {
        "Core（知識内）"
    };
    println!("   - 高効率・構造異質[{}]: {}", high_eff_unusual, unusual_layer);

    println!("\n   【部屋の主の視点】");
    println!("   効率だけで判断すると: 低効率 = ゴミ");
    println!("   構造も見ると: 低効率でも構造OK → 捨てるな");
    println!("\n   KDFは「類似度」で判断するため、");
    println!("   専門家が重視する「構造」の情報を自然に考慮できる");
}

/// 核心3: 判断保留
fn demo_deferred_judgment() {
    println!("\n## 3. 判断保留（捨てない理由）\n");

    println!("誤解: 「宝だから保存する」");
    println!("正解: 「ゴミかどうか確信がないから保存する」\n");

    let kdf = Kdf::with_defaults();

    // シナリオ: 新しいデータが来た時の判断
    let mut data = Vec::new();

    // 既知の領域（十分なデータあり）
    for i in 0..30 {
        let x = (i % 6) as f64 * 0.2;
        let y = (i / 6) as f64 * 0.2;
        data.push(vec![x, y]);
    }

    // 新しいデータ（判断材料不足）
    let new_data_indices = vec![
        data.len(), // インデックスを記録
        data.len() + 1,
        data.len() + 2,
    ];
    data.push(vec![5.0, 5.0]);  // 孤立
    data.push(vec![2.0, 2.0]);  // やや離れた位置
    data.push(vec![0.5, 0.5]);  // 既知領域に近い

    let result = kdf.process(&data, 0.85, euclidean_similarity);

    let rare_items = result.rare_items();
    let edge_items = result.edge_items();
    let core_items = result.core_items();

    println!("   新しいデータの分類:\n");

    for (i, &idx) in new_data_indices.iter().enumerate() {
        let pos = &data[idx];
        let (layer, reason) = if rare_items.contains(&idx) {
            ("Rare", "判断材料不足 → 保留")
        } else if edge_items.contains(&idx) {
            ("Edge", "知識に余地 → 注目")
        } else if core_items.contains(&idx) {
            ("Core", "既知の領域 → 冗長")
        } else {
            ("不明", "分類失敗")
        };

        println!("   新データ{}: 位置({:.1}, {:.1})", i + 1, pos[0], pos[1]);
        println!("      → {}: {}\n", layer, reason);
    }

    println!("   【判断保留の本質】");
    println!("   Rare ≠ 宝がある場所");
    println!("   Rare = 判断できないから捨てない場所");
    println!("\n   RAREに宝があるかどうかは分からない。");
    println!("   分からないから捨てない。それだけ。");
}

/// まとめ
fn summary() {
    println!("\n{}", "=".repeat(60));
    println!("\n## まとめ: KDFの本当の価値\n");

    println!("【価値があるもの】");
    println!("1. 冗長情報の圧縮");
    println!("   → 1000本のPb論文を「Pb系は高効率」という1つの知識に");
    println!();
    println!("2. 早まった廃棄の防止");
    println!("   → 「効率2%だからゴミ」という判断を保留");
    println!();
    println!("3. 判断の透明化");
    println!("   → なぜ捨てないのか（RARE）、なぜ集約するのか（CORE）を明示");

    println!("\n【価値がないもの（主張すべきでない）】");
    println!("1. 効率的な発見");
    println!("   → 発見効率の向上は証明されていない");
    println!();
    println!("2. 宝の予測");
    println!("   → RAREに宝がある確率が高いとは言えない");
    println!();
    println!("3. 優先順位付け");
    println!("   → 類似度による優先順位の効果は弱い");

    println!("\n{}", "=".repeat(60));
    println!("\nKDF = 情報整理 + 保守的保存ポリシー");
    println!();
    println!("核心: 「冗長は減らせ、でも確信がないなら捨てるな」");
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
