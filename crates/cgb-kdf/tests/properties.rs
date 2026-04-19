//! Phase 3 property-based tests (proptest)
//!
//! Validates algebraic/monotonic properties over wide randomly-generated
//! input ranges. Each `proptest!` macro block runs 256 randomized cases by
//! default (configurable via PROPTEST_CASES env var).

use cgb_kdf::{Layer, MasterSpecParams, MetaController};
use proptest::prelude::*;

proptest! {
    /// Claim 8-9: λ(C) monotone-increasing in C for any positive C.
    #[test]
    fn lambda_monotone_in_c(c1 in 0.0f64..1000.0, dc in 0.001f64..100.0) {
        let p = MasterSpecParams::default();
        let c2 = c1 + dc;
        for layer in [Layer::Edge, Layer::Rare, Layer::Core] {
            let l1 = p.lambda(c1, layer);
            let l2 = p.lambda(c2, layer);
            prop_assert!(l2 >= l1, "λ must be monotone; layer={:?}, c1={}, c2={}, l1={}, l2={}", layer, c1, c2, l1, l2);
        }
    }

    /// Claim 14: survival factor exp(-λdt) ∈ (0, 1].
    #[test]
    fn survival_in_unit_interval(c in 0.0f64..1000.0) {
        let p = MasterSpecParams::default();
        for layer in [Layer::Edge, Layer::Rare, Layer::Core] {
            let lambda = p.lambda(c, layer);
            let dt = p.dt_for_layer(layer);
            let s = (-lambda * dt).exp();
            prop_assert!(s > 0.0 && s <= 1.0, "survival out of bounds: {} (c={}, layer={:?})", s, c, layer);
        }
    }

    /// P_decay probability lies in [0, 1] for all valid congestion values.
    #[test]
    fn p_decay_is_probability(c in 0.0f64..10_000.0) {
        let p = MasterSpecParams::default();
        for layer in [Layer::Edge, Layer::Rare, Layer::Core] {
            let lambda = p.lambda(c, layer);
            let dt = p.dt_for_layer(layer);
            let p_decay = 1.0 - (-lambda * dt).exp();
            prop_assert!((0.0..=1.0).contains(&p_decay), "P_decay out of [0,1]: {}", p_decay);
        }
    }

    /// N-step iteration agrees with closed-form exp(-N·λ·dt) within tight tolerance.
    #[test]
    fn n_step_agrees_with_closed_form(
        c in 0.01f64..100.0,
        n in 10usize..5_000,
    ) {
        let p = MasterSpecParams::default();
        let lambda = p.lambda(c, Layer::Edge);
        let dt = p.dt_for_layer(Layer::Edge);
        let mut w = 1.0_f64;
        let survival = (-lambda * dt).exp();
        for _ in 0..n { w *= survival; }
        let closed = (-lambda * dt * n as f64).exp();
        let rel = ((w - closed) / closed.max(f64::MIN_POSITIVE)).abs();
        prop_assert!(rel < 1e-10, "n-step rel_err {} at n={}, c={}", rel, n, c);
    }

    /// Claim 29: δk^4 scaling is exact (up to floating rounding).
    #[test]
    fn fourth_power_scaling_exact(k1 in 7.0f64..50.0, scale in 1.1f64..5.0) {
        // Set up controller with η=0, H_target=1 so only the μ·δk^4 term contributes.
        let mc = MetaController { eta: 0.0, mu: 1.0, health_target: 1.0, ..MetaController::default() };
        let k_opt = mc.k_opt_edge;
        // Ensure avg_k > k_opt so δk > 0
        prop_assume!(k1 > k_opt);
        let k2 = k_opt + (k1 - k_opt) * scale; // δk2 = scale · δk1
        let d1 = mc.alpha_update(Layer::Edge, k1);
        let d2 = mc.alpha_update(Layer::Edge, k2);
        let expected_ratio = scale.powi(4);
        let actual_ratio = d2 / d1;
        let rel = (actual_ratio - expected_ratio).abs() / expected_ratio;
        prop_assert!(rel < 1e-9, "Δα ratio {} vs scale^4 {} rel_err={}", actual_ratio, expected_ratio, rel);
    }

    /// Claim 30: α stays inside declared bounds no matter how we push it.
    #[test]
    fn alpha_clamped_under_extreme_kick(
        avg_k_edge in 0.0f64..1_000.0,
        avg_k_core in 0.0f64..1_000.0,
    ) {
        let mc = MetaController::default();
        let mut params = MasterSpecParams::default();
        // 100 harsh steps
        for _ in 0..100 {
            mc.step(&mut params, avg_k_edge, avg_k_core);
            prop_assert!(params.alpha_edge >= mc.alpha_edge_bounds.0);
            prop_assert!(params.alpha_edge <= mc.alpha_edge_bounds.1);
            prop_assert!(params.alpha_core >= mc.alpha_core_bounds.0);
            prop_assert!(params.alpha_core <= mc.alpha_core_bounds.1);
        }
    }

    /// Claim 32: enabled=false must make step a no-op (idempotent on params).
    #[test]
    fn disabled_meta_is_noop(avg_k_edge in 0.0f64..100.0, avg_k_core in 0.0f64..100.0) {
        let mut mc = MetaController::default();
        mc.set_enabled(false);
        let mut params = MasterSpecParams::default();
        let before = params.clone();
        for _ in 0..50 {
            mc.step(&mut params, avg_k_edge, avg_k_core);
        }
        prop_assert_eq!(params.alpha_edge.to_bits(), before.alpha_edge.to_bits());
        prop_assert_eq!(params.alpha_core.to_bits(), before.alpha_core.to_bits());
    }
}
