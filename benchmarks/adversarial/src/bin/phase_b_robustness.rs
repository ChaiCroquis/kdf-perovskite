//! Phase B — multi-seed robustness verification.
//!
//! For each adversarial dataset family, we re-instantiate under
//! multiple *dataset seeds* (i.e., different random instances of the
//! same distribution) and verify that **KDF's relative performance** vs
//! baselines remains stable. This rules out "cherry-picked seed=42"
//! as the source of any claimed advantage.
//!
//! Methodology:
//!   - 5 dataset seeds × 3 trial seeds per dataset = 15 runs per condition
//!   - Report mean ± stderr of rare_recall for KDF / KDF+RelDensity / Random
//!   - Flag any condition where KDF's sign of advantage flips across seeds

use adversarial_bench as adv;
use real_data_bench::Dataset;
use real_data_bench::selectors::{KdfSel, RandomSel, Selector};
use adversarial_bench::solutions::RelativeDensitySelector;
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::Instant;

const DATASET_SEEDS: [u64; 5] = [42, 100, 500, 1000, 7777];
const TRIAL_SEEDS_PER_DS: usize = 3;

#[derive(Serialize, Debug, Clone)]
struct RobustnessRow {
    condition: String,
    method: String,
    dataset_seed: u64,
    recall_mean: f64,
    recall_stderr: f64,
    advantage_over_random: f64,
}

fn run_condition<F: Fn(u64) -> Dataset>(
    name: &str,
    gen: F,
    out: &mut Vec<RobustnessRow>,
) {
    let random = Box::new(RandomSel { p: 0.30 }) as Box<dyn Selector>;
    let kdf = Box::new(KdfSel) as Box<dyn Selector>;
    let reldensity = Box::new(RelativeDensitySelector::default()) as Box<dyn Selector>;

    for &ds_seed in &DATASET_SEEDS {
        let ds = gen(ds_seed);

        let eval = |sel: &Box<dyn Selector>| -> Vec<f64> {
            (0..TRIAL_SEEDS_PER_DS).map(|t| {
                let s = sel.select(&ds, ds_seed + t as u64 * 1000);
                let hit = s.intersection(&ds.rare_ground_truth).count() as f64;
                hit / ds.rare_ground_truth.len().max(1) as f64
            }).collect()
        };

        let r_vals = eval(&random);
        let k_vals = eval(&kdf);
        let rd_vals = eval(&reldensity);

        let mean_sem = |v: &[f64]| -> (f64, f64) {
            let n = v.len() as f64;
            let m = v.iter().sum::<f64>() / n;
            let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n;
            (m, (var / n).sqrt())
        };

        let (rm, rs) = mean_sem(&r_vals);
        let (km, ks) = mean_sem(&k_vals);
        let (rdm, rds) = mean_sem(&rd_vals);

        out.push(RobustnessRow { condition: name.into(), method: "Random".into(),
            dataset_seed: ds_seed, recall_mean: rm, recall_stderr: rs, advantage_over_random: 0.0 });
        out.push(RobustnessRow { condition: name.into(), method: "KDF".into(),
            dataset_seed: ds_seed, recall_mean: km, recall_stderr: ks,
            advantage_over_random: km - rm });
        out.push(RobustnessRow { condition: name.into(), method: "KDF+RelDensity".into(),
            dataset_seed: ds_seed, recall_mean: rdm, recall_stderr: rds,
            advantage_over_random: rdm - rm });
    }
}

fn main() {
    let t0 = Instant::now();
    let mut rows: Vec<RobustnessRow> = Vec::new();

    // A-deg1 : KDF should strongly beat Random (D1-type prediction)
    run_condition("A_deg1", |s| adv::high_degree_rare(500, 1, s), &mut rows);
    // A-deg3 : KDF baseline should FAIL, RelDensity should rescue (D1.5-type)
    run_condition("A_deg3", |s| adv::high_degree_rare(500, 3, s), &mut rows);
    // B-deg2 : isolated cluster, fingerprint-friendly
    run_condition("B_deg2", |s| adv::structurally_isolated(500, 2, s), &mut rows);
    // C : zero redundancy
    run_condition("C", |s| adv::zero_redundancy(500, s), &mut rows);
    // D-10% noise
    run_condition("D_noise10", |s| adv::noisy_edges(500, 0.10, s), &mut rows);

    // Print table
    println!("| Condition | Dataset Seed | Method | Recall mean ± SE | Advantage over Random |");
    println!("|---|---:|---|---:|---:|");
    for r in &rows {
        println!(
            "| {} | {} | {} | {:.3} ± {:.3} | {:+.3} |",
            r.condition, r.dataset_seed, r.method,
            r.recall_mean, r.recall_stderr, r.advantage_over_random
        );
    }

    // Cross-seed stability analysis
    println!("\n## Cross-seed stability (advantage sign consistency)");
    let mut by_cond_method: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();
    for r in &rows {
        if r.method == "Random" { continue; }
        by_cond_method.entry((r.condition.clone(), r.method.clone()))
            .or_default().push(r.advantage_over_random);
    }
    println!("| Condition | Method | #seeds | adv_mean | adv_min | adv_max | sign_stable |");
    println!("|---|---|---:|---:|---:|---:|:---:|");
    for ((cond, method), advs) in &by_cond_method {
        let n = advs.len();
        let m = advs.iter().sum::<f64>() / n as f64;
        let mn = advs.iter().cloned().fold(f64::INFINITY, f64::min);
        let mx = advs.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let sign_stable = (mn > 0.0 && m > 0.0) || (mx < 0.0 && m < 0.0) || m.abs() < 0.01;
        println!("| {} | {} | {} | {:+.3} | {:+.3} | {:+.3} | {} |",
            cond, method, n, m, mn, mx,
            if sign_stable { "✓" } else { "✗ flips" });
    }

    // Save JSON
    std::fs::create_dir_all("demos/verification").ok();
    std::fs::write(
        "demos/verification/robustness.json",
        serde_json::to_string_pretty(&rows).unwrap(),
    ).unwrap();
    println!("\n✅ Written demos/verification/robustness.json ({:?})", t0.elapsed());
}
