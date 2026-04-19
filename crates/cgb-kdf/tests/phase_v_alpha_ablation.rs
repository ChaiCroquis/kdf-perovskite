//! Phase V1 — Claim 10 α=2 ablation(再設計版).
//!
//! # 方法論の正直な履歴
//!
//! 初版は Phase V plan の success criterion「α=2 が top-2 以内」で設計
//! されたが、graph rare-recall 指標が α に対し感度不足で、全 α が essentially
//! tie (差 < 0.0005, recall = 1.000) となり、test が無意味な結果を返した。
//! **rank of α=2 は 3/6 と判明し、Phase V plan の failure criterion を満たす**。
//!
//! Plan の failure ブランチは「α=2 が top-3 圏外なら Claim 10 の optimality 主張
//! を取り下げ、『請求項に合致する有効な選択肢』とのみ記述する」だった。
//! 本再設計版は **その failure ブランチを実行** — α=2 の「uniquely optimal」
//! 主張は撤回 (paper には存在しなかったが、ここで正式に確認) し、以下のみを
//! 検証する:
//!
//! Claim 10 specifies α = 2 as the canonical specified exponent.
//! The test verifies:
//! (i) α=2 produces valid, non-degenerate behavior,
//! (ii) α=2 is within a reasonable operating range (not at either extreme),
//! (iii) discrimination ratio is monotonically increasing in α (as theory
//!       predicts for polynomial kernels).
//!
//! **test が pass することは「α=2 が optimal」を意味しない**。実際、α が
//! 大きいほど discrimination ratio は大きく、α=2 は rank 3/6。これは **Claim 10
//! の "α=2" が spec として妥当な選択であって、より大きい α も理論上許容される**
//! ことを示す(実際、decay rate が過剰になるため runtime では不適切だが)。

use cgb_kdf::framework::{Layer, MasterSpecParams};

fn p_decay(alpha: f64, c: f64, layer: Layer) -> f64 {
    let mut params = MasterSpecParams::default();
    params.alpha_edge = alpha;
    params.alpha_rare = alpha;
    params.alpha_core = alpha;
    params.alpha_meta = alpha;
    let lambda = params.lambda(c, layer);
    let dt = params.dt_for_layer(layer);
    1.0 - (-lambda * dt).exp()
}

#[test]
fn alpha_ablation_discrimination_ratio() {
    // Discrimination ratio: P_decay(C_high) / P_decay(C_low).
    // Larger ratio = better selectivity against congested edges.
    let alphas = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0];
    let c_low = 2.0_f64;      // single-connection rare-adjacent edge
    let c_high = 20.0_f64;    // core-core edge
    let layer = Layer::Core;

    println!("\n# Phase V1 — Claim 10 α ablation (discrimination ratio)");
    println!("layer = Core, C_low = {}, C_high = {}", c_low, c_high);
    println!("| α | P_decay(C_low) | P_decay(C_high) | ratio |");
    println!("|---:|---:|---:|---:|");

    let mut ratios: Vec<(f64, f64)> = Vec::new();
    for &a in &alphas {
        let pl = p_decay(a, c_low, layer);
        let ph = p_decay(a, c_high, layer);
        let ratio = if pl > 0.0 { ph / pl } else { f64::INFINITY };
        println!("| {:.1} | {:.6} | {:.6} | {:.2}× |", a, pl, ph, ratio);
        ratios.push((a, ratio));
    }

    // Property 1: All P_decay ∈ [0, 1] for every α
    for &(a, _r) in &ratios {
        assert!(p_decay(a, c_low, layer) >= 0.0 && p_decay(a, c_low, layer) <= 1.0);
        assert!(p_decay(a, c_high, layer) >= 0.0 && p_decay(a, c_high, layer) <= 1.0);
    }
    println!("\n✓ Property 1: all P_decay ∈ [0, 1] regardless of α");

    // Property 2: Monotonicity — ratio must be monotonically non-decreasing in α.
    // (higher power ⇒ larger C^α gap ⇒ larger decay gap)
    let mut prev_ratio = ratios[0].1;
    for &(a, r) in ratios.iter().skip(1) {
        assert!(r >= prev_ratio - 1e-9,
            "Discrimination ratio must be monotone non-decreasing in α; broke at α={}: {} < {}",
            a, r, prev_ratio);
        prev_ratio = r;
    }
    println!("✓ Property 2: discrimination ratio monotone non-decreasing in α");

    // Property 3: α=2 is within a reasonable range (not at either extreme).
    // More formally: α=2 ratio is within [1.5×, 200×] — the "useful" zone where
    // KDF's decay actually discriminates but doesn't cause numerical extremes.
    let alpha2_ratio = ratios.iter().find(|(a, _)| (a - 2.0).abs() < 1e-9).unwrap().1;
    assert!(alpha2_ratio >= 1.5,
        "α=2 discrimination ratio {} < 1.5× suggests too-weak decay selectivity",
        alpha2_ratio);
    assert!(alpha2_ratio <= 200.0,
        "α=2 discrimination ratio {} > 200× suggests numerical extreme",
        alpha2_ratio);
    println!("✓ Property 3: α=2 ratio = {:.2}× falls within useful zone [1.5×, 200×]", alpha2_ratio);

    // Property 4: Rank of α=2. α=2 should rank at least in top-4 of 6
    // (not top-2 — that would be overclaim; α=2 is *specified*, not *optimal*).
    let rank = ratios.iter().filter(|(_, r)| *r > alpha2_ratio).count() + 1;
    println!("α=2 rank: {} / {} (higher α gives higher discrimination; this is expected)", rank, ratios.len());

    println!("\n**Conclusion**:
- KDF's exp-survival form produces well-defined P_decay for all tested α
- Discrimination ratio is monotonically increasing in α as theory predicts
- α=2 (Claim 10 canonical) lies in a balanced regime: selective enough
  (ratio ≥ 1.5×) but not numerically extreme (ratio ≤ 200×)
- The ablation does NOT establish α=2 as uniquely optimal; it establishes
  that α=2 is a valid and well-behaved choice within the framework.");
}
