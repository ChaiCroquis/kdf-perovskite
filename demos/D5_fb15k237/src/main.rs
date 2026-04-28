//! Demo D5 — Knowledge graph curation: preserve rare entities + surface analogies.
//!
//! Problem: knowledge graphs like FB15K-237 contain tens of thousands of
//! entities with a Zipf-distributed degree. For downstream tasks (link
//! prediction, KG completion) we often want to reduce the graph to a
//! working subset while preserving entities that touch *rare relations*
//! (freq ≤ 5) — these are where novel facts usually live.
//!
//! # Data policy
//! FB15K-237 is not redistributed here. Place the three files under
//! `demos/D5_fb15k237/data/fb15k-237/{train,valid,test}.txt`. If missing,
//! a synthetic Freebase-shaped KG is generated.

use kdf_demos_common::{
    visualizer::emit_artifacts, Axis, Conclusion, DemoReport, MethodResult, Metric,
};
use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::{public_datasets, Dataset};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const DEMO_ID: &str = "D5";
const DEMO_TITLE: &str = "知識グラフ (FB15K-237) 希少 entity 保存付き curation";
const N_TRIALS: usize = 10;
const SELECTION_FRAC: f64 = 0.30;

fn main() {
    // rare_freq_max tuned for real FB15K-237: relations with ≤200 triples are
    // in the bottom ~10% tail (empirical min ≈45, max ≈16000).
    let (ds, synthetic) = match public_datasets::load_fb15k_237(200) {
        Some(d) => {
            println!(
                "Loaded real FB15K-237: n={} entities, {} edges, rare={}",
                d.n_nodes,
                d.edges.len(),
                d.rare_ground_truth.len()
            );
            (d, false)
        }
        None => {
            println!("Real FB15K-237 not found. Using synthetic KG (Freebase-shaped).");
            println!("To use real data: place train/valid/test.txt under demos/D5_fb15k237/data/fb15k-237/");
            let d = synthesize_kg(5_000, 50, 42);
            println!(
                "  synthetic KG: n={}, edges={}, rare_truth={}",
                d.n_nodes,
                d.edges.len(),
                d.rare_ground_truth.len()
            );
            (d, true)
        }
    };

    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 6000 + i).collect();
    let keep = (ds.n_nodes as f64 * SELECTION_FRAC).ceil() as usize;

    let methods: Vec<(&str, bool, Box<dyn Fn(&Dataset, u64) -> HashSet<u32>>)> = vec![
        (
            "Random",
            false,
            Box::new(|ds, seed| sample_random(ds, SELECTION_FRAC, seed)),
        ),
        (
            "FreqCutoff",
            false,
            Box::new(move |ds, _s| sample_freq_cutoff(ds, keep)),
        ),
        (
            "DegreeTopK",
            false,
            Box::new(move |ds, _s| sample_degree_top_k(ds, keep)),
        ),
        (
            "TransE-like",
            false,
            Box::new(move |ds, seed| sample_transe_like(ds, keep, seed)),
        ),
        ("KDF", false, Box::new(|ds, _s| sample_kdf(ds))),
        (
            "KDF+RelDensity",
            false,
            Box::new(|ds, _s| sample_kdf_reldensity(ds)),
        ),
        (
            "KDF+Analogy",
            false,
            Box::new(|ds, _s| sample_kdf_with_analogy(ds)),
        ),
    ];

    let mut method_results: Vec<MethodResult> = Vec::new();
    let mut raw_trials: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (name, needs_label, sampler) in &methods {
        let mut recalls = Vec::new();
        let mut analogy_counts = Vec::new();
        let mut compressions = Vec::new();
        let mut walls = Vec::new();
        for &seed in &seeds {
            let t0 = Instant::now();
            let sel = sampler(&ds, seed);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let rare_sel = sel.intersection(&ds.rare_ground_truth).count();
            let rare_tot = ds.rare_ground_truth.len().max(1);
            let recall = rare_sel as f64 / rare_tot as f64;
            let comp = 1.0 - sel.len() as f64 / ds.n_nodes.max(1) as f64;
            let analogy = count_cross_cluster_pairs(&ds, &sel);
            recalls.push(recall);
            compressions.push(comp);
            analogy_counts.push(analogy as f64);
            walls.push(ms);
            raw_trials
                .entry(format!("{}/rare_recall", name))
                .or_default()
                .push(recall);
            raw_trials
                .entry(format!("{}/analogy_pairs", name))
                .or_default()
                .push(analogy as f64);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&recalls);
        let a = mean(&analogy_counts);
        let c = mean(&compressions);
        let w = mean(&walls);
        println!(
            "{:18} recall={:.3}  comp={:.3}  analogy={:.0}  ms={:.2}",
            name, r, c, a, w
        );
        let mut m = BTreeMap::new();
        m.insert("rare_recall".into(), r);
        m.insert("analogy_pairs".into(), a);
        m.insert("compression".into(), c);
        m.insert("wall_ms".into(), w);
        method_results.push(MethodResult {
            method: name.to_string(),
            requires_labels: *needs_label,
            metrics: m,
            wall_ms: w,
            notes: String::new(),
        });
    }

    let metric_definitions = vec![
        Metric {
            name: "rare_recall".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        },
        Metric {
            name: "analogy_pairs".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        },
        Metric {
            name: "compression".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::Tie,
        },
        Metric {
            name: "wall_ms".into(),
            higher_is_better: false,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfWeakness,
        },
    ];

    let mut limits = vec![
        "rare entity = 出現関係 freq ≤ 5 の端点、という定義に対する評価。他の定義(betweenness 等)は別途検証要".into(),
        "TransE-like は「次数 top-K を embedding-top-K の近似プロキシ」として実装(訓練しない)".into(),
    ];
    if synthetic {
        limits.insert(0, format!(
            "本実行は合成 KG (n={}, Freebase-shaped) を使用。実 FB15K-237 使用時は `demos/D5_fb15k237/data/fb15k-237/` に train/valid/test.txt を配置",
            ds.n_nodes
        ));
    }

    let report = DemoReport {
        demo_id: DEMO_ID.to_string(),
        title: DEMO_TITLE.to_string(),
        dataset_name: ds.name.clone(),
        n_items: ds.n_nodes,
        patent_section: "明細書 §0002 (知識グラフ) / Claim 1, 42, 46 (整合性発見)".into(),
        metric_definitions,
        method_results,
        raw_trials,
        conclusion: Conclusion {
            kdf_recommended_for: vec![
                "KG の長尾 entity(出現稀な relation を触れる)を保護しつつ graph 縮約".into(),
                "既存の確立 cluster と孤立 entity 間の構造類似 pair を発見する用途".into(),
            ],
            kdf_not_recommended_for: vec![
                "純粋な新規 link prediction → TransE/ComplEx などの embedding 系が適切".into(),
                "高頻度 entity の重要度ランキング → 次数ランキングで十分".into(),
            ],
            honest_limits: limits,
        },
    };

    let out_dir = std::path::Path::new("demos/D5_fb15k237/out");
    emit_artifacts(&report, out_dir).expect("emit");
    println!("\n✅ D5 artifacts written to {}", out_dir.display());
}

// ============================================================================
// Synthetic KG generator
// ============================================================================

fn synthesize_kg(n_entities: usize, n_relations: usize, seed: u64) -> Dataset {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut edge_rels: Vec<usize> = Vec::new();
    let mut relation_counts: HashMap<usize, u32> = HashMap::new();

    // Main relations 0..(n_relations-N_RARE) get the bulk of edges.
    // Last N_RARE relations are explicitly rare (1-5 edges each).
    const N_RARE: usize = 10;
    let n_main = n_relations - N_RARE;
    let main_weights: Vec<f64> = (1..=n_main).map(|r| 1.0 / r as f64).collect();
    let mw_sum: f64 = main_weights.iter().sum();

    let n_main_edges = n_entities * 4 - N_RARE * 3; // rare relations get 3 edges each
    for _ in 0..n_main_edges {
        let h = rng.gen_range(0..n_entities) as u32;
        let t = rng.gen_range(0..n_entities) as u32;
        if h == t {
            continue;
        }
        let r_idx = {
            let u = rng.gen::<f64>() * mw_sum;
            let mut acc = 0.0;
            let mut idx = 0;
            for (i, w) in main_weights.iter().enumerate() {
                acc += w;
                if acc >= u {
                    idx = i;
                    break;
                }
            }
            idx
        };
        *relation_counts.entry(r_idx).or_insert(0) += 1;
        edges.push((h, t, 1.0));
        edge_rels.push(r_idx);
    }
    // Now add rare-relation edges: exactly 3 per rare relation, involving
    // *random existing entities* so rare entities look structurally normal.
    for ri in 0..N_RARE {
        let r_idx = n_main + ri;
        for _ in 0..3 {
            let h = rng.gen_range(0..n_entities) as u32;
            let t = rng.gen_range(0..n_entities) as u32;
            if h == t {
                continue;
            }
            *relation_counts.entry(r_idx).or_insert(0) += 1;
            edges.push((h, t, 1.0));
            edge_rels.push(r_idx);
        }
    }

    let rare_rels: HashSet<usize> = ((n_main)..n_relations).collect();

    // Rare ground truth = entities touching ANY rare-relation edge.
    // This is independent of the entity's total degree, making the benchmark
    // NOT trivially solvable by simple FreqCutoff.
    let mut rare: HashSet<u32> = HashSet::new();
    for (&(h, t, _), &r) in edges.iter().zip(edge_rels.iter()) {
        if rare_rels.contains(&r) {
            rare.insert(h);
            rare.insert(t);
        }
    }
    Dataset {
        name: format!("FB15K-237_synth_n{}_rel{}", n_entities, n_relations),
        n_nodes: n_entities,
        edges,
        rare_ground_truth: rare,
        description: format!(
            "Synthetic Freebase-shaped KG; rare entities touch one of {} rare relations (freq ≤ 5). \
             Rare entities may themselves have high degree.",
            rare_rels.len()
        ),
    }
}

// ============================================================================
// Samplers
// ============================================================================

fn sample_random(ds: &Dataset, p: f64, seed: u64) -> HashSet<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    (0..ds.n_nodes as u32).filter(|_| rng.gen_bool(p)).collect()
}

fn sample_freq_cutoff(ds: &Dataset, keep: usize) -> HashSet<u32> {
    // Keep entities with lowest degree (prioritize rare-looking).
    let mut deg = vec![0u32; ds.n_nodes];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < ds.n_nodes {
            deg[u as usize] += 1;
        }
        if (v as usize) < ds.n_nodes {
            deg[v as usize] += 1;
        }
    }
    let mut order: Vec<u32> = (0..ds.n_nodes as u32)
        .filter(|&i| deg[i as usize] > 0)
        .collect();
    order.sort_by_key(|&i| deg[i as usize]);
    order.into_iter().take(keep).collect()
}

fn sample_degree_top_k(ds: &Dataset, keep: usize) -> HashSet<u32> {
    // Keep highest-degree entities (conventional KG pruning).
    let mut deg = vec![0u32; ds.n_nodes];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < ds.n_nodes {
            deg[u as usize] += 1;
        }
        if (v as usize) < ds.n_nodes {
            deg[v as usize] += 1;
        }
    }
    let mut order: Vec<u32> = (0..ds.n_nodes as u32).collect();
    order.sort_by_key(|&i| std::cmp::Reverse(deg[i as usize]));
    order.into_iter().take(keep).collect()
}

fn sample_transe_like(ds: &Dataset, keep: usize, seed: u64) -> HashSet<u32> {
    // TransE proxy: mix of degree-top-K (anchor entities) + random (diversity).
    // Real TransE would train embeddings; here we approximate deterministically.
    let mut rng = SmallRng::seed_from_u64(seed);
    let top_k_portion = (keep as f64 * 0.7) as usize;
    let rand_portion = keep.saturating_sub(top_k_portion);
    let mut out = sample_degree_top_k(ds, top_k_portion);
    while out.len() < top_k_portion + rand_portion {
        let id = rng.gen_range(0..ds.n_nodes) as u32;
        out.insert(id);
    }
    out
}

fn sample_kdf(ds: &Dataset) -> HashSet<u32> {
    // Budgeted KDF: use classifier priority to pick top-K under the same
    // compression budget as other methods.
    use cgb_kdf::{Layer, NodeClassifier};
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let budget = (ds.n_nodes as f64 * SELECTION_FRAC).ceil() as usize;

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
    scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
    scored.into_iter().take(budget).map(|(id, _)| id).collect()
}

/// KDF + Phase 7 S2 RelativeDensity — score entities by how much their
/// degree is below their 1-hop neighborhood average. Surfaces entities that
/// look "relatively rare in context" regardless of absolute degree.
fn sample_kdf_reldensity(ds: &Dataset) -> HashSet<u32> {
    let n = ds.n_nodes;
    let budget = (n as f64 * SELECTION_FRAC).ceil() as usize;
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < n && (v as usize) < n {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
            adj[u as usize].push(v);
            adj[v as usize].push(u);
        }
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

fn sample_kdf_with_analogy(ds: &Dataset) -> HashSet<u32> {
    // KDF budgeted + Phase 7 S3 fingerprint isolation bonus for rare-looking
    // structures. Reserve 20% of budget for fingerprint-isolated nodes.
    use cgb_kdf::{Layer, NodeClassifier};
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let budget = (ds.n_nodes as f64 * SELECTION_FRAC).ceil() as usize;
    let analogy_budget = budget / 5;
    let layer_budget = budget - analogy_budget;

    let score_layer = |l: Layer| -> i32 {
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
                score_layer(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
    let mut out: HashSet<u32> = scored
        .into_iter()
        .take(layer_budget)
        .map(|(id, _)| id)
        .collect();

    // Analogy bonus: pick fingerprint-isolated nodes not yet selected
    let fps = compute_degree_histograms(ds);
    let median = median_histogram(&fps);
    let mut ranked: Vec<(u32, f64)> = (0..ds.n_nodes as u32)
        .filter(|id| !out.contains(id))
        .map(|id| (id, l1_dist(&fps[id as usize], &median)))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    for (id, _) in ranked.into_iter().take(analogy_budget) {
        out.insert(id);
    }
    out
}

fn compute_degree_histograms(ds: &Dataset) -> Vec<[f64; 4]> {
    let n = ds.n_nodes;
    let mut deg = vec![0u32; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < n && (v as usize) < n {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
            adj[u as usize].push(v);
            adj[v as usize].push(u);
        }
    }
    (0..n)
        .map(|i| {
            let mut bins = [0.0f64; 4];
            for &v in &adj[i] {
                let d = deg[v as usize];
                let idx = if d < 2 {
                    0
                } else if d < 5 {
                    1
                } else if d < 20 {
                    2
                } else {
                    3
                };
                bins[idx] += 1.0;
            }
            let tot: f64 = bins.iter().sum();
            if tot > 0.0 {
                for b in bins.iter_mut() {
                    *b /= tot;
                }
            }
            bins
        })
        .collect()
}

fn median_histogram(fps: &[[f64; 4]]) -> [f64; 4] {
    let mut median = [0.0f64; 4];
    for dim in 0..4 {
        let mut v: Vec<f64> = fps.iter().map(|fp| fp[dim]).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        median[dim] = v[v.len() / 2];
    }
    median
}

fn l1_dist(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

// ============================================================================
// Analogy count: pairs of selected entities with similar degree histograms
// but no direct edge
// ============================================================================

fn count_cross_cluster_pairs(ds: &Dataset, selected: &HashSet<u32>) -> usize {
    let adj_set: HashSet<(u32, u32)> = ds
        .edges
        .iter()
        .map(|&(u, v, _)| if u < v { (u, v) } else { (v, u) })
        .collect();
    let fps = compute_degree_histograms(ds);
    // Sort for determinism — HashSet iteration order is unspecified
    let mut selected_vec: Vec<u32> = selected.iter().copied().collect();
    selected_vec.sort();
    let mut count = 0;
    // Sample to keep this O(selected^2) bounded: only check first 200
    let cap = selected_vec.len().min(200);
    for i in 0..cap {
        for j in (i + 1)..cap {
            let (a, b) = (selected_vec[i], selected_vec[j]);
            let key = if a < b { (a, b) } else { (b, a) };
            if adj_set.contains(&key) {
                continue;
            }
            let dist = l1_dist(&fps[a as usize], &fps[b as usize]);
            if dist < 0.1 && fps[a as usize].iter().sum::<f64>() > 0.0 {
                count += 1;
            }
        }
    }
    count
}
