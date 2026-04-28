//! Phase 7 comparative benchmark.
//!
//! Runs baseline KDF + 4 Phase 7 solution candidates on the same adversarial
//! suite as Phase 6, and reports improvements/regressions per failure mode.

use adversarial_bench::{
    self as adv,
    solutions::{FingerprintIsolationSelector, PersistentRareMemory, RelativeDensitySelector},
};
use real_data_bench::{
    Dataset, TrialResult, metrics,
    selectors::{KdfSel, RandomSel, Selector, StratifiedSel},
    wilcoxon::wilcoxon_signed_rank,
};
use std::collections::BTreeMap;
use std::time::Instant;

const N_TRIALS: usize = 10;
const N_NODES: usize = 500;

fn all_methods() -> Vec<Box<dyn Selector>> {
    vec![
        Box::new(RandomSel { p: 0.30 }),
        Box::new(StratifiedSel { p_non_rare: 0.30 }),
        Box::new(KdfSel),                                      // baseline
        Box::new(PersistentRareMemory::new(Box::new(KdfSel))), // S1
        Box::new(RelativeDensitySelector::default()),          // S2
        Box::new(FingerprintIsolationSelector::default()),     // S3
        Box::new(adv::solutions::s4_hybrid()),                 // S4
    ]
}

fn run(ds: &Dataset, selectors: &[Box<dyn Selector>], seed: u64, trial: usize) -> Vec<TrialResult> {
    selectors
        .iter()
        .map(|s| {
            let start = Instant::now();
            let sel = s.select(ds, seed);
            let ms = start.elapsed().as_secs_f64() * 1000.0;
            metrics::evaluate(&ds.name, s.name(), seed, trial, ds, &sel, ms)
        })
        .collect()
}

fn main() {
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 3000 + i).collect();

    let mut all: Vec<TrialResult> = Vec::new();
    let selectors = all_methods();

    // Focused attack: we mostly care about the failure modes (A-deg3, E-temporal).
    // Keep 1 clean case (A-deg1) as sanity check.
    println!("## Phase 7 — testing solutions against known failure modes\n");

    for (trial, &seed) in seeds.iter().enumerate() {
        // (A) Clean — baseline sanity
        let ds = adv::high_degree_rare(N_NODES, 1, seed);
        all.extend(run(&ds, &selectors, seed, trial));

        // (A) Failure — KDF missed deg=3
        let ds = adv::high_degree_rare(N_NODES, 3, seed);
        all.extend(run(&ds, &selectors, seed, trial));

        // (A) Failure — KDF missed deg=5
        let ds = adv::high_degree_rare(N_NODES, 5, seed);
        all.extend(run(&ds, &selectors, seed, trial));

        // (B) Structural isolation
        let ds = adv::structurally_isolated(N_NODES, 3, seed);
        all.extend(run(&ds, &selectors, seed, trial));

        // (E) Failure — temporal, all snapshots
        let snaps = adv::temporal_snapshots(N_NODES, 5, seed);
        for ds in &snaps {
            all.extend(run(ds, &selectors, seed, trial));
        }
    }

    print_aggregated(&all);
    run_wilcoxon_vs_baseline(&all);

    std::fs::create_dir_all("benchmarks/results").ok();
    let p = "benchmarks/results/phase7_solutions.json";
    std::fs::write(p, serde_json::to_string_pretty(&all).unwrap()).unwrap();
    println!("\nWritten: {}", p);
}

fn print_aggregated(all: &[TrialResult]) {
    let mut by: BTreeMap<(String, String), Vec<&TrialResult>> = BTreeMap::new();
    for r in all {
        by.entry((r.dataset.clone(), r.method.clone()))
            .or_default()
            .push(r);
    }

    println!("| Dataset | Method | Recall | Precision | F1 | Comp | ms |");
    println!("|---|---|---:|---:|---:|---:|---:|");
    for ((ds, m), rs) in &by {
        let n = rs.len() as f64;
        let mean = |f: fn(&TrialResult) -> f64| rs.iter().map(|r| f(r)).sum::<f64>() / n;
        println!(
            "| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.2} |",
            ds,
            m,
            mean(|r| r.rare_recall),
            mean(|r| r.precision_at_rare),
            mean(|r| r.f1_at_rare),
            mean(|r| r.compression_rate),
            mean(|r| r.elapsed_ms)
        );
    }
}

fn run_wilcoxon_vs_baseline(all: &[TrialResult]) {
    println!("\n## Wilcoxon signed-rank: Solutions vs baseline KDF (Rare Recall)");
    println!("| Dataset | Method | n | median diff | p | sig@0.01 | direction |");
    println!("|---|---|---:|---:|---:|:---:|---|");

    let datasets: std::collections::BTreeSet<String> =
        all.iter().map(|r| r.dataset.clone()).collect();
    let methods = ["KDF+PersistMem", "KDF+RelDensity", "KDF+Fingerprint"];

    for ds in &datasets {
        let kdf: Vec<f64> = all
            .iter()
            .filter(|r| r.dataset == *ds && r.method == "KDF")
            .map(|r| r.rare_recall)
            .collect();
        for m in &methods {
            let sol: Vec<f64> = all
                .iter()
                .filter(|r| r.dataset == *ds && r.method == *m)
                .map(|r| r.rare_recall)
                .collect();
            if kdf.len() != sol.len() || kdf.is_empty() {
                continue;
            }
            if let Some(w) = wilcoxon_signed_rank(&sol, &kdf) {
                let dir = if w.median_diff > 0.0 {
                    "improves"
                } else if w.median_diff < 0.0 {
                    "regresses"
                } else {
                    "equal"
                };
                println!(
                    "| {} | {} | {} | {:+.3} | {:.3} | {} | {} |",
                    ds,
                    m,
                    w.n_effective,
                    w.median_diff,
                    w.p_value_two_sided,
                    if w.significant_at_01 { "YES" } else { "no" },
                    dir
                );
            }
        }
    }
}
