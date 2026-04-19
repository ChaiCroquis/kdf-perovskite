//! Phase S v3 — push toward TRUE O(n log n) for KDF selection.
//!
//! v2 used FastClassifier (CSR-based) but still sort-based top-K → O(n log n)
//! per sort + lots of HashMap. v3 replaces:
//!   - HashMap<u32, Layer> → Vec<Layer> (direct indexing)
//!   - sort_by top-K → bucket-based O(n) selection
//! and measures if exponent approaches 1.0 or log n.

use adversarial_bench as adv;
use cgb_kdf::{FastNodeClassifier, Layer};
use real_data_bench::Dataset;
use std::collections::HashSet;
use std::time::Instant;

fn kdf_select_v3_linear(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let n = ds.n_nodes;
    let c = FastNodeClassifier::default();
    let class = c.classify(n, &ds.edges);

    // Dense Vec<Layer> instead of HashMap
    let mut layers_vec = vec![Layer::Edge; n];
    for (&id, &l) in &class.layers {
        if (id as usize) < n { layers_vec[id as usize] = l; }
    }

    // Bucket each node into its layer bucket (O(n))
    let mut rare = Vec::with_capacity(class.stats.rare_count);
    let mut core = Vec::with_capacity(class.stats.core_count);
    let mut edge = Vec::with_capacity(class.stats.edge_count);
    for i in 0..n {
        match layers_vec[i] {
            Layer::Rare => rare.push(i as u32),
            Layer::Core => core.push(i as u32),
            Layer::Edge => edge.push(i as u32),
            Layer::Garbage => {}
        }
    }

    // Priority fill: Rare → Core → Edge
    let mut out: HashSet<u32> = HashSet::with_capacity(keep);
    let take = |src: &[u32], out: &mut HashSet<u32>, keep: usize| {
        for &id in src {
            if out.len() >= keep { break; }
            out.insert(id);
        }
    };
    take(&rare, &mut out, keep);
    take(&core, &mut out, keep);
    take(&edge, &mut out, keep);
    out
}

/// Build a sparse graph with O(n) edges (not O(n²)) — the default
/// adversarial synthesizer creates n_hubs²/2 hub-hub edges which dominates
/// at large n. For scaling tests we need true sparse graphs.
fn build_sparse_graph(n: usize, seed: u64) -> Dataset {
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut edges = Vec::with_capacity(n * 5);
    let mut rare = HashSet::new();
    let n_hubs = (n / 1000).max(10); // Sparse hubs
    let n_rare = n / 20;

    // Hub degrees (each hub gets ~20 connections to neighbors, not all-to-all)
    for u in 0..n_hubs as u32 {
        for _ in 0..20 {
            let v = rng.gen_range(0..n_hubs) as u32;
            if u != v { edges.push((u, v, 1.0)); }
        }
    }
    for i in 0..n_rare {
        let id = (n_hubs + i) as u32;
        let h = rng.gen_range(0..n_hubs) as u32;
        edges.push((id, h, 1.0));
        rare.insert(id);
    }
    for i in (n_hubs + n_rare)..n {
        let id = i as u32;
        for _ in 0..3 {
            let h = rng.gen_range(0..n_hubs) as u32;
            edges.push((id, h, 1.0));
        }
    }
    Dataset { name: "sparse".into(), n_nodes: n, edges, rare_ground_truth: rare, description: "sparse".into() }
}

fn main() {
    let sizes = [10_000_usize, 50_000, 100_000, 200_000, 500_000, 1_000_000, 2_000_000];
    println!("| n | classifier ms | bucket ms | total ms | ns/n |");
    println!("|---:|---:|---:|---:|---:|");

    let mut pts: Vec<(f64, f64)> = Vec::new();
    for &n in &sizes {
        let ds = build_sparse_graph(n, 42);
        let keep = (n as f64 * 0.30) as usize;

        // Time classifier alone
        let c = FastNodeClassifier::default();
        let t0 = Instant::now();
        let _ = c.classify(n, &ds.edges);
        let class_ms = t0.elapsed().as_secs_f64() * 1000.0;

        // Time full selection
        let t0 = Instant::now();
        let _ = kdf_select_v3_linear(&ds, keep);
        let total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let bucket_ms = total_ms - class_ms;
        let ns_per_node = total_ms * 1e6 / n as f64;

        println!("| {} | {:.1} | {:.1} | {:.1} | {:.1} |",
            n, class_ms, bucket_ms, total_ms, ns_per_node);
        pts.push(((n as f64).ln(), total_ms.ln()));
    }

    let m_x: f64 = pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64;
    let m_y: f64 = pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64;
    let cov: f64 = pts.iter().map(|p| (p.0 - m_x) * (p.1 - m_y)).sum::<f64>();
    let var_x: f64 = pts.iter().map(|p| (p.0 - m_x).powi(2)).sum::<f64>();
    let slope = cov / var_x;

    println!("\n## Log-log regression\n");
    println!("v3 empirical exponent: **O(n^{:.3})**", slope);
    if slope < 1.1 {
        println!("→ Achieves TRUE O(n) scaling (ns/n approximately constant)");
    } else if slope < 1.3 {
        println!("→ Close to O(n log n) (log n grows slowly)");
    } else {
        println!("→ Still super-linear, further work needed");
    }
}
