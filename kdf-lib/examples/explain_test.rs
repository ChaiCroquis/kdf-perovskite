//! Explanation generation demonstration
use kdf::{Kdf, cosine_similarity};

fn main() {
    println!("=== KDF Explanation Generation ===\n");

    // Create test dataset
    let items = vec![
        vec![1.0, 0.0, 0.0],    // 0: Cluster A representative
        vec![0.98, 0.1, 0.0],   // 1: Cluster A member
        vec![0.95, 0.15, 0.0],  // 2: Cluster A member
        vec![0.0, 1.0, 0.0],    // 3: Cluster B representative
        vec![0.05, 0.98, 0.0],  // 4: Cluster B member
        vec![-1.0, 0.0, 0.0],   // 5: Rare item
        vec![0.5, 0.5, 0.5],    // 6: Edge item
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&items, 0.9, |a, b| cosine_similarity(a, b));

    // ========================================================================
    // 1. Summary
    // ========================================================================
    println!("## 1. Processing Summary\n");
    println!("{}", result.summary());

    // ========================================================================
    // 2. Individual Explanations
    // ========================================================================
    println!("\n## 2. Individual Item Explanations\n");

    for i in 0..items.len() {
        println!("{}", result.explain(i));
    }

    // ========================================================================
    // 3. Short Explanations
    // ========================================================================
    println!("## 3. Short Explanations\n");

    for i in 0..items.len() {
        println!("   Item {}: {}", i, result.explain_short(i));
    }

    // ========================================================================
    // 4. Use Case: Data Audit Report
    // ========================================================================
    println!("\n## 4. Use Case: Data Audit Report\n");

    let documents = vec![
        "Machine learning improves accuracy",
        "Deep learning improves accuracy",
        "AI improves model accuracy",
        "Natural language processing overview",
        "Quantum computing introduction",  // Rare
    ];

    let result = kdf.process(&documents, 0.5, |a, b| kdf::levenshtein_similarity(a, b));

    println!("   Document Processing Report:");
    println!("   ----------------------------");
    for (i, doc) in documents.iter().enumerate() {
        let status = if result.is_selected(i) { "✓" } else { "✗" };
        let reason = result.explain_short(i);
        println!("   {} [{}] \"{}\"", status, i, doc);
        println!("      └─ {}", reason);
    }

    let stats = result.stats();
    println!("\n   Audit Summary:");
    println!("   - Total documents: {}", stats.total_items);
    println!("   - Unique documents: {}", stats.selected_count);
    println!("   - Redundancy: {:.1}%", stats.redundancy_ratio * 100.0);

    println!("\n✅ Explanation Generation 正常動作");
}
