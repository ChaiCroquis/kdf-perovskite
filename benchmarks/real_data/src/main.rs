//! Phase 6 real-data benchmark runner (Obsidian + public datasets).

use real_data_bench::{
    metrics, obsidian, public_datasets, selectors::all_selectors, wilcoxon::wilcoxon_signed_rank,
    Dataset, TrialResult,
};
use std::collections::{BTreeMap, HashSet};
use std::path::PathBuf;
use std::time::Instant;

const N_TRIALS: usize = 10;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scale = args.get(1).map(|s| s.as_str()).unwrap_or("small");

    let datasets: Vec<Dataset> = collect_datasets(scale);

    if datasets.is_empty() {
        eprintln!("No datasets available. Check:");
        eprintln!("{}", public_datasets::download_instructions());
        eprintln!("Or set OBSIDIAN_VAULT to a directory of *.md files.");
        std::process::exit(1);
    }

    let selectors = all_selectors(0.30);
    let mut all: Vec<TrialResult> = Vec::new();

    for ds in &datasets {
        println!(
            "## Dataset: {} — n={}, edges={}, rare={}",
            ds.name,
            ds.n_nodes,
            ds.n_edges(),
            ds.n_rare()
        );
        for trial in 0..N_TRIALS {
            let seed = 2000 + trial as u64;
            for sel in &selectors {
                let start = Instant::now();
                let selected = sel.select(ds, seed);
                let elapsed = start.elapsed().as_secs_f64() * 1000.0;
                all.push(metrics::evaluate(
                    &ds.name,
                    sel.name(),
                    seed,
                    trial,
                    ds,
                    &selected,
                    elapsed,
                ));
            }
        }
    }

    print_table(&all);
    run_wilcoxon_vs_random(&all);

    std::fs::create_dir_all("benchmarks/results").ok();
    let out_path = "benchmarks/results/real_data.json";
    std::fs::write(out_path, serde_json::to_string_pretty(&all).unwrap()).expect("write results");
    println!("\nResults written to {}", out_path);
}

fn collect_datasets(_scale: &str) -> Vec<Dataset> {
    let mut out = Vec::new();
    // Obsidian Vault content is read in full regardless of scale because the
    // interesting link density is concentrated in a minority of notes.
    // `_scale` currently only serves as a CLI hook for future per-dataset
    // size overrides (e.g. subsample FB15K-237 for quick runs).
    let max_notes: Option<usize> = None;

    // Obsidian Vault (highest priority: always available locally)
    let vault = std::env::var("OBSIDIAN_VAULT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(r"C:\Users\user\Documents\Obsidian Vault"));
    if vault.exists() {
        let cfg = obsidian::ObsidianBuildConfig {
            vault_root: vault,
            max_notes,
            rare_indegree_max: 2,
        };
        match obsidian::build(&cfg) {
            Ok(ds) => {
                println!(
                    "Loaded {} ({} nodes, {} edges, {} rare)",
                    ds.name,
                    ds.n_nodes,
                    ds.n_edges(),
                    ds.n_rare()
                );
                out.push(ds);
            }
            Err(e) => eprintln!("Obsidian build failed: {}", e),
        }
    }

    if let Some(ds) = public_datasets::load_fb15k_237(5) {
        out.push(ds);
    } else {
        eprintln!("FB15K-237 not found (data not redistributed). See download_instructions().");
    }
    if let Some(ds) = public_datasets::load_ogbn_arxiv(2) {
        out.push(ds);
    }
    let rare_codes: HashSet<u16> = [400, 404, 500, 503].into_iter().collect();
    if let Some(ds) = public_datasets::load_nasa_log(&rare_codes) {
        out.push(ds);
    }

    out
}

fn print_table(all: &[TrialResult]) {
    let mut by_key: BTreeMap<(String, String), Vec<&TrialResult>> = BTreeMap::new();
    for r in all {
        by_key
            .entry((r.dataset.clone(), r.method.clone()))
            .or_default()
            .push(r);
    }
    println!("\n## Aggregated results");
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
        if kdf.is_empty() || rand.is_empty() || kdf.len() != rand.len() {
            continue;
        }
        if let Some(w) = wilcoxon_signed_rank(kdf, rand) {
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
