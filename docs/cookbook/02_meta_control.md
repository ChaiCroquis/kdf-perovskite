# Cookbook 02 — メタ制御手段(Claim 27-32)

長期運用で平均接続量が目標から乖離したときに、減衰パラメータ α を自動調整する例。

## コード

```rust
use cgb_kdf::{MasterSpecParams, MetaController, Layer};

fn main() {
    let mc = MetaController::default();
    let mut params = MasterSpecParams::default();

    println!("initial α_E = {:.4}, α_C = {:.4}", params.alpha_edge, params.alpha_core);

    // Lyapunov 安定性を確認 (Rev.11 §7.4)
    assert!(mc.check_lyapunov_stability(), "η² > μ² must hold");

    // 1000 ティックのシミュレーション
    for t in 0..1000 {
        // 観測された平均接続量(外部センサから来ると仮定)
        let avg_k_edge = 8.0 + 2.0 * ((t as f64) / 100.0).sin();
        let avg_k_core = 4.0 + 1.0 * ((t as f64) / 80.0).cos();

        let (d_alpha_e, d_alpha_c) = mc.step(&mut params, avg_k_edge, avg_k_core);

        if t % 100 == 0 {
            println!(
                "t={} avg_k_edge={:.2} → α_E={:.4} (Δ={:+.4})",
                t, avg_k_edge, params.alpha_edge, d_alpha_e
            );
        }
    }
}
```

## ポイント

- `MetaController::default()` は **[Claim 39 数値範囲を自動で満たす canonical 値** を持つ
  - η = 0.15, μ = 0.08, H_target = 0.70
  - α_E ∈ [1.0, 2.5], α_C ∈ [1.5, 3.0] (Claim 30 範囲)
- `check_lyapunov_stability()` は η² > μ² を実行時検証 (Rev.11 §7.4)
- Claim 29 により、δk が 2 倍になれば Δα は **16 倍** にスケール

## 緊急介入 (Claim 31)

健全性が緊急条件を満たす場合、低重みエッジを優先削除:

```rust
use cgb_kdf::MetaController;

let mut mc = MetaController::default();
let edges = vec![
    ((0, 1), 0.01),
    ((1, 2), 0.02),
    ((2, 3), 0.50),
];
// 健全性 h < 0.30 (default emergency_health_threshold) を擬似的に発動
let picked = mc.emergency_intervention(/* avg_k */ 0.0, edges.into_iter());
println!("緊急削除対象: {:?}", picked);
assert_eq!(mc.emergency_count, 1);
```

## モード切替 (Claim 32)

```rust
let mut mc = MetaController::default();
mc.set_enabled(false);  // メタ制御オフ
// step() は no-op になる
```
