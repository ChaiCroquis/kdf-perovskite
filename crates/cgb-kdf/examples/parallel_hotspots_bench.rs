//! Parallel hotspots benchmark.
//!
//! Measures wall-clock time of three embarrassingly-parallel hotspots in cgb-kdf:
//!   - `DecayManager::apply_edge_decay` over n=10^5 edges (Step 3)
//!   - `CausalEngine::batch_compute` over n_series=200, n_pairs≈5000 (Step 2)
//!   - `KdfProcessorRev12::process_review_cycle` over n_rare=300, n_cand=600 (Step 4)
//!
//! The same workload is run twice with the same seed and the resulting state is
//! compared bit-exactly to verify Claim 15 determinism is preserved when rayon
//! is enabled.
//!
//! Run: cargo run --release -p cgb-kdf --example parallel_hotspots_bench

use cgb_kdf::causal::{CausalEngine, TeStrategy};
use cgb_kdf::framework::{ClassificationStats, DecayManager, Layer, NodeClassification};
use cgb_kdf::framework::{KdfProcessorRev12, Rev12Stats};
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

// Rev12 review cycle bench parameters
// N_REV12_TOTAL nodes split as ~50% RARE / ~50% Edge so candidates are non-empty
// and analogy_engine.compute_analogy does meaningful work per RARE node.
const N_REV12_TOTAL: usize = 600;
const N_REV12_RARE: usize = 300;
const N_REV12_CYCLES: usize = 5;

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

/// Build a spoke-graph: RARE leaves (each with exactly 1 neighbor) attached
/// to a clique of CORE hubs. The classifier sees:
///   - nodes [0, N_REV12_RARE)         → RARE (deg=1)
///   - nodes [N_REV12_RARE, N_REV12_TOTAL) → Core/Edge (deg>=2 via inter-hub edges)
fn build_rev12_processor(seed: u64) -> KdfProcessorRev12 {
    let mut rng = SmallRng::seed_from_u64(seed);
    let n_total = N_REV12_TOTAL;
    let n_rare = N_REV12_RARE;
    let n_hubs = n_total - n_rare;

    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    // 1) RARE leaf → hub (each leaf gets exactly 1 edge)
    for leaf in 0..n_rare as u32 {
        let hub = (n_rare as u32) + (leaf % n_hubs as u32);
        edges.push((leaf, hub, 1.0));
    }
    // 2) Hub-hub edges so each hub has degree >= 2 (Core/Edge eligible).
    // Add ~3 × n_hubs random hub-pair edges with deduplication.
    let mut seen = std::collections::HashSet::new();
    let target_hub_edges = n_hubs * 3;
    while edges.len() - n_rare < target_hub_edges {
        let u = (n_rare as u32) + rng.gen_range(0..n_hubs as u32);
        let v = (n_rare as u32) + rng.gen_range(0..n_hubs as u32);
        if u == v {
            continue;
        }
        let key = if u < v { (u, v) } else { (v, u) };
        if !seen.insert(key) {
            continue;
        }
        edges.push((key.0, key.1, 1.0));
    }

    let mut processor = KdfProcessorRev12::default();
    processor.initialize(n_total, &edges);
    processor
}

fn bench_rev12(label: &str) -> (Rev12Stats, Vec<(u32, u64)>) {
    let mut processor = build_rev12_processor(SEED);
    let start = Instant::now();
    let mut total_actions = 0;
    for _ in 0..N_REV12_CYCLES {
        let actions = processor.process_review_cycle();
        total_actions += actions.len();
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let stats = processor.rev12_stats().clone();

    // Capture per-RARE wait_count snapshot in deterministic order for bit-exact compare.
    let mut wait_snapshot: Vec<(u32, u64)> = (0..N_REV12_RARE as u32)
        .filter_map(|n| processor.get_rare_state(n).map(|s| (n, s.wait_count)))
        .collect();
    wait_snapshot.sort_by_key(|(n, _)| *n);

    println!(
        "[rev12/{}] {} cycles × {} RARE × {} candidates = {:.1} ms ({:.2} ms/cycle, actions={}, attempts={}, spoke_up={})",
        label,
        N_REV12_CYCLES,
        N_REV12_RARE,
        N_REV12_TOTAL - N_REV12_RARE,
        elapsed,
        elapsed / N_REV12_CYCLES as f64,
        total_actions,
        stats.discovery_attempts,
        stats.spoke_up_count
    );

    (stats, wait_snapshot)
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

    // rev12 review cycle benchmark + determinism snapshot (Step 4)
    let (stats_a, snap_a_rev12) = bench_rev12(&label);
    let (stats_b, snap_b_rev12) = bench_rev12(&format!("{}-rerun", label));
    let determ_rev12_stats = stats_a.discovery_attempts == stats_b.discovery_attempts
        && stats_a.spoke_up_count == stats_b.spoke_up_count
        && stats_a.demoted_count == stats_b.demoted_count
        && stats_a.successful_discoveries == stats_b.successful_discoveries
        && stats_a.promoted_count == stats_b.promoted_count;
    let determ_rev12_state = snap_a_rev12 == snap_b_rev12;
    let determ_rev12 = determ_rev12_stats && determ_rev12_state;
    println!(
        "[rev12/determinism] {} (stats match: {}, state match: {}, snapshot rare: {})",
        if determ_rev12 { "PASS" } else { "FAIL" },
        determ_rev12_stats,
        determ_rev12_state,
        snap_a_rev12.len()
    );
    if !determ_rev12 {
        std::process::exit(4);
    }

    println!("=== done ===");
}
