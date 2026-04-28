//! W3 — Keep-rate ablation for KDF retrieval on LongMemEval 500Q.
//!
//! Sweeps keep_rate over {0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.40, 0.50, 0.70, 1.00}
//! and reports answer_turn_recall for:
//!   - Random (lower bound, = keep_rate in expectation)
//!   - TTL_recent (baseline: last-k turns)
//!   - KDF (structural-only)
//!   - KDF+TextSim (full KDF with text rareness)
//!
//! Also reports answer_session_recall (coarser metric: at least 1 turn from each
//! answer-bearing session).
//!
//! Writes CSV to demos/D8_llm_memory/out/w3_keep_rate_ablation.csv

use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct Turn {
    #[allow(dead_code)]
    role: String,
    content: String,
    has_answer: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Question {
    #[allow(dead_code)]
    question_id: String,
    #[allow(dead_code)]
    question: String,
    #[allow(dead_code)]
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

fn build_graph(
    q: &Question,
) -> (
    Vec<(String, String, usize)>,
    Vec<(u32, u32, f64)>,
    HashSet<u32>,
    HashSet<String>,
) {
    let mut flat: Vec<(String, String, usize)> = Vec::new();
    let mut answer_turn_ids: HashSet<u32> = HashSet::new();
    let mut answer_sessions_present: HashSet<String> = HashSet::new();
    for (i, session) in q.haystack_sessions.iter().enumerate() {
        let sid = q.haystack_session_ids.get(i).cloned().unwrap_or_default();
        let is_answer_sess = q.answer_session_ids.contains(&sid);
        if is_answer_sess {
            answer_sessions_present.insert(sid.clone());
        }
        for turn in session {
            let global_idx = flat.len();
            if is_answer_sess && turn.has_answer.unwrap_or(false) {
                answer_turn_ids.insert(global_idx as u32);
            }
            flat.push((sid.clone(), turn.content.clone(), global_idx));
        }
    }
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    if flat.len() >= 2 {
        for i in 0..flat.len() - 1 {
            if flat[i].0 == flat[i + 1].0 {
                edges.push((i as u32, (i + 1) as u32, 1.0));
            }
        }
    }
    (flat, edges, answer_turn_ids, answer_sessions_present)
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
    use cgb_kdf::framework::multimodal::{MultiModalWeights, select_top_k_multi_modal};
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

fn session_recall(
    sel: &HashSet<u32>,
    flat: &[(String, String, usize)],
    answer_sessions: &HashSet<String>,
) -> f64 {
    if answer_sessions.is_empty() {
        return 1.0;
    }
    let mut sessions_hit: HashSet<String> = HashSet::new();
    for &i in sel {
        if let Some(row) = flat.get(i as usize)
            && answer_sessions.contains(&row.0)
        {
            sessions_hit.insert(row.0.clone());
        }
    }
    sessions_hit.len() as f64 / answer_sessions.len() as f64
}

fn main() {
    let path = "demos/D8_llm_memory/data/longmemeval_oracle.json";
    println!("# W3 — Keep-rate ablation on LongMemEval 500Q\n");

    let data = std::fs::read_to_string(path).expect("Load LongMemEval oracle");
    let questions: Vec<Question> = serde_json::from_str(&data).expect("Parse JSON");
    println!("Loaded {} questions\n", questions.len());

    let rates: Vec<f64> = vec![0.05, 0.10, 0.15, 0.20, 0.25, 0.30, 0.40, 0.50, 0.70, 1.00];
    let methods: Vec<&str> = vec!["Random", "TTL_recent", "KDF", "KDF+TextSim"];

    // Pre-compute per-question graph and text_rare once (reuse across rates)
    struct PrecomputedQ {
        n: usize,
        edges: Vec<(u32, u32, f64)>,
        answer_turns: HashSet<u32>,
        answer_sessions: HashSet<String>,
        flat: Vec<(String, String, usize)>,
        text_rare: Vec<f64>,
    }
    let pre: Vec<PrecomputedQ> = questions
        .iter()
        .map(|q| {
            let (flat, edges, answer_turns, answer_sessions) = build_graph(q);
            let text_rare = text_rareness_scores(&flat);
            PrecomputedQ {
                n: flat.len(),
                edges,
                answer_turns,
                answer_sessions,
                flat,
                text_rare,
            }
        })
        .collect();
    println!("Pre-computed graphs for {} questions\n", pre.len());

    // Table header
    println!(
        "| method | keep_rate | answer_turn_recall | answer_session_recall | compression | ms/q |"
    );
    println!("|---|---:|---:|---:|---:|---:|");

    let mut csv =
        String::from("method,keep_rate,turn_recall,session_recall,compression,ms_per_q\n");

    for &rate in &rates {
        for method in &methods {
            let mut recalls = Vec::new();
            let mut session_recalls = Vec::new();
            let mut compressions = Vec::new();
            let mut walls = Vec::new();
            for (i, q) in pre.iter().enumerate() {
                if q.n == 0 {
                    continue;
                }
                let keep = ((q.n as f64) * rate).ceil() as usize;
                let keep = keep.max(1).min(q.n);

                let t0 = std::time::Instant::now();
                let sel = match *method {
                    "Random" => random_select(q.n, keep, (i as u64) * 7 + 42),
                    "TTL_recent" => ttl_select(q.n, keep),
                    "KDF" => kdf_select(q.n, &q.edges, keep),
                    "KDF+TextSim" => kdf_textsim_select(q.n, &q.edges, &q.text_rare, keep),
                    _ => HashSet::new(),
                };
                let ms = t0.elapsed().as_secs_f64() * 1000.0;

                let hit = sel.intersection(&q.answer_turns).count() as f64;
                let recall = if q.answer_turns.is_empty() {
                    1.0
                } else {
                    hit / q.answer_turns.len() as f64
                };
                let sr = session_recall(&sel, &q.flat, &q.answer_sessions);
                let comp = 1.0 - sel.len() as f64 / q.n as f64;
                recalls.push(recall);
                session_recalls.push(sr);
                compressions.push(comp);
                walls.push(ms);
            }
            let mean = |v: &[f64]| -> f64 {
                if v.is_empty() {
                    return 0.0;
                }
                v.iter().sum::<f64>() / v.len() as f64
            };
            let r = mean(&recalls);
            let sr = mean(&session_recalls);
            let c = mean(&compressions);
            let w = mean(&walls);
            println!(
                "| {} | {:.2} | {:.4} | {:.4} | {:.4} | {:.2} |",
                method, rate, r, sr, c, w
            );
            csv.push_str(&format!(
                "{},{},{:.4},{:.4},{:.4},{:.2}\n",
                method, rate, r, sr, c, w
            ));
        }
    }

    // Write CSV
    let out = "demos/D8_llm_memory/out/w3_keep_rate_ablation.csv";
    if let Err(e) = std::fs::write(out, &csv) {
        eprintln!("Failed to write CSV: {}", e);
    } else {
        println!("\nSaved: {}", out);
    }
}
