//! Custom similarity function example for KDF

use kdf::{euclidean_similarity, jaccard_similarity, Kdf};
use std::collections::HashSet;

/// Custom item type with tokens
#[derive(Clone)]
struct Document {
    id: String,
    tokens: HashSet<String>,
}

impl Document {
    fn new(id: &str, tokens: Vec<&str>) -> Self {
        Self {
            id: id.to_string(),
            tokens: tokens.into_iter().map(|s| s.to_string()).collect(),
        }
    }
}

fn main() {
    println!("=== KDF Custom Similarity Example ===\n");

    // Example 1: Euclidean similarity for vectors
    println!("--- Euclidean Similarity ---\n");

    let vectors = vec![
        vec![0.0, 0.0],
        vec![0.1, 0.1],
        vec![0.2, 0.2],
        vec![10.0, 10.0], // Outlier (rare)
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&vectors, 0.9, |a, b| euclidean_similarity(a, b));

    println!("Vectors: {} items", vectors.len());
    println!("Selected: {:?}", result.selected_indices());
    println!("Layers: {:?}", result.layers);

    // Example 2: Jaccard similarity for documents
    println!("\n--- Jaccard Similarity ---\n");

    let documents = vec![
        // Tech cluster
        Document::new("doc1", vec!["machine", "learning", "neural", "network"]),
        Document::new("doc2", vec!["machine", "learning", "deep", "network"]),
        Document::new("doc3", vec!["machine", "learning", "neural", "model"]),
        // Database cluster
        Document::new("doc4", vec!["database", "sql", "query", "index"]),
        Document::new("doc5", vec!["database", "sql", "table", "index"]),
        // Rare document
        Document::new("doc6", vec!["quantum", "physics", "entanglement"]),
    ];

    let result = kdf.process(&documents, 0.5, |a, b| {
        jaccard_similarity(&a.tokens, &b.tokens)
    });

    println!("Documents: {} items", documents.len());
    println!("Selected indices: {:?}", result.selected_indices());
    println!("\nSelected documents:");
    for &idx in result.selected_indices() {
        println!(
            "  - {} (layer: {:?})",
            documents[idx].id, result.layers[idx]
        );
    }

    // Example 3: Custom weighted similarity
    println!("\n--- Custom Weighted Similarity ---\n");

    struct WeightedItem {
        values: Vec<f64>,
        importance: f64,
    }

    let weighted_items = vec![
        WeightedItem {
            values: vec![1.0, 0.0],
            importance: 1.0,
        },
        WeightedItem {
            values: vec![1.0, 0.1],
            importance: 1.0,
        },
        WeightedItem {
            values: vec![0.0, 1.0],
            importance: 0.5,
        },
        WeightedItem {
            values: vec![-1.0, 0.0],
            importance: 2.0,
        }, // High importance
    ];

    // Custom similarity that considers importance
    let weighted_similarity = |a: &WeightedItem, b: &WeightedItem| -> f64 {
        let base_sim = euclidean_similarity(&a.values, &b.values);
        // Higher importance items are less similar to encourage preservation
        let importance_factor = 1.0 - (a.importance * b.importance).min(1.0) * 0.3;
        base_sim * importance_factor
    };

    let result = kdf.process(&weighted_items, 0.8, weighted_similarity);

    println!("Weighted items: {} items", weighted_items.len());
    println!("Selected indices: {:?}", result.selected_indices());
    println!("Layers: {:?}", result.layers);
}
