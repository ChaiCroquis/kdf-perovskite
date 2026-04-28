//! Code Review Prioritization with KDF
//!
//! This example demonstrates how to prioritize code changes for review:
//! - Core: Routine changes (quick review)
//! - Edge: Moderate complexity (standard review)
//! - Rare: Unusual patterns (deep review required)
//!
//! Run: cargo run --example kdf_code_review

use kdf::{Kdf, Layer, cosine_similarity};

/// Represents a code change (diff hunk)
#[derive(Clone)]
struct CodeChange {
    file_path: String,
    /// Feature vector: [lines_added, lines_deleted, complexity_delta,
    ///                  is_security_sensitive, is_test, author_experience]
    features: Vec<f64>,
    description: String,
}

/// Review priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ReviewPriority {
    Critical = 0,
    High = 1,
    Medium = 2,
    Low = 3,
}

/// Review recommendation
struct ReviewRecommendation {
    change_idx: usize,
    priority: ReviewPriority,
    reason: String,
    suggested_reviewer: String,
    estimated_time_min: u32,
}

fn prioritize_reviews(changes: &[CodeChange], threshold: f64) -> Vec<ReviewRecommendation> {
    let kdf = Kdf::with_defaults();

    let result = kdf.process(changes, threshold, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    let mut recommendations = Vec::new();

    for (i, change) in changes.iter().enumerate() {
        let layer = result.layers[i];
        let _degree = result.degrees[i];

        // Determine priority based on layer and features
        let is_security = change.features[3] > 0.5;
        let is_large = change.features[0] + change.features[1] > 1.0;

        let (priority, reason) = match (layer, is_security, is_large) {
            (_, true, _) => (ReviewPriority::Critical, "Security-sensitive change"),
            (Layer::Rare, _, true) => (ReviewPriority::Critical, "Large unusual change"),
            (Layer::Rare, _, false) => (ReviewPriority::High, "Unusual pattern detected"),
            (Layer::Edge, _, true) => (ReviewPriority::High, "Significant edge case"),
            (Layer::Edge, _, false) => (ReviewPriority::Medium, "Moderate complexity"),
            (Layer::Core, _, _) => (ReviewPriority::Low, "Routine change"),
        };

        // Suggest reviewer based on file path
        let suggested_reviewer = if change.file_path.contains("security") {
            "security-team"
        } else if change.file_path.contains("test") {
            "qa-team"
        } else if layer == Layer::Rare {
            "senior-engineer"
        } else {
            "any-reviewer"
        };

        // Estimate review time
        let base_time = match priority {
            ReviewPriority::Critical => 30,
            ReviewPriority::High => 20,
            ReviewPriority::Medium => 10,
            ReviewPriority::Low => 5,
        };
        let size_factor = 1.0 + (change.features[0] + change.features[1]) * 0.5;
        let estimated_time = (base_time as f64 * size_factor) as u32;

        recommendations.push(ReviewRecommendation {
            change_idx: i,
            priority,
            reason: reason.into(),
            suggested_reviewer: suggested_reviewer.into(),
            estimated_time_min: estimated_time,
        });
    }

    // Sort by priority (Critical first)
    recommendations.sort_by_key(|a| a.priority);

    recommendations
}

fn main() {
    println!("=== Code Review Prioritization with KDF ===\n");

    // Simulated code changes in a PR
    let changes = vec![
        // Routine changes (Core)
        CodeChange {
            file_path: "src/utils/helpers.rs".into(),
            features: vec![0.1, 0.05, 0.1, 0.0, 0.0, 0.8], // Small, simple
            description: "Fix typo in helper function".into(),
        },
        CodeChange {
            file_path: "src/utils/format.rs".into(),
            features: vec![0.15, 0.1, 0.1, 0.0, 0.0, 0.7],
            description: "Update date formatting".into(),
        },
        CodeChange {
            file_path: "src/config/defaults.rs".into(),
            features: vec![0.1, 0.0, 0.05, 0.0, 0.0, 0.9],
            description: "Add new config option".into(),
        },
        // Moderate changes (Edge)
        CodeChange {
            file_path: "src/api/handlers.rs".into(),
            features: vec![0.4, 0.2, 0.3, 0.0, 0.0, 0.6],
            description: "Add new API endpoint".into(),
        },
        CodeChange {
            file_path: "src/database/queries.rs".into(),
            features: vec![0.3, 0.1, 0.4, 0.0, 0.0, 0.5],
            description: "Optimize database query".into(),
        },
        // Unusual changes (Rare)
        CodeChange {
            file_path: "src/security/auth.rs".into(),
            features: vec![0.5, 0.3, 0.6, 1.0, 0.0, 0.4], // Security sensitive!
            description: "Modify authentication flow".into(),
        },
        CodeChange {
            file_path: "src/core/engine.rs".into(),
            features: vec![0.8, 0.6, 0.8, 0.0, 0.0, 0.3], // Large, complex
            description: "Refactor core processing engine".into(),
        },
        CodeChange {
            file_path: "src/experimental/new_algo.rs".into(),
            features: vec![0.9, 0.0, 0.9, 0.0, 0.0, 0.2], // New code, junior dev
            description: "Implement experimental algorithm".into(),
        },
        // Test changes
        CodeChange {
            file_path: "tests/integration_test.rs".into(),
            features: vec![0.3, 0.1, 0.2, 0.0, 1.0, 0.7],
            description: "Add integration tests".into(),
        },
    ];

    println!("Total changes: {}\n", changes.len());

    // =========================================================================
    // 1. KDF Layer Analysis
    // =========================================================================
    println!("--- KDF Layer Analysis ---\n");

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&changes, 0.85, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    for (i, change) in changes.iter().enumerate() {
        let layer = result.layers[i];
        let icon = match layer {
            Layer::Rare => "🔴",
            Layer::Edge => "🟡",
            Layer::Core => "🟢",
        };
        println!("{} [{:?}] {}", icon, layer, change.file_path);
        println!("    {}", change.description);
    }
    println!();

    // =========================================================================
    // 2. Generate Review Recommendations
    // =========================================================================
    println!("--- Review Recommendations ---\n");

    let recommendations = prioritize_reviews(&changes, 0.85);
    let mut total_time = 0;

    for rec in &recommendations {
        let change = &changes[rec.change_idx];
        let icon = match rec.priority {
            ReviewPriority::Critical => "🚨",
            ReviewPriority::High => "⚠️ ",
            ReviewPriority::Medium => "📝",
            ReviewPriority::Low => "✓ ",
        };

        println!("{} {:?}: {}", icon, rec.priority, change.file_path);
        println!("    Reason: {}", rec.reason);
        println!("    Reviewer: {}", rec.suggested_reviewer);
        println!("    Est. time: {}min", rec.estimated_time_min);
        println!();

        total_time += rec.estimated_time_min;
    }

    // =========================================================================
    // 3. Review Summary
    // =========================================================================
    println!("--- Review Summary ---\n");

    let critical = recommendations
        .iter()
        .filter(|r| r.priority == ReviewPriority::Critical)
        .count();
    let high = recommendations
        .iter()
        .filter(|r| r.priority == ReviewPriority::High)
        .count();
    let medium = recommendations
        .iter()
        .filter(|r| r.priority == ReviewPriority::Medium)
        .count();
    let low = recommendations
        .iter()
        .filter(|r| r.priority == ReviewPriority::Low)
        .count();

    println!("Priority breakdown:");
    println!("  🚨 Critical: {}", critical);
    println!("  ⚠️  High: {}", high);
    println!("  📝 Medium: {}", medium);
    println!("  ✓  Low: {}", low);
    println!();
    println!("Total estimated review time: {} minutes", total_time);
    println!();

    // =========================================================================
    // 4. Parallel Review Suggestion
    // =========================================================================
    println!("--- Parallel Review Suggestion ---\n");

    println!("Assign to multiple reviewers:");
    println!("  Security Team: {} critical changes", critical);
    println!("  Senior Engineer: {} high-priority changes", high);
    println!("  Any Reviewer: {} routine changes", medium + low);
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("KDF Code Review benefits:");
    println!("1. Automatically detect unusual code patterns");
    println!("2. Prioritize security-sensitive changes");
    println!("3. Estimate review time based on complexity");
    println!("4. Assign appropriate reviewers");
    println!("5. Reduce review fatigue on routine changes");
}
