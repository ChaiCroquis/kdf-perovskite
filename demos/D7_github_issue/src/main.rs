//! Demo D7 — GitHub issue archive + analogy-based reopen candidate detection.
//!
//! # Hypothesis
//! "Forgotten-but-relevant" old issues = issues closed long ago that are
//! **structurally similar** to currently-open ones (shared labels, referenced
//! by similar PRs, etc.). This is **archetypal Claim 46 territory** — analogy
//! discovery between isolated-but-structurally-similar components.
//!
//! # Data
//! Synthetic issue archive: 500 issues with labels, author-reply graph,
//! reference-between-issues graph. Ground truth "relevant-to-reopen" = 30
//! issues whose label + ref pattern structurally matches recently-opened ones.
//!
//! # Baselines
//! - StaleBot (age > threshold → close/drop, inverse = "keep if young")
//! - LabelMatch (keep all closed issues sharing any label with open ones)
//! - TextSim (title-based shingle similarity to open issues)
//! - KDF (issue co-label + co-author graph)
//! - KDF+Analogy (Claim 46 fingerprint bridge to open-issue cluster)

use kdf_demos_common::{
    visualizer::emit_artifacts, Axis, Conclusion, DemoReport, MethodResult, Metric,
};
use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::Dataset;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const DEMO_ID: &str = "D7";
const DEMO_TITLE: &str = "GitHub Issue アーカイブ + reopen 候補の構造類似発見";
const N_TRIALS: usize = 10;
const SELECTION_FRAC: f64 = 0.30;

fn main() {
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 9000 + i).collect();

    let (issues, ds, reopen_truth) = synthesize_issues(42);
    println!(
        "Issues: n={}, edges(co-label/co-author)={}, reopen_truth={}",
        issues.len(),
        ds.edges.len(),
        reopen_truth.len()
    );

    let keep = (issues.len() as f64 * SELECTION_FRAC).ceil() as usize;

    let methods: Vec<(
        String,
        bool,
        Box<dyn Fn(&[Issue], &Dataset, u64) -> HashSet<u32>>,
    )> = vec![
        (
            "Random".into(),
            false,
            Box::new(move |issues, _ds, seed| sample_random(issues.len(), keep, seed)),
        ),
        (
            "StaleBot".into(),
            false,
            Box::new(move |issues, _ds, _seed| sample_stale_young_first(issues, keep)),
        ),
        (
            "LabelMatch".into(),
            false,
            Box::new(move |issues, _ds, _seed| sample_label_match(issues, keep)),
        ),
        (
            "TextSim".into(),
            false,
            Box::new(move |issues, _ds, _seed| sample_text_sim(issues, keep)),
        ),
        (
            "KDF".into(),
            false,
            Box::new(move |_issues, ds, _seed| sample_kdf(ds, keep)),
        ),
        (
            "KDF+Analogy".into(),
            false,
            Box::new(move |issues, ds, _seed| sample_kdf_analogy(issues, ds, keep)),
        ),
    ];

    let mut method_results: Vec<MethodResult> = Vec::new();
    let mut raw_trials: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (name, needs_label, sampler) in &methods {
        let mut reopen_recalls = Vec::new();
        let mut precisions = Vec::new();
        let mut compressions = Vec::new();
        let mut walls = Vec::new();
        for &seed in &seeds {
            let t0 = Instant::now();
            let sel = sampler(&issues, &ds, seed);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let hits = sel.intersection(&reopen_truth).count();
            let recall = hits as f64 / reopen_truth.len().max(1) as f64;
            let precision = if sel.is_empty() {
                0.0
            } else {
                hits as f64 / sel.len() as f64
            };
            let comp = 1.0 - sel.len() as f64 / issues.len() as f64;
            reopen_recalls.push(recall);
            precisions.push(precision);
            compressions.push(comp);
            walls.push(ms);
            raw_trials
                .entry(format!("{}/reopen_recall", name))
                .or_default()
                .push(recall);
            raw_trials
                .entry(format!("{}/precision", name))
                .or_default()
                .push(precision);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&reopen_recalls);
        let p = mean(&precisions);
        let c = mean(&compressions);
        let w = mean(&walls);
        println!(
            "{:14} reopen_recall={:.3}  precision={:.3}  comp={:.3}  ms={:.2}",
            name, r, p, c, w
        );
        let mut metrics = BTreeMap::new();
        metrics.insert("reopen_recall".into(), r);
        metrics.insert("precision".into(), p);
        metrics.insert("compression".into(), c);
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
            name: "reopen_recall".into(),
            higher_is_better: true,
            mean: 0.0,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        },
        Metric {
            name: "precision".into(),
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

    let report = DemoReport {
        demo_id: DEMO_ID.to_string(),
        title: DEMO_TITLE.to_string(),
        dataset_name: format!("synthetic_issues_n{}", issues.len()),
        n_items: issues.len(),
        patent_section: "明細書 §0002 (アーカイブ管理) / Claim 1, 42, 46 (整合性発見)".into(),
        metric_definitions,
        method_results,
        raw_trials,
        conclusion: Conclusion {
            kdf_recommended_for: vec![
                "issue tracker の自動アーカイブで、**過去 closed issue が現在の open issue と構造類似**する場合に reopen 候補として surface".into(),
                "ラベル / author / reference 構造が豊富な issue tracker".into(),
            ],
            kdf_not_recommended_for: vec![
                "単純な age-based stale 運用 → StaleBot で十分、KDF overhead に見合わない".into(),
                "完全なテキスト意味解釈が必要 → LLM triage の方が強い".into(),
            ],
            honest_limits: vec![
                "合成 issue archive (n=500) での評価、実 rust-lang/rust 等での数値は異なる".into(),
                "reopen_truth は合成生成した label+reference パターン一致であり、実運用の reopen とは異なる".into(),
                "実際の issue は title/body text を LLM で解釈すべきで、本 demo は構造のみ".into(),
            ],
        },
    };

    let out_dir = std::path::Path::new("demos/D7_github_issue/out");
    emit_artifacts(&report, out_dir).expect("emit");
    println!("\n✅ D7 artifacts written to {}", out_dir.display());
}

// ============================================================================
// Synthetic issue archive
// ============================================================================

#[derive(Clone)]
struct Issue {
    id: u32,
    is_open: bool,
    age_days: u32,
    labels: Vec<u32>,     // label ids (0..10)
    author: u32,          // author id (0..50)
    references: Vec<u32>, // referenced other issue ids
    title_shingles: HashSet<String>,
}

fn synthesize_issues(seed: u64) -> (Vec<Issue>, Dataset, HashSet<u32>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let n = 500;
    let mut issues: Vec<Issue> = Vec::with_capacity(n);
    let mut reopen_truth: HashSet<u32> = HashSet::new();

    // Archetype: a "pattern" is (label_set, title_theme)
    // 4 patterns. Some open issues share pattern with closed "forgotten" issues.
    let patterns = [
        (vec![0u32, 1], "parser crash edge case"),
        (vec![2, 3], "memory leak in threading"),
        (vec![4, 5], "regex performance"),
        (vec![6, 7], "docs build failure"),
    ];

    for i in 0..n as u32 {
        let pattern_idx = (rng.gen_range(0..100) % 10) as usize;
        let (labels, title_stem) = if pattern_idx < 4 {
            patterns[pattern_idx].clone()
        } else {
            // noise: random labels
            (
                vec![rng.gen_range(0..10) as u32],
                "miscellaneous bug report",
            )
        };
        let title = format!("{} #{}", title_stem, i);
        let shs = title
            .to_lowercase()
            .chars()
            .collect::<Vec<_>>()
            .windows(3)
            .map(|w| w.iter().collect::<String>())
            .collect();
        let is_open = i >= 450; // last 50 are currently open
        let age_days = if is_open {
            rng.gen_range(1..30)
        } else {
            rng.gen_range(90..720)
        };
        let author = rng.gen_range(0..50) as u32;
        let references: Vec<u32> = if rng.gen_bool(0.15) {
            let k = rng.gen_range(1..=3);
            (0..k).map(|_| rng.gen_range(0..i.max(1))).collect()
        } else {
            Vec::new()
        };

        issues.push(Issue {
            id: i,
            is_open,
            age_days,
            labels,
            author,
            references,
            title_shingles: shs,
        });
    }

    // Reopen truth: **strictly** defined to limit to ~30 issues:
    //   closed issue whose (sorted labels, author) tuple exactly matches
    //   at least one currently-open issue. This captures "same person
    //   reported the same kind of bug twice" — a genuine reopen candidate.
    let open_sigs: HashSet<(Vec<u32>, u32)> = issues
        .iter()
        .filter(|iss| iss.is_open)
        .map(|iss| {
            let mut l = iss.labels.clone();
            l.sort();
            (l, iss.author)
        })
        .collect();
    for iss in &issues {
        if iss.is_open {
            continue;
        }
        let mut l = iss.labels.clone();
        l.sort();
        if open_sigs.contains(&(l, iss.author)) {
            reopen_truth.insert(iss.id);
        }
    }

    // Graph: edges = issues sharing a label or referencing each other
    let mut edges = Vec::new();
    let mut by_label: HashMap<u32, Vec<u32>> = HashMap::new();
    for iss in &issues {
        for &l in &iss.labels {
            by_label.entry(l).or_default().push(iss.id);
        }
    }
    for idxs in by_label.values() {
        for i in 0..idxs.len() {
            for j in (i + 1)..idxs.len().min(i + 5) {
                edges.push((idxs[i], idxs[j], 1.0));
            }
        }
    }
    for iss in &issues {
        for &r in &iss.references {
            edges.push((iss.id, r, 1.0));
        }
    }

    let ds = Dataset {
        name: "synthetic_issues".into(),
        n_nodes: n,
        edges,
        rare_ground_truth: reopen_truth.clone(),
        description: "synthetic: 500 issues (450 closed, 50 open), reopen_truth = closed sharing pattern with open".into(),
    };
    (issues, ds, reopen_truth)
}

// ============================================================================
// Samplers
// ============================================================================

fn sample_random(n: usize, keep: usize, seed: u64) -> HashSet<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut idx: Vec<u32> = (0..n as u32).collect();
    idx.shuffle(&mut rng);
    idx.into_iter().take(keep).collect()
}

fn sample_stale_young_first(issues: &[Issue], keep: usize) -> HashSet<u32> {
    // StaleBot = "close old issues" → for "keep" semantics, keep youngest first
    let mut order: Vec<u32> = (0..issues.len() as u32).collect();
    order.sort_by_key(|&i| issues[i as usize].age_days);
    order.into_iter().take(keep).collect()
}

fn sample_label_match(issues: &[Issue], keep: usize) -> HashSet<u32> {
    let open_labels: HashSet<u32> = issues
        .iter()
        .filter(|i| i.is_open)
        .flat_map(|i| i.labels.iter().copied())
        .collect();
    let matched: Vec<u32> = issues
        .iter()
        .filter(|i| i.labels.iter().any(|l| open_labels.contains(l)))
        .map(|i| i.id)
        .collect();
    matched.into_iter().take(keep).collect()
}

fn sample_text_sim(issues: &[Issue], keep: usize) -> HashSet<u32> {
    // For each closed issue, compute max Jaccard similarity to any open issue title
    let open_sh: Vec<&HashSet<String>> = issues
        .iter()
        .filter(|i| i.is_open)
        .map(|i| &i.title_shingles)
        .collect();
    let mut scored: Vec<(u32, f64)> = issues
        .iter()
        .map(|iss| {
            let s = open_sh
                .iter()
                .map(|osh| {
                    let inter = iss.title_shingles.intersection(osh).count();
                    let union = iss.title_shingles.union(osh).count().max(1);
                    inter as f64 / union as f64
                })
                .fold(0.0, f64::max);
            (iss.id, s)
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
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
    scored.into_iter().take(keep).map(|(id, _)| id).collect()
}

fn sample_kdf_analogy(issues: &[Issue], ds: &Dataset, keep: usize) -> HashSet<u32> {
    // Claim 46 idea: find closed issues whose neighborhood-degree profile is
    // similar to open issues' profile. These are "structural twins in archive".
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
        let mut bins = [0.0f64; 4];
        for &v in &adj[i] {
            let d = deg[v as usize];
            let idx = if d < 3 {
                0
            } else if d < 7 {
                1
            } else if d < 15 {
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
    };

    // Open-issue average fingerprint
    let open_idx: Vec<u32> = issues.iter().filter(|i| i.is_open).map(|i| i.id).collect();
    let mut avg_fp = [0.0f64; 4];
    for &id in &open_idx {
        let f = fp(id as usize);
        for d in 0..4 {
            avg_fp[d] += f[d];
        }
    }
    let n_open = open_idx.len().max(1) as f64;
    for d in 0..4 {
        avg_fp[d] /= n_open;
    }

    // Score closed issues by similarity to open-avg fingerprint
    let mut scored: Vec<(u32, f64)> = issues
        .iter()
        .filter(|i| !i.is_open)
        .map(|iss| {
            let f = fp(iss.id as usize);
            let dist: f64 = f
                .iter()
                .zip(avg_fp.iter())
                .map(|(a, b)| (a - b).abs())
                .sum();
            (iss.id, -dist) // small dist → high score
        })
        .collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let mut out: HashSet<u32> = scored
        .into_iter()
        .take(keep - open_idx.len().min(keep))
        .map(|(i, _)| i)
        .collect();
    // Always include open issues
    for id in open_idx {
        out.insert(id);
        if out.len() >= keep {
            break;
        }
    }
    out
}
