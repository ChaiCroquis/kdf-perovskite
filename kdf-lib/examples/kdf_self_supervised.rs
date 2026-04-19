//! Self-Supervised Learning Integration with KDF
//!
//! This example demonstrates how to use KDF to improve contrastive learning:
//! - Better negative sampling using layer structure
//! - Hard negative mining from Edge layer
//! - Curriculum learning based on item difficulty
//!
//! Run: cargo run --example kdf_self_supervised

use kdf::{cosine_similarity, Kdf, Layer};
use std::collections::HashMap;

/// Representation learning sample
#[derive(Clone)]
struct Sample {
    embedding: Vec<f64>,
    augmentation_id: usize,  // Samples with same ID are positive pairs
}

/// Contrastive learning batch with KDF-guided negative selection
struct KdfContrastiveBatch {
    anchors: Vec<usize>,
    positives: Vec<usize>,
    hard_negatives: Vec<Vec<usize>>,  // Per anchor
    layer_distribution: HashMap<String, usize>,
}

fn main() {
    println!("=== Self-Supervised Learning with KDF ===\n");

    let kdf = Kdf::with_defaults();

    // =========================================================================
    // 1. Create embeddings with augmentations
    // =========================================================================
    println!("--- Contrastive Learning Setup ---\n");

    // Simulated embeddings (in practice, from a neural network)
    // Same augmentation_id = positive pair
    let samples = vec![
        // Group 1: Similar embeddings (same semantic content)
        Sample { embedding: vec![1.0, 0.0, 0.0, 0.0], augmentation_id: 0 },
        Sample { embedding: vec![0.95, 0.05, 0.0, 0.0], augmentation_id: 0 },  // Aug of 0

        Sample { embedding: vec![0.0, 1.0, 0.0, 0.0], augmentation_id: 1 },
        Sample { embedding: vec![0.05, 0.95, 0.0, 0.0], augmentation_id: 1 },  // Aug of 2

        // Group 2: Different semantic content
        Sample { embedding: vec![0.0, 0.0, 1.0, 0.0], augmentation_id: 2 },
        Sample { embedding: vec![0.0, 0.0, 0.9, 0.1], augmentation_id: 2 },  // Aug of 4

        // Hard negatives: Similar embedding but different semantic
        Sample { embedding: vec![0.7, 0.3, 0.0, 0.0], augmentation_id: 3 },  // Close to 0 but different
        Sample { embedding: vec![0.3, 0.7, 0.0, 0.0], augmentation_id: 4 },  // Close to 2 but different

        // Easy negatives: Very different
        Sample { embedding: vec![0.0, 0.0, 0.0, 1.0], augmentation_id: 5 },
    ];

    println!("Total samples: {}", samples.len());
    println!("Augmentation groups: {} unique",
        samples.iter().map(|s| s.augmentation_id).collect::<std::collections::HashSet<_>>().len());
    println!();

    // =========================================================================
    // 2. KDF Layer Analysis
    // =========================================================================
    println!("--- KDF Layer Analysis ---\n");

    let result = kdf.process(&samples, 0.9, |a, b| {
        cosine_similarity(&a.embedding, &b.embedding)
    });

    println!("Layers: {:?}", result.layers);
    println!("Degrees: {:?}", result.degrees);
    println!();

    for (i, sample) in samples.iter().enumerate() {
        let layer = result.layers[i];
        let degree = result.degrees[i];
        println!("  {} [aug={}]: {:?} (degree={})",
            i, sample.augmentation_id, layer, degree);
    }
    println!();

    // =========================================================================
    // 3. KDF-Guided Negative Sampling
    // =========================================================================
    println!("--- KDF-Guided Negative Mining ---\n");

    // Strategy: Use Edge layer items as hard negatives
    // They are similar enough to be challenging but still different
    let edge_items: Vec<usize> = (0..samples.len())
        .filter(|&i| result.layers[i] == Layer::Edge)
        .collect();

    let rare_items: Vec<usize> = (0..samples.len())
        .filter(|&i| result.layers[i] == Layer::Rare)
        .collect();

    let core_items: Vec<usize> = (0..samples.len())
        .filter(|&i| result.layers[i] == Layer::Core)
        .collect();

    println!("Hard negatives (Edge): {:?}", edge_items);
    println!("Easy negatives (Core): {:?}", core_items);
    println!("Avoid (Rare): {:?}", rare_items);
    println!();

    // =========================================================================
    // 4. Contrastive Batch Construction
    // =========================================================================
    println!("--- Contrastive Batch Example ---\n");

    // For each anchor, find positive and negatives
    let anchor_idx = 0;
    let anchor = &samples[anchor_idx];

    // Positive: same augmentation_id
    let positive_idx = samples.iter()
        .position(|s| s.augmentation_id == anchor.augmentation_id && std::ptr::eq(s, anchor) == false)
        .unwrap_or(anchor_idx);

    // Hard negatives: Edge items with different augmentation_id
    let hard_negatives: Vec<usize> = edge_items.iter()
        .filter(|&&i| samples[i].augmentation_id != anchor.augmentation_id)
        .cloned()
        .collect();

    // Easy negatives: Core items with different augmentation_id
    let easy_negatives: Vec<usize> = core_items.iter()
        .filter(|&&i| samples[i].augmentation_id != anchor.augmentation_id)
        .cloned()
        .collect();

    println!("Anchor: {} (aug={})", anchor_idx, anchor.augmentation_id);
    println!("Positive: {} (aug={})", positive_idx, samples[positive_idx].augmentation_id);
    println!("Hard negatives: {:?}", hard_negatives);
    println!("Easy negatives: {:?}", easy_negatives);
    println!();

    // =========================================================================
    // 5. Curriculum Learning
    // =========================================================================
    println!("--- Curriculum Learning with KDF ---\n");

    println!("Training curriculum based on layer difficulty:");
    println!();
    println!("Phase 1 (Easy): Use Core negatives only");
    println!("  - Clear distinction between positive/negative");
    println!("  - Model learns basic separation");
    println!();
    println!("Phase 2 (Medium): Add Edge negatives");
    println!("  - Harder negatives challenge the model");
    println!("  - Fine-grained discrimination");
    println!();
    println!("Phase 3 (Hard): Include difficult Edge cases");
    println!("  - Samples with high similarity but different semantics");
    println!("  - Model learns subtle differences");
    println!();
    println!("Avoid: Rare samples as negatives");
    println!("  - Too unique, may confuse the model");
    println!("  - Could represent outliers or noise");
    println!();

    // =========================================================================
    // 6. Benefits Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("KDF for Contrastive Learning:");
    println!("1. Hard negative mining: Edge layer provides challenging negatives");
    println!("2. Curriculum: Progress from Core (easy) to Edge (hard) negatives");
    println!("3. Noise filtering: Rare samples may be outliers - avoid as negatives");
    println!("4. Batch balance: Use KDF to ensure diverse negative sampling");
    println!("5. Sample efficiency: Focus training on informative samples");
}
