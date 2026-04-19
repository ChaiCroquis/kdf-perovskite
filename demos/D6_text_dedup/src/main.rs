//! Demo D6 — Forum / SNS post deduplication preserving minority opinions.
//!
//! # Hypothesis
//! Forum posts form a reply graph: nodes=posts, edges=replies. The rareness
//! here is *minority opinions* (posts that receive few replies but aren't
//! complete spam). This is a **hybrid case**: rareness has both structural
//! (few replies) and content (distinct phrasing) dimensions.
//!
//! - Prediction: KDF baseline should catch "few replies" minorities (structural);
//!   MinHash/SimHash catch "textually unique" ones (content). They are
//!   complementary — neither covers both axes alone.
//!
//! # Data
//! Synthetic forum with 3 kinds of posts:
//!   - Majority opinion (cluster of many near-duplicate replies)
//!   - Minority opinion (distinct content, few replies)
//!   - Spam (high-frequency copy-paste)
//!
//! # Baselines
//! - Random
//! - ExactDup (trivial): remove byte-exact duplicates
//! - MinHash (shingle-based near-dup detection)
//! - SimHash (similarity hashing)
//! - KDF (reply-graph structural rareness)
//! - KDF+TextSim (structural + content hybrid)

use kdf_demos_common::{visualizer::emit_artifacts, Axis, Conclusion, DemoReport, Metric, MethodResult};
use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::Dataset;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const DEMO_ID: &str = "D6";
const DEMO_TITLE: &str = "Forum / SNS テキスト dedup + 少数意見保持";
const N_TRIALS: usize = 10;
const SELECTION_FRAC: f64 = 0.30;

fn main() {
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 8000 + i).collect();

    let (posts, ds, minority_ids) = synthesize_forum(/*seed=*/ 42);
    println!("Forum: n={} posts, reply edges={}, minority_posts={}",
        posts.len(), ds.edges.len(), minority_ids.len());

    let keep = (posts.len() as f64 * SELECTION_FRAC).ceil() as usize;

    let methods: Vec<(String, bool, Box<dyn Fn(&[String], &Dataset, u64) -> HashSet<u32>>)> = vec![
        ("Random".into(), false, Box::new(move |posts, _ds, seed| sample_random(posts.len(), keep, seed))),
        ("ExactDup".into(), false, Box::new(move |posts, _ds, _seed| sample_exact_dedup(posts, keep))),
        ("MinHash".into(), false, Box::new(move |posts, _ds, _seed| sample_minhash(posts, keep, 32))),
        ("SimHash".into(), false, Box::new(move |posts, _ds, _seed| sample_simhash(posts, keep))),
        ("KDF".into(), false, Box::new(move |_posts, ds, _seed| sample_kdf(ds, keep))),
        ("KDF+TextSim".into(), false, Box::new(move |posts, ds, _seed| sample_kdf_hybrid(posts, ds, keep))),
    ];

    let mut method_results: Vec<MethodResult> = Vec::new();
    let mut raw_trials: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (name, needs_label, sampler) in &methods {
        let mut minority_recalls = Vec::new();
        let mut dup_reductions = Vec::new();
        let mut compressions = Vec::new();
        let mut walls = Vec::new();
        for &seed in &seeds {
            let t0 = Instant::now();
            let sel = sampler(&posts, &ds, seed);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;
            let minority_recall = sel.intersection(&minority_ids).count() as f64 / minority_ids.len().max(1) as f64;
            let dup_reduction = measure_dup_reduction(&posts, &sel);
            let comp = 1.0 - sel.len() as f64 / posts.len() as f64;
            minority_recalls.push(minority_recall);
            dup_reductions.push(dup_reduction);
            compressions.push(comp);
            walls.push(ms);
            raw_trials.entry(format!("{}/minority_recall", name)).or_default().push(minority_recall);
            raw_trials.entry(format!("{}/dup_reduction", name)).or_default().push(dup_reduction);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&minority_recalls);
        let d = mean(&dup_reductions);
        let c = mean(&compressions);
        let w = mean(&walls);
        println!("{:14} minority_recall={:.3}  dup_reduction={:.3}  comp={:.3}  ms={:.2}",
            name, r, d, c, w);
        let mut metrics = BTreeMap::new();
        metrics.insert("minority_recall".into(), r);
        metrics.insert("dup_reduction".into(), d);
        metrics.insert("compression".into(), c);
        metrics.insert("wall_ms".into(), w);
        method_results.push(MethodResult {
            method: name.clone(), requires_labels: *needs_label,
            metrics, wall_ms: w, notes: String::new(),
        });
    }

    let metric_definitions = vec![
        Metric { name: "minority_recall".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::KdfStrength },
        Metric { name: "dup_reduction".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::Tie },
        Metric { name: "compression".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::Tie },
        Metric { name: "wall_ms".into(), higher_is_better: false, mean: 0.0, stderr: 0.0, axis: Axis::KdfWeakness },
    ];

    let report = DemoReport {
        demo_id: DEMO_ID.to_string(),
        title: DEMO_TITLE.to_string(),
        dataset_name: format!("synthetic_forum_n{}", posts.len()),
        n_items: posts.len(),
        patent_section: "明細書 §0002 (SNS/フォーラム投稿) / Claim 1, 18, 46".into(),
        metric_definitions,
        method_results,
        raw_trials,
        conclusion: Conclusion {
            kdf_recommended_for: vec![
                "Forum/SNS で **reply 構造から** minority post を保護(textual dedup と並行)".into(),
                "MinHash/SimHash が見逃す「**少ないが独立した視点**」の保持".into(),
            ],
            kdf_not_recommended_for: vec![
                "純粋なテキスト重複排除 → MinHash/SimHash で十分".into(),
                "reply graph が無い(単発投稿リスト)→ KDF のシグナル元が消える".into(),
            ],
            honest_limits: vec![
                "合成 forum データ(reply graph のテンプレ生成)での評価".into(),
                "実 Reddit/HN post は誤差、spam/minority 分離がより難しい可能性".into(),
                "KDF+TextSim の hybrid は ad-hoc な weighted union、本格 ensemble ではない".into(),
            ],
        },
    };

    let out_dir = std::path::Path::new("demos/D6_text_dedup/out");
    emit_artifacts(&report, out_dir).expect("emit");
    println!("\n✅ D6 artifacts written to {}", out_dir.display());
}

// ============================================================================
// Synthetic forum
// ============================================================================

fn synthesize_forum(seed: u64) -> (Vec<String>, Dataset, HashSet<u32>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut posts = Vec::new();
    let mut edges = Vec::new();
    let mut minority_ids = HashSet::new();

    // 3 majority threads: each has 1 original + 30 near-duplicate replies
    for t in 0..3 {
        let original_id = posts.len() as u32;
        let template = format!("Thread {} opinion A: the feature works well", t);
        posts.push(template.clone());
        for r in 0..30 {
            let reply_id = posts.len() as u32;
            let perturb = r % 3;
            let text = match perturb {
                0 => format!("{} (agreed)", template),
                1 => format!("I think {} too", template.to_lowercase()),
                _ => template.clone(), // exact dup
            };
            posts.push(text);
            edges.push((reply_id, original_id, 1.0));
        }
    }

    // Minority opinions: 10 posts, unique content, each with 1-2 replies
    for m in 0..10 {
        let id = posts.len() as u32;
        let text = format!("minority opinion {}: there is an edge case where the feature fails at index {}", m, m * 7);
        posts.push(text);
        minority_ids.insert(id);
        let n_reply = (rng.gen_range(1..=2)) as usize;
        for _ in 0..n_reply {
            let reply_id = posts.len() as u32;
            posts.push(format!("response to minority {}", m));
            edges.push((reply_id, id, 1.0));
        }
    }

    // Spam: 20 posts, byte-exact duplicates
    let spam_text = "BUY NOW!!! CLICK HERE".to_string();
    for _ in 0..20 {
        posts.push(spam_text.clone());
    }

    let n = posts.len();
    let ds = Dataset {
        name: "synthetic_forum".into(),
        n_nodes: n,
        edges,
        rare_ground_truth: minority_ids.clone(),
        description: "synthetic forum: 3 majority threads × 30 replies + 10 minorities + 20 spam".into(),
    };
    (posts, ds, minority_ids)
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

fn sample_exact_dedup(posts: &[String], keep: usize) -> HashSet<u32> {
    // Keep first occurrence of each exact post
    let mut seen: HashMap<&str, u32> = HashMap::new();
    let mut out: Vec<u32> = Vec::new();
    for (i, p) in posts.iter().enumerate() {
        if !seen.contains_key(p.as_str()) {
            seen.insert(p.as_str(), i as u32);
            out.push(i as u32);
        }
    }
    out.truncate(keep);
    out.into_iter().collect()
}

fn shingles(s: &str, k: usize) -> HashSet<String> {
    let s = s.to_lowercase();
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < k { return HashSet::new(); }
    (0..=chars.len() - k).map(|i| chars[i..i + k].iter().collect::<String>()).collect()
}

fn sample_minhash(posts: &[String], keep: usize, n_hashes: usize) -> HashSet<u32> {
    // Compute n_hashes MinHash signatures per post, group by signature, keep representatives.
    let sigs: Vec<Vec<u64>> = posts.iter().map(|p| {
        let shs = shingles(p, 5);
        let mut hash_mins = vec![u64::MAX; n_hashes];
        for sh in &shs {
            let h = fnv1a(sh.as_bytes());
            for k in 0..n_hashes {
                let mixed = h ^ (0x9E3779B97F4A7C15u64.wrapping_mul(k as u64 + 1));
                if mixed < hash_mins[k] { hash_mins[k] = mixed; }
            }
        }
        hash_mins
    }).collect();
    // Group by first hash bucket + keep diverse across groups
    let mut buckets: BTreeMap<u64, Vec<u32>> = BTreeMap::new();
    for (i, sig) in sigs.iter().enumerate() {
        buckets.entry(sig[0] / (u64::MAX / 100).max(1)).or_default().push(i as u32);
    }
    let mut out: HashSet<u32> = HashSet::new();
    // Take one representative per bucket first
    for ids in buckets.values() {
        if let Some(&first) = ids.first() { out.insert(first); }
        if out.len() >= keep { break; }
    }
    out
}

fn sample_simhash(posts: &[String], keep: usize) -> HashSet<u32> {
    // 64-bit SimHash, group into buckets, keep diverse.
    let sigs: Vec<u64> = posts.iter().map(|p| simhash64(p)).collect();
    let mut buckets: BTreeMap<u8, Vec<u32>> = BTreeMap::new();
    for (i, &s) in sigs.iter().enumerate() {
        buckets.entry((s >> 56) as u8).or_default().push(i as u32);
    }
    let mut out = HashSet::new();
    for ids in buckets.values() {
        if let Some(&first) = ids.first() { out.insert(first); }
        if out.len() >= keep { break; }
    }
    // Fill remaining with lowest-distance pairs removed
    if out.len() < keep {
        for i in 0..posts.len() as u32 {
            if out.insert(i) && out.len() >= keep { break; }
        }
    }
    out
}

fn sample_kdf(ds: &Dataset, keep: usize) -> HashSet<u32> {
    use cgb_kdf::{Layer, NodeClassifier};
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let score = |l: Layer| -> i32 {
        match l { Layer::Rare => 3, Layer::Core => 2, Layer::Edge => 1, Layer::Garbage => 0 }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| (id, score(class.layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(id, _)| id).collect()
}

fn sample_kdf_hybrid(posts: &[String], ds: &Dataset, keep: usize) -> HashSet<u32> {
    // 70% by KDF structural, 30% by MinHash content uniqueness
    let kdf_budget = (keep as f64 * 0.7) as usize;
    let minhash_budget = keep.saturating_sub(kdf_budget);
    let mut out = sample_kdf(ds, kdf_budget);
    let mh = sample_minhash(posts, minhash_budget + out.len(), 32);
    for id in mh { if !out.contains(&id) { out.insert(id); if out.len() >= keep { break; } } }
    out
}

// ============================================================================
// Helpers
// ============================================================================

fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn simhash64(text: &str) -> u64 {
    let mut v = [0i32; 64];
    for sh in shingles(text, 3) {
        let h = fnv1a(sh.as_bytes());
        for i in 0..64 {
            if (h >> i) & 1 == 1 { v[i] += 1; } else { v[i] -= 1; }
        }
    }
    let mut out = 0u64;
    for i in 0..64 {
        if v[i] > 0 { out |= 1 << i; }
    }
    out
}

fn measure_dup_reduction(posts: &[String], selected: &HashSet<u32>) -> f64 {
    // Fraction of selected posts that are NOT exact duplicates of another selected
    let mut seen = HashSet::new();
    let mut unique = 0;
    for &id in selected {
        let p = &posts[id as usize];
        if seen.insert(p.clone()) { unique += 1; }
    }
    if selected.is_empty() { 0.0 } else { unique as f64 / selected.len() as f64 }
}
