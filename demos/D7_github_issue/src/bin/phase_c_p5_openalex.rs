//! Phase C-P5 — Late-bloomer paper detection on OpenAlex data.
//!
//! World problem: research paper rediscovery — papers whose citations jump
//! years after publication (e.g. 1982 Hopfield net → 2024 Nobel). Can KDF
//! preserve these "late bloomers" from a large corpus without using citation
//! time-series as a feature?
//!
//! Data: 200 papers 2000-2008, cite count 30-500 (mid-range), via OpenAlex API.
//! Ground truth: papers with ≥50% of their lifetime citations in 2020-2026
//! (late bloomer: recent surge).
//!
//! Graph: paper × concept bipartite (papers share a concept → edge).

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct Response { results: Vec<Paper> }

#[derive(Deserialize, Debug, Clone)]
#[allow(dead_code)]
struct Paper {
    #[serde(default)] id: String,
    #[serde(default)] title: Option<String>,
    #[serde(default)] publication_year: Option<i32>,
    #[serde(default)] cited_by_count: Option<i64>,
    #[serde(default)] counts_by_year: Vec<CountYear>,
    #[serde(default)] concepts: Vec<Concept>,
}

#[derive(Deserialize, Debug, Clone)]
struct CountYear { year: i32, cited_by_count: i64 }

#[derive(Deserialize, Debug, Clone)]
struct Concept { #[serde(default)] id: String, #[serde(default)] score: f64 }

fn main() {
    let path = "benchmarks/real_data/data/arxiv-cs/openalex_2000_2008.json";
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => {
            eprintln!("Missing {}. Fetch with:", path);
            eprintln!("  curl -sL 'https://api.openalex.org/works?filter=publication_year:2000-2008,cited_by_count:30-500&per-page=200&sort=cited_by_count:desc&select=id,title,publication_year,cited_by_count,counts_by_year,concepts' -o {}", path);
            std::process::exit(1);
        }
    };
    let resp: Response = serde_json::from_str(&text).expect("bad JSON");
    let papers = resp.results;
    let n = papers.len();
    println!("Loaded {} papers from OpenAlex (publication_year 2000-2008, cite 30-500)", n);

    // Ground truth: late bloomers = >=50% of lifetime citations in 2020+
    let mut rare_gt: HashSet<u32> = HashSet::new();
    for (i, p) in papers.iter().enumerate() {
        let total = p.cited_by_count.unwrap_or(0).max(1);
        let recent: i64 = p.counts_by_year.iter().filter(|c| c.year >= 2020).map(|c| c.cited_by_count).sum();
        let ratio = recent as f64 / total as f64;
        if ratio >= 0.5 {
            rare_gt.insert(i as u32);
        }
    }
    println!("Rare ground truth (late bloomers: ≥50% cites in 2020+): {} / {} = {:.1}%",
        rare_gt.len(), n, 100.0 * rare_gt.len() as f64 / n as f64);

    if rare_gt.is_empty() {
        eprintln!("No late bloomers found; experiment not meaningful on this sample");
        std::process::exit(2);
    }

    // Build graph: concept-sharing edges (score-weighted). Cap per concept to avoid quadratic.
    let mut concept_groups: HashMap<String, Vec<u32>> = HashMap::new();
    for (i, p) in papers.iter().enumerate() {
        for c in &p.concepts {
            if c.score >= 0.3 {  // top-relevance concepts only
                concept_groups.entry(c.id.clone()).or_default().push(i as u32);
            }
        }
    }
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for (_, idxs) in &concept_groups {
        for a in 0..idxs.len() {
            for b in (a + 1)..idxs.len().min(a + 4) {
                edges.push((idxs[a], idxs[b], 1.0));
            }
        }
    }
    println!("Edges: {} (concept-sharing)", edges.len());

    let keep = (n as f64 * 0.30).ceil() as usize;

    let recall = |sel: &HashSet<u32>| -> f64 {
        sel.intersection(&rare_gt).count() as f64 / rare_gt.len() as f64
    };

    // Random averaged over 10 seeds
    let mut r_rand = 0.0;
    for s in 0..10u64 {
        use rand::{rngs::SmallRng, SeedableRng, seq::SliceRandom};
        let mut rng = SmallRng::seed_from_u64(50000 + s);
        let mut idx: Vec<u32> = (0..n as u32).collect();
        idx.shuffle(&mut rng);
        let sel: HashSet<u32> = idx.into_iter().take(keep).collect();
        r_rand += recall(&sel);
    }
    r_rand /= 10.0;

    // TopCiteAll: keep the highest cited papers (conventional "find impactful papers" baseline).
    let mut by_cite: Vec<(u32, i64)> = papers.iter().enumerate()
        .map(|(i, p)| (i as u32, p.cited_by_count.unwrap_or(0))).collect();
    by_cite.sort_by(|a, b| b.1.cmp(&a.1));
    let sel_topcite: HashSet<u32> = by_cite.into_iter().take(keep).map(|(i, _)| i).collect();
    let r_topcite = recall(&sel_topcite);

    // KDF
    use cgb_kdf::{Layer, NodeClassifier};
    let mut c = NodeClassifier::default();
    let class = c.classify(n, &edges);
    let score = |l: Layer| -> i32 { match l { Layer::Rare => 3, Layer::Core => 2, Layer::Edge => 1, Layer::Garbage => 0 } };
    let mut scored: Vec<(u32, i32)> = (0..n as u32)
        .map(|id| (id, score(class.layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    let sel_kdf: HashSet<u32> = scored.into_iter().take(keep).map(|(i, _)| i).collect();
    let r_kdf = recall(&sel_kdf);

    println!("\n| Method | Recall (late-bloomer) |");
    println!("|---|---:|");
    println!("| Random (30%) | {:.3} |", r_rand);
    println!("| TopCite (30%) | {:.3} |", r_topcite);
    println!("| **KDF** | **{:.3}** |", r_kdf);
    println!("\nKDF/Random = ×{:.2}", r_kdf / r_rand.max(1e-9));

    println!("\n## Interpretation");
    println!("Late-bloomer = paper with ≥50% of lifetime cites in 2020+. Graph = concept-sharing.");
    println!("If KDF > Random and KDF > TopCite → structural (not popularity) signal predicts late rise.");
    println!("If KDF ≈ Random → late bloom is independent of concept-graph structure (D5 type, as predicted).");
}
