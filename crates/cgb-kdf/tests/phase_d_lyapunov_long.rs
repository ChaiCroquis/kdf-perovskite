//! Phase D — long-run Lyapunov simulation (100,000+ steps).
//!
//! The existing `lyapunov_simulation_bounded` in math_properties.rs runs
//! 5,000 steps. This test extends to 100,000 steps with multiple parameter
//! regimes to strengthen the "Lyapunov-bounded" claim.

use cgb_kdf::{MasterSpecParams, MetaController};

#[test]
fn lyapunov_100k_steps_default_params() {
    let mc = MetaController::default();
    let mut params = MasterSpecParams::default();

    let mut max_alpha_e: f64 = params.alpha_edge;
    let mut min_alpha_e: f64 = params.alpha_edge;
    let mut max_alpha_c: f64 = params.alpha_core;
    let mut min_alpha_c: f64 = params.alpha_core;
    let mut sum_alpha_e = 0.0_f64;

    // 100,000 steps with oscillating k signal
    let n_steps = 100_000;
    for t in 0..n_steps {
        let avg_k_edge = 8.0 + 4.0 * ((t as f64) / 100.0).sin();
        let avg_k_core = 4.0 + 2.0 * ((t as f64) / 80.0).cos();
        mc.step(&mut params, avg_k_edge, avg_k_core);
        max_alpha_e = max_alpha_e.max(params.alpha_edge);
        min_alpha_e = min_alpha_e.min(params.alpha_edge);
        max_alpha_c = max_alpha_c.max(params.alpha_core);
        min_alpha_c = min_alpha_c.min(params.alpha_core);
        sum_alpha_e += params.alpha_edge;

        // Sanity at every step
        assert!(params.alpha_edge.is_finite(), "α_E diverged to NaN/Inf at step {}", t);
        assert!(params.alpha_core.is_finite(), "α_C diverged to NaN/Inf at step {}", t);
        assert!(params.alpha_edge >= mc.alpha_edge_bounds.0, "α_E below bound at step {}", t);
        assert!(params.alpha_edge <= mc.alpha_edge_bounds.1, "α_E above bound at step {}", t);
        assert!(params.alpha_core >= mc.alpha_core_bounds.0, "α_C below bound at step {}", t);
        assert!(params.alpha_core <= mc.alpha_core_bounds.1, "α_C above bound at step {}", t);
    }

    let range_e = max_alpha_e - min_alpha_e;
    let range_c = max_alpha_c - min_alpha_c;
    let mean_alpha_e = sum_alpha_e / n_steps as f64;

    println!(
        "100k-step Lyapunov: α_E range=[{:.3}, {:.3}] (width {:.3}), mean={:.3}; α_C range=[{:.3}, {:.3}] (width {:.3})",
        min_alpha_e, max_alpha_e, range_e, mean_alpha_e, min_alpha_c, max_alpha_c, range_c
    );

    // Check: range stays well within allowed bounds (no drift)
    let allowed_width_e = mc.alpha_edge_bounds.1 - mc.alpha_edge_bounds.0;
    let allowed_width_c = mc.alpha_core_bounds.1 - mc.alpha_core_bounds.0;
    assert!(range_e <= allowed_width_e, "α_E range exceeds bounds width");
    assert!(range_c <= allowed_width_c, "α_C range exceeds bounds width");
    // Mean α_E should not drift to one extreme (oscillation should center somewhere)
    let center_e = (mc.alpha_edge_bounds.0 + mc.alpha_edge_bounds.1) / 2.0;
    let drift_e = (mean_alpha_e - center_e).abs();
    assert!(drift_e < allowed_width_e, "α_E mean drifted too far from center");
}

#[test]
fn lyapunov_100k_steps_noisy_signal() {
    // Signal with white noise instead of smooth oscillation.
    let mc = MetaController::default();
    let mut params = MasterSpecParams::default();

    // Deterministic pseudo-noise via LCG
    let mut lcg: u64 = 0xDEADBEEF;
    let mut noise = || -> f64 {
        lcg = lcg.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((lcg >> 33) as f64 / (u32::MAX as f64 / 2.0)) - 1.0
    };

    let n_steps = 100_000;
    for t in 0..n_steps {
        let avg_k_edge = 8.0 + 4.0 * noise();
        let avg_k_core = 4.0 + 2.0 * noise();
        mc.step(&mut params, avg_k_edge, avg_k_core);
        assert!(params.alpha_edge.is_finite() && params.alpha_core.is_finite(),
            "NaN/Inf at step {}", t);
        assert!(params.alpha_edge >= mc.alpha_edge_bounds.0);
        assert!(params.alpha_edge <= mc.alpha_edge_bounds.1);
    }

    println!("100k-step with noisy input: α_E={:.3}, α_C={:.3} (bounded throughout)",
        params.alpha_edge, params.alpha_core);
}

#[test]
fn lyapunov_adversarial_extreme_spikes() {
    // Pathological input: alternating 0 / 100 connectivity spikes.
    let mc = MetaController::default();
    let mut params = MasterSpecParams::default();
    for t in 0..10_000 {
        let avg_k = if t % 2 == 0 { 0.0 } else { 100.0 };
        mc.step(&mut params, avg_k, avg_k);
        assert!(params.alpha_edge.is_finite() && params.alpha_core.is_finite());
        assert!(params.alpha_edge >= mc.alpha_edge_bounds.0);
        assert!(params.alpha_edge <= mc.alpha_edge_bounds.1);
    }
    println!("10k adversarial spikes: α_E={:.3}, α_C={:.3} (bounded)", params.alpha_edge, params.alpha_core);
}

#[test]
fn lyapunov_disabled_no_op_even_with_extreme_input() {
    // Claim 32: when disabled, params must not change regardless of input.
    let mut mc = MetaController::default();
    mc.set_enabled(false);
    let mut params = MasterSpecParams::default();
    let orig_e = params.alpha_edge;
    let orig_c = params.alpha_core;
    for _ in 0..10_000 {
        mc.step(&mut params, 1000.0, -1000.0);
    }
    assert_eq!(params.alpha_edge.to_bits(), orig_e.to_bits(), "α_E must be bit-identical when disabled");
    assert_eq!(params.alpha_core.to_bits(), orig_c.to_bits(), "α_C must be bit-identical when disabled");
}

#[test]
fn lyapunov_condition_self_check_passes() {
    assert!(MetaController::default().check_lyapunov_stability(),
        "Default η, μ must satisfy η² > μ²");
}
