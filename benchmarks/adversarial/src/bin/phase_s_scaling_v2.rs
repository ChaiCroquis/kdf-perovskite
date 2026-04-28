//! Phase S — re-measure KDF scaling with FastNodeClassifier (Phase 8 candidate 2).
//!
//! Prior Phase M showed O(n^1.747) using the default NodeClassifier. This
//! bench compares the **fast path** (CSR-based classifier, no fingerprint
//! generation) to see if true O(n log n) is achievable.

use adversarial_bench as adv;
use cgb_kdf::{FastNodeClassifier, Layer, NodeClassifier};
use real_data_bench::Dataset;
use std::collections::HashSet;
use std::time::Instant;

fn kdf_select_default(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let mut c = NodeClassifier::default();
    let class = c.classify(ds.n_nodes, &ds.edges);
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn kdf_select_fast(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let c = FastNodeClassifier::default();
    let class = c.classify(ds.n_nodes, &ds.edges);
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn main() {
    let sizes = [10_000_usize, 50_000, 100_000, 200_000, 500_000, 1_000_000];
    println!("| n | default select_ms | fast select_ms | speedup |");
    println!("|---:|---:|---:|---:|");

    let mut default_pts: Vec<(f64, f64)> = Vec::new();
    let mut fast_pts: Vec<(f64, f64)> = Vec::new();

    for &n in &sizes {
        let ds = adv::high_degree_rare(n, 1, 42);
        let keep = (n as f64 * 0.30) as usize;

        // Default classifier
        let def_ms = if n <= 500_000 {
            let t0 = Instant::now();
            let _ = kdf_select_default(&ds, keep);
            t0.elapsed().as_secs_f64() * 1000.0
        } else {
            f64::NAN // skip at 1M (would be ~40s)
        };

        // Fast classifier
        let t0 = Instant::now();
        let _ = kdf_select_fast(&ds, keep);
        let fast_ms = t0.elapsed().as_secs_f64() * 1000.0;

        let speedup = if def_ms.is_nan() {
            f64::NAN
        } else {
            def_ms / fast_ms
        };
        println!(
            "| {} | {:.1} | {:.1} | {:.1}x |",
            n,
            if def_ms.is_nan() {
                f64::INFINITY
            } else {
                def_ms
            },
            fast_ms,
            speedup
        );

        if !def_ms.is_nan() {
            default_pts.push(((n as f64).ln(), def_ms.ln()));
        }
        fast_pts.push(((n as f64).ln(), fast_ms.ln()));
    }

    let fit = |pts: &[(f64, f64)]| -> f64 {
        let m_x: f64 = pts.iter().map(|p| p.0).sum::<f64>() / pts.len() as f64;
        let m_y: f64 = pts.iter().map(|p| p.1).sum::<f64>() / pts.len() as f64;
        let cov: f64 = pts.iter().map(|p| (p.0 - m_x) * (p.1 - m_y)).sum::<f64>();
        let var_x: f64 = pts.iter().map(|p| (p.0 - m_x).powi(2)).sum::<f64>();
        cov / var_x
    };

    println!("\n## Log-log regression (empirical exponent)\n");
    println!("- Default classifier: O(n^{:.3})", fit(&default_pts));
    println!("- FastClassifier:     O(n^{:.3})", fit(&fast_pts));
}
