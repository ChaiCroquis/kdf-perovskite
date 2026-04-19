//! Phase δ-2 — KDF on multi-repo real GitHub issues (P6 広域検証).
//!
//! Runs the phase_delta_real_issues experiment across 3 repositories to check
//! whether the ×1.15 signal on rust-lang/rust generalizes.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug, Clone)]
struct Label { name: String }

#[derive(Deserialize, Debug)]
struct Issue {
    #[serde(default)] #[allow(dead_code)] number: u32,
    #[serde(default)] state_reason: Option<String>,
    #[serde(default)] labels: Vec<Label>,
    #[serde(default)] user: Option<User>,
    #[serde(default)] pull_request: Option<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
struct User { login: String }

fn load(dir: &str) -> Vec<Issue> {
    let mut all = Vec::new();
    for p in 1..=5 {
        let path = format!("benchmarks/real_data/data/{}/page{}.json", dir, p);
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(batch) = serde_json::from_str::<Vec<Issue>>(&text) {
                all.extend(batch);
            }
        }
    }
    all
}

struct Result_ { repo: &'static str, n: usize, rare: usize, r_kdf: f64, r_rand: f64, r_label: f64 }

fn run_one(repo: &'static str, dir: &str) -> Option<Result_> {
    let issues = load(dir);
    if issues.is_empty() { eprintln!("No data for {}", repo); return None; }
    let pure: Vec<&Issue> = issues.iter().filter(|i| i.pull_request.is_none()).collect();
    let n = pure.len();

    let mut label_groups: HashMap<String, Vec<u32>> = HashMap::new();
    let mut author_groups: HashMap<String, Vec<u32>> = HashMap::new();
    for (i, iss) in pure.iter().enumerate() {
        for l in &iss.labels { label_groups.entry(l.name.clone()).or_default().push(i as u32); }
        if let Some(u) = &iss.user { author_groups.entry(u.login.clone()).or_default().push(i as u32); }
    }
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for (_, idxs) in &label_groups {
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len().min(a + 5) { edges.push((idxs[a], idxs[b], 1.0)); }
        }
    }
    for (_, idxs) in &author_groups {
        if idxs.len() < 2 || idxs.len() > 20 { continue; }
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len().min(a + 3) { edges.push((idxs[a], idxs[b], 0.5)); }
        }
    }

    let mut rare_gt: HashSet<u32> = HashSet::new();
    for (i, iss) in pure.iter().enumerate() {
        if matches!(iss.state_reason.as_deref(), Some("duplicate") | Some("not_planned")) {
            rare_gt.insert(i as u32);
        }
    }
    if rare_gt.is_empty() {
        eprintln!("{}: no rare ground truth", repo); return None;
    }

    let keep = (n as f64 * 0.30).ceil() as usize;
    use cgb_kdf::{Layer, NodeClassifier};
    let recall = |sel: &HashSet<u32>| -> f64 {
        sel.intersection(&rare_gt).count() as f64 / rare_gt.len() as f64
    };

    // Random averaged over 10 seeds
    let mut r_rand = 0.0;
    for s in 0..10u64 {
        use rand::{rngs::SmallRng, SeedableRng, seq::SliceRandom};
        let mut rng = SmallRng::seed_from_u64(40000 + s);
        let mut idx: Vec<u32> = (0..n as u32).collect();
        idx.shuffle(&mut rng);
        let sel: HashSet<u32> = idx.into_iter().take(keep).collect();
        r_rand += recall(&sel);
    }
    r_rand /= 10.0;

    // LabelMatch
    let mut hot_labels: HashSet<String> = HashSet::new();
    for &i in &rare_gt {
        for l in &pure[i as usize].labels { hot_labels.insert(l.name.clone()); }
    }
    let mut lm_sel: HashSet<u32> = HashSet::new();
    for (i, iss) in pure.iter().enumerate() {
        if iss.labels.iter().any(|l| hot_labels.contains(&l.name)) {
            lm_sel.insert(i as u32);
            if lm_sel.len() >= keep { break; }
        }
    }
    let r_label = recall(&lm_sel);

    // KDF
    let mut c = NodeClassifier::default();
    let class = c.classify(n, &edges);
    let score = |l: Layer| -> i32 { match l { Layer::Rare => 3, Layer::Core => 2, Layer::Edge => 1, Layer::Garbage => 0 } };
    let mut scored: Vec<(u32, i32)> = (0..n as u32)
        .map(|id| (id, score(class.layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let sel_kdf: HashSet<u32> = scored.into_iter().take(keep).map(|(i, _)| i).collect();
    let r_kdf = recall(&sel_kdf);

    Some(Result_ { repo, n, rare: rare_gt.len(), r_kdf, r_rand, r_label })
}

fn main() {
    let targets = [
        ("rust-lang/rust", "rust-issues"),
        ("tokio-rs/tokio",  "tokio-issues"),
        ("golang/go",        "golang-issues"),
    ];
    let mut results = Vec::new();
    for (repo, dir) in targets {
        if let Some(r) = run_one(repo, dir) { results.push(r); }
    }

    println!("\n# P6 multi-repo OSS generalization\n");
    println!("| repo | n | rare | KDF | Random | LabelMatch | KDF/Random |");
    println!("|---|---:|---:|---:|---:|---:|---:|");
    for r in &results {
        let ratio = if r.r_rand > 0.0 { r.r_kdf / r.r_rand } else { 0.0 };
        println!("| {} | {} | {} | {:.3} | {:.3} | {:.3} | ×{:.2} |",
            r.repo, r.n, r.rare, r.r_kdf, r.r_rand, r.r_label, ratio);
    }
    if results.is_empty() { std::process::exit(1); }
    let mean_ratio: f64 = results.iter()
        .filter(|r| r.r_rand > 0.0)
        .map(|r| r.r_kdf / r.r_rand)
        .sum::<f64>() / results.len() as f64;
    println!("\nMean KDF/Random ratio across repos: ×{:.2}", mean_ratio);
    println!("\nInterpretation: a ratio > 1 means KDF surfaces rare-labeled issues faster than Random");
    println!("on a structure-only graph (shared labels + shared authors). No text read, no ground-truth labels.");
}
