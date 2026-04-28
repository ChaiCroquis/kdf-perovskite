//! Phase δ — KDF on REAL rust-lang/rust issues.
//!
//! World problem: OSS maintenance at scale — 60M+ issues across GitHub.
//! Can KDF help surface duplicate / should-reopen candidates without reading
//! text or requiring human labeling?
//!
//! Data: 500 most-recent closed issues from rust-lang/rust (5 pages × 100,
//! fetched via `curl -sL https://api.github.com/repos/rust-lang/rust/issues`).
//!
//! Ground truth: issues with `state_reason = "duplicate"` or `"not_planned"`
//! (19 of 500 = 3.8%) = "structurally suspicious" issues an OSS maintainer
//! would want to surface.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug, Clone)]
struct Label {
    name: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Issue {
    number: u32,
    title: String,
    #[serde(default)]
    state_reason: Option<String>,
    #[serde(default)]
    labels: Vec<Label>,
    #[serde(default)]
    user: Option<User>,
    #[serde(default)]
    pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct User {
    login: String,
}

fn load_issues() -> Vec<Issue> {
    let mut all = Vec::new();
    for p in 1..=5 {
        let path = format!("benchmarks/real_data/data/rust-issues/page{}.json", p);
        if let Ok(text) = std::fs::read_to_string(&path)
            && let Ok(batch) = serde_json::from_str::<Vec<Issue>>(&text)
        {
            all.extend(batch);
        }
    }
    all
}

fn main() {
    let issues = load_issues();
    if issues.is_empty() {
        eprintln!("No issue data found. Fetch with:");
        eprintln!("  for p in 1 2 3 4 5; do");
        eprintln!(
            "    curl -sL \"https://api.github.com/repos/rust-lang/rust/issues?state=closed&per_page=100&page=$p\" -o benchmarks/real_data/data/rust-issues/page$p.json;"
        );
        eprintln!("  done");
        std::process::exit(1);
    }
    println!(
        "Loaded {} issues from rust-lang/rust (real GitHub API dump)",
        issues.len()
    );

    // Filter to PURE issues (not PRs)
    let pure_issues: Vec<&Issue> = issues.iter().filter(|i| i.pull_request.is_none()).collect();
    println!("Pure issues (not PRs): {}", pure_issues.len());

    // Build id map: number → local index
    let index_of: HashMap<u32, u32> = pure_issues
        .iter()
        .enumerate()
        .map(|(i, iss)| (iss.number, i as u32))
        .collect();
    let n = pure_issues.len();

    // Build graph: edges from shared labels + shared author
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut label_groups: HashMap<String, Vec<u32>> = HashMap::new();
    let mut author_groups: HashMap<String, Vec<u32>> = HashMap::new();
    for (i, iss) in pure_issues.iter().enumerate() {
        for l in &iss.labels {
            label_groups
                .entry(l.name.clone())
                .or_default()
                .push(i as u32);
        }
        if let Some(u) = &iss.user {
            author_groups
                .entry(u.login.clone())
                .or_default()
                .push(i as u32);
        }
    }
    // Edges: limit per group to avoid quadratic
    for idxs in label_groups.values() {
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len().min(a + 5) {
                edges.push((idxs[a], idxs[b], 1.0));
            }
        }
    }
    for idxs in author_groups.values() {
        if idxs.len() < 2 || idxs.len() > 20 {
            continue;
        }
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len().min(a + 3) {
                edges.push((idxs[a], idxs[b], 0.5));
            }
        }
    }

    // Rare ground truth: duplicate or not_planned issues
    let mut rare_gt: HashSet<u32> = HashSet::new();
    for (i, iss) in pure_issues.iter().enumerate() {
        if matches!(
            iss.state_reason.as_deref(),
            Some("duplicate") | Some("not_planned")
        ) {
            rare_gt.insert(i as u32);
        }
    }
    println!(
        "Edges: {}, Rare ground truth (duplicate + not_planned): {} / {}",
        edges.len(),
        rare_gt.len(),
        n
    );
    let _ = index_of;

    // Samplers
    let keep = (n as f64 * 0.30).ceil() as usize;

    use cgb_kdf::{Layer, NodeClassifier};
    let random_select = |seed: u64| -> HashSet<u32> {
        use rand::prelude::*;
        use rand::rngs::SmallRng;
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut idx: Vec<u32> = (0..n as u32).collect();
        idx.shuffle(&mut rng);
        idx.into_iter().take(keep).collect()
    };

    let label_match = || -> HashSet<u32> {
        // Baseline: keep issues sharing any label with `duplicate` / `not_planned`
        // This is a "cheat" baseline using ground truth structure — we keep it to show
        // how hard it is even WITH partial labeling hint.
        let mut hot_labels: HashSet<String> = HashSet::new();
        for &i in &rare_gt {
            for l in &pure_issues[i as usize].labels {
                hot_labels.insert(l.name.clone());
            }
        }
        let mut out: HashSet<u32> = HashSet::new();
        for (i, iss) in pure_issues.iter().enumerate() {
            if iss.labels.iter().any(|l| hot_labels.contains(&l.name)) {
                out.insert(i as u32);
                if out.len() >= keep {
                    break;
                }
            }
        }
        out
    };

    let kdf_select = || -> HashSet<u32> {
        let mut c = NodeClassifier::default();
        let class = c.classify(n, &edges);
        let score = |l: Layer| -> i32 {
            match l {
                Layer::Rare => 3,
                Layer::Core => 2,
                Layer::Edge => 1,
                Layer::Garbage => 0,
            }
        };
        let mut scored: Vec<(u32, i32)> = (0..n as u32)
            .map(|id| {
                (
                    id,
                    score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
                )
            })
            .collect();
        scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));
        scored.into_iter().take(keep).map(|(i, _)| i).collect()
    };

    // Evaluate
    let recall = |sel: &HashSet<u32>| -> f64 {
        if rare_gt.is_empty() {
            return 0.0;
        }
        sel.intersection(&rare_gt).count() as f64 / rare_gt.len() as f64
    };
    let precision = |sel: &HashSet<u32>| -> f64 {
        if sel.is_empty() {
            return 0.0;
        }
        sel.intersection(&rare_gt).count() as f64 / sel.len() as f64
    };

    let seeds: Vec<u64> = (0..10).map(|i| 20000 + i).collect();
    println!("\n| Method | recall | precision | time |");
    println!("|---|---:|---:|---:|");

    // Random avg over seeds
    let mut r_vals = Vec::new();
    let mut p_vals = Vec::new();
    for &s in &seeds {
        let sel = random_select(s);
        r_vals.push(recall(&sel));
        p_vals.push(precision(&sel));
    }
    let r_mean: f64 = r_vals.iter().sum::<f64>() / r_vals.len() as f64;
    let p_mean: f64 = p_vals.iter().sum::<f64>() / p_vals.len() as f64;
    println!("| Random | {:.3} | {:.3} | ~0ms |", r_mean, p_mean);

    // Label match (deterministic)
    let sel_lm = label_match();
    println!(
        "| LabelMatch | {:.3} | {:.3} | ~0ms |",
        recall(&sel_lm),
        precision(&sel_lm)
    );

    // KDF (deterministic)
    let t0 = std::time::Instant::now();
    let sel_kdf = kdf_select();
    let ms = t0.elapsed().as_secs_f64() * 1000.0;
    println!(
        "| KDF | {:.3} | {:.3} | {:.1}ms |",
        recall(&sel_kdf),
        precision(&sel_kdf),
        ms
    );

    println!("\n## Interpretation");
    println!(
        "- Rare ground truth = duplicate + not_planned state_reason ({}/{} = {:.1}%)",
        rare_gt.len(),
        n,
        100.0 * rare_gt.len() as f64 / n as f64
    );
    println!("- Random baseline ≈ 30% (should match selection budget)");
    println!("- LabelMatch uses ground-truth labels indirectly — upper bound");
    println!("- KDF: structure-only, no text read, no labels besides graph");
}
