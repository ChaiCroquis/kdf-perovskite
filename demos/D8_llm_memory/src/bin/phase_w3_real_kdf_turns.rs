//! Dump per-question KDF-selected turn INDICES for LongMemEval 500Q at a
//! configurable keep_rate, for use by Python answer-generation pipelines.
//!
//! This replaces the Python `kdf_retrieve()` approximation that assumed a
//! global 0.821 recall constant; instead we run the real KDF selector
//! (Rare/Core/Edge/Garbage classification, optional text-rareness MM).
//!
//! Output: JSON with schema
//!   {
//!     "keep_rate": 0.30,
//!     "method": "KDF",
//!     "n_questions": 500,
//!     "results": [
//!       { "question_id": "...",
//!         "kept_turn_indices": [3, 7, 12, ...],    // 0-based global within flattened turns
//!         "n_total_turns": 36,
//!         "answer_turn_recall": 0.5 },
//!       ...
//!     ]
//!   }
//!
//! Flatten order: session 0 turns in order, then session 1, ... (matches
//! `phase_w_longmemeval.rs` canonical order).
//!
//! Usage:
//!   cargo run --release -p demo-d8-llm-memory --bin phase_w3_real_kdf_turns -- \
//!       --keep-rate 0.30 --method KDF
//!   cargo run --release -p demo-d8-llm-memory --bin phase_w3_real_kdf_turns -- \
//!       --keep-rate 0.30 --method KDF+TextSim

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct Turn {
    role: String,
    content: String,
    has_answer: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Question {
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

#[derive(Serialize)]
struct PerQ {
    question_id: String,
    kept_turn_indices: Vec<u32>,
    n_total_turns: usize,
    n_answer_turns: usize,
    answer_turn_recall: f64,
}

#[derive(Serialize)]
struct Output {
    keep_rate: f64,
    method: String,
    n_questions: usize,
    mean_answer_turn_recall: f64,
    results: Vec<PerQ>,
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
    Vec<(String, String, String, usize)>, // (session_id, role, content, idx)
    Vec<(u32, u32, f64)>,
    HashSet<u32>,
) {
    let mut flat: Vec<(String, String, String, usize)> = Vec::new();
    let mut answer_turn_ids: HashSet<u32> = HashSet::new();
    for (i, session) in q.haystack_sessions.iter().enumerate() {
        let sid = q.haystack_session_ids.get(i).cloned().unwrap_or_default();
        let is_answer_sess = q.answer_session_ids.contains(&sid);
        for turn in session {
            let global_idx = flat.len();
            if is_answer_sess && turn.has_answer.unwrap_or(false) {
                answer_turn_ids.insert(global_idx as u32);
            }
            flat.push((
                sid.clone(),
                turn.role.clone(),
                turn.content.clone(),
                global_idx,
            ));
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
    (flat, edges, answer_turn_ids)
}

fn text_rareness_scores(flat: &[(String, String, String, usize)]) -> Vec<f64> {
    let all_sh: Vec<HashSet<String>> = flat.iter().map(|(_, _, c, _)| shingles(c, 5)).collect();
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
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
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

fn main() {
    // Parse args
    let args: Vec<String> = std::env::args().collect();
    let mut keep_rate: f64 = 0.30;
    let mut method: String = "KDF".to_string();
    let mut out_path: String = String::new();
    let mut input_path: String = "demos/D8_llm_memory/data/longmemeval_oracle.json".to_string();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--keep-rate" => {
                keep_rate = args[i + 1].parse().expect("keep_rate must be a float");
                i += 2;
            }
            "--method" => {
                method = args[i + 1].clone();
                i += 2;
            }
            "--out" => {
                out_path = args[i + 1].clone();
                i += 2;
            }
            "--input" => {
                input_path = args[i + 1].clone();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    if out_path.is_empty() {
        out_path = format!(
            "demos/D8_llm_memory/out/w3_real_kdf_turns_{}_{:03}.json",
            method.replace("+", "_"),
            (keep_rate * 100.0).round() as u32
        );
    }
    assert!(
        method == "KDF" || method == "KDF+TextSim",
        "method must be 'KDF' or 'KDF+TextSim', got {}",
        method
    );

    let path = input_path.as_str();
    eprintln!("Loading {} ...", path);
    let data = std::fs::read_to_string(path).expect("Load LongMemEval oracle");
    let questions: Vec<Question> = serde_json::from_str(&data).expect("Parse JSON");
    eprintln!(
        "Loaded {} questions; method={}, keep_rate={:.2}",
        questions.len(),
        method,
        keep_rate
    );

    let mut results: Vec<PerQ> = Vec::with_capacity(questions.len());
    let mut recall_sum = 0.0;
    let mut recall_count = 0;

    for q in &questions {
        let (flat, edges, answer_turns) = build_graph(q);
        let n = flat.len();
        if n == 0 {
            results.push(PerQ {
                question_id: q.question_id.clone(),
                kept_turn_indices: vec![],
                n_total_turns: 0,
                n_answer_turns: 0,
                answer_turn_recall: 1.0,
            });
            continue;
        }
        let keep = ((n as f64) * keep_rate).ceil() as usize;
        let keep = keep.max(1).min(n);

        let sel: HashSet<u32> = match method.as_str() {
            "KDF" => kdf_select(n, &edges, keep),
            "KDF+TextSim" => {
                let tr = text_rareness_scores(&flat);
                kdf_textsim_select(n, &edges, &tr, keep)
            }
            _ => unreachable!(),
        };

        let recall = if answer_turns.is_empty() {
            1.0
        } else {
            sel.intersection(&answer_turns).count() as f64 / answer_turns.len() as f64
        };
        recall_sum += recall;
        recall_count += 1;

        let mut kept: Vec<u32> = sel.into_iter().collect();
        kept.sort();
        results.push(PerQ {
            question_id: q.question_id.clone(),
            kept_turn_indices: kept,
            n_total_turns: n,
            n_answer_turns: answer_turns.len(),
            answer_turn_recall: recall,
        });
    }

    let mean_recall = if recall_count == 0 {
        0.0
    } else {
        recall_sum / recall_count as f64
    };
    eprintln!(
        "mean answer_turn_recall = {:.4} over {} questions",
        mean_recall, recall_count
    );

    let output = Output {
        keep_rate,
        method: method.clone(),
        n_questions: results.len(),
        mean_answer_turn_recall: mean_recall,
        results,
    };
    let json = serde_json::to_string(&output).expect("serialize");
    std::fs::write(&out_path, json).expect("write output");
    eprintln!("Wrote: {}", out_path);
    println!("{}", out_path);
}
