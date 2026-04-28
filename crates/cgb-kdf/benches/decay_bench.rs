//! DecayManager micro-benchmarks.
//!
//! Used to measure the impact of internal-representation refactors
//! (HashMap → sorted-Vec / dense-Vec). Run with:
//!   cargo bench --bench decay_bench -p cgb-kdf
//!
//! Scenario: n = 100,000 nodes, |E| = 1,000,000 directed-as-undirected edges,
//! seeded RNG for reproducibility.

use cgb_kdf::framework::{ClassificationStats, DecayManager, Layer, NodeClassification};
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::collections::{HashMap, HashSet};

const N_NODES: u32 = 100_000;
const M_EDGES: u32 = 1_000_000;
const SEED: u64 = 0xDECA_F100_DEAD_BEEFu64;

/// Reproducible random graph: spanning path + random edges, deduplicated.
fn make_graph(n: u32, m: u32, seed: u64) -> Vec<(u32, u32, f64)> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut edges: Vec<(u32, u32, f64)> = Vec::with_capacity(m as usize);
    let mut seen: HashSet<(u32, u32)> = HashSet::with_capacity(m as usize);
    // spanning path so every node has at least 1 edge
    for i in 0..n.saturating_sub(1) {
        let key = (i, i + 1);
        if seen.insert(key) {
            edges.push((i, i + 1, rng.r#gen::<f64>().max(1e-3)));
        }
    }
    while (edges.len() as u32) < m {
        let u = rng.gen_range(0..n);
        let v = rng.gen_range(0..n);
        if u == v {
            continue;
        }
        let (a, b) = if u < v { (u, v) } else { (v, u) };
        if seen.insert((a, b)) {
            edges.push((a, b, rng.r#gen::<f64>().max(1e-3)));
        }
    }
    edges
}

fn make_classification(n: u32) -> NodeClassification {
    let mut layers = HashMap::with_capacity(n as usize);
    for i in 0..n {
        layers.insert(i, Layer::Edge);
    }
    NodeClassification {
        layers,
        rare_fingerprints: HashMap::new(),
        stats: ClassificationStats::default(),
    }
}

fn bench_apply_edge_decay(c: &mut Criterion) {
    let edges = make_graph(N_NODES, M_EDGES, SEED);
    let class = make_classification(N_NODES);
    eprintln!(
        "[bench] make_graph done: n={}, |E|={}",
        N_NODES,
        edges.len()
    );

    let mut group = c.benchmark_group("decay_apply");
    group.sample_size(10);
    group.bench_function("apply_edge_decay_1M", |b| {
        b.iter_batched(
            || {
                let mut mgr = DecayManager::master_spec();
                mgr.initialize_with_edges(class.clone(), &edges);
                mgr
            },
            |mut mgr| {
                mgr.apply_edge_decay();
                black_box(mgr);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_record_edge_access(c: &mut Criterion) {
    let edges = make_graph(N_NODES, M_EDGES, SEED);
    let class = make_classification(N_NODES);

    let mut rng = SmallRng::seed_from_u64(SEED.wrapping_add(1));
    let access_pattern: Vec<(u32, u32)> = (0..100_000)
        .map(|_| {
            let (u, v, _) = edges[rng.gen_range(0..edges.len())];
            (u, v)
        })
        .collect();

    let mut group = c.benchmark_group("decay_access");
    group.sample_size(10);
    group.bench_function("record_edge_access_100K", |b| {
        b.iter_batched(
            || {
                let mut mgr = DecayManager::master_spec();
                mgr.initialize_with_edges(class.clone(), &edges);
                mgr
            },
            |mut mgr| {
                for &(u, v) in &access_pattern {
                    mgr.record_edge_access(black_box(u), black_box(v));
                }
                black_box(mgr);
            },
            BatchSize::LargeInput,
        );
    });
    group.finish();
}

fn bench_compute_evaluation(c: &mut Criterion) {
    let edges = make_graph(N_NODES, M_EDGES, SEED);
    let class = make_classification(N_NODES);
    let mut mgr = DecayManager::master_spec();
    mgr.initialize_with_edges(class, &edges);
    // touch some edges so time-component path is exercised
    for &(u, v, _) in edges.iter().take(10_000) {
        mgr.record_edge_access(u, v);
    }

    let mut rng = SmallRng::seed_from_u64(SEED.wrapping_add(2));
    let lookup_pattern: Vec<(u32, u32)> = (0..100_000)
        .map(|_| {
            let (u, v, _) = edges[rng.gen_range(0..edges.len())];
            (u, v)
        })
        .collect();

    let mut group = c.benchmark_group("decay_eval");
    group.sample_size(10);
    group.bench_function("compute_evaluation_value_100K", |b| {
        b.iter(|| {
            let mut sum = 0.0;
            for &(u, v) in &lookup_pattern {
                sum += mgr.compute_evaluation_value(black_box(u), black_box(v), Layer::Edge);
            }
            black_box(sum);
        });
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_apply_edge_decay,
    bench_record_edge_access,
    bench_compute_evaluation
);
criterion_main!(benches);
