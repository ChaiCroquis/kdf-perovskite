//! Phase X Step 1 — Claim 5 / 14 / 17 realistic time-series benchmark on LoCoMo.
//!
//! ## Motivation
//!
//! Claims 5 (time evaluation component), 14 (exponential decay), 17 (distributed
//! local decay) are currently backed by unit tests only (F-002, F-037). This
//! binary promotes them to a realistic LoCoMo temporal-recall benchmark
//! (`locomo_oracle_temporal_all.json`, 321 Q, 19-32 sessions each).
//!
//! ## Method
//!
//! Session index serves as the discrete time step (sessions are ordered in the
//! haystack). A turn arriving in session `s` has `age = max_session - s` when
//! the question is asked.
//!
//! ### Temporal score variants (passed to `select_top_k_multi_modal` as
//! `temporal_score`):
//!
//! - **Decay (Claim 14)**: `P_decay(age) = exp(-λ · age)` — younger = higher
//!   score. Penalises old content; expected to HURT on LoCoMo temporal since
//!   answers often sit in early sessions.
//! - **Staleness (Claim 5 alone)**: `T(age) = 1 - exp(-age / τ_ref)` — older =
//!   higher score. Boosts stale information; expected to HELP LoCoMo temporal.
//! - **Eval (Claim 5 + 14)**: `V(age) = P_decay · (1 + κ · T(age))` — the full
//!   Claim 5 evaluation value combining decay and time component.
//!
//! Each variant is combined with KDF layer score via
//! `MultiModalWeights { alpha: 0.5, beta: 0.0, gamma: 0.5 }` (graph-time
//! balanced, no text signal).
//!
//! ### Conditions
//!
//! 1. Random (baseline, seeded)
//! 2. TTL_recent (keep most-recent turns)
//! 3. TTL_oldest (keep oldest turns — mirror baseline for LoCoMo)
//! 4. KDF static (layer score only, matches F-057/F-058 setup)
//! 5. KDF + Claim 14 decay
//! 6. KDF + Claim 5 staleness
//! 7. KDF + Claim 5+14 eval
//!
//! ### Claim 17 sanity check
//!
//! A small parity test verifies that `DecayManager::apply_edge_decay_local`
//! (distributed shard-wise decay) matches `apply_edge_decay` (global) on a
//! sample graph. This promotes F-037's unit-level coverage to an integration
//! check on a realistic LoCoMo graph.
//!
//! ## Metric
//!
//! `answer_turn_recall` @ 30% keep_rate, averaged over 321 questions. Also
//! reports per-condition standard error and wall-clock time.

use cgb_kdf::framework::multimodal::{select_top_k_multi_modal, MultiModalWeights};
use cgb_kdf::{DecayManager, Layer, NodeClassifier};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

#[derive(Deserialize, Debug)]
struct Turn {
    #[allow(dead_code)]
    role: String,
    #[allow(dead_code)]
    content: String,
    has_answer: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct Question {
    #[allow(dead_code)]
    question_id: String,
    #[allow(dead_code)]
    question: String,
    #[serde(default)]
    #[allow(dead_code)]
    answer: serde_json::Value,
    haystack_session_ids: Vec<String>,
    haystack_sessions: Vec<Vec<Turn>>,
    answer_session_ids: Vec<String>,
}

struct Graph {
    n: usize,
    edges: Vec<(u32, u32, f64)>,
    session_of: Vec<u32>,
    answer_turns: HashSet<u32>,
    max_session: u32,
}

fn build_graph(q: &Question) -> Graph {
    let mut session_of: Vec<u32> = Vec::new();
    let mut answer_turns: HashSet<u32> = HashSet::new();
    for (i, session) in q.haystack_sessions.iter().enumerate() {
        let sid = q.haystack_session_ids.get(i).cloned().unwrap_or_default();
        let is_answer_sess = q.answer_session_ids.contains(&sid);
        for turn in session {
            let gid = session_of.len() as u32;
            if is_answer_sess && turn.has_answer.unwrap_or(false) {
                answer_turns.insert(gid);
            }
            session_of.push(i as u32);
        }
    }
    let n = session_of.len();
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    for i in 0..n.saturating_sub(1) {
        if session_of[i] == session_of[i + 1] {
            edges.push((i as u32, (i + 1) as u32, 1.0));
        }
    }
    let max_session = session_of.iter().copied().max().unwrap_or(0);
    Graph {
        n,
        edges,
        session_of,
        answer_turns,
        max_session,
    }
}

// --- temporal score variants ---

fn score_decay(g: &Graph, lambda: f64) -> Vec<f64> {
    (0..g.n)
        .map(|i| {
            let age = (g.max_session.saturating_sub(g.session_of[i])) as f64;
            (-lambda * age).exp()
        })
        .collect()
}

fn score_staleness(g: &Graph, tau_ref: f64) -> Vec<f64> {
    (0..g.n)
        .map(|i| {
            let age = (g.max_session.saturating_sub(g.session_of[i])) as f64;
            1.0 - (-age / tau_ref).exp()
        })
        .collect()
}

fn score_eval(g: &Graph, lambda: f64, tau_ref: f64, kappa: f64) -> Vec<f64> {
    (0..g.n)
        .map(|i| {
            let age = (g.max_session.saturating_sub(g.session_of[i])) as f64;
            let p = (-lambda * age).exp();
            let t = 1.0 - (-age / tau_ref).exp();
            (p * (1.0 + kappa * t)).clamp(0.0, 1.0)
        })
        .collect()
}

// --- selection methods ---

fn random_select(n: usize, keep: usize, seed: u64) -> HashSet<u32> {
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut idx: Vec<u32> = (0..n as u32).collect();
    idx.shuffle(&mut rng);
    idx.into_iter().take(keep).collect()
}

fn ttl_recent(n: usize, keep: usize) -> HashSet<u32> {
    (n.saturating_sub(keep) as u32..n as u32).collect()
}

fn ttl_oldest(_n: usize, keep: usize) -> HashSet<u32> {
    (0u32..keep as u32).collect()
}

fn kdf_static(g: &Graph, keep: usize) -> HashSet<u32> {
    let mut c = NodeClassifier::default();
    let class = c.classify(g.n, &g.edges);
    let layer_of: HashMap<u32, Layer> = class.layers;
    select_top_k_multi_modal(
        g.n,
        &layer_of,
        None,
        None,
        keep,
        &MultiModalWeights::graph_only(),
    )
}

fn kdf_with_temporal(g: &Graph, keep: usize, temporal: &[f64], gamma: f64) -> HashSet<u32> {
    let mut c = NodeClassifier::default();
    let class = c.classify(g.n, &g.edges);
    let layer_of: HashMap<u32, Layer> = class.layers;
    let weights = MultiModalWeights {
        alpha: 1.0 - gamma,
        beta: 0.0,
        gamma,
    };
    select_top_k_multi_modal(g.n, &layer_of, None, Some(temporal), keep, &weights)
}

// --- Claim 17 parity check ---

fn claim17_parity_check(g: &Graph) -> (f64, f64) {
    // Returns (max_diff, total_diff) between global and local decay on all edges.
    let mut c = NodeClassifier::default();
    let class = c.classify(g.n, &g.edges);
    let layer_of = class.layers.clone();

    // Global path
    let mut dm_global = DecayManager::master_spec();
    dm_global.initialize_with_edges(class.clone(), &g.edges);
    dm_global.apply_edge_decay();

    // Local path (shard into 2 halves)
    let dm_local = DecayManager::master_spec();
    let mut degrees: HashMap<u32, usize> = HashMap::new();
    for &(u, v, _) in &g.edges {
        *degrees.entry(u).or_insert(0) += 1;
        *degrees.entry(v).or_insert(0) += 1;
    }
    let items: Vec<((u32, u32), f64, f64, Layer)> = g
        .edges
        .iter()
        .map(|&(u, v, w)| {
            let layer_u = layer_of.get(&u).copied().unwrap_or(Layer::Edge);
            let layer_v = layer_of.get(&v).copied().unwrap_or(Layer::Edge);
            let layer = if layer_u.priority() > layer_v.priority() {
                layer_u
            } else {
                layer_v
            };
            let c = (degrees.get(&u).copied().unwrap_or(0) + degrees.get(&v).copied().unwrap_or(0))
                as f64;
            ((u, v), w, c, layer)
        })
        .collect();
    let local_results = dm_local.apply_edge_decay_local(items);

    let mut max_diff = 0f64;
    let mut total_diff = 0f64;
    for ((u, v), w_local) in local_results {
        let w_global = dm_global.get_edge_weight(u, v).unwrap_or(0.0);
        let d = (w_local - w_global).abs();
        if d > max_diff {
            max_diff = d;
        }
        total_diff += d;
    }
    (max_diff, total_diff)
}

// --- main ---

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        0.0
    } else {
        v.iter().sum::<f64>() / v.len() as f64
    }
}

fn stderr(v: &[f64]) -> f64 {
    if v.len() < 2 {
        return 0.0;
    }
    let m = mean(v);
    let var = v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / (v.len() - 1) as f64;
    (var / v.len() as f64).sqrt()
}

fn main() {
    let path = "demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json";
    println!(
        "# Phase X Step 1 — Claim 5/14/17 realistic time-series benchmark on LoCoMo temporal\n"
    );

    let data = std::fs::read_to_string(path).expect("Load LoCoMo temporal JSON");
    let questions: Vec<Question> = serde_json::from_str(&data).expect("Parse LoCoMo JSON");
    println!(
        "Loaded {} temporal questions from LoCoMo oracle\n",
        questions.len()
    );

    let keep_rate = 0.30_f64;
    // Hyperparameters for temporal score (chosen to span the age range [0, 30]):
    let lambda = 0.10; // decay rate per session
    let tau_ref = 10.0; // staleness scale in sessions
    let kappa = 1.0; // weight for time component in eval formula

    println!(
        "Config: keep_rate={}, λ={}, τ_ref={}, κ={}\n",
        keep_rate, lambda, tau_ref, kappa
    );

    struct Res {
        name: String,
        recalls: Vec<f64>,
        walls_ms: Vec<f64>,
    }

    // Build ablation grid: 3 temporal variants × 3 γ levels + 4 baselines
    let gammas = [0.2_f64, 0.5, 0.8];
    let mut method_names: Vec<String> = vec![
        "Random".into(),
        "TTL_recent".into(),
        "TTL_oldest".into(),
        "KDF_static".into(),
    ];
    for g in &gammas {
        method_names.push(format!("KDF+Decay(C14) γ={}", g));
        method_names.push(format!("KDF+Staleness(C5) γ={}", g));
        method_names.push(format!("KDF+Eval(C5+14) γ={}", g));
    }
    let mut results: Vec<Res> = method_names
        .iter()
        .map(|n| Res {
            name: n.clone(),
            recalls: Vec::new(),
            walls_ms: Vec::new(),
        })
        .collect();

    // Claim 17 parity accumulator
    let mut parity_max_diffs: Vec<f64> = Vec::new();

    for (q_idx, q) in questions.iter().enumerate() {
        let g = build_graph(q);
        if g.n == 0 || g.answer_turns.is_empty() {
            continue;
        }
        let keep = (g.n as f64 * keep_rate).ceil() as usize;

        // Pre-compute temporal scores once per question
        let s_decay = score_decay(&g, lambda);
        let s_stale = score_staleness(&g, tau_ref);
        let s_eval = score_eval(&g, lambda, tau_ref, kappa);

        for (mi, name) in method_names.iter().enumerate() {
            let t0 = std::time::Instant::now();
            let sel: HashSet<u32> = if name == "Random" {
                random_select(g.n, keep, (q_idx as u64) * 7 + 42)
            } else if name == "TTL_recent" {
                ttl_recent(g.n, keep)
            } else if name == "TTL_oldest" {
                ttl_oldest(g.n, keep)
            } else if name == "KDF_static" {
                kdf_static(&g, keep)
            } else if name.starts_with("KDF+Decay(C14) γ=") {
                let gamma: f64 = name.rsplit('=').next().unwrap().parse().unwrap();
                kdf_with_temporal(&g, keep, &s_decay, gamma)
            } else if name.starts_with("KDF+Staleness(C5) γ=") {
                let gamma: f64 = name.rsplit('=').next().unwrap().parse().unwrap();
                kdf_with_temporal(&g, keep, &s_stale, gamma)
            } else if name.starts_with("KDF+Eval(C5+14) γ=") {
                let gamma: f64 = name.rsplit('=').next().unwrap().parse().unwrap();
                kdf_with_temporal(&g, keep, &s_eval, gamma)
            } else {
                HashSet::new()
            };
            let ms = t0.elapsed().as_secs_f64() * 1000.0;

            let hit = sel.intersection(&g.answer_turns).count() as f64;
            let recall = hit / g.answer_turns.len() as f64;
            results[mi].recalls.push(recall);
            results[mi].walls_ms.push(ms);
        }

        // Claim 17 parity check on first 10 questions (to bound runtime)
        if q_idx < 10 {
            let (md, _) = claim17_parity_check(&g);
            parity_max_diffs.push(md);
        }
    }

    println!("## Recall @ keep_rate={}", keep_rate);
    println!();
    println!("| Method | mean recall | SE | wall ms/q |");
    println!("|---|---:|---:|---:|");
    for r in &results {
        let m = mean(&r.recalls);
        let se = stderr(&r.recalls);
        let w = mean(&r.walls_ms);
        println!("| {} | {:.4} | ±{:.4} | {:.2} |", r.name, m, se, w);
    }

    // Pairwise delta vs KDF_static
    let static_idx = method_names.iter().position(|n| n == "KDF_static").unwrap();
    let m_static = mean(&results[static_idx].recalls);
    println!();
    println!("## Δ vs KDF_static ({:.4})", m_static);
    println!();
    println!("| Method | Δ | n positive | n tie | n negative |");
    println!("|---|---:|---:|---:|---:|");
    for (mi, r) in results.iter().enumerate() {
        if mi == static_idx {
            continue;
        }
        let m = mean(&r.recalls);
        let n_pos = r
            .recalls
            .iter()
            .zip(&results[static_idx].recalls)
            .filter(|(a, b)| a > b)
            .count();
        let n_tie = r
            .recalls
            .iter()
            .zip(&results[static_idx].recalls)
            .filter(|(a, b)| a == b)
            .count();
        let n_neg = r
            .recalls
            .iter()
            .zip(&results[static_idx].recalls)
            .filter(|(a, b)| a < b)
            .count();
        println!(
            "| {} | {:+.4} | {} | {} | {} |",
            r.name,
            m - m_static,
            n_pos,
            n_tie,
            n_neg
        );
    }

    // Claim 17 parity report
    let max_parity = parity_max_diffs.iter().cloned().fold(0f64, f64::max);
    println!();
    println!("## Claim 17 parity check (global vs local decay, 10 sampled questions)");
    println!(
        "- Max absolute edge-weight diff across all sampled graphs: **{:.3e}**",
        max_parity
    );
    println!(
        "- {} (threshold 1e-10)",
        if max_parity < 1e-10 { "PASS" } else { "FAIL" }
    );

    println!();
    println!("## Interpretation notes");
    println!("- LoCoMo temporal answers often sit in **early sessions** → TTL_oldest expected > TTL_recent.");
    println!("- Naive decay (C14) penalises old turns → expected to HURT on this task.");
    println!("- Staleness (C5) boosts old turns → expected to HELP on this task.");
    println!("- Combined eval (C5+C14) trades off: C14 decay × (1 + κ·C5 staleness).");
    println!("- Any condition beating KDF_static with p<0.05 (sign test) validates the time component for this realistic scenario.");
}
