//! Phase C — Ablation study.
//!
//! Turn off each KDF component in turn and measure the drop in headline
//! metrics. Confirms which components are *actually necessary* and which
//! are redundant / might be simplifiable.
//!
//! Ablations:
//!   A0: Full KDF (baseline)
//!   A1: Remove Rare-first priority (treat Rare like Edge)
//!   A2: Remove Core priority (treat Core like Edge)
//!   A3: Remove Garbage filtering (include deg=0 nodes)
//!   A4: Remove cluster-representative dedup (keep all Edge)
//!   A5: KDF+RelDensity (add relative-density on top of full)
//!
//! Dataset: Adv A deg=1 (D1-type: KDF should win)
//! and Adv A deg=3 (D1.5-type: baseline fails)

use adversarial_bench::{high_degree_rare, structurally_isolated};
use cgb_kdf::{Layer, NodeClassifier};
use real_data_bench::Dataset;
use std::collections::{BTreeMap, HashSet};

const N_SEEDS: usize = 5;
// Aggressive budget to differentiate ablations — at 30% all pass trivially.
const SELECTION_FRAC: f64 = 0.08;

#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)] // ablation index Aₙ_Description preserves output-string compatibility
enum Ablation {
    A0_Full,
    A1_NoRarePriority,
    A2_NoCorePriority,
    A3_NoGarbageFilter,
    A4_NoClusterDedup,
    A5_WithRelDensity,
}

fn select(ds: &Dataset, ablation: Ablation) -> HashSet<u32> {
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let budget = (ds.n_nodes as f64 * SELECTION_FRAC).ceil() as usize;

    match ablation {
        Ablation::A0_Full => select_full(ds, &class.layers, budget),
        Ablation::A1_NoRarePriority => select_no_rare(ds, &class.layers, budget),
        Ablation::A2_NoCorePriority => select_no_core(ds, &class.layers, budget),
        Ablation::A3_NoGarbageFilter => select_keep_garbage(ds, &class.layers, budget),
        Ablation::A4_NoClusterDedup => select_no_dedup(ds, &class.layers, budget),
        Ablation::A5_WithRelDensity => select_with_reldensity(ds, budget),
    }
}

fn score_for_layer(l: Layer) -> i32 {
    match l {
        Layer::Rare => 3,
        Layer::Core => 2,
        Layer::Edge => 1,
        Layer::Garbage => 0,
    }
}

fn select_full(
    ds: &Dataset,
    layers: &std::collections::HashMap<u32, Layer>,
    budget: usize,
) -> HashSet<u32> {
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| {
            (
                id,
                score_for_layer(layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(budget).map(|(i, _)| i).collect()
}

fn select_no_rare(
    ds: &Dataset,
    layers: &std::collections::HashMap<u32, Layer>,
    budget: usize,
) -> HashSet<u32> {
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 1,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| (id, score(layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(budget).map(|(i, _)| i).collect()
}

fn select_no_core(
    ds: &Dataset,
    layers: &std::collections::HashMap<u32, Layer>,
    budget: usize,
) -> HashSet<u32> {
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 1,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| (id, score(layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(budget).map(|(i, _)| i).collect()
}

fn select_keep_garbage(
    ds: &Dataset,
    layers: &std::collections::HashMap<u32, Layer>,
    budget: usize,
) -> HashSet<u32> {
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 1,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| (id, score(layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(budget).map(|(i, _)| i).collect()
}

fn select_no_dedup(
    ds: &Dataset,
    layers: &std::collections::HashMap<u32, Layer>,
    budget: usize,
) -> HashSet<u32> {
    // Same as full — full Kdf already has no dedup (we do single-pick).
    // We keep this as a sanity row: ablation identical to full, verify same output.
    select_full(ds, layers, budget)
}

fn select_with_reldensity(ds: &Dataset, budget: usize) -> HashSet<u32> {
    let n = ds.n_nodes;
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }
    let mut scored: Vec<(u32, f64)> = (0..n as u32)
        .map(|id| {
            let neighbors = &adj[id as usize];
            if neighbors.is_empty() {
                return (id, -1.0);
            }
            let local_avg: f64 = neighbors
                .iter()
                .map(|&v| deg[v as usize] as f64)
                .sum::<f64>()
                / neighbors.len() as f64;
            let ratio = deg[id as usize] as f64 / local_avg.max(1.0);
            (id, 1.0 - ratio)
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(budget).map(|(i, _)| i).collect()
}

fn eval_recall(ds: &Dataset, sel: &HashSet<u32>) -> f64 {
    let hit = sel.intersection(&ds.rare_ground_truth).count() as f64;
    hit / ds.rare_ground_truth.len().max(1) as f64
}

fn main() {
    let ablations = [
        (Ablation::A0_Full, "A0_Full"),
        (Ablation::A1_NoRarePriority, "A1_NoRarePriority"),
        (Ablation::A2_NoCorePriority, "A2_NoCorePriority"),
        (Ablation::A3_NoGarbageFilter, "A3_NoGarbageFilter"),
        (Ablation::A4_NoClusterDedup, "A4_NoClusterDedup"),
        (Ablation::A5_WithRelDensity, "A5_WithRelDensity"),
    ];

    let conditions: Vec<(String, Box<dyn Fn(u64) -> Dataset>)> = vec![
        (
            "A_deg1_D1type".into(),
            Box::new(|s| high_degree_rare(500, 1, s)),
        ),
        (
            "A_deg3_D1.5type".into(),
            Box::new(|s| high_degree_rare(500, 3, s)),
        ),
        (
            "B_deg2".into(),
            Box::new(|s| structurally_isolated(500, 2, s)),
        ),
    ];

    let seeds: Vec<u64> = (0..N_SEEDS as u64).map(|i| 42 + i * 100).collect();
    let mut table: BTreeMap<(String, String), Vec<f64>> = BTreeMap::new();

    for (cond_name, gen) in &conditions {
        for &seed in &seeds {
            let ds = gen(seed);
            for (abl, name) in &ablations {
                let sel = select(&ds, *abl);
                let r = eval_recall(&ds, &sel);
                table
                    .entry((cond_name.clone(), name.to_string()))
                    .or_default()
                    .push(r);
            }
        }
    }

    println!("# Phase C — Ablation study\n");
    println!("Each row: recall mean ± stderr over 5 dataset seeds.\n");
    println!("| Condition | Ablation | Recall mean ± SE | Δ vs A0_Full |");
    println!("|---|---|---:|---:|");

    let cond_names: Vec<String> = conditions.iter().map(|(n, _)| n.clone()).collect();
    for cond in &cond_names {
        let full = table
            .get(&(cond.clone(), "A0_Full".into()))
            .cloned()
            .unwrap_or_default();
        let full_mean: f64 = full.iter().sum::<f64>() / full.len().max(1) as f64;
        for (_, abl_name) in &ablations {
            let vals = table
                .get(&(cond.clone(), abl_name.to_string()))
                .cloned()
                .unwrap_or_default();
            let n = vals.len() as f64;
            let m = vals.iter().sum::<f64>() / n.max(1.0);
            let var = vals.iter().map(|x| (x - m).powi(2)).sum::<f64>() / n.max(1.0);
            let se = (var / n.max(1.0)).sqrt();
            let delta = if *abl_name == "A0_Full" {
                0.0
            } else {
                m - full_mean
            };
            let marker = if delta.abs() < 0.001 {
                "  ≈"
            } else if delta > 0.0 {
                " +"
            } else {
                "  "
            };
            println!(
                "| {} | {} | {:.3} ± {:.3} | {}{:+.3} |",
                cond, abl_name, m, se, marker, delta
            );
        }
    }

    println!("\n## Interpretation");
    println!("- A0_Full: current KDF baseline selection policy");
    println!("- A1: Rare layer no longer has priority → expect drop on D1 type");
    println!("- A2: Core priority removed → minor impact (Cores are already selected)");
    println!("- A3: Garbage kept → dilutes selection, Recall may drop slightly");
    println!("- A4: sanity check (implementation identical to A0 currently)");
    println!("- A5: RelDensity replaces layer-based score → expected win on D1.5");

    std::fs::create_dir_all("demos/verification").ok();
    let json = serde_json::json!({"ablation_table": table.iter()
        .map(|(k, v)| serde_json::json!({"condition": k.0, "ablation": k.1, "values": v}))
        .collect::<Vec<_>>()});
    std::fs::write(
        "demos/verification/ablation.json",
        serde_json::to_string_pretty(&json).unwrap(),
    )
    .unwrap();
    println!("\n✅ Written demos/verification/ablation.json");
}
