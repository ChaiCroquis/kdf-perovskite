//! Phase 6 adversarial benchmark runner.
//!
//! Runs all 6 selectors against all 6 adversarial generators.
//! Writes `benchmarks/results/adversarial.json`.

use adversarial_bench::*;
use real_data_bench::wilcoxon::wilcoxon_signed_rank;
use real_data_bench::{metrics, selectors::all_selectors, Dataset, TrialResult};
use std::collections::BTreeMap;
use std::time::Instant;

const N_TRIALS: usize = 10;
const N_NODES: usize = 500;

fn run_one(
    ds: &Dataset,
    selectors: &[Box<dyn real_data_bench::selectors::Selector>],
    seeds: &[u64],
) -> Vec<TrialResult> {
    let mut out = Vec::new();
    for (trial, &seed) in seeds.iter().enumerate() {
        for sel in selectors {
            let start = Instant::now();
            let sel_result = sel.select(ds, seed);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0;
            out.push(metrics::evaluate(
                &ds.name,
                sel.name(),
                seed,
                trial,
                ds,
                &sel_result,
                elapsed,
            ));
        }
    }
    out
}

fn main() {
    let selectors = all_selectors(0.30);
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 1000 + i).collect();

    let mut all: Vec<TrialResult> = Vec::new();

    // (A) High-degree rare (degrees 1, 3, 10)
    for deg in [1, 3, 10] {
        for &seed in &seeds {
            let ds = high_degree_rare(N_NODES, deg, seed);
            all.extend(run_one(&ds, &selectors, &[seed]));
        }
    }

    // (B) Structurally isolated
    for deg in [2, 5] {
        for &seed in &seeds {
            let ds = structurally_isolated(N_NODES, deg, seed);
            all.extend(run_one(&ds, &selectors, &[seed]));
        }
    }

    // (C) Zero redundancy
    for &seed in &seeds {
        let ds = zero_redundancy(N_NODES, seed);
        all.extend(run_one(&ds, &selectors, &[seed]));
    }

    // (D) Noisy edges
    for rate in [0.10, 0.30] {
        for &seed in &seeds {
            let ds = noisy_edges(N_NODES, rate, seed);
            all.extend(run_one(&ds, &selectors, &[seed]));
        }
    }

    // (E) Temporal (pick 3 snapshots)
    for &seed in &seeds {
        let snaps = temporal_snapshots(N_NODES, 5, seed);
        for ds in snaps {
            all.extend(run_one(&ds, &selectors, &[seed]));
        }
    }

    // (F) Large scale — only if N_NODES small enough; here a bigger single run
    for &seed in &seeds {
        let ds = large_scale(2_000, seed);
        all.extend(run_one(&ds, &selectors, &[seed]));
    }

    // Aggregate & report
    print_table(&all);
    run_wilcoxon_vs_random(&all);

    std::fs::create_dir_all("benchmarks/results").ok();
    let out_path = "benchmarks/results/adversarial.json";
    std::fs::write(out_path, serde_json::to_string_pretty(&all).unwrap()).expect("write results");
    println!("\nResults written to {}", out_path);
}

fn print_table(all: &[TrialResult]) {
    let mut by_key: BTreeMap<(String, String), Vec<&TrialResult>> = BTreeMap::new();
    for r in all {
        by_key
            .entry((r.dataset.clone(), r.method.clone()))
            .or_default()
            .push(r);
    }

    println!("| Dataset | Method | Rare Recall | Precision@Rare | F1 | Compression | Time (ms) | trials |");
    println!("|---|---|---:|---:|---:|---:|---:|---:|");
    for ((ds, method), rs) in &by_key {
        let n = rs.len() as f64;
        let mean = |f: fn(&TrialResult) -> f64| rs.iter().map(|r| f(r)).sum::<f64>() / n;
        let se = |f: fn(&TrialResult) -> f64| {
            let m = mean(f);
            let v: f64 = rs.iter().map(|r| (f(r) - m).powi(2)).sum::<f64>() / n;
            (v / n).sqrt()
        };
        println!(
            "| {} | {} | {:.3} ± {:.3} | {:.3} | {:.3} | {:.3} | {:.2} | {} |",
            ds,
            method,
            mean(|r| r.rare_recall),
            se(|r| r.rare_recall),
            mean(|r| r.precision_at_rare),
            mean(|r| r.f1_at_rare),
            mean(|r| r.compression_rate),
            mean(|r| r.elapsed_ms),
            rs.len()
        );
    }
}

fn run_wilcoxon_vs_random(all: &[TrialResult]) {
    // For each dataset, test KDF Rare Recall > Random Rare Recall
    println!("\n## Wilcoxon signed-rank: KDF vs Random (Rare Recall)");
    println!("| Dataset | n | median diff | z | p | sig@0.01 |");
    println!("|---|---:|---:|---:|---:|:---:|");

    let mut by_ds: BTreeMap<String, (Vec<f64>, Vec<f64>)> = BTreeMap::new();
    for r in all {
        let entry = by_ds.entry(r.dataset.clone()).or_default();
        if r.method == "KDF" {
            entry.0.push(r.rare_recall);
        } else if r.method == "Random" {
            entry.1.push(r.rare_recall);
        }
    }
    for (ds, (kdf, rand)) in &by_ds {
        if kdf.len() != rand.len() || kdf.is_empty() {
            continue;
        }
        let n = kdf.len().min(rand.len());
        let kdf_s = &kdf[..n];
        let rand_s = &rand[..n];
        if let Some(w) = wilcoxon_signed_rank(kdf_s, rand_s) {
            println!(
                "| {} | {} | {:+.3} | {:.2} | {:.3} | {} |",
                ds,
                w.n_effective,
                w.median_diff,
                w.z,
                w.p_value_two_sided,
                if w.significant_at_01 { "YES" } else { "no" }
            );
        } else {
            println!("| {} | — | — | — | — | (no diff) |", ds);
        }
    }
}
