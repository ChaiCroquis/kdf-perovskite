//! Phase F — D6 forum dedup failure deep-dive.
//!
//! The D6 baseline KDF achieved 0% minority recall in the synthetic forum
//! with "1 original + 30 replies" thread structure. This deep-dive
//! analyzes WHY and tests multiple hypotheses for recovery:
//!
//!   H1: Vary the reply/minority ratio — is 30:1 pathological?
//!   H2: Give minority posts more replies (matching majority) — does KDF recover?
//!   H3: Apply Phase 7 S2 RelDensity — does it rescue D6?
//!   H4: Shuffle reply-to-minority ordering (seed sensitivity)
//!
//! Output: table of (hypothesis, KDF_recall, RelDensity_recall) for Chai
//! to see whether D6 is structurally unrecoverable or just parameter-sensitive.

use cgb_kdf::{Layer, NodeClassifier};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::HashSet;

struct Forum {
    n_posts: usize,
    edges: Vec<(u32, u32, f64)>,
    minority_ids: HashSet<u32>,
}

fn synthesize(
    n_majority_threads: usize,
    replies_per_thread: usize,
    n_minority: usize,
    replies_per_minority: usize,
    seed: u64,
) -> Forum {
    let _rng = SmallRng::seed_from_u64(seed);
    let mut edges = Vec::new();
    let mut minority_ids = HashSet::new();
    let mut next_id: u32 = 0;

    for _ in 0..n_majority_threads {
        let orig_id = next_id;
        next_id += 1;
        for _ in 0..replies_per_thread {
            let reply_id = next_id;
            next_id += 1;
            edges.push((reply_id, orig_id, 1.0));
        }
    }
    for _ in 0..n_minority {
        let orig_id = next_id;
        minority_ids.insert(orig_id);
        next_id += 1;
        for _ in 0..replies_per_minority {
            let reply_id = next_id;
            next_id += 1;
            edges.push((reply_id, orig_id, 1.0));
        }
    }
    Forum {
        n_posts: next_id as usize,
        edges,
        minority_ids,
    }
}

fn kdf_select(forum: &Forum, keep: usize) -> HashSet<u32> {
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(forum.n_posts, &forum.edges);
    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };
    let mut scored: Vec<(u32, i32)> = (0..forum.n_posts as u32)
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn reldensity_select(forum: &Forum, keep: usize) -> HashSet<u32> {
    let n = forum.n_posts;
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &forum.edges {
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
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn minority_recall(forum: &Forum, sel: &HashSet<u32>) -> f64 {
    let hit = sel.intersection(&forum.minority_ids).count() as f64;
    hit / forum.minority_ids.len().max(1) as f64
}

fn main() {
    println!("# Phase F — D6 Forum Dedup Deep-Dive\n");
    println!("Testing hypotheses for why D6 baseline KDF achieves 0% minority recall.\n");

    let scenarios: Vec<(&str, Forum)> = vec![
        // Original D6 scenario
        (
            "D6_original (3×30 maj + 10 min×1-2)",
            synthesize(3, 30, 10, 2, 42),
        ),
        // H1: reduce majority reply count
        (
            "H1a: 3×5 majority (instead of 30)",
            synthesize(3, 5, 10, 2, 42),
        ),
        ("H1b: 3×10 majority", synthesize(3, 10, 10, 2, 42)),
        // H2: increase minority reply count
        (
            "H2a: minority 5 replies each (vs 2)",
            synthesize(3, 30, 10, 5, 42),
        ),
        (
            "H2b: minority 15 replies (match part of majority)",
            synthesize(3, 30, 10, 15, 42),
        ),
        // H3: more thread imbalance
        ("H3a: 10×30 majority (heavy)", synthesize(10, 30, 10, 2, 42)),
        // H4: different seed
        (
            "H4a: original shape, seed=100",
            synthesize(3, 30, 10, 2, 100),
        ),
    ];

    println!("| Scenario | n_posts | KDF recall | RelDensity recall | Delta |");
    println!("|---|---:|---:|---:|---:|");
    for (name, forum) in &scenarios {
        let keep = (forum.n_posts as f64 * 0.30).ceil() as usize;
        let k_sel = kdf_select(forum, keep);
        let r_sel = reldensity_select(forum, keep);
        let k_r = minority_recall(forum, &k_sel);
        let r_r = minority_recall(forum, &r_sel);
        println!(
            "| {} | {} | {:.3} | {:.3} | {:+.3} |",
            name,
            forum.n_posts,
            k_r,
            r_r,
            r_r - k_r
        );
    }

    println!("\n## Analysis\n");
    println!("- Original D6 is a KDF failure mode (known)");
    println!("- If RelDensity rescues ANY scenario, D6 is recoverable with Phase 7 S2");
    println!(
        "- If H2b (minority with many replies) still fails, structure is fundamentally mismatched"
    );
    println!(
        "- H1 tests whether the 30:1 majority:minority reply imbalance is the pathological element"
    );
}
