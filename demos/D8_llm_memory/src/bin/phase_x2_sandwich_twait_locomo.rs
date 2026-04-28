//! Phase X Step 2 — Claim 36-41 (T_wait 二段階審査) + Claim 47-48 (θ_L/θ_U sandwich)
//! realistic benchmark.
//!
//! ## Background
//!
//! F-068 validated Claim 1 pillar 3 (analogy discovery) via AnalogyDiscoveryEngine
//! with θ_disc=0.75. Claims 36-41 (二段階審査 T_wait1/T_wait2) and 47-48
//! (sandwich upper bound θ_U=0.80) are currently backed only by unit tests
//! (F-040, F-041 Phase V3). F-041 already partially falsified θ_U on Hopfield
//! spurious attractors. This binary extends the sandwich test to realistic
//! analogy discovery tasks and streaming RARE-node review.
//!
//! ## Part A — Sandwich sensitivity on analogy discovery (Claim 47-48)
//!
//! Reuses F-068's analogy scenarios. For each pair, captures the raw
//! `overall_score` from `AnalogyDiscoveryEngine::find_analogy` and applies
//! multiple sandwich configurations post-hoc. Measures:
//!   - TP: positive pair with θ_L ≤ score ≤ θ_U (correctly admitted)
//!   - FN: positive pair rejected by sandwich
//!   - TN: negative pair score < θ_L (correctly rejected)
//!   - FP: negative pair inside band
//!
//! Configurations: (0.70, 0.75), (0.70, 0.80 canonical), (0.70, 0.90),
//! (0.70, 0.95), (0.70, 1.00 = no upper bound).
//!
//! ## Part B — T_wait 2-stage streaming review on LoCoMo (Claim 36-41)
//!
//! Takes 30 LoCoMo temporal questions, builds turn-adjacency graph, runs
//! `KdfProcessorRev12` review cycles with (t_wait1=30, t_wait2=30). For each
//! answer-turn RARE node, tracks whether it:
//!   - spoke_up: analogy found within sandwich (preserved)
//!   - demoted: wait period expired without analogy (lost to Garbage)
//!
//! Compares 3 sandwich settings: θ_U ∈ {0.80, 0.90, 1.00}.
//!
//! Outcome question: does sandwich admit meaningful analogies in realistic
//! chain-graph substrate, or does it systematically reject structurally-
//! identical RARE boundary nodes?

use cgb_kdf::analogy::{AnalogyDiscoveryEngine, NodeFeatures, RelationType};
use cgb_kdf::fingerprint::NodeLabel;
use cgb_kdf::framework::rev12::KdfProcessorRev12;
use cgb_kdf::{Layer, NodeClassifier};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

// =============================================================================
// Data loader (shared with phase_x1)
// =============================================================================

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
    haystack_session_ids: Vec<String>,
    haystack_sessions: Vec<Vec<Turn>>,
    answer_session_ids: Vec<String>,
}

struct TurnGraph {
    n: usize,
    edges: Vec<(u32, u32, f64)>,
    answer_turns: HashSet<u32>,
}

fn build_turn_graph(q: &Question) -> TurnGraph {
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
    TurnGraph {
        n,
        edges,
        answer_turns,
    }
}

// =============================================================================
// Part A — Sandwich sensitivity on analogy discovery
// =============================================================================

#[allow(dead_code)]
struct AnalogyPair {
    source_id: String,
    target_id: String,
    score: f64,
    is_positive: bool,
    scenario: &'static str,
}

/// Build AnalogyDiscoveryEngine with threshold=0.0 so ALL scores come through.
fn build_permissive_engine() -> AnalogyDiscoveryEngine {
    AnalogyDiscoveryEngine::new(
        0.1, 0.2,
        0.7, // Claim 44 weights 7:2:1 (attribute/relational/systematic normalized to 0.1/0.2/0.7)
        0.0, // discovery_threshold = 0 (capture ALL scores)
        32,  // fingerprint_dim
        true, // screening_enabled
        0.05, // top_k_percent
    )
}

#[allow(clippy::too_many_arguments)]
fn register_node(
    engine: &mut AnalogyDiscoveryEngine,
    id: &str,
    degree: u32,
    clustering: f64,
    outgoing: Vec<RelationType>,
    incoming: Vec<RelationType>,
    domain: &str,
    label: NodeLabel,
) {
    let mut features = NodeFeatures::new(id.to_string());
    features.degree = degree;
    features.clustering_coef = clustering;
    features.domain = domain.to_string();
    features.outgoing_relation_types = outgoing.into_iter().collect::<HashSet<_>>();
    features.incoming_relation_types = incoming.into_iter().collect::<HashSet<_>>();
    engine.register_node(id, features, &label);
}

fn part_a_solar_atom() -> Vec<AnalogyPair> {
    let mut e = build_permissive_engine();
    // Solar system
    register_node(
        &mut e,
        "sun",
        8,
        0.9,
        vec![RelationType::Causal, RelationType::Enables],
        vec![],
        "astronomy",
        NodeLabel::IsolatedTruth,
    );
    register_node(
        &mut e,
        "earth",
        2,
        0.1,
        vec![RelationType::Temporal],
        vec![RelationType::Causal, RelationType::Enables],
        "astronomy",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "mars",
        2,
        0.1,
        vec![RelationType::Temporal],
        vec![RelationType::Causal, RelationType::Enables],
        "astronomy",
        NodeLabel::Normal,
    );
    // Atom
    register_node(
        &mut e,
        "nucleus",
        8,
        0.9,
        vec![RelationType::Causal, RelationType::Enables],
        vec![],
        "physics",
        NodeLabel::IsolatedTruth,
    );
    register_node(
        &mut e,
        "electron1",
        2,
        0.1,
        vec![RelationType::Temporal],
        vec![RelationType::Causal, RelationType::Enables],
        "physics",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "electron2",
        2,
        0.1,
        vec![RelationType::Temporal],
        vec![RelationType::Causal, RelationType::Enables],
        "physics",
        NodeLabel::Normal,
    );

    let targets: Vec<String> = vec!["nucleus".into(), "electron1".into(), "electron2".into()];
    let gt = [
        ("sun", "nucleus"),
        ("earth", "electron1"),
        ("mars", "electron1"),
    ];
    gt.iter()
        .filter_map(|(src, expected)| {
            e.find_analogy(src, &targets).map(|m| AnalogyPair {
                source_id: src.to_string(),
                target_id: m.target_node.clone(),
                score: m.overall_score,
                is_positive: m.target_node == *expected,
                scenario: "Gentner sun↔atom",
            })
        })
        .collect()
}

fn part_a_git_paper() -> Vec<AnalogyPair> {
    let mut e = build_permissive_engine();
    // git side
    register_node(
        &mut e,
        "bug_issue",
        3,
        0.2,
        vec![RelationType::Causal, RelationType::Enables],
        vec![],
        "git",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "fix_branch",
        2,
        0.3,
        vec![RelationType::Temporal, RelationType::PartOf],
        vec![RelationType::Causal, RelationType::Enables],
        "git",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "merge_commit",
        4,
        0.5,
        vec![
            RelationType::Enables,
            RelationType::PartOf,
            RelationType::Temporal,
        ],
        vec![RelationType::Temporal, RelationType::PartOf],
        "git",
        NodeLabel::IsolatedTruth,
    );
    register_node(
        &mut e,
        "release_tag",
        3,
        0.4,
        vec![RelationType::PartOf],
        vec![RelationType::Enables, RelationType::PartOf],
        "git",
        NodeLabel::Normal,
    );
    // paper side
    register_node(
        &mut e,
        "problem_stmt",
        3,
        0.2,
        vec![RelationType::Causal, RelationType::Enables],
        vec![],
        "paper",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "solution_draft",
        2,
        0.3,
        vec![RelationType::Temporal, RelationType::PartOf],
        vec![RelationType::Causal, RelationType::Enables],
        "paper",
        NodeLabel::Normal,
    );
    register_node(
        &mut e,
        "peer_review_merge",
        4,
        0.5,
        vec![
            RelationType::Enables,
            RelationType::PartOf,
            RelationType::Temporal,
        ],
        vec![RelationType::Temporal, RelationType::PartOf],
        "paper",
        NodeLabel::IsolatedTruth,
    );
    register_node(
        &mut e,
        "publication",
        3,
        0.4,
        vec![RelationType::PartOf],
        vec![RelationType::Enables, RelationType::PartOf],
        "paper",
        NodeLabel::Normal,
    );

    let targets: Vec<String> = vec![
        "problem_stmt".into(),
        "solution_draft".into(),
        "peer_review_merge".into(),
        "publication".into(),
    ];
    let gt = [
        ("bug_issue", "problem_stmt"),
        ("fix_branch", "solution_draft"),
        ("merge_commit", "peer_review_merge"),
        ("release_tag", "publication"),
    ];
    gt.iter()
        .filter_map(|(src, expected)| {
            e.find_analogy(src, &targets).map(|m| AnalogyPair {
                source_id: src.to_string(),
                target_id: m.target_node.clone(),
                score: m.overall_score,
                is_positive: m.target_node == *expected,
                scenario: "git↔paper",
            })
        })
        .collect()
}

fn part_a_negative_control() -> Vec<AnalogyPair> {
    let mut e = build_permissive_engine();
    register_node(
        &mut e,
        "hub_src",
        15,
        0.9,
        vec![
            RelationType::Causal,
            RelationType::Enables,
            RelationType::PartOf,
        ],
        vec![RelationType::Causal],
        "net_a",
        NodeLabel::IsolatedTruth,
    );
    register_node(
        &mut e,
        "isolated_a",
        0,
        0.0,
        vec![],
        vec![],
        "net_b",
        NodeLabel::Garbage,
    );
    register_node(
        &mut e,
        "isolated_b",
        1,
        0.0,
        vec![RelationType::Contrast],
        vec![],
        "net_b",
        NodeLabel::Garbage,
    );

    let targets: Vec<String> = vec!["isolated_a".into(), "isolated_b".into()];
    // Truly isomorphic-negative: hub_src score with ANY target is "negative" ground truth
    e.find_analogy("hub_src", &targets)
        .map(|m| AnalogyPair {
            source_id: "hub_src".to_string(),
            target_id: m.target_node.clone(),
            score: m.overall_score,
            is_positive: false, // engineered to be negative
            scenario: "non-isomorphic negative",
        })
        .into_iter()
        .collect()
}

/// Additional synthetic 30 pairs: 15 positive (isomorphic structures) + 15 negative (mismatched)
fn part_a_synthetic_bulk() -> Vec<AnalogyPair> {
    let mut pairs = Vec::new();
    for trial in 0..15 {
        let mut e = build_permissive_engine();
        // Source: hub + leaves
        let src_deg = 5 + trial as u32;
        let n_leaves = 3 + (trial % 3) as usize;
        register_node(
            &mut e,
            "src_hub",
            src_deg,
            0.5,
            vec![RelationType::Causal, RelationType::Enables],
            vec![],
            "sA",
            NodeLabel::IsolatedTruth,
        );
        for i in 0..n_leaves {
            let id = format!("src_leaf{}", i);
            register_node(
                &mut e,
                &id,
                1,
                0.0,
                vec![],
                vec![RelationType::Causal],
                "sA",
                NodeLabel::Normal,
            );
        }
        // Target: SAME structure, different domain
        register_node(
            &mut e,
            "tgt_hub",
            src_deg,
            0.5,
            vec![RelationType::Causal, RelationType::Enables],
            vec![],
            "sB",
            NodeLabel::IsolatedTruth,
        );
        let mut targets = vec!["tgt_hub".into()];
        for i in 0..n_leaves {
            let id = format!("tgt_leaf{}", i);
            register_node(
                &mut e,
                &id,
                1,
                0.0,
                vec![],
                vec![RelationType::Causal],
                "sB",
                NodeLabel::Normal,
            );
            targets.push(id);
        }
        if let Some(m) = e.find_analogy("src_hub", &targets) {
            pairs.push(AnalogyPair {
                source_id: "src_hub".into(),
                target_id: m.target_node.clone(),
                score: m.overall_score,
                is_positive: m.target_node == "tgt_hub",
                scenario: "synthetic_iso",
            });
        }
    }
    for trial in 0..15 {
        let mut e = build_permissive_engine();
        // Source: deg high, outgoing-heavy
        register_node(
            &mut e,
            "src",
            10 + trial as u32,
            0.8,
            vec![
                RelationType::Causal,
                RelationType::Enables,
                RelationType::PartOf,
            ],
            vec![RelationType::Causal],
            "hA",
            NodeLabel::IsolatedTruth,
        );
        // Target: deg 1, no outgoing
        register_node(
            &mut e,
            "tgt1",
            1,
            0.0,
            vec![],
            vec![],
            "hB",
            NodeLabel::Garbage,
        );
        register_node(
            &mut e,
            "tgt2",
            0,
            0.0,
            vec![RelationType::Contrast],
            vec![],
            "hB",
            NodeLabel::Garbage,
        );
        let targets: Vec<String> = vec!["tgt1".into(), "tgt2".into()];
        if let Some(m) = e.find_analogy("src", &targets) {
            pairs.push(AnalogyPair {
                source_id: "src".into(),
                target_id: m.target_node.clone(),
                score: m.overall_score,
                is_positive: false,
                scenario: "synthetic_nonIso",
            });
        }
    }
    pairs
}

fn part_a_run() {
    println!("## Part A — Sandwich sensitivity on analogy discovery (Claim 47-48)\n");

    let mut all_pairs: Vec<AnalogyPair> = Vec::new();
    all_pairs.extend(part_a_solar_atom());
    all_pairs.extend(part_a_git_paper());
    all_pairs.extend(part_a_negative_control());
    all_pairs.extend(part_a_synthetic_bulk());

    println!("Total pairs captured: {}", all_pairs.len());
    let n_pos = all_pairs.iter().filter(|p| p.is_positive).count();
    let n_neg = all_pairs.len() - n_pos;
    println!(
        "  Positive (expected match): {}\n  Negative (expected reject): {}\n",
        n_pos, n_neg
    );

    // Raw score distribution
    println!("### Raw score distribution");
    println!();
    println!("| Scenario | n | mean score | min | max | expected |");
    println!("|---|---:|---:|---:|---:|---|");
    let mut scenarios: HashMap<&'static str, (usize, f64, f64, f64, bool)> = HashMap::new();
    for p in &all_pairs {
        let e = scenarios
            .entry(p.scenario)
            .or_insert((0, 0.0, f64::MAX, f64::MIN, p.is_positive));
        e.0 += 1;
        e.1 += p.score;
        e.2 = e.2.min(p.score);
        e.3 = e.3.max(p.score);
    }
    let mut keys: Vec<_> = scenarios.keys().collect();
    keys.sort();
    for k in keys {
        let (n, sum, mn, mx, pos) = scenarios[k];
        println!(
            "| {} | {} | {:.4} | {:.4} | {:.4} | {} |",
            k,
            n,
            sum / n as f64,
            mn,
            mx,
            if pos { "POS" } else { "NEG" }
        );
    }

    // Sandwich sweep
    println!();
    println!("### Sandwich filter evaluation");
    println!();
    println!("| (θ_L, θ_U) | TP | FN | TN | FP | Precision | Recall | F1 |");
    println!("|---|---:|---:|---:|---:|---:|---:|---:|");

    let configs = [
        (0.70, 0.75, "narrow"),
        (0.70, 0.80, "canonical"),
        (0.70, 0.90, "wide"),
        (0.70, 0.95, "wider"),
        (0.70, 1.00, "no-upper"),
    ];
    for (tl, tu, _name) in &configs {
        let mut tp = 0;
        let mut fn_ = 0;
        let mut tn = 0;
        let mut fp = 0;
        for p in &all_pairs {
            let admitted = p.score >= *tl && p.score <= *tu;
            match (p.is_positive, admitted) {
                (true, true) => tp += 1,
                (true, false) => fn_ += 1,
                (false, false) => tn += 1,
                (false, true) => fp += 1,
            }
        }
        let prec = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let rec = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1 = if prec + rec > 0.0 {
            2.0 * prec * rec / (prec + rec)
        } else {
            0.0
        };
        println!(
            "| ({:.2}, {:.2}) | {} | {} | {} | {} | {:.3} | {:.3} | {:.3} |",
            tl, tu, tp, fn_, tn, fp, prec, rec, f1
        );
    }

    // Top-level verdict
    println!();
    println!("**Verdict(Part A)**:");
    let canonical_config = configs.iter().find(|c| c.1 == 0.80).unwrap();
    let wider_config = configs.iter().find(|c| c.1 == 1.00).unwrap();
    let mut canonical_f1 = 0.0;
    let mut wider_f1 = 0.0;
    for (tl, tu, _) in [canonical_config, wider_config] {
        let mut tp = 0;
        let mut fn_ = 0;
        let mut fp = 0;
        for p in &all_pairs {
            let admitted = p.score >= *tl && p.score <= *tu;
            match (p.is_positive, admitted) {
                (true, true) => tp += 1,
                (true, false) => fn_ += 1,
                (false, true) => fp += 1,
                _ => {}
            }
        }
        let prec = if tp + fp > 0 {
            tp as f64 / (tp + fp) as f64
        } else {
            0.0
        };
        let rec = if tp + fn_ > 0 {
            tp as f64 / (tp + fn_) as f64
        } else {
            0.0
        };
        let f1 = if prec + rec > 0.0 {
            2.0 * prec * rec / (prec + rec)
        } else {
            0.0
        };
        if (tu - 0.80f64).abs() < 1e-6 {
            canonical_f1 = f1;
        } else {
            wider_f1 = f1;
        }
    }
    println!("- Canonical (θ_U=0.80) F1: **{:.3}**", canonical_f1);
    println!("- No-upper (θ_U=1.00) F1: **{:.3}**", wider_f1);
    if wider_f1 > canonical_f1 + 0.01 {
        println!(
            "- **θ_U=0.80 is too strict**: relaxing to 1.00 improves F1 by {:.3}",
            wider_f1 - canonical_f1
        );
    } else if (canonical_f1 - wider_f1).abs() < 0.01 {
        println!("- **θ_U=0.80 is neutral**: relaxing upper bound yields no F1 change");
    } else {
        println!(
            "- **θ_U=0.80 adds value**: removing it degrades F1 by {:.3}",
            canonical_f1 - wider_f1
        );
    }
}

// =============================================================================
// Part B — T_wait 2-stage streaming review on LoCoMo (Claim 36-41)
// =============================================================================

#[derive(Debug, Default)]
struct PartBAgg {
    n_questions: usize,
    total_rare: usize,
    total_answer_rare: usize,
    spoke_up_answer: usize,
    spoke_up_nonanswer: usize,
    demoted_answer: usize,
    demoted_nonanswer: usize,
    cycles_sum: u64,
}

fn part_b_run_single(g: &TurnGraph, t_wait1: u64, t_wait2: u64, theta_u: f64) -> PartBAgg {
    let mut agg = PartBAgg {
        n_questions: 1,
        ..Default::default()
    };

    // Initial classification (for truth mask on answer-RARE)
    let mut cls = NodeClassifier::default();
    let class = cls.classify(g.n, &g.edges);
    let rare_ids: HashSet<u32> = class
        .layers
        .iter()
        .filter(|&(_, &l)| l == Layer::Rare)
        .map(|(&id, _)| id)
        .collect();
    let answer_rare: HashSet<u32> = rare_ids.intersection(&g.answer_turns).copied().collect();
    agg.total_rare = rare_ids.len();
    agg.total_answer_rare = answer_rare.len();

    if rare_ids.is_empty() {
        return agg;
    }

    // Init Rev12 processor
    let mut proc = match KdfProcessorRev12::with_upper_threshold(t_wait1, t_wait2, 0.70, theta_u) {
        Ok(p) => p,
        Err(_) => return agg,
    };
    proc.initialize(g.n, &g.edges);

    // Run cycles up to t_wait1 + t_wait2 (max theoretical) with early termination.
    // Apply actions BEFORE checking pending so demote layer-updates land.
    let max_cycles = t_wait1 + t_wait2 + 5;
    let mut cycles_used = 0u64;
    for cycle in 0..max_cycles {
        let actions = proc.process_review_cycle();
        cycles_used = cycle + 1;
        for (node, action) in actions {
            match action {
                "promote" => proc.apply_promotion(node),
                "demote" => proc.apply_demotion(node),
                _ => {}
            }
        }
        let pending = proc
            .get_original_rare_nodes()
            .iter()
            .filter(|&&n| {
                proc.get_rare_state(n)
                    .map(|s| s.phase != cgb_kdf::framework::rev12::ReviewPhase::Complete)
                    .unwrap_or(false)
            })
            .count();
        if pending == 0 {
            break;
        }
    }
    agg.cycles_sum = cycles_used;

    // Tally outcomes
    for &rare_id in &rare_ids {
        let is_answer = answer_rare.contains(&rare_id);
        let state = proc.get_rare_state(rare_id);
        let spoke_up = state.map(|s| s.spoke_up).unwrap_or(false);
        let demoted = matches!(proc.get_layer(rare_id), Some(Layer::Garbage));

        if is_answer {
            if spoke_up {
                agg.spoke_up_answer += 1;
            }
            if demoted {
                agg.demoted_answer += 1;
            }
        } else {
            if spoke_up {
                agg.spoke_up_nonanswer += 1;
            }
            if demoted {
                agg.demoted_nonanswer += 1;
            }
        }
    }
    agg
}

fn part_b_aggregate(a: PartBAgg, b: &PartBAgg) -> PartBAgg {
    PartBAgg {
        n_questions: a.n_questions + b.n_questions,
        total_rare: a.total_rare + b.total_rare,
        total_answer_rare: a.total_answer_rare + b.total_answer_rare,
        spoke_up_answer: a.spoke_up_answer + b.spoke_up_answer,
        spoke_up_nonanswer: a.spoke_up_nonanswer + b.spoke_up_nonanswer,
        demoted_answer: a.demoted_answer + b.demoted_answer,
        demoted_nonanswer: a.demoted_nonanswer + b.demoted_nonanswer,
        cycles_sum: a.cycles_sum + b.cycles_sum,
    }
}

fn part_b_run() {
    println!();
    println!("## Part B — T_wait 2-stage streaming review on LoCoMo (Claim 36-41)\n");

    let path = "demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json";
    let data = std::fs::read_to_string(path).expect("Load LoCoMo temporal JSON");
    let questions: Vec<Question> = serde_json::from_str(&data).expect("Parse LoCoMo JSON");
    let n_sample = 30.min(questions.len()); // bound runtime

    println!(
        "Running Rev12 review on first {} LoCoMo questions",
        n_sample
    );
    println!("t_wait1=30, t_wait2=30 (Claim 37/39 canonical)\n");

    println!(
        "| θ_U | total RARE | answer-RARE | spoke_up(ans) | demoted(ans) | spoke_up(non) | demoted(non) | avg cycles |"
    );
    println!("|---|---:|---:|---:|---:|---:|---:|---:|");

    for theta_u in [0.80_f64, 0.90, 1.00] {
        let mut total = PartBAgg::default();
        for q in questions.iter().take(n_sample) {
            let g = build_turn_graph(q);
            if g.n == 0 {
                continue;
            }
            let a = part_b_run_single(&g, 30, 30, theta_u);
            total = part_b_aggregate(total, &a);
        }

        let avg_cycles = if total.n_questions > 0 {
            total.cycles_sum as f64 / total.n_questions as f64
        } else {
            0.0
        };

        println!(
            "| {:.2} | {} | {} | {} | {} | {} | {} | {:.1} |",
            theta_u,
            total.total_rare,
            total.total_answer_rare,
            total.spoke_up_answer,
            total.demoted_answer,
            total.spoke_up_nonanswer,
            total.demoted_nonanswer,
            avg_cycles,
        );
    }

    println!();
    println!("**Verdict(Part B)**:");
    println!("- answer-RARE turns の spoke_up 率が高い → sandwich が意味ある analogy を admit");
    println!(
        "- 両 group demote 率が高い → sandwich too strict or graph structure lacks cross-domain targets"
    );
    println!("- avg cycles ≈ 60 → 大半の node が t_wait1+t_wait2 timeout に到達(spoke_up 希少)");
}

// =============================================================================
// Main
// =============================================================================

fn main() {
    println!(
        "# Phase X Step 2 — Claim 36-41 (T_wait) + Claim 47-48 (sandwich) realistic benchmark\n"
    );
    part_a_run();
    part_b_run();

    println!();
    println!("## Overall interpretation");
    println!();
    println!("- **Part A** quantifies sandwich cost on the analogy discovery task");
    println!(
        "  - If canonical (0.70, 0.80) F1 < no-upper F1 → θ_U=0.80 rejects legitimate high-score analogies"
    );
    println!(
        "  - This generalizes F-041 (Hopfield falsification) from associative memory to analogy task"
    );
    println!("- **Part B** exercises the T_wait 2-stage review on realistic LoCoMo graph");
    println!(
        "  - answer-RARE spoke_up rate = KDF's cross-domain preservation of information-rich rare nodes"
    );
    println!("  - demote rate = fraction of RARE lost after 60 cycles without analogy");
}
