//! Phase U — full TransitionController loop value test.
//!
//! Problem: Phase N's "dynamic loop" only used ActivationScore + MetaController.step.
//! TransitionController struct was instantiated but never called. This test
//! wires the FULL Claim 23-32 pipeline (with real promotion/demotion decisions)
//! and compares to Phase N's partial loop. Does adding TransitionController
//! ADD value, or is ActivationScore alone sufficient?

use adversarial_bench as adv;
use cgb_kdf::{
    ActivationScore, Layer, MasterSpecParams, MetaController, NodeClassifier,
    RegionKind, SemanticImportance, TransitionController, TransitionScore,
};
use real_data_bench::Dataset;
use std::collections::HashSet;

const N_DATASET_SEEDS: usize = 5;
const N_STEPS: usize = 5;

fn avg_connectivity(ds: &Dataset) -> f64 {
    if ds.n_nodes == 0 { return 0.0; }
    let mut deg = vec![0u32; ds.n_nodes];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < ds.n_nodes { deg[u as usize] += 1; }
        if (v as usize) < ds.n_nodes { deg[v as usize] += 1; }
    }
    let non_iso: Vec<u32> = deg.into_iter().filter(|&d| d > 0).collect();
    if non_iso.is_empty() { 0.0 } else {
        non_iso.iter().sum::<u32>() as f64 / non_iso.len() as f64
    }
}

fn run_full_loop(dataset_seed: u64) -> Vec<(usize, f64, f64, f64)> {
    // Returns (step, static_recall, phase_n_partial_recall, phase_u_full_recall)
    let snapshots = adv::temporal_snapshots(500, N_STEPS, dataset_seed);

    // Phase N style: ActivationScore + MetaController only
    let mut activation_partial = ActivationScore::default();
    activation_partial.decay_rate = 0.05;
    let mc_partial = MetaController::default();
    let mut params_partial = MasterSpecParams::default();

    // Phase U style: + TransitionController with explicit promote/demote
    let mut activation_full = ActivationScore::default();
    activation_full.decay_rate = 0.05;
    let mc_full = MetaController::default();
    let mut params_full = MasterSpecParams::default();
    let tc = TransitionController {
        score_config: TransitionScore { w_connectivity: 0.3, w_activation: 0.5, w_semantic: 0.2 },
        promote_threshold: 0.6,
        demote_threshold: 0.3,
        ..Default::default()
    };
    let si = SemanticImportance::default();

    // Track per-node "current region" — transitions between these are where
    // TransitionController exerts its influence
    let mut region_of: std::collections::HashMap<u32, RegionKind> = std::collections::HashMap::new();
    let mut transitions_triggered = 0u32;

    let mut results = Vec::new();
    for (step, ds) in snapshots.iter().enumerate() {
        let keep = (ds.n_nodes as f64 * 0.30).ceil() as usize;

        // ---- shared: classify current snapshot ----
        let mut classifier = NodeClassifier::default();
        let class = classifier.classify(ds.n_nodes, &ds.edges);

        // ---- update ActivationScore (both variants) ----
        activation_partial.advance_tick();
        activation_full.advance_tick();
        for (&id, &l) in &class.layers {
            if matches!(l, Layer::Rare) {
                activation_partial.record_event(id);
                activation_full.record_event(id);
            }
        }

        // ---- MetaController adapts α ----
        let avg_k = avg_connectivity(ds);
        mc_partial.step(&mut params_partial, avg_k, avg_k / 2.0);
        mc_full.step(&mut params_full, avg_k, avg_k / 2.0);

        // ---- FULL PATH: TransitionController makes region decisions ----
        let mut neighbors: std::collections::HashMap<u32, Vec<u32>> = std::collections::HashMap::new();
        for &(u, v, _) in &ds.edges {
            neighbors.entry(u).or_default().push(v);
            neighbors.entry(v).or_default().push(u);
        }
        for (&id, &layer) in &class.layers {
            let current_region = *region_of.entry(id).or_insert(match layer {
                Layer::Core => RegionKind::LongTerm,
                Layer::Rare => RegionKind::Rare,
                Layer::Edge => RegionKind::ShortTerm,
                Layer::Garbage => RegionKind::ShortTerm,
            });
            let connectivity = neighbors.get(&id).map(|v| v.len() as f64).unwrap_or(0.0);
            let ns = neighbors.get(&id).cloned().unwrap_or_default();
            if let Some(new_region) = tc.step(id, current_region, connectivity, &ns) {
                region_of.insert(id, new_region);
                transitions_triggered += 1;
            }
        }
        let _ = si; // unused (Claim 26 reference set empty here)

        // ---- Scoring ----
        let score_static_only = |id: u32| -> f64 {
            let l = class.layers.get(&id).copied().unwrap_or(Layer::Edge);
            match l { Layer::Rare => 3.0, Layer::Core => 2.0, Layer::Edge => 1.0, Layer::Garbage => 0.0 }
        };
        let score_partial = |id: u32| -> f64 {
            score_static_only(id) + activation_partial.get(id) * 5.0
        };
        let score_full = |id: u32| -> f64 {
            let base = score_static_only(id) + activation_full.get(id) * 5.0;
            // FULL path: region membership adds signal
            let region_bonus = match region_of.get(&id).copied().unwrap_or(RegionKind::ShortTerm) {
                RegionKind::Rare => 2.0,
                RegionKind::LongTerm => 0.5,
                RegionKind::ShortTerm => 0.0,
            };
            base + region_bonus
        };

        let pick = |scorer: &dyn Fn(u32) -> f64| -> HashSet<u32> {
            let mut scored: Vec<(u32, f64)> = (0..ds.n_nodes as u32).map(|id| (id, scorer(id))).collect();
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored.into_iter().take(keep).map(|(i, _)| i).collect()
        };

        let static_sel = pick(&score_static_only);
        let partial_sel = pick(&score_partial);
        let full_sel = pick(&score_full);

        let recall = |sel: &HashSet<u32>| -> f64 {
            if ds.rare_ground_truth.is_empty() { return 0.0; }
            sel.intersection(&ds.rare_ground_truth).count() as f64
                / ds.rare_ground_truth.len() as f64
        };

        results.push((step, recall(&static_sel), recall(&partial_sel), recall(&full_sel)));
    }
    println!("   (dataset_seed={}: TransitionController fired {} times over {} steps)",
        dataset_seed, transitions_triggered, N_STEPS);
    results
}

fn main() {
    println!("# Phase U — Full TransitionController Loop Value Test\n");
    println!("Does adding TransitionController + region-based scoring IMPROVE over");
    println!("Phase N's ActivationScore-only partial loop?\n");

    let seeds: Vec<u64> = (0..N_DATASET_SEEDS as u64).map(|i| 42 + i * 100).collect();
    let mut agg: Vec<(f64, f64, f64, usize)> = vec![(0.0, 0.0, 0.0, 0); N_STEPS];

    for &seed in &seeds {
        let results = run_full_loop(seed);
        for (step, rs, rp, rf) in results {
            agg[step].0 += rs;
            agg[step].1 += rp;
            agg[step].2 += rf;
            agg[step].3 += 1;
        }
    }

    println!("\n## Aggregated recall across {} seeds\n", N_DATASET_SEEDS);
    println!("| Step | Static | Phase-N (Partial) | **Phase-U (Full)** | Δ (Full-Partial) |");
    println!("|---:|---:|---:|---:|---:|");
    for step in 0..N_STEPS {
        let n = agg[step].3 as f64;
        let rs = agg[step].0 / n;
        let rp = agg[step].1 / n;
        let rf = agg[step].2 / n;
        let delta = rf - rp;
        let marker = if delta.abs() < 0.01 { "≈" }
                     else if delta > 0.0 { "✅" }
                     else { "❌" };
        println!("| t={} | {:.3} | {:.3} | {:.3} | {}{:+.3} |", step, rs, rp, rf, marker, delta);
    }

    println!("\n## Interpretation");
    println!("- If Full-Partial Δ ≈ 0: TransitionController adds NO value over ActivationScore");
    println!("- If Full-Partial Δ > 0: TransitionController genuinely improves temporal handling");
    println!("- If Full-Partial Δ < 0: TransitionController HURTS (region-bonus wrongly boosts non-rares)");
}
