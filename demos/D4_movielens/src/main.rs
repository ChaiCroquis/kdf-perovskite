//! Demo D4 — Long-tail recommendation item preservation (MovieLens-style).
//!
//! # Hypothesis
//! Per Stage 1 meta-analysis: MovieLens items form a user-item bipartite
//! graph where long-tail items = low-degree items. This is "D1-type"
//! (structure encodes rareness) → predict strong KDF baseline.
//!
//! # Data
//! Synthetic MovieLens shape: 500 users × 300 items with Zipf popularity.
//! Long-tail = items with fewer than `n_users / n_items` ratings (below
//! mean popularity). Goal: when selecting 30% of items for index/cache,
//! how well do we preserve the long tail?
//!
//! # Baselines
//! - Random
//! - PopularityTop (conventional: keep top-rated items)
//! - MatrixFactorization proxy (keep items with highest rating variance)
//! - KDF (bipartite graph rareness)
//! - KDF+Analogy (fingerprint bridge)

use kdf_demos_common::{
    Axis, Conclusion, DemoReport, MethodResult, Metric, visualizer::emit_artifacts,
};
use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::Dataset;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const DEMO_ID: &str = "D4";
const DEMO_TITLE: &str = "推薦システム long-tail アイテム保持 curation";
const N_TRIALS: usize = 10;
const SELECTION_FRAC: f64 = 0.30;
const N_USERS: usize = 500;
const N_ITEMS: usize = 300;

fn main() {
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 10000 + i).collect();

    let (ratings, ds, long_tail) = synthesize_movielens(42);
    println!(
        "MovieLens: {} ratings, {} users, {} items, long_tail={}",
        ratings.len(),
        N_USERS,
        N_ITEMS,
        long_tail.len()
    );

    let keep = (N_ITEMS as f64 * SELECTION_FRAC).ceil() as usize;

    let methods: Vec<(
        String,
        bool,
        Box<dyn Fn(&[Rating], &Dataset, u64) -> HashSet<u32>>,
    )> = vec![
        (
            "Random".into(),
            false,
            Box::new(move |_r, _d, seed| sample_random_items(keep, seed)),
        ),
        (
            "PopularityTop".into(),
            false,
            Box::new(move |ratings, _d, _s| sample_popularity_top(ratings, keep)),
        ),
        (
            "MF-proxy".into(),
            false,
            Box::new(move |ratings, _d, _s| sample_mf_proxy(ratings, keep)),
        ),
        (
            "KDF".into(),
            false,
            Box::new(move |_r, ds, _s| sample_kdf(ds, keep)),
        ),
        (
            "KDF+RelDensity".into(),
            false,
            Box::new(move |_r, ds, _s| sample_kdf_reldensity(ds, keep)),
        ),
        (
            "KDF+Analogy".into(),
            false,
            Box::new(move |_r, ds, _s| sample_kdf_analogy(ds, keep)),
        ),
    ];

    let mut method_results: Vec<MethodResult> = Vec::new();
    let mut raw_trials: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (name, needs_label, sampler) in &methods {
        let mut tail_recalls = Vec::new();
        let mut coverages = Vec::new();
        let mut ndcg_tails = Vec::new();
        let mut walls = Vec::new();
        for &seed in &seeds {
            let t0 = Instant::now();
            let sel = sampler(&ratings, &ds, seed);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let hits = sel.intersection(&long_tail).count();
            let tail_recall = hits as f64 / long_tail.len().max(1) as f64;
            let coverage = sel.len() as f64 / N_ITEMS as f64;
            // Simple tail-NDCG proxy: sum of 1/log(popularity+2) for selected tail items
            let tail_ndcg = ndcg_tail(&sel, &ratings);
            tail_recalls.push(tail_recall);
            coverages.push(coverage);
            ndcg_tails.push(tail_ndcg);
            walls.push(ms);
            raw_trials
                .entry(format!("{}/tail_recall", name))
                .or_default()
                .push(tail_recall);
            raw_trials
                .entry(format!("{}/tail_ndcg", name))
                .or_default()
                .push(tail_ndcg);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&tail_recalls);
        let cov = mean(&coverages);
        let ng = mean(&ndcg_tails);
        let w = mean(&walls);
        println!(
            "{:14} tail_recall={:.3}  coverage={:.3}  tail_ndcg={:.3}  ms={:.2}",
            name, r, cov, ng, w
        );
        let mut metrics = BTreeMap::new();
        metrics.insert("tail_recall".into(), r);
        metrics.insert("coverage".into(), cov);
        metrics.insert("tail_ndcg".into(), ng);
        metrics.insert("wall_ms".into(), w);
        method_results.push(MethodResult {
            method: name.clone(),
            requires_labels: *needs_label,
            metrics,
            wall_ms: w,
            notes: String::new(),
        });
    }

    let metric_definitions = vec![
        Metric {
            name: "tail_recall".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        },
        Metric {
            name: "tail_ndcg".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        },
        Metric {
            name: "coverage".into(),
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

    let report = DemoReport {
        demo_id: DEMO_ID.to_string(),
        title: DEMO_TITLE.to_string(),
        dataset_name: "synthetic_movielens_n500x300".into(),
        n_items: N_ITEMS,
        patent_section: "明細書 §0002 (検索又は推薦) / Claim 1, 18, 42".into(),
        metric_definitions,
        method_results,
        raw_trials,
        conclusion: Conclusion {
            kdf_recommended_for: vec![
                "推薦システムの item index / cache の **long-tail 保持付き縮減**".into(),
                "popularity top-K が多様性を下げすぎる環境の対策".into(),
                "user-item bipartite の構造シグナルで rare item を検出".into(),
            ],
            kdf_not_recommended_for: vec![
                "NDCG@10 のような精度第一指標 → MF / Neural CF に完敗".into(),
                "popularity-dominated item 推薦 → PopularityTop で十分".into(),
            ],
            honest_limits: vec![
                "合成 MovieLens 風データ(実 MovieLens 100K/1M の分布近似)".into(),
                "本 demo は item **selection** 品質のみ、実際の推薦精度は未測定".into(),
                "MF-proxy は variance-based heuristic、実 NMF / Neural CF ではない".into(),
            ],
        },
    };

    let out_dir = std::path::Path::new("demos/D4_movielens/out");
    emit_artifacts(&report, out_dir).expect("emit");
    println!("\n✅ D4 artifacts written to {}", out_dir.display());
}

// ============================================================================
// Synthetic MovieLens
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
struct Rating {
    user: u32,
    item: u32,
    rating: f64,
}

fn synthesize_movielens(seed: u64) -> (Vec<Rating>, Dataset, HashSet<u32>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    // Zipf popularity
    let item_weights: Vec<f64> = (1..=N_ITEMS).map(|r| 1.0 / r as f64).collect();
    let iw_sum: f64 = item_weights.iter().sum();

    let target_ratings = 20_000;
    let mut ratings: Vec<Rating> = Vec::with_capacity(target_ratings);
    let mut edges = Vec::with_capacity(target_ratings);

    for _ in 0..target_ratings {
        let user = rng.gen_range(0..N_USERS) as u32;
        let u = rng.r#gen::<f64>() * iw_sum;
        let mut acc = 0.0;
        let mut item_idx = N_ITEMS - 1;
        for (i, w) in item_weights.iter().enumerate() {
            acc += w;
            if u < acc {
                item_idx = i;
                break;
            }
        }
        let item = (N_USERS + item_idx) as u32; // offset item IDs after users
        let rating = rng.gen_range(10..=50) as f64 / 10.0;
        ratings.push(Rating { user, item, rating });
        edges.push((user, item, 1.0));
    }

    // long_tail: items with degree below global mean
    let mut item_deg: HashMap<u32, u32> = HashMap::new();
    for r in &ratings {
        *item_deg.entry(r.item).or_insert(0) += 1;
    }
    let mean_deg = item_deg.values().sum::<u32>() as f64 / item_deg.len().max(1) as f64;
    let long_tail: HashSet<u32> = item_deg
        .iter()
        .filter(|&(_, &d)| (d as f64) < mean_deg)
        .map(|(&i, _)| i)
        .collect();

    let ds = Dataset {
        name: "synth_movielens".into(),
        n_nodes: N_USERS + N_ITEMS,
        edges,
        rare_ground_truth: long_tail.clone(),
        description: format!(
            "synthetic MovieLens: {} users × {} items, Zipf popularity",
            N_USERS, N_ITEMS
        ),
    };
    (ratings, ds, long_tail)
}

// ============================================================================
// Samplers
// ============================================================================

fn sample_random_items(keep: usize, seed: u64) -> HashSet<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut item_ids: Vec<u32> = ((N_USERS as u32)..((N_USERS + N_ITEMS) as u32)).collect();
    item_ids.shuffle(&mut rng);
    item_ids.into_iter().take(keep).collect()
}

fn sample_popularity_top(ratings: &[Rating], keep: usize) -> HashSet<u32> {
    let mut item_deg: HashMap<u32, u32> = HashMap::new();
    for r in ratings {
        *item_deg.entry(r.item).or_insert(0) += 1;
    }
    let mut order: Vec<(u32, u32)> = item_deg.into_iter().collect();
    order.sort_by_key(|b| std::cmp::Reverse(b.1));
    order.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn sample_mf_proxy(ratings: &[Rating], keep: usize) -> HashSet<u32> {
    // Items with highest rating variance (proxy for informative latent factors)
    let mut item_rs: HashMap<u32, Vec<f64>> = HashMap::new();
    for r in ratings {
        item_rs.entry(r.item).or_default().push(r.rating);
    }
    let mut scored: Vec<(u32, f64)> = item_rs
        .into_iter()
        .map(|(i, rs)| {
            if rs.len() < 2 {
                return (i, 0.0);
            }
            let m = rs.iter().sum::<f64>() / rs.len() as f64;
            let v = rs.iter().map(|x| (x - m).powi(2)).sum::<f64>() / rs.len() as f64;
            (i, v * (rs.len() as f64).sqrt())
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn sample_kdf(ds: &Dataset, keep: usize) -> HashSet<u32> {
    use cgb_kdf::{Layer, NodeClassifier};
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    // Only score items (exclude user nodes)
    let mut scored: Vec<(u32, i32)> = ((N_USERS as u32)..((N_USERS + N_ITEMS) as u32))
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

/// KDF + Phase 7 S2 RelativeDensity — rank items by "how much below local
/// average degree" they are, selecting **relatively rare** items regardless
/// of absolute degree. This is the recommended extension for D1.5-type
/// datasets (bipartite with dense-ish items).
fn sample_kdf_reldensity(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let n = ds.n_nodes;
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }
    // Score each item by (local_avg - deg) / local_avg — positive = below local avg = rare
    let mut scored: Vec<(u32, f64)> = ((N_USERS as u32)..((N_USERS + N_ITEMS) as u32))
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
            (id, 1.0 - ratio) // higher = more relatively rare
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn sample_kdf_analogy(ds: &Dataset, keep: usize) -> HashSet<u32> {
    // 80% KDF + 20% fingerprint-isolated items
    let kdf_budget = (keep as f64 * 0.8) as usize;
    let mut out = sample_kdf(ds, kdf_budget);

    let n = ds.n_nodes;
    let mut deg = vec![0u32; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }
    let fp = |i: usize| -> [f64; 4] {
        let mut b = [0.0; 4];
        for &v in &adj[i] {
            let d = deg[v as usize];
            let idx = if d < 5 {
                0
            } else if d < 20 {
                1
            } else if d < 50 {
                2
            } else {
                3
            };
            b[idx] += 1.0;
        }
        let tot: f64 = b.iter().sum();
        if tot > 0.0 {
            for x in b.iter_mut() {
                *x /= tot;
            }
        }
        b
    };
    let items: Vec<u32> = ((N_USERS as u32)..((N_USERS + N_ITEMS) as u32)).collect();
    let mut median = [0.0f64; 4];
    for dim in 0..4 {
        let mut v: Vec<f64> = items.iter().map(|&i| fp(i as usize)[dim]).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        median[dim] = v[v.len() / 2];
    }
    let mut ranked: Vec<(u32, f64)> = items
        .iter()
        .filter(|i| !out.contains(i))
        .map(|&i| {
            let f = fp(i as usize);
            let d: f64 = f
                .iter()
                .zip(median.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            (i, d)
        })
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let bonus = keep.saturating_sub(out.len());
    for (i, _) in ranked.into_iter().take(bonus) {
        out.insert(i);
    }
    out
}

fn ndcg_tail(sel: &HashSet<u32>, ratings: &[Rating]) -> f64 {
    let mut item_deg: HashMap<u32, u32> = HashMap::new();
    for r in ratings {
        *item_deg.entry(r.item).or_insert(0) += 1;
    }
    let mut score = 0.0;
    for &id in sel {
        let d = *item_deg.get(&id).unwrap_or(&0);
        if d > 0 {
            score += 1.0 / ((d as f64 + 2.0).ln());
        }
    }
    score / sel.len().max(1) as f64
}
