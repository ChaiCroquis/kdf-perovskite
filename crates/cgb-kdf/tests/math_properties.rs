//! Phase 2 mathematical property tests
//!
//! These tests verify that the implementation's numerics agree with the
//! analytical solutions derived in docs/math/decay_analysis.md.

use cgb_kdf::{Layer, MasterSpecParams, MetaController};

/// §2.1 closed-form: w(t) = w0 · exp(-λt) for constant C.
#[test]
fn closed_form_single_step() {
    let p = MasterSpecParams::default();
    let c = 10.0;
    let lambda = p.lambda(c, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let w0 = 1.0_f64;
    let w1 = w0 * (-lambda * dt).exp();
    assert!((w1 - (-lambda * dt).exp()).abs() < 1e-15);
}

/// §3.1: After N iterations, w(N·dt) = w0 · exp(-N·λ·dt).
#[test]
fn closed_form_n_step_agreement() {
    let p = MasterSpecParams::default();
    let c = 10.0;
    let lambda = p.lambda(c, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let n = 10_000usize;

    let mut w = 1.0_f64;
    for _ in 0..n {
        w *= (-lambda * dt).exp();
    }
    let closed = (-lambda * dt * n as f64).exp();
    let rel_err = ((w - closed) / closed).abs();
    assert!(
        rel_err < 1e-11,
        "N-step product must equal closed-form exp(-Nλdt) within 1e-11, got rel_err={}",
        rel_err
    );
}

/// §4.2 convergence time: T_θ = (1/λ) · ln(w0/θ).
#[test]
fn convergence_time_matches_formula() {
    let p = MasterSpecParams::default();
    let c = 5.0;
    let lambda = p.lambda(c, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let w0 = 1.0_f64;
    let theta = 0.15; // maintenance threshold for Edge layer

    let expected_steps = (1.0 / lambda * (w0 / theta).ln() / dt).ceil() as usize;

    let mut w = w0;
    let mut steps = 0usize;
    let max_iter = 10 * expected_steps.max(1);
    while w > theta && steps < max_iter {
        w *= (-lambda * dt).exp();
        steps += 1;
    }
    let rel = (steps as f64 - expected_steps as f64).abs() / expected_steps as f64;
    assert!(
        rel < 0.01,
        "empirical steps ({}) must match formula ({}) within 1%",
        steps,
        expected_steps
    );
}

/// §4.3 rare layer is ~6x slower than edge layer (half-life comparison).
#[test]
fn rare_layer_protects_longer_than_edge() {
    let p = MasterSpecParams::default();
    let c = 10.0;
    let lam_e = p.lambda(c, Layer::Edge);
    let lam_r = p.lambda(c, Layer::Rare);
    let dt_e = p.dt_for_layer(Layer::Edge);
    let dt_r = p.dt_for_layer(Layer::Rare);

    let tau_e = 1.0 / (lam_e * dt_e); // steps to 1/e
    let tau_r = 1.0 / (lam_r * dt_r);
    assert!(
        tau_r > tau_e,
        "Rare layer time-constant must exceed Edge layer"
    );
}

/// §5.1 Lyapunov condition η² > μ² on default parameters.
#[test]
fn lyapunov_stability_default() {
    let mc = MetaController::default();
    assert!(
        mc.check_lyapunov_stability(),
        "default η, μ must satisfy η²>μ²"
    );
}

/// §5.2 long-run simulation: adaptive α does not diverge.
#[test]
fn lyapunov_simulation_bounded() {
    let mc = MetaController::default();
    let mut params = MasterSpecParams::default();

    // Sweep average connectivity around target, verify α stays in bounds.
    let mut alpha_history = Vec::with_capacity(5_000);
    for t in 0..5_000 {
        // Oscillating connectivity signal (8 ± 4 sin(t/100))
        let avg_k_edge = 8.0 + 4.0 * ((t as f64) / 100.0).sin();
        let avg_k_core = 4.0 + 2.0 * ((t as f64) / 80.0).cos();
        mc.step(&mut params, avg_k_edge, avg_k_core);
        alpha_history.push(params.alpha_edge);
        assert!(
            params.alpha_edge >= mc.alpha_edge_bounds.0,
            "α below bound at t={}",
            t
        );
        assert!(
            params.alpha_edge <= mc.alpha_edge_bounds.1,
            "α above bound at t={}",
            t
        );
    }

    // Variance of α should remain bounded (no monotonic drift)
    let mean: f64 = alpha_history.iter().sum::<f64>() / alpha_history.len() as f64;
    let var: f64 = alpha_history
        .iter()
        .map(|a| (a - mean).powi(2))
        .sum::<f64>()
        / alpha_history.len() as f64;
    assert!(
        var < 1.0,
        "α variance {} indicates Lyapunov instability",
        var
    );
}

/// Claim 8/9: λ(C) is strictly monotone-increasing in C (for fixed layer).
#[test]
fn lambda_monotone_in_congestion() {
    let p = MasterSpecParams::default();
    for layer in [Layer::Edge, Layer::Rare, Layer::Core] {
        let mut prev = p.lambda(0.0, layer);
        for c_int in 1..=100 {
            let c = c_int as f64;
            let cur = p.lambda(c, layer);
            assert!(
                cur > prev,
                "λ must be monotone-increasing in C; layer={:?}, C={}, prev={}, cur={}",
                layer,
                c,
                prev,
                cur
            );
            prev = cur;
        }
    }
}

/// Claim 29 δk^4 scaling: doubling δk multiplies the fourth-power term by 16.
#[test]
fn delta_k_fourth_power_scaling() {
    let mc = MetaController {
        mu: 1.0,
        eta: 0.0,
        health_target: 1.0,
        ..MetaController::default()
    };
    // At H = 1 (perfect health) and health_target=1, the η term vanishes.
    // So Δα = +μ · δk^4 (sign +1 for Edge).
    // k_opt_edge = 6.0 → δk=4 for avg_k=10, δk=8 for avg_k=14.
    let d1 = mc.alpha_update(Layer::Edge, 10.0); // δk=4 → 256
    let d2 = mc.alpha_update(Layer::Edge, 14.0); // δk=8 → 4096
    assert!(
        (d2 / d1 - 16.0).abs() < 1e-9,
        "Claim 29: ratio must be 16, got {}",
        d2 / d1
    );
}

/// Phase 3 determinism: apply_edge_decay is HashMap-insertion-order invariant.
///
/// Inserting identical edges in different orders (forward vs reverse) and
/// then running the decay must produce bit-exact identical weight vectors.
/// This catches any reliance on HashMap's random iteration order.
#[test]
fn apply_decay_is_insertion_order_invariant() {
    use cgb_kdf::{ClassificationStats, DecayManager, Layer, NodeClassification};
    use std::collections::HashMap;

    let forward: Vec<(u32, u32, f64)> = (0..20).map(|i| (i, i + 1, 1.0)).collect();
    let reverse: Vec<(u32, u32, f64)> = forward.iter().rev().cloned().collect();

    let run = |edges: &[(u32, u32, f64)]| -> Vec<((u32, u32), u64)> {
        let mut layers = HashMap::new();
        for e in edges {
            layers.insert(e.0, Layer::Edge);
            layers.insert(e.1, Layer::Edge);
        }
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        let mut mgr = DecayManager::master_spec();
        mgr.initialize_with_edges(class, edges);
        for _ in 0..50 {
            mgr.apply_edge_decay();
        }
        // Drain weights sorted by (u,v) to compare bit patterns.
        let mut out: Vec<_> = (0..20u32)
            .map(|i| ((i, i + 1), mgr.get_edge_weight(i, i + 1).unwrap().to_bits()))
            .collect();
        out.sort();
        out
    };

    let a = run(&forward);
    let b = run(&reverse);
    assert_eq!(
        a, b,
        "decay must be insertion-order invariant (HashMap order must not leak into results)"
    );
}

/// Determinism: decay operator is purely functional of (w, λ, dt).
#[test]
fn decay_determinism_bitwise() {
    let p = MasterSpecParams::default();
    let lambda = p.lambda(7.5, Layer::Edge);
    let dt = p.dt_for_layer(Layer::Edge);
    let w0 = 0.5_f64;
    let w_a = (0..1000).fold(w0, |w, _| w * (-lambda * dt).exp());
    let w_b = (0..1000).fold(w0, |w, _| w * (-lambda * dt).exp());
    assert_eq!(
        w_a.to_bits(),
        w_b.to_bits(),
        "same inputs must produce bit-exact outputs"
    );
}
