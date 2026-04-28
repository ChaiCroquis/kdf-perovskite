//! Test Case Selection with KDF
//!
//! This example demonstrates how to use KDF to optimize test execution:
//! - Group similar tests to avoid redundancy
//! - Prioritize edge cases (Rare tests)
//! - Reduce test suite execution time while maintaining coverage
//!
//! Run: cargo run --example kdf_test_selection

use kdf::{Kdf, Layer, cosine_similarity};

/// Represents a test case with its characteristics
#[derive(Clone)]
struct TestCase {
    name: String,
    /// Feature vector representing test characteristics:
    /// [code_path_hash, input_complexity, mutation_score, execution_time, coverage_area]
    features: Vec<f64>,
    execution_time_ms: u64,
    last_failure: Option<u64>, // Days since last failure
}

/// Test selection strategy
#[derive(Debug, Clone, Copy)]
enum SelectionStrategy {
    /// Run all selected tests
    Full,
    /// Run only Core representatives + all Rare
    Fast,
    /// Run Edge + Rare only (skip redundant Core)
    Focused,
}

/// Result of test selection
#[allow(dead_code)]
struct TestSelectionResult {
    /// Tests to run
    selected_tests: Vec<usize>,
    /// Estimated time savings
    time_saved_ms: u64,
    /// Coverage estimate (0-1)
    estimated_coverage: f64,
    /// Strategy used
    strategy: SelectionStrategy,
}

fn select_tests(
    tests: &[TestCase],
    strategy: SelectionStrategy,
    threshold: f64,
) -> TestSelectionResult {
    let kdf = Kdf::with_defaults();

    let result = kdf.process(tests, threshold, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    let mut selected_tests = Vec::new();
    let total_time: u64 = tests.iter().map(|t| t.execution_time_ms).sum();

    match strategy {
        SelectionStrategy::Full => {
            // Run all KDF-selected tests
            selected_tests = result.selected.clone();
        }
        SelectionStrategy::Fast => {
            // Core: only 1 representative per cluster
            // Edge + Rare: all
            for &idx in &result.selected {
                match result.layers[idx] {
                    Layer::Core => {
                        // Include if it's the first Core in its cluster
                        let cluster_id = result.clusters[idx];
                        let already_has_core = selected_tests.iter().any(|&i| {
                            result.layers[i] == Layer::Core && result.clusters[i] == cluster_id
                        });
                        if !already_has_core {
                            selected_tests.push(idx);
                        }
                    }
                    Layer::Edge | Layer::Rare => {
                        selected_tests.push(idx);
                    }
                }
            }
        }
        SelectionStrategy::Focused => {
            // Skip Core entirely, focus on Edge + Rare
            for &idx in &result.selected {
                if result.layers[idx] != Layer::Core {
                    selected_tests.push(idx);
                }
            }
            // Always include Rare even if not in selected
            for (i, &layer) in result.layers.iter().enumerate() {
                if layer == Layer::Rare && !selected_tests.contains(&i) {
                    selected_tests.push(i);
                }
            }
        }
    }

    let selected_time: u64 = selected_tests
        .iter()
        .map(|&i| tests[i].execution_time_ms)
        .sum();

    let time_saved_ms = total_time.saturating_sub(selected_time);

    // Estimate coverage based on layer distribution
    let rare_count = result.layers.iter().filter(|&&l| l == Layer::Rare).count();
    let selected_rare = selected_tests
        .iter()
        .filter(|&&i| result.layers[i] == Layer::Rare)
        .count();

    let rare_coverage = if rare_count > 0 {
        selected_rare as f64 / rare_count as f64
    } else {
        1.0
    };

    // Core coverage is assumed high if at least one per cluster
    let core_coverage = if !result.selected.is_empty() {
        selected_tests.len() as f64 / result.selected.len() as f64
    } else {
        1.0
    };

    let estimated_coverage = 0.7 * rare_coverage + 0.3 * core_coverage;

    TestSelectionResult {
        selected_tests,
        time_saved_ms,
        estimated_coverage,
        strategy,
    }
}

fn main() {
    println!("=== Test Case Selection with KDF ===\n");

    // Create a simulated test suite
    let tests = vec![
        // Unit tests for module A (similar)
        TestCase {
            name: "test_module_a_basic".into(),
            features: vec![1.0, 0.2, 0.5, 0.1, 0.8],
            execution_time_ms: 50,
            last_failure: None,
        },
        TestCase {
            name: "test_module_a_edge".into(),
            features: vec![1.0, 0.3, 0.5, 0.1, 0.8],
            execution_time_ms: 55,
            last_failure: Some(30),
        },
        TestCase {
            name: "test_module_a_null".into(),
            features: vec![1.0, 0.2, 0.6, 0.1, 0.8],
            execution_time_ms: 45,
            last_failure: None,
        },
        // Unit tests for module B (similar)
        TestCase {
            name: "test_module_b_basic".into(),
            features: vec![0.0, 1.0, 0.5, 0.2, 0.3],
            execution_time_ms: 80,
            last_failure: None,
        },
        TestCase {
            name: "test_module_b_complex".into(),
            features: vec![0.1, 1.0, 0.6, 0.3, 0.3],
            execution_time_ms: 120,
            last_failure: Some(7),
        },
        // Integration tests (unique patterns)
        TestCase {
            name: "test_integration_a_b".into(),
            features: vec![0.5, 0.5, 0.8, 0.5, 0.6],
            execution_time_ms: 500,
            last_failure: Some(3),
        },
        // Edge case tests (rare, important)
        TestCase {
            name: "test_edge_case_overflow".into(),
            features: vec![0.3, 0.3, 0.9, 0.8, 0.2],
            execution_time_ms: 200,
            last_failure: Some(1),
        },
        TestCase {
            name: "test_edge_case_unicode".into(),
            features: vec![0.2, 0.4, 0.95, 0.7, 0.1],
            execution_time_ms: 150,
            last_failure: None,
        },
        // Performance tests (expensive)
        TestCase {
            name: "test_perf_large_input".into(),
            features: vec![0.1, 0.9, 0.3, 0.9, 0.5],
            execution_time_ms: 2000,
            last_failure: None,
        },
        TestCase {
            name: "test_perf_stress".into(),
            features: vec![0.1, 0.95, 0.3, 0.95, 0.5],
            execution_time_ms: 3000,
            last_failure: None,
        },
    ];

    let total_time: u64 = tests.iter().map(|t| t.execution_time_ms).sum();
    println!("Total tests: {}", tests.len());
    println!("Total execution time: {}ms\n", total_time);

    // =========================================================================
    // 1. Analyze test suite with KDF
    // =========================================================================
    println!("--- Test Suite Analysis ---\n");

    let kdf = Kdf::with_defaults();
    let analysis = kdf.process(&tests, 0.85, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    for (i, test) in tests.iter().enumerate() {
        let layer = analysis.layers[i];
        let selected = if analysis.selected.contains(&i) {
            "*"
        } else {
            " "
        };
        println!(
            "{} [{:?}] {} ({}ms)",
            selected, layer, test.name, test.execution_time_ms
        );
    }
    println!();

    // =========================================================================
    // 2. Compare selection strategies
    // =========================================================================
    println!("--- Selection Strategies ---\n");

    for strategy in [
        SelectionStrategy::Full,
        SelectionStrategy::Fast,
        SelectionStrategy::Focused,
    ] {
        let result = select_tests(&tests, strategy, 0.85);

        println!("{:?} Strategy:", strategy);
        println!(
            "  Tests selected: {} / {}",
            result.selected_tests.len(),
            tests.len()
        );
        println!(
            "  Time saved: {}ms ({:.1}%)",
            result.time_saved_ms,
            100.0 * result.time_saved_ms as f64 / total_time as f64
        );
        println!("  Est. coverage: {:.1}%", result.estimated_coverage * 100.0);
        println!(
            "  Selected: {:?}",
            result
                .selected_tests
                .iter()
                .map(|&i| tests[i].name.as_str())
                .collect::<Vec<_>>()
        );
        println!();
    }

    // =========================================================================
    // 3. Prioritized execution order
    // =========================================================================
    println!("--- Prioritized Execution Order ---\n");

    let mut prioritized: Vec<(usize, i32)> = (0..tests.len())
        .map(|i| {
            let priority = match analysis.layers[i] {
                Layer::Rare => 100, // Highest priority
                Layer::Edge => 50,
                Layer::Core => 10,
            };
            // Boost recently failed tests
            let failure_boost = tests[i]
                .last_failure
                .map(|days| {
                    if days < 7 {
                        30
                    } else if days < 30 {
                        10
                    } else {
                        0
                    }
                })
                .unwrap_or(0);
            (i, priority + failure_boost)
        })
        .collect();

    prioritized.sort_by_key(|b| std::cmp::Reverse(b.1));

    println!("Recommended execution order:");
    for (rank, (idx, priority)) in prioritized.iter().enumerate() {
        println!(
            "  {}. {} (priority: {})",
            rank + 1,
            tests[*idx].name,
            priority
        );
    }
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("KDF Test Selection benefits:");
    println!("1. Reduce redundant test execution (similar tests grouped)");
    println!("2. Never skip edge cases (Rare layer = important)");
    println!("3. Prioritize recently failed tests");
    println!("4. Balance coverage vs execution time");
}
