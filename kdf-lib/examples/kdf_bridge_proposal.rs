//! KDF Bridge Proposal: Connecting Rare to Core
//!
//! Concept: How can minority opinions (Rare) be modified minimally
//! to connect with majority (Core) while preserving uniqueness?
//!
//! 1. Rare layer: Isolated unique perspectives
//! 2. Core layer: Well-connected mainstream views
//! 3. Gap: The difference between Rare and nearest Core
//! 4. Bridge: Minimal modification to become "hearable"
//!
//! Applications:
//! - Negotiation: Find common ground while keeping principles
//! - Product design: Bridge niche and mainstream features
//! - Communication: Reframe ideas for broader acceptance

use kdf::Kdf;

fn euclidean_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    1.0 / (1.0 + euclidean_distance(a, b))
}

/// Bridge proposal from Rare to Core
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct BridgeProposal {
    /// Original Rare item index
    rare_idx: usize,
    /// Target Core item index (nearest connectable point)
    target_core_idx: usize,
    /// Original position
    original: Vec<f64>,
    /// Proposed bridged position
    bridged: Vec<f64>,
    /// Uniqueness preserved (0.0 = completely compromised, 1.0 = unchanged)
    uniqueness_preserved: f64,
    /// Connectivity gained (0.0 = still isolated, 1.0 = fully connected)
    connectivity_gained: f64,
    /// Modification vector (what needs to change)
    modification: Vec<f64>,
    /// Key dimensions that were modified
    key_changes: Vec<(usize, f64, String)>,  // (dim, change, interpretation)
}

/// Analyze the gap between Rare and Core
struct GapAnalyzer {
    kdf: Kdf,
    sim_threshold: f64,
}

impl GapAnalyzer {
    fn new(sim_threshold: f64) -> Self {
        Self {
            kdf: Kdf::with_defaults(),
            sim_threshold,
        }
    }

    /// Find bridge proposals for all Rare items
    fn analyze(
        &self,
        data: &[Vec<f64>],
        dim_names: Option<&[&str]>,
    ) -> Vec<BridgeProposal>
    {
        let result = self.kdf.process(data, self.sim_threshold, |a, b| euclidean_similarity(a, b));

        let rare_items = result.rare_items();
        let core_items = result.core_items();
        let edge_items = result.edge_items();

        // Potential bridge targets (Core + Edge)
        let targets: Vec<usize> = core_items.iter()
            .chain(edge_items.iter())
            .copied()
            .collect();

        if targets.is_empty() {
            return vec![];
        }

        let mut proposals = Vec::new();

        for &rare_idx in &rare_items {
            // Find nearest target
            let (nearest_idx, _nearest_dist) = targets.iter()
                .map(|&t| (t, euclidean_distance(&data[rare_idx], &data[t])))
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            // Calculate bridge position (partial move towards Core)
            // Bridge ratio: how much to move towards Core (0.5 = halfway)
            let bridge_ratio = self.calculate_optimal_bridge_ratio(
                &data[rare_idx],
                &data[nearest_idx],
                data,
                &result,
            );

            let original = data[rare_idx].clone();
            let target = &data[nearest_idx];

            // Bridged position
            let bridged: Vec<f64> = original.iter()
                .zip(target)
                .map(|(o, t)| o + (t - o) * bridge_ratio)
                .collect();

            // Modification vector
            let modification: Vec<f64> = bridged.iter()
                .zip(&original)
                .map(|(b, o)| b - o)
                .collect();

            // Uniqueness preserved = 1 - bridge_ratio
            let uniqueness_preserved = 1.0 - bridge_ratio;

            // Connectivity gained = similarity to target after bridging
            let connectivity_gained = euclidean_similarity(&bridged, target);

            // Identify key changes
            let key_changes = self.identify_key_changes(&modification, dim_names);

            proposals.push(BridgeProposal {
                rare_idx,
                target_core_idx: nearest_idx,
                original,
                bridged,
                uniqueness_preserved,
                connectivity_gained,
                modification,
                key_changes,
            });
        }

        proposals
    }

    /// Calculate optimal bridge ratio that achieves connectivity
    fn calculate_optimal_bridge_ratio(
        &self,
        rare: &[f64],
        core: &[f64],
        data: &[Vec<f64>],
        result: &kdf::KdfResult,
    ) -> f64 {
        // Binary search for minimum ratio that achieves similarity threshold
        let mut low = 0.0;
        let mut high = 1.0;

        for _ in 0..20 {
            let mid = (low + high) / 2.0;

            // Calculate bridged position
            let bridged: Vec<f64> = rare.iter()
                .zip(core)
                .map(|(r, c)| r + (c - r) * mid)
                .collect();

            // Check if this achieves connectivity
            let connected = result.core_items().iter()
                .any(|&i| euclidean_similarity(&bridged, &data[i]) >= self.sim_threshold);

            if connected {
                high = mid;  // Can achieve with less modification
            } else {
                low = mid;   // Need more modification
            }
        }

        // Return slightly above threshold to ensure connectivity
        (high + 0.05).min(0.8)  // Cap at 0.8 to preserve some uniqueness
    }

    /// Identify which dimensions changed the most
    fn identify_key_changes(
        &self,
        modification: &[f64],
        dim_names: Option<&[&str]>,
    ) -> Vec<(usize, f64, String)> {
        let mut changes: Vec<(usize, f64)> = modification.iter()
            .enumerate()
            .map(|(i, &m)| (i, m.abs()))
            .collect();

        changes.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        changes.into_iter()
            .take(3)
            .map(|(dim, _change)| {
                let name = dim_names
                    .and_then(|names| names.get(dim))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Dim{}", dim));

                let direction = if modification[dim] > 0.0 { "↑" } else { "↓" };
                let interpretation = format!("{} {}", name, direction);

                (dim, modification[dim], interpretation)
            })
            .collect()
    }
}

/// Generate consultation/recommendation text
#[allow(dead_code)]
fn generate_consultation(proposal: &BridgeProposal, _dim_names: Option<&[&str]>) -> String {
    let mut text = String::new();

    text.push_str(&format!("【提案】アイテム {} の橋渡し戦略\n\n", proposal.rare_idx));

    text.push_str(&format!("  現状: 孤立点 (Core層との接続なし)\n"));
    text.push_str(&format!("  目標: Core層アイテム {} に接続\n\n", proposal.target_core_idx));

    text.push_str("  推奨される調整:\n");
    for (_dim, change, interp) in &proposal.key_changes {
        let magnitude = if change.abs() > 0.5 { "大幅に" }
            else if change.abs() > 0.2 { "中程度" }
            else { "少し" };
        text.push_str(&format!("    - {} {} 調整 ({:+.2})\n", interp, magnitude, change));
    }

    text.push_str(&format!("\n  予測効果:\n"));
    text.push_str(&format!("    - ユニークさ保持: {:.0}%\n", proposal.uniqueness_preserved * 100.0));
    text.push_str(&format!("    - 接続性獲得: {:.0}%\n", proposal.connectivity_gained * 100.0));

    text
}

// ============================================================================
// Demo: Opinion bridging
// ============================================================================

fn opinion_bridging_demo() {
    println!("## 1. 意見の橋渡しシミュレーション\n");

    // Opinions as vectors: [革新性, 実用性, コスト意識, リスク許容度]
    let dim_names = ["革新性", "実用性", "コスト意識", "リスク許容"];

    let mut opinions = Vec::new();
    let mut labels = Vec::new();

    // Mainstream opinions (Core候補)
    for i in 0..20 {
        opinions.push(vec![
            0.3 + (i as f64 * 0.01),  // 中程度の革新性
            0.7 + (i as f64 * 0.01),  // 高い実用性
            0.6,                       // 中程度のコスト意識
            0.3,                       // 低いリスク許容度
        ]);
        labels.push("主流派");
    }

    // Minority unique opinions (Rare候補)
    // 革新派: 高い革新性、低い実用性
    opinions.push(vec![0.9, 0.2, 0.4, 0.8]);
    labels.push("革新派A");

    // コスト重視派: 極端なコスト意識
    opinions.push(vec![0.2, 0.5, 0.95, 0.1]);
    labels.push("コスト重視派");

    // リスクテイカー: 高いリスク許容度
    opinions.push(vec![0.6, 0.4, 0.3, 0.95]);
    labels.push("リスクテイカー");

    let analyzer = GapAnalyzer::new(0.8);
    let proposals = analyzer.analyze(
        &opinions,
        Some(&dim_names),
    );

    println!("   少数派の意見と橋渡し提案:\n");

    for proposal in &proposals {
        let label = labels[proposal.rare_idx];

        println!("   ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("   【{}】(Index: {})", label, proposal.rare_idx);
        println!();

        // Original position
        print!("   現在の立場: ");
        for (i, (&val, name)) in proposal.original.iter().zip(dim_names.iter()).enumerate() {
            if i > 0 { print!(", "); }
            print!("{}={:.1}", name, val);
        }
        println!();

        // Target Core
        print!("   最寄りの主流派: ");
        let target = &opinions[proposal.target_core_idx];
        for (i, (&val, name)) in target.iter().zip(dim_names.iter()).enumerate() {
            if i > 0 { print!(", "); }
            print!("{}={:.1}", name, val);
        }
        println!();

        // Recommended adjustments
        println!();
        println!("   💡 橋渡し提案:");
        for (dim, change, _) in &proposal.key_changes {
            let name = dim_names[*dim];
            let original = proposal.original[*dim];
            let new_val = original + change;
            let direction = if *change > 0.0 { "↑上げる" } else { "↓下げる" };

            println!("      {} を {:.1} → {:.1} ({}) ",
                     name, original, new_val, direction);
        }

        println!();
        println!("   📊 効果予測:");
        println!("      ユニークさ: {:.0}% 維持", proposal.uniqueness_preserved * 100.0);
        println!("      接続性: {:.0}% 獲得", proposal.connectivity_gained * 100.0);
        println!();
    }
}

// ============================================================================
// Demo: Product feature bridging
// ============================================================================

fn product_bridging_demo() {
    println!("\n## 2. プロダクト機能の橋渡し\n");

    // Product features: [先進性, 使いやすさ, 価格競争力, 信頼性]
    let dim_names = ["先進性", "使いやすさ", "価格", "信頼性"];

    let mut products = Vec::new();
    let mut names = Vec::new();

    // Mainstream products
    for i in 0..15 {
        products.push(vec![
            0.4 + (i as f64 * 0.01),
            0.8 - (i as f64 * 0.01),
            0.5,
            0.7,
        ]);
        names.push(format!("標準製品{}", i));
    }

    // Niche products
    products.push(vec![0.95, 0.3, 0.2, 0.5]);  // 超先進的だが使いにくい
    names.push("先進製品X".to_string());

    products.push(vec![0.2, 0.9, 0.9, 0.4]);   // 安くて簡単だが古い
    names.push("廉価製品Y".to_string());

    let analyzer = GapAnalyzer::new(0.85);
    let proposals = analyzer.analyze(
        &products,
        Some(&dim_names),
    );

    println!("   ニッチ製品の主流市場参入戦略:\n");

    for proposal in &proposals {
        let name = &names[proposal.rare_idx];

        println!("   【{}】", name);

        // Current features
        println!("   現在: {:?}", proposal.original.iter()
            .zip(dim_names.iter())
            .map(|(v, n)| format!("{}:{:.1}", n, v))
            .collect::<Vec<_>>()
            .join(", "));

        // Recommended changes
        println!("   提案:");
        for (dim, change, _) in &proposal.key_changes {
            if change.abs() > 0.1 {
                let action = if *change > 0.0 { "強化" } else { "抑制" };
                println!("     → {} を {} ({:+.1})", dim_names[*dim], action, change);
            }
        }

        println!("   効果: 独自性{:.0}%維持 / 市場適合{:.0}%\n",
                 proposal.uniqueness_preserved * 100.0,
                 proposal.connectivity_gained * 100.0);
    }
}

// ============================================================================
// Demo: Communication reframing
// ============================================================================

fn communication_demo() {
    println!("\n## 3. コミュニケーション・リフレーミング\n");

    // Message dimensions: [論理性, 感情訴求, 具体性, 新規性]
    let dim_names = ["論理性", "感情訴求", "具体性", "新規性"];

    let mut messages = Vec::new();

    // Messages that resonate (Core)
    for i in 0..12 {
        messages.push(vec![
            0.6 + (i as f64 * 0.02),
            0.5,
            0.7,
            0.3,
        ]);
    }

    // Messages that don't resonate (Rare)
    messages.push(vec![0.9, 0.1, 0.3, 0.8]);  // Too abstract and new
    messages.push(vec![0.2, 0.9, 0.2, 0.2]);  // Too emotional, vague

    let analyzer = GapAnalyzer::new(0.8);
    let proposals = analyzer.analyze(
        &messages,
        Some(&dim_names),
    );

    println!("   伝わりにくいメッセージの改善提案:\n");

    for (i, proposal) in proposals.iter().enumerate() {
        let msg_type = if i == 0 { "論理重視型" } else { "感情重視型" };

        println!("   【メッセージ: {}】", msg_type);
        println!("   問題: 聴衆に届いていない\n");

        println!("   リフレーミング提案:");
        for (dim, change, _) in &proposal.key_changes {
            let name = dim_names[*dim];
            let suggestion = match (*dim, *change > 0.0) {
                (0, false) => "論理を減らし、直感的な表現に",
                (0, true) => "根拠を追加して説得力を",
                (1, false) => "感情表現を控えめに",
                (1, true) => "感情に訴える要素を追加",
                (2, false) => "抽象度を上げる",
                (2, true) => "具体例を追加する",
                (3, false) => "既知の概念と結びつける",
                (3, true) => "新しさを強調する",
                _ => "調整する",
            };
            println!("     → {}: {}", name, suggestion);
        }

        println!("\n   期待効果: 独自性{:.0}%維持、共感{:.0}%獲得\n",
                 proposal.uniqueness_preserved * 100.0,
                 proposal.connectivity_gained * 100.0);
    }
}

fn main() {
    println!("=== KDF Bridge Proposal: 少数意見の橋渡し ===\n");

    println!("コンセプト:");
    println!("┌─────────────────────────────────────────────────────┐");
    println!("│  Rare層 (少数意見)                                   │");
    println!("│    ↓                                                │");
    println!("│  Gap分析: 何が違うのか？                             │");
    println!("│    ↓                                                │");
    println!("│  Bridge提案: 最小限の調整で接続可能に                │");
    println!("│    ↓                                                │");
    println!("│  Core層に届く + ユニークさを維持                     │");
    println!("└─────────────────────────────────────────────────────┘\n");

    opinion_bridging_demo();
    product_bridging_demo();
    communication_demo();

    println!("## まとめ\n");

    println!("   【橋渡しの原則】");
    println!("   1. 完全な迎合ではなく「最小限の調整」");
    println!("   2. ユニークさの核心部分は維持");
    println!("   3. 「接続可能性」を獲得するための変更点を特定");
    println!();

    println!("   【KDFの役割】");
    println!("   - Rare/Coreの自動分類");
    println!("   - Gap (距離) の定量化");
    println!("   - 最適な橋渡し比率の計算");
    println!("   - どの次元を調整すべきかの特定");

    println!("\n✅ 橋渡し提案システム完了");
}
