//! Phase K — does TEXT content rescue D6?
//!
//! F-022 claimed D6's ground truth (minority = thread original) is
//! orthogonal to GRAPH signal. This implies TEXT content could help —
//! minority posts have unique shingle patterns (rare opinion), majorities
//! share shingles (reposts). We test 3 hypotheses:
//!
//!   K1: Pure TextSim (MinHash-like shingle uniqueness) — does it alone work?
//!   K2: Graph-only KDF — baseline confirmation
//!   K3: KDF ∪ TextSim hybrid — does union restore recall?
//!   K4: KDF ∩ TextSim (intersection) — does precision rise?
//!
//! If K1 or K3 achieves meaningful minority recall, F-022's claim is
//! **partially refuted**: D6 IS solvable, just not by graph-only methods.
//! If all fail, the claim stands.

use cgb_kdf::{Layer, NodeClassifier};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
struct Post {
    id: u32,
    text: String,
}

struct Forum {
    posts: Vec<Post>,
    edges: Vec<(u32, u32, f64)>,
    minority_ids: HashSet<u32>,
}

fn synthesize(seed: u64) -> Forum {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut posts: Vec<Post> = Vec::new();
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut minority_ids: HashSet<u32> = HashSet::new();
    let mut next_id: u32 = 0;
    let new_post = |text: String, posts: &mut Vec<Post>, next_id: &mut u32| -> u32 {
        let id = *next_id;
        posts.push(Post { id, text });
        *next_id += 1;
        id
    };

    // 3 majority threads: 30 near-duplicate replies each
    for t in 0..3 {
        let template = format!("Thread {} opinion A: the feature works well", t);
        let orig_id = new_post(template.clone(), &mut posts, &mut next_id);
        for r in 0..30 {
            let reply = match r % 3 {
                0 => format!("{} (agreed)", template),
                1 => format!("I think {} too", template.to_lowercase()),
                _ => template.clone(), // exact dup
            };
            let reply_id = new_post(reply, &mut posts, &mut next_id);
            edges.push((reply_id, orig_id, 1.0));
        }
    }

    // 10 minority: unique text patterns, 1-2 replies each
    for m in 0..10 {
        let text = format!(
            "minority opinion {}: there is an edge case at index {} with different implications",
            m,
            m * 7
        );
        let orig_id = new_post(text, &mut posts, &mut next_id);
        minority_ids.insert(orig_id);
        let n = rng.gen_range(1..=2);
        for _ in 0..n {
            let reply_id = new_post(
                format!("response to minority {}", m),
                &mut posts,
                &mut next_id,
            );
            edges.push((reply_id, orig_id, 1.0));
        }
    }

    // 20 spam (exact dup)
    let spam = "BUY NOW!!! CLICK HERE".to_string();
    for _ in 0..20 {
        new_post(spam.clone(), &mut posts, &mut next_id);
    }

    Forum {
        posts,
        edges,
        minority_ids,
    }
}

// ============================================================================
// TextSim: shingle-based uniqueness ranking
// ============================================================================

fn shingles(s: &str, k: usize) -> HashSet<String> {
    let s = s.to_lowercase();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < k {
        return HashSet::new();
    }
    (0..=chars.len() - k)
        .map(|i| chars[i..i + k].iter().collect::<String>())
        .collect()
}

/// Score each post by uniqueness: fewer global co-occurrences = higher score.
fn textsim_scores(posts: &[Post]) -> Vec<(u32, f64)> {
    let all_shingles: Vec<HashSet<String>> = posts.iter().map(|p| shingles(&p.text, 5)).collect();
    // Global shingle frequency
    let mut global: HashMap<String, u32> = HashMap::new();
    for shs in &all_shingles {
        for sh in shs {
            *global.entry(sh.clone()).or_insert(0) += 1;
        }
    }
    posts
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let shs = &all_shingles[i];
            if shs.is_empty() {
                return (p.id, 0.0);
            }
            // Average rarity of post's shingles
            let inv_freq_sum: f64 = shs
                .iter()
                .map(|sh| 1.0 / (*global.get(sh).unwrap_or(&1) as f64))
                .sum();
            (p.id, inv_freq_sum / shs.len() as f64)
        })
        .collect()
}

fn textsim_select(posts: &[Post], keep: usize) -> HashSet<u32> {
    let mut scored = textsim_scores(posts);
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(keep).map(|(id, _)| id).collect()
}

// ============================================================================
// KDF graph-only
// ============================================================================

fn kdf_select(forum: &Forum, keep: usize) -> HashSet<u32> {
    let n = forum.posts.len();
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(n, &forum.edges);
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
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(id, _)| id).collect()
}

// ============================================================================
// Hybrids
// ============================================================================

fn kdf_union_textsim(forum: &Forum, keep: usize) -> HashSet<u32> {
    let half = keep / 2;
    let mut out = kdf_select(forum, half);
    let text_picks = textsim_select(&forum.posts, keep);
    for id in text_picks {
        if !out.contains(&id) {
            out.insert(id);
            if out.len() >= keep {
                break;
            }
        }
    }
    out
}

fn kdf_intersect_textsim(forum: &Forum, keep: usize) -> HashSet<u32> {
    // Keep items that rank high in BOTH KDF and TextSim.
    // Take 2*keep in each, intersect, then fallback to union if short.
    let budget = keep * 2;
    let kdf_picks = kdf_select(forum, budget);
    let text_picks = textsim_select(&forum.posts, budget);
    let intersect: HashSet<u32> = kdf_picks.intersection(&text_picks).copied().collect();
    if intersect.len() >= keep {
        // Take arbitrary `keep` from intersection
        let mut v: Vec<u32> = intersect.into_iter().collect();
        v.sort();
        v.into_iter().take(keep).collect()
    } else {
        // Pad with union
        let mut out = intersect;
        for id in kdf_picks {
            if !out.contains(&id) {
                out.insert(id);
                if out.len() >= keep {
                    break;
                }
            }
        }
        for id in text_picks {
            if !out.contains(&id) {
                out.insert(id);
                if out.len() >= keep {
                    break;
                }
            }
        }
        out
    }
}

fn minority_recall(forum: &Forum, sel: &HashSet<u32>) -> f64 {
    let hit = sel.intersection(&forum.minority_ids).count() as f64;
    hit / forum.minority_ids.len().max(1) as f64
}

fn precision_at_minority(forum: &Forum, sel: &HashSet<u32>) -> f64 {
    if sel.is_empty() {
        return 0.0;
    }
    let hit = sel.intersection(&forum.minority_ids).count() as f64;
    hit / sel.len() as f64
}

fn main() {
    println!("# Phase K — D6 Text-Hybrid Rescue Attempt\n");
    println!("F-022 claimed D6 ground truth is orthogonal to graph signal.");
    println!("Testing whether TEXT content can rescue it (alone or with KDF).\n");

    let seeds: Vec<u64> = (0..5).map(|i| 42 + i * 100).collect();
    let methods: Vec<(&str, Box<dyn Fn(&Forum, usize) -> HashSet<u32>>)> = vec![
        ("K2_KDF", Box::new(kdf_select)),
        (
            "K1_TextSim",
            Box::new(|forum, keep| textsim_select(&forum.posts, keep)),
        ),
        ("K3_KDF∪TextSim", Box::new(kdf_union_textsim)),
        ("K4_KDF∩TextSim", Box::new(kdf_intersect_textsim)),
    ];

    println!("| Method | Recall mean ± SE | Precision mean ± SE |");
    println!("|---|---:|---:|");
    for (name, sampler) in &methods {
        let mut recalls = Vec::new();
        let mut precisions = Vec::new();
        for &seed in &seeds {
            let forum = synthesize(seed);
            let keep = (forum.posts.len() as f64 * 0.30).ceil() as usize;
            let sel = sampler(&forum, keep);
            recalls.push(minority_recall(&forum, &sel));
            precisions.push(precision_at_minority(&forum, &sel));
        }
        let rm = recalls.iter().sum::<f64>() / recalls.len() as f64;
        let rsem = ((recalls.iter().map(|x| (x - rm).powi(2)).sum::<f64>() / recalls.len() as f64)
            / recalls.len() as f64)
            .sqrt();
        let pm = precisions.iter().sum::<f64>() / precisions.len() as f64;
        let psem = ((precisions.iter().map(|x| (x - pm).powi(2)).sum::<f64>()
            / precisions.len() as f64)
            / precisions.len() as f64)
            .sqrt();
        println!(
            "| {} | {:.3} ± {:.3} | {:.3} ± {:.3} |",
            name, rm, rsem, pm, psem
        );
    }

    println!("\n## Interpretation");
    println!("- If K1 (TextSim alone) achieves > 0, then D6 IS solvable via text.");
    println!("- If K3 (union) achieves > K2 (KDF), hybrid recovers what graph loses.");
    println!("- F-022's claim was: 'graph-only cannot'. Text-rescue would REFUTE");
    println!("  F-022's implicit upper bound but PRESERVE the core finding that");
    println!("  KDF alone can't.");
}
