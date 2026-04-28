//! Parallel hotspots benchmark.
//!
//! Measures wall-clock time of two embarrassingly-parallel hotspots in cgb-kdf:
//!   - `DecayManager::apply_edge_decay` over n=10^5 edges
//!   - `CausalEngine::batch_compute` over n_series=200, n_pairs≈5000
//!
//! The same workload is run twice with the same seed and the resulting state is
//! compared bit-exactly to verify Claim 15 determinism is preserved when rayon
//! is enabled.
//!
//! Run: cargo run --release -p cgb-kdf --example parallel_hotspots_bench

use cgb_kdf::causal::{CausalEngine, TeStrategy};
use cgb_kdf::framework::{ClassificationStats, DecayManager, Layer, NodeClassification};
use rand::Rng;
use rand::SeedableRng;
use rand::rngs::SmallRng;
use std::collections::HashMap;
use std::time::Instant;

const N_NODES: usize = 20_000;
const N_EDGES: usize = 100_000;
const N_DECAY_ITERS: usize = 5;
const N_SERIES: usize = 200;
const SERIES_LEN: usize = 200;
const N_PAIRS: usize = 5_000;
const N_BATCH_RUNS: usize = 3;
const SEED: u64 = 0xCAFE_F00D;

fn build_decay_manager(seed: u64) -> (DecayManager, Vec<(u32, u32)>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut layers = HashMap::with_capacity(N_NODES);
    for n in 0..N_NODES as u32 {
        // 5% Rare, 25% Core, 70% Edge — realistic skew.
        let r: f64 = rng.r#gen();
        let layer = if r < 0.05 {
            Layer::Rare
        } else if r < 0.30 {
            Layer::Core
        } else {
            Layer::Edge
        };
        layers.insert(n, layer);
    }
    let stats = ClassificationStats::default();
    let class = NodeClassification {
        layers,
        rare_fingerprints: HashMap::new(),
        stats,
    };

    // Random edges with weights in [0.5, 1.0]
    let mut edges: Vec<(u32, u32, f64)> = Vec::with_capacity(N_EDGES);
    let mut seen = std::collections::HashSet::new();
    while edges.len() < N_EDGES {
        let u: u32 = rng.gen_range(0..N_NODES as u32);
        let v: u32 = rng.gen_range(0..N_NODES as u32);
        if u == v {
            continue;
        }
        let key = if u < v { (u, v) } else { (v, u) };
        if !seen.insert(key) {
            continue;
        }
        let w = 0.5 + 0.5 * rng.r#gen::<f64>();
        edges.push((key.0, key.1, w));
    }

    let mut mgr = DecayManager::master_spec();
    mgr.initialize_with_edges(class, &edges);
    let mut keys: Vec<(u32, u32)> = edges.iter().map(|(u, v, _)| (*u, *v)).collect();
    keys.sort();
    (mgr, keys)
}

fn snapshot_weights(mgr: &DecayManager, keys: &[(u32, u32)]) -> Vec<((u32, u32), f64)> {
    keys.iter()
        .map(|&(u, v)| ((u, v), mgr.get_edge_weight(u, v).unwrap_or(0.0)))
        .collect()
}

fn build_causal_inputs(seed: u64) -> (HashMap<String, Vec<f64>>, Vec<(String, String)>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut data = HashMap::with_capacity(N_SERIES);
    for i in 0..N_SERIES {
        let mut series = Vec::with_capacity(SERIES_LEN);
        for j in 0..SERIES_LEN {
            let phase = (j as f64 * 0.1).sin() + 0.05 * rng.r#gen::<f64>();
            series.push(phase + 0.001 * (i as f64));
        }
        data.insert(format!("S{}", i), series);
    }
    let mut candidates = Vec::with_capacity(N_PAIRS);
    while candidates.len() < N_PAIRS {
        let s = rng.gen_range(0..N_SERIES);
        let t = rng.gen_range(0..N_SERIES);
        if s != t {
            candidates.push((format!("S{}", s), format!("S{}", t)));
        }
    }
    (data, candidates)
}

fn bench_decay(label: &str) -> Vec<((u32, u32), f64)> {
    let (mut mgr, keys) = build_decay_manager(SEED);
    let start = Instant::now();
    for _ in 0..N_DECAY_ITERS {
        mgr.apply_edge_decay();
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    println!(
        "[decay/{}] {} iters × {} edges = {:.1} ms ({:.2} ms/iter)",
        label,
        N_DECAY_ITERS,
        N_EDGES,
        elapsed,
        elapsed / N_DECAY_ITERS as f64
    );
    snapshot_weights(&mgr, &keys)
}

fn bench_batch(label: &str, strategy: TeStrategy, strategy_label: &str) -> f64 {
    let (data, candidates) = build_causal_inputs(SEED);
    let mut engine = CausalEngine::default();
    engine.cache_enabled = false; // Force every pair to be computed
    let start = Instant::now();
    let mut total_pairs = 0;
    let mut sum_te = 0.0_f64;
    for _ in 0..N_BATCH_RUNS {
        let (links, stats) = engine.batch_compute(&data, &candidates, strategy);
        total_pairs += stats.pairs_computed;
        sum_te += links.iter().map(|l| l.te).sum::<f64>();
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    println!(
        "[batch/{}/{}] {} runs × {} pairs = {:.1} ms ({:.2} ms/run, {} computed, te_sum={:.6})",
        label,
        strategy_label,
        N_BATCH_RUNS,
        N_PAIRS,
        elapsed,
        elapsed / N_BATCH_RUNS as f64,
        total_pairs,
        sum_te
    );
    sum_te
}

fn main() {
    let label = std::env::args().nth(1).unwrap_or_else(|| "run".to_string());
    println!(
        "=== parallel_hotspots_bench ({}, n_threads={}) ===",
        label,
        rayon::current_num_threads()
    );

    // Decay benchmark + determinism snapshot
    let snap_a = bench_decay(&label);
    let snap_b = bench_decay(&format!("{}-rerun", label));
    let determ = snap_a == snap_b;
    println!(
        "[decay/determinism] {} (snapshot edges: {})",
        if determ { "PASS" } else { "FAIL" },
        snap_a.len()
    );
    if !determ {
        let mismatches: usize = snap_a
            .iter()
            .zip(snap_b.iter())
            .filter(|(a, b)| a != b)
            .count();
        eprintln!("[decay/determinism] mismatched edges: {}", mismatches);
        std::process::exit(2);
    }

    // Batch compute benchmark + determinism snapshot
    let te_a_screen = bench_batch(&label, TeStrategy::Screening, "Screening");
    let te_b_screen = bench_batch(
        &format!("{}-rerun", label),
        TeStrategy::Screening,
        "Screening",
    );
    let determ_screen = (te_a_screen - te_b_screen).abs() < 1e-12;
    println!(
        "[batch/Screening/determinism] {} (te_sum drift = {:.3e})",
        if determ_screen { "PASS" } else { "FAIL" },
        (te_a_screen - te_b_screen).abs()
    );

    let te_a_dp = bench_batch(&label, TeStrategy::DeepProbe, "DeepProbe");
    let te_b_dp = bench_batch(
        &format!("{}-rerun", label),
        TeStrategy::DeepProbe,
        "DeepProbe",
    );
    let determ_dp = (te_a_dp - te_b_dp).abs() < 1e-12;
    println!(
        "[batch/DeepProbe/determinism] {} (te_sum drift = {:.3e})",
        if determ_dp { "PASS" } else { "FAIL" },
        (te_a_dp - te_b_dp).abs()
    );

    if !(determ_screen && determ_dp) {
        std::process::exit(3);
    }
    println!("=== done ===");
}
