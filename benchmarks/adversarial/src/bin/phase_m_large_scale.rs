//! Phase M — Large-scale scaling verification.
//!
//! Phase 7 scaling tested up to n=50k. This extends to n=500k to
//! re-estimate the empirical complexity exponent with more decades.

use adversarial_bench as adv;
use real_data_bench::selectors::{KdfSel, RandomSel, KMedoidsSel, Selector};
use std::time::Instant;

fn main() {
    let sizes = [10_000_usize, 50_000, 100_000, 200_000, 500_000];
    let seed = 42;
    let selectors: Vec<(String, Box<dyn Selector>)> = vec![
        ("Random".into(), Box::new(RandomSel { p: 0.30 })),
        ("KMedoids".into(), Box::new(KMedoidsSel { frac: 0.30 })),
        ("KDF".into(), Box::new(KdfSel)),
    ];

    println!("| n | method | build_ms | select_ms | total_ms | ns/(n·log2 n) |");
    println!("|---:|---|---:|---:|---:|---:|");
    let mut kdf_ns_per_nlogn: Vec<(usize, f64)> = Vec::new();

    for &n in &sizes {
        let build_start = Instant::now();
        let ds = adv::high_degree_rare(n, 1, seed);
        let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;

        for (name, sel) in &selectors {
            let start = Instant::now();
            let _selected = sel.select(&ds, seed);
            let sel_ms = start.elapsed().as_secs_f64() * 1000.0;
            let n_f = n as f64;
            let ns_per_nlogn = (sel_ms * 1e6) / (n_f * n_f.log2());
            println!(
                "| {} | {} | {:.1} | {:.1} | {:.1} | {:.2} |",
                n, name, build_ms, sel_ms, build_ms + sel_ms, ns_per_nlogn
            );
            if name == "KDF" {
                kdf_ns_per_nlogn.push((n, ns_per_nlogn));
            }
        }
    }

    // Regression: fit log-log of (n, select_ms) for KDF
    // select_ms = A * n^k  →  log(select_ms) = log(A) + k log(n)
    println!("\n## KDF empirical exponent estimation (log-log regression)\n");

    // We didn't store select_ms; recompute roughly from ns/(n log n)
    // select_ms ≈ ns_per_nlogn * n * log2(n) / 1e6
    let points: Vec<(f64, f64)> = kdf_ns_per_nlogn.iter().map(|&(n, r)| {
        let n_f = n as f64;
        let select_ms = r * n_f * n_f.log2() / 1e6;
        (n_f.ln(), select_ms.ln())
    }).collect();

    let mean_x: f64 = points.iter().map(|p| p.0).sum::<f64>() / points.len() as f64;
    let mean_y: f64 = points.iter().map(|p| p.1).sum::<f64>() / points.len() as f64;
    let cov: f64 = points.iter().map(|p| (p.0 - mean_x) * (p.1 - mean_y)).sum::<f64>();
    let var_x: f64 = points.iter().map(|p| (p.0 - mean_x).powi(2)).sum::<f64>();
    let slope = cov / var_x;

    println!("Points (ln n, ln select_ms):");
    for (n, rate) in &kdf_ns_per_nlogn {
        let n_f = *n as f64;
        let select_ms = rate * n_f * n_f.log2() / 1e6;
        println!("  n={:>7}  → select_ms={:.2}  (ln n={:.2}, ln ms={:.2})",
            n, select_ms, n_f.ln(), select_ms.ln());
    }
    println!("\nEstimated KDF empirical exponent: **O(n^{:.3})**", slope);
    println!("\n- If ≈ 1.0, KDF is truly O(n)");
    println!("- If ≈ 1.2, consistent with Phase 7 finding");
    println!("- If > 1.5, complexity claim collapses at scale");
}
