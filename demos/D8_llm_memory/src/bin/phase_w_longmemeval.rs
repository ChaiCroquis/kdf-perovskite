//! Phase W — LongMemEval (real LLM benchmark) evaluation for D8.
//!
//! Uses the **Oracle** subset of LongMemEval (500 questions, 15MB), loaded
//! from `data/longmemeval_oracle.json`. For each question, we have:
//!   - haystack_sessions: list of conversation sessions (most irrelevant)
//!   - answer_session_ids: ground-truth IDs of sessions containing the answer
//!
//! Task: select a fraction of turns from the entire haystack that preserves
//! answer-bearing content. Measure:
//!   - answer_session_recall: % of answer sessions with ≥1 turn selected
//!   - answer_turn_recall: % of has_answer=true turns selected
//!
//! This is the realistic LLM memory curation problem KDF+TextSim claims to solve.

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Turn {
    role: String,
    content: String,
    has_answer: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
struct Question {
    question_id: String,
    question: String,
    // answer may be string or int (depends on question type); we don't use it
    #[serde(default)]
    answer: serde_json::Value,
    haystack_session_ids: Vec<String>,
    haystack_sessions: Vec<Vec<Turn>>,
    answer_session_ids: Vec<String>,
}

fn shingles(s: &str, k: usize) -> HashSet<String> {
    let chars: Vec<char> = s.to_lowercase().chars().collect();
    if chars.len() < k {
        return HashSet::new();
    }
    (0..=chars.len() - k)
        .map(|i| chars[i..i + k].iter().collect::<String>())
        .collect()
}

/// Build a flat turn list + graph: nodes = turns, edges = same-session co-occurrence
fn build_graph(
    q: &Question,
) -> (
    Vec<(String, String, usize)>,
    Vec<(u32, u32, f64)>,
    HashSet<u32>,
) {
    // (session_id, content, turn_global_idx)
    let mut flat: Vec<(String, String, usize)> = Vec::new();
    let mut answer_turn_ids: HashSet<u32> = HashSet::new();
    for (i, session) in q.haystack_sessions.iter().enumerate() {
        let sid = q.haystack_session_ids.get(i).cloned().unwrap_or_default();
        let is_answer_sess = q.answer_session_ids.contains(&sid);
        for turn in session {
            let global_idx = flat.len();
            if is_answer_sess && turn.has_answer.unwrap_or(false) {
                answer_turn_ids.insert(global_idx as u32);
            }
            flat.push((sid.clone(), turn.content.clone(), global_idx));
        }
    }
    // Edges: same-session adjacency (chain)
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for i in 0..flat.len() - 1 {
        if flat[i].0 == flat[i + 1].0 {
            edges.push((i as u32, (i + 1) as u32, 1.0));
        }
    }
    (flat, edges, answer_turn_ids)
}

fn text_rareness_scores(flat: &[(String, String, usize)]) -> Vec<f64> {
    let all_sh: Vec<HashSet<String>> = flat.iter().map(|(_, c, _)| shingles(c, 5)).collect();
    let mut freq: HashMap<String, u32> = HashMap::new();
    for shs in &all_sh {
        for sh in shs {
            *freq.entry(sh.clone()).or_insert(0) += 1;
        }
    }
    all_sh
        .iter()
        .map(|shs| {
            if shs.is_empty() {
                return 0.0;
            }
            let inv: f64 = shs
                .iter()
                .map(|sh| 1.0 / *freq.get(sh).unwrap_or(&1) as f64)
                .sum();
            (inv / shs.len() as f64).min(1.0)
        })
        .collect()
}

fn kdf_select(n: usize, edges: &[(u32, u32, f64)], keep: usize) -> HashSet<u32> {
    use cgb_kdf::{Layer, NodeClassifier};
    let mut c = NodeClassifier::default();
    let class = c.classify(n, edges);
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
}

fn kdf_textsim_select(
    n: usize,
    edges: &[(u32, u32, f64)],
    text_rare: &[f64],
    keep: usize,
) -> HashSet<u32> {
    use cgb_kdf::framework::multimodal::{select_top_k_multi_modal, MultiModalWeights};
    use cgb_kdf::{Layer, NodeClassifier};
    let mut c = NodeClassifier::default();
    let class = c.classify(n, edges);
    let layer_of: HashMap<u32, Layer> = class.layers;
    select_top_k_multi_modal(
        n,
        &layer_of,
        Some(text_rare),
        None,
        keep,
        &MultiModalWeights::balanced(),
    )
}

fn ttl_select(n: usize, keep: usize) -> HashSet<u32> {
    (n.saturating_sub(keep) as u32..n as u32).collect()
}

fn random_select(n: usize, keep: usize, seed: u64) -> HashSet<u32> {
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut idx: Vec<u32> = (0..n as u32).collect();
    idx.shuffle(&mut rng);
    idx.into_iter().take(keep).collect()
}

fn main() {
    let path = "demos/D8_llm_memory/data/longmemeval_oracle.json";
    println!("# Phase W — LongMemEval (real LLM benchmark) D8 Evaluation\n");

    let data = std::fs::read_to_string(path).expect("Load LongMemEval oracle");
    let questions: Vec<Question> = serde_json::from_str(&data).expect("Parse JSON");
    println!(
        "Loaded {} questions from LongMemEval oracle subset\n",
        questions.len()
    );

    // Sample to bound runtime. By default sample the first 100, but allow
    // --random-sample flag to pick a random subset (checks cherry-pick bias).
    let sample_size = 100.min(questions.len());
    let use_random = std::env::args().any(|a| a == "--random-sample");
    let sample: Vec<&Question> = if use_random {
        use rand::prelude::*;
        use rand::rngs::SmallRng;
        let mut rng = SmallRng::seed_from_u64(12345);
        let mut idxs: Vec<usize> = (0..questions.len()).collect();
        idxs.shuffle(&mut rng);
        idxs.into_iter()
            .take(sample_size)
            .map(|i| &questions[i])
            .collect()
    } else {
        questions.iter().take(sample_size).collect()
    };
    println!(
        "Sampling strategy: {}",
        if use_random {
            "random"
        } else {
            "first 100 (deterministic)"
        }
    );

    println!(
        "Evaluating {} questions, selection rate = 30%\n",
        sample_size
    );
    println!("| Method | answer_turn_recall | compression | wall_ms/q |");
    println!("|---|---:|---:|---:|");

    let methods: Vec<&str> = vec!["Random", "TTL_recent", "KDF", "KDF+TextSim"];
    for method in &methods {
        let mut recalls = Vec::new();
        let mut compressions = Vec::new();
        let mut walls = Vec::new();
        for (i, q) in sample.iter().enumerate() {
            let (flat, edges, answer_turns) = build_graph(q);
            let n = flat.len();
            let keep = (n as f64 * 0.30).ceil() as usize;
            let text_rare = text_rareness_scores(&flat);

            let t0 = std::time::Instant::now();
            let sel = match *method {
                "Random" => random_select(n, keep, (i as u64) * 7 + 42),
                "TTL_recent" => ttl_select(n, keep),
                "KDF" => kdf_select(n, &edges, keep),
                "KDF+TextSim" => kdf_textsim_select(n, &edges, &text_rare, keep),
                _ => HashSet::new(),
            };
            let ms = t0.elapsed().as_secs_f64() * 1000.0;

            let hit = sel.intersection(&answer_turns).count() as f64;
            let recall = if answer_turns.is_empty() {
                1.0
            } else {
                hit / answer_turns.len() as f64
            };
            let comp = 1.0 - sel.len() as f64 / n.max(1) as f64;
            recalls.push(recall);
            compressions.push(comp);
            walls.push(ms);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&recalls);
        let c = mean(&compressions);
        let w = mean(&walls);
        println!("| {} | {:.3} | {:.3} | {:.2} |", method, r, c, w);
    }

    println!("\n## Interpretation");
    println!("- Answer turns are the subset of turns containing the answer (has_answer=true)");
    println!("- Random baseline: expected ≈ selection rate = 0.30");
    println!("- KDF baseline graph-only: depends on conversation structure");
    println!("- KDF+TextSim: claimed to be superior (F-028 synthetic, 1.000)");
    println!("- **This is the REAL benchmark check for F-028**");
}
