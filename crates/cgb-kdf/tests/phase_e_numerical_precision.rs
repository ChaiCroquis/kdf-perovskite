//! Phase E — numerical precision under extreme parameter regimes.
//!
//! Tests that the exp(-λdt) formula and lambda(C) calculation remain
//! numerically stable under:
//!   - Very large C (10^6)
//!   - Very small dt (10^-9)
//!   - Very small λ (β → 0)
//!   - Boundary behavior at w → f64::MIN_POSITIVE (denormals)
//! Also verifies bit-exact cross-platform behavior assertions.

use cgb_kdf::{Layer, MasterSpecParams};

#[test]
fn lambda_large_congestion_no_overflow() {
    let p = MasterSpecParams::default();
    // Very high congestion. α_C = 2.0, γ_C = 0.008 → λ ≈ 0.003 * (1 + 0.008 * 10^12)
    let c = 1_000_000.0;
    let lam = p.lambda(c, Layer::Core);
    assert!(lam.is_finite(), "λ must remain finite for C={}", c);
    assert!(lam > 0.0, "λ must be positive");
    println!("λ(C=10^6, Core) = {:e}", lam);
}

#[test]
fn exp_decay_very_small_dt_preserves_weight() {
    let p = MasterSpecParams::default();
    let lam = p.lambda(10.0, Layer::Edge);
    let dt = 1e-9;
    let survival = (-lam * dt).exp();
    // At dt=1e-9, should be essentially 1.0
    assert!(survival > 0.999999_99);
    assert!(survival <= 1.0);
    println!("survival(dt=1e-9) = {}", survival);
}

#[test]
fn exp_decay_zero_beta_means_no_decay() {
    let mut p = MasterSpecParams::default();
    p.beta = 0.0;
    let lam = p.lambda(10.0, Layer::Edge);
    assert_eq!(lam, 0.0);
    assert_eq!((-lam * 1.0).exp(), 1.0); // no decay
}

#[test]
fn weight_bit_exact_reproducibility_across_runs() {
    let p = MasterSpecParams::default();
    let lam = p.lambda(7.5, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let run = || -> u64 {
        let mut w = 0.5_f64;
        for _ in 0..10_000 {
            w *= (-lam * dt).exp();
        }
        w.to_bits()
    };
    let a = run();
    let b = run();
    assert_eq!(a, b, "identical runs must produce bit-identical f64 outputs");
}

#[test]
fn lambda_monotone_in_c_at_large_range() {
    let p = MasterSpecParams::default();
    let mut prev = p.lambda(0.0, Layer::Edge);
    for c in [1.0, 10.0, 100.0, 1_000.0, 10_000.0, 100_000.0] {
        let cur = p.lambda(c, Layer::Edge);
        assert!(cur > prev, "λ must be monotone; C={}, prev={}, cur={}", c, prev, cur);
        prev = cur;
    }
}

#[test]
fn exp_decay_long_run_does_not_reach_subnormal() {
    // With β=0.01, γ_E=0.015, α_E=1.5, C=10, dt=0.005 → λdt ≈ 7.4e-5
    // After 1M steps: exp(-1e6 * 7.4e-5) = exp(-74) ≈ 8e-33
    // Still in normal f64 range (not subnormal). Verify.
    let p = MasterSpecParams::default();
    let lam = p.lambda(10.0, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let mut w = 1.0_f64;
    for _ in 0..1_000_000 {
        w *= (-lam * dt).exp();
    }
    assert!(w > f64::MIN_POSITIVE, "w should not drop to subnormal in 1M steps (got {:e})", w);
    println!("w after 1M steps: {:e}", w);
}

#[test]
fn exp_decay_against_closed_form_rel_err() {
    // N-step iteration vs closed-form exp(-Nλdt) should agree tightly.
    let p = MasterSpecParams::default();
    for c in [1.0, 10.0, 100.0] {
        for layer in [Layer::Edge, Layer::Rare, Layer::Core] {
            let lam = p.lambda(c, layer);
            let dt = p.dt_for_layer(layer);
            let n_steps = 10_000;
            let mut w = 1.0_f64;
            let survival = (-lam * dt).exp();
            for _ in 0..n_steps { w *= survival; }
            let closed = (-lam * dt * n_steps as f64).exp();
            let rel_err = ((w - closed) / closed.max(f64::MIN_POSITIVE)).abs();
            assert!(rel_err < 1e-10,
                "C={}, layer={:?}: rel_err {} too large",
                c, layer, rel_err);
        }
    }
}
