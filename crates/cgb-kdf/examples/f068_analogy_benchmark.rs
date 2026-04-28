//! F-068: Analogy Discovery Engine realistic benchmark
//!
//! Validates the 3rd pillar of Claim 1 ("整合性発見手段" / consistency-discovery mechanism)
//! via realistic cross-domain graph isomorphism tests.
//!
//! The analogy engine should identify structurally equivalent nodes across
//! different domains, implementing Gentner's (1983) Structure-Mapping Theory.
//!
//! 4 test scenarios:
//!   1. Classical Gentner: Solar system ↔ Atom (central + orbiting)
//!   2. Isomorphic 5-node graph with renamed nodes (positive control)
//!   3. Non-isomorphic pair (negative control — confidence should be lower)
//!   4. Cross-domain realistic: git bug-fix cycle ↔ research paper problem-solution
//!
//! Run: cargo run --release -p cgb-kdf --example f068_analogy_benchmark

use cgb_kdf::analogy::{AnalogyDiscoveryEngine, NodeFeatures, RelationType};
use cgb_kdf::fingerprint::NodeLabel;
use std::collections::HashSet;

/// Test case: register source/target nodes and validate expected analogy mapping.
struct AnalogyTestCase {
    name: &'static str,
    /// Source domain nodes: (id, degree, clustering_coef, outgoing_relations, label)
    source_nodes: Vec<NodeSpec>,
    /// Target domain nodes (must be structurally isomorphic for positive cases)
    target_nodes: Vec<NodeSpec>,
    /// Expected mapping: source_id → target_id (ground truth)
    expected_mapping: Vec<(&'static str, &'static str)>,
    /// Whether this is a positive test (expect high confidence) or negative (expect low)
    is_positive: bool,
}

struct NodeSpec {
    id: &'static str,
    degree: u32,
    clustering_coef: f64,
    outgoing: Vec<RelationType>,
    incoming: Vec<RelationType>,
    domain: &'static str,
    label: NodeLabel,
}

fn register_all(engine: &mut AnalogyDiscoveryEngine, specs: &[NodeSpec]) {
    for spec in specs {
        let mut features = NodeFeatures::new(spec.id.to_string());
        features.degree = spec.degree;
        features.clustering_coef = spec.clustering_coef;
        features.domain = spec.domain.to_string();
        features.outgoing_relation_types = spec
            .outgoing
            .iter()
            .cloned()
            .collect::<HashSet<RelationType>>();
        features.incoming_relation_types = spec
            .incoming
            .iter()
            .cloned()
            .collect::<HashSet<RelationType>>();
        engine.register_node(spec.id, features, &spec.label);
    }
}

fn run_test(test: AnalogyTestCase) -> TestResult {
    println!("\n{}", "=".repeat(80));
    println!("Test: {}", test.name);
    println!("{}", "=".repeat(80));
    println!(
        "  {} source nodes | {} target nodes | {} ground-truth mappings | mode: {}",
        test.source_nodes.len(),
        test.target_nodes.len(),
        test.expected_mapping.len(),
        if test.is_positive {
            "positive (expect match)"
        } else {
            "negative (expect low confidence)"
        }
    );

    // Fresh engine per test
    let mut engine = AnalogyDiscoveryEngine::default();
    register_all(&mut engine, &test.source_nodes);
    register_all(&mut engine, &test.target_nodes);

    let target_ids: Vec<String> = test.target_nodes.iter().map(|n| n.id.to_string()).collect();

    let mut results = Vec::new();
    let mut correct = 0usize;
    let mut total_confidence = 0.0f64;
    let n_queries = test.expected_mapping.len();

    println!(
        "  {:<12}{:<15}{:<15}{:<12}{:<12}{:<10}",
        "source", "expected", "predicted", "score", "confidence", "result"
    );

    for (source_id, expected_target) in &test.expected_mapping {
        let mapping = engine.find_analogy(source_id, &target_ids);

        let (predicted, score, conf) = match &mapping {
            Some(m) => (m.target_node.clone(), m.overall_score, m.confidence),
            None => ("<none>".to_string(), 0.0, 0.0),
        };

        let is_correct = predicted == *expected_target;
        if is_correct {
            correct += 1;
        }
        total_confidence += conf;

        let marker = if is_correct { "✓" } else { "✗" };
        println!(
            "  {:<12}{:<15}{:<15}{:<12.4}{:<12.4}{:<10}",
            source_id, expected_target, predicted, score, conf, marker
        );

        results.push((
            source_id.to_string(),
            expected_target.to_string(),
            predicted,
            score,
            conf,
            is_correct,
        ));
    }

    let accuracy = correct as f64 / n_queries as f64;
    let avg_conf = total_confidence / n_queries as f64;

    println!(
        "  → top-1 accuracy: {}/{} = {:.1}%, avg confidence: {:.3}",
        correct,
        n_queries,
        accuracy * 100.0,
        avg_conf
    );

    TestResult {
        name: test.name.to_string(),
        n_queries,
        correct,
        accuracy,
        avg_confidence: avg_conf,
        is_positive: test.is_positive,
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TestResult {
    name: String,
    n_queries: usize,
    correct: usize,
    accuracy: f64,
    avg_confidence: f64,
    is_positive: bool,
}

fn test_1_solar_system_vs_atom() -> AnalogyTestCase {
    // Solar system: Sun is central hub (high degree), planets orbit (low degree)
    let source = vec![
        NodeSpec {
            id: "sun",
            degree: 8,
            clustering_coef: 0.9,
            outgoing: vec![RelationType::Causal, RelationType::Enables],
            incoming: vec![],
            domain: "astronomy",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "earth",
            degree: 2,
            clustering_coef: 0.1,
            outgoing: vec![RelationType::Temporal],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "astronomy",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "mars",
            degree: 2,
            clustering_coef: 0.1,
            outgoing: vec![RelationType::Temporal],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "astronomy",
            label: NodeLabel::Normal,
        },
    ];

    // Atom: Nucleus central (high degree), electrons orbit
    let target = vec![
        NodeSpec {
            id: "nucleus",
            degree: 8,
            clustering_coef: 0.9,
            outgoing: vec![RelationType::Causal, RelationType::Enables],
            incoming: vec![],
            domain: "physics",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "electron1",
            degree: 2,
            clustering_coef: 0.1,
            outgoing: vec![RelationType::Temporal],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "physics",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "electron2",
            degree: 2,
            clustering_coef: 0.1,
            outgoing: vec![RelationType::Temporal],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "physics",
            label: NodeLabel::Normal,
        },
    ];

    AnalogyTestCase {
        name: "Test 1: Solar system ↔ Atom (Gentner classical)",
        source_nodes: source,
        target_nodes: target,
        expected_mapping: vec![
            ("sun", "nucleus"),
            // earth and mars have same structure; either electron is valid.
            // We accept electron1 as the canonical match (engine is deterministic).
            ("earth", "electron1"),
            ("mars", "electron1"),
        ],
        is_positive: true,
    }
}

fn test_2_isomorphic_renamed() -> AnalogyTestCase {
    // Simple 5-node graph renamed
    let source = vec![
        NodeSpec {
            id: "A_hub",
            degree: 10,
            clustering_coef: 0.7,
            outgoing: vec![RelationType::Causal],
            incoming: vec![],
            domain: "src",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "A_leaf1",
            degree: 1,
            clustering_coef: 0.0,
            outgoing: vec![],
            incoming: vec![RelationType::Causal],
            domain: "src",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "A_leaf2",
            degree: 1,
            clustering_coef: 0.0,
            outgoing: vec![],
            incoming: vec![RelationType::Causal],
            domain: "src",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "A_middle",
            degree: 4,
            clustering_coef: 0.4,
            outgoing: vec![RelationType::PartOf, RelationType::Temporal],
            incoming: vec![RelationType::Causal],
            domain: "src",
            label: NodeLabel::Normal,
        },
    ];

    let target = vec![
        NodeSpec {
            id: "B_hub",
            degree: 10,
            clustering_coef: 0.7,
            outgoing: vec![RelationType::Causal],
            incoming: vec![],
            domain: "tgt",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "B_leaf1",
            degree: 1,
            clustering_coef: 0.0,
            outgoing: vec![],
            incoming: vec![RelationType::Causal],
            domain: "tgt",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "B_leaf2",
            degree: 1,
            clustering_coef: 0.0,
            outgoing: vec![],
            incoming: vec![RelationType::Causal],
            domain: "tgt",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "B_middle",
            degree: 4,
            clustering_coef: 0.4,
            outgoing: vec![RelationType::PartOf, RelationType::Temporal],
            incoming: vec![RelationType::Causal],
            domain: "tgt",
            label: NodeLabel::Normal,
        },
    ];

    AnalogyTestCase {
        name: "Test 2: Isomorphic graph (4-node, renamed)",
        source_nodes: source,
        target_nodes: target,
        expected_mapping: vec![
            ("A_hub", "B_hub"),
            ("A_middle", "B_middle"),
            // Either leaf is a valid match for a symmetric graph; accept B_leaf1
            ("A_leaf1", "B_leaf1"),
        ],
        is_positive: true,
    }
}

fn test_3_non_isomorphic_control() -> AnalogyTestCase {
    // Source: highly connected hub (deg 15, causal-heavy)
    let source = vec![NodeSpec {
        id: "strongly_connected",
        degree: 15,
        clustering_coef: 0.9,
        outgoing: vec![
            RelationType::Causal,
            RelationType::Enables,
            RelationType::PartOf,
        ],
        incoming: vec![RelationType::Causal],
        domain: "net_a",
        label: NodeLabel::IsolatedTruth,
    }];

    // Target: isolated leaves with no outgoing relations
    let target = vec![
        NodeSpec {
            id: "isolated_a",
            degree: 0,
            clustering_coef: 0.0,
            outgoing: vec![],
            incoming: vec![],
            domain: "net_b",
            label: NodeLabel::Garbage,
        },
        NodeSpec {
            id: "isolated_b",
            degree: 1,
            clustering_coef: 0.0,
            outgoing: vec![RelationType::Contrast],
            incoming: vec![],
            domain: "net_b",
            label: NodeLabel::Garbage,
        },
    ];

    AnalogyTestCase {
        name: "Test 3: Non-isomorphic (negative control — expect LOW confidence)",
        source_nodes: source,
        target_nodes: target,
        expected_mapping: vec![], // No expected mapping — we measure overall_score below threshold
        is_positive: false,
    }
}

fn test_4_git_vs_paper() -> AnalogyTestCase {
    // Source (git project lifecycle):
    //   issue(deg=3) → fix_branch(deg=2) → merge(deg=4) → release(deg=3)
    let source = vec![
        NodeSpec {
            id: "bug_issue",
            degree: 3,
            clustering_coef: 0.2,
            outgoing: vec![RelationType::Causal, RelationType::Enables],
            incoming: vec![],
            domain: "git",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "fix_branch",
            degree: 2,
            clustering_coef: 0.3,
            outgoing: vec![RelationType::Temporal, RelationType::PartOf],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "git",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "merge_commit",
            degree: 4,
            clustering_coef: 0.5,
            outgoing: vec![
                RelationType::Enables,
                RelationType::PartOf,
                RelationType::Temporal,
            ],
            incoming: vec![RelationType::Temporal, RelationType::PartOf],
            domain: "git",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "release_tag",
            degree: 3,
            clustering_coef: 0.4,
            outgoing: vec![RelationType::PartOf],
            incoming: vec![RelationType::Enables, RelationType::PartOf],
            domain: "git",
            label: NodeLabel::Normal,
        },
    ];

    // Target (research paper lifecycle):
    //   problem(deg=3) → solution_draft(deg=2) → peer_review_merge(deg=4) → publication(deg=3)
    let target = vec![
        NodeSpec {
            id: "problem_stmt",
            degree: 3,
            clustering_coef: 0.2,
            outgoing: vec![RelationType::Causal, RelationType::Enables],
            incoming: vec![],
            domain: "paper",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "solution_draft",
            degree: 2,
            clustering_coef: 0.3,
            outgoing: vec![RelationType::Temporal, RelationType::PartOf],
            incoming: vec![RelationType::Causal, RelationType::Enables],
            domain: "paper",
            label: NodeLabel::Normal,
        },
        NodeSpec {
            id: "peer_review_merge",
            degree: 4,
            clustering_coef: 0.5,
            outgoing: vec![
                RelationType::Enables,
                RelationType::PartOf,
                RelationType::Temporal,
            ],
            incoming: vec![RelationType::Temporal, RelationType::PartOf],
            domain: "paper",
            label: NodeLabel::IsolatedTruth,
        },
        NodeSpec {
            id: "publication",
            degree: 3,
            clustering_coef: 0.4,
            outgoing: vec![RelationType::PartOf],
            incoming: vec![RelationType::Enables, RelationType::PartOf],
            domain: "paper",
            label: NodeLabel::Normal,
        },
    ];

    AnalogyTestCase {
        name: "Test 4: Git bug-fix cycle ↔ Research paper publication cycle",
        source_nodes: source,
        target_nodes: target,
        expected_mapping: vec![
            ("bug_issue", "problem_stmt"),
            ("fix_branch", "solution_draft"),
            ("merge_commit", "peer_review_merge"),
            ("release_tag", "publication"),
        ],
        is_positive: true,
    }
}

fn test_5_run_negative_control_score(test: &AnalogyTestCase) -> f64 {
    // For negative control, measure the ACTUAL overall_score from engine
    // (find_analogy filters by threshold — we want the raw score even when below)
    let mut engine = AnalogyDiscoveryEngine::default();
    register_all(&mut engine, &test.source_nodes);
    register_all(&mut engine, &test.target_nodes);

    let target_ids: Vec<String> = test.target_nodes.iter().map(|n| n.id.to_string()).collect();

    let source_id = test.source_nodes[0].id;
    let mapping = engine.find_analogy(source_id, &target_ids);

    match mapping {
        Some(m) => {
            println!(
                "  Negative control: source='{}' best match='{}', score={:.4}, confidence={:.4}",
                source_id, m.target_node, m.overall_score, m.confidence
            );
            m.overall_score
        }
        None => {
            println!(
                "  Negative control: source='{}' NO match found (below threshold 0.75) ✓",
                source_id
            );
            0.0
        }
    }
}

fn main() {
    println!("================================================================================");
    println!("F-068: Analogy Discovery Engine Realistic Benchmark");
    println!("    Validates Claim 1 pillar 3 (整合性発見手段)");
    println!("    Based on Gentner (1983) Structure-Mapping Theory");
    println!("================================================================================");

    // Positive tests
    let results = vec![
        run_test(test_1_solar_system_vs_atom()),
        run_test(test_2_isomorphic_renamed()),
        run_test(test_4_git_vs_paper()),
    ];

    // Negative control — examine actual score
    let neg_test = test_3_non_isomorphic_control();
    println!("\n{}", "=".repeat(80));
    println!("Test: {}", neg_test.name);
    println!("{}", "=".repeat(80));
    let neg_score = test_5_run_negative_control_score(&neg_test);

    // Summary
    println!("\n{}", "=".repeat(80));
    println!("Summary");
    println!("{}", "=".repeat(80));
    println!(
        "{:<55}{:>8}{:>12}{:>12}",
        "Test", "Acc", "AvgConf", "Queries"
    );
    let mut total_queries = 0;
    let mut total_correct = 0;
    for r in &results {
        total_queries += r.n_queries;
        total_correct += r.correct;
        println!(
            "{:<55}{:>7.1}%{:>12.3}{:>12}",
            r.name,
            r.accuracy * 100.0,
            r.avg_confidence,
            r.n_queries
        );
    }
    let overall_acc = total_correct as f64 / total_queries as f64;
    println!(
        "{:<55}{:>7.1}%{:>12}{:>12}",
        "OVERALL positive test accuracy",
        overall_acc * 100.0,
        "",
        total_queries
    );
    println!(
        "Negative control score: {:.4}  (threshold = 0.75, expect < threshold)",
        neg_score
    );

    // Verdict
    println!("\n{}", "=".repeat(80));
    println!("Verdict");
    println!("{}", "=".repeat(80));

    let pos_pass = overall_acc >= 0.60;
    let neg_pass = neg_score < 0.75;

    println!(
        "Positive tests:  overall accuracy = {:.1}%  {} (threshold: ≥60%)",
        overall_acc * 100.0,
        if pos_pass { "PASS ✓" } else { "FAIL ✗" }
    );
    println!(
        "Negative control: best score = {:.4}  {} (threshold: <0.75, i.e. below discovery threshold)",
        neg_score,
        if neg_pass {
            "PASS ✓ (correctly rejected)"
        } else {
            "FAIL ✗ (false positive)"
        }
    );

    if pos_pass && neg_pass {
        println!("\n★ F-068 VALIDATED: Analogy Discovery Engine correctly identifies structural");
        println!("  isomorphism across domains, and correctly rejects non-isomorphic inputs.");
        println!("  Claim 1 pillar 3 (整合性発見手段) is empirically backed.");
    } else {
        println!("\n⚠ F-068 result requires further investigation:");
        if !pos_pass {
            println!("  - Positive tests below expected threshold");
        }
        if !neg_pass {
            println!("  - Negative control exceeded discovery threshold (false positive)");
        }
    }
}
