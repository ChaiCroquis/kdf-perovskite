//! Adversarial Sample Detection with KDF
//!
//! This example demonstrates how to use KDF to detect potential adversarial samples:
//! - Samples with unusual similarity patterns
//! - Samples that don't fit well into any cluster
//! - Potential perturbations in ML input data
//!
//! Key insight: Adversarial samples often become Rare in KDF
//! because they're designed to be close to decision boundaries.
//!
//! Run: cargo run --example kdf_adversarial

use kdf::{cosine_similarity, Kdf, Layer};

/// Sample with features and ground truth label
#[derive(Clone)]
struct Sample {
    features: Vec<f64>,
    label: usize,
    is_adversarial: bool,
}

/// Adversarial detection result
struct AdversarialAnalysis {
    suspicious_indices: Vec<usize>,
    confidence_scores: Vec<f64>,
    layer_distribution: (usize, usize, usize),
}

/// Detect potential adversarial samples using KDF
fn detect_adversarial(samples: &[Sample], threshold: f64) -> AdversarialAnalysis {
    let kdf = Kdf::with_defaults();

    // Apply KDF
    let result = kdf.process(samples, threshold, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    // Rare items are suspicious
    let _rare_indices = result.rare_items();

    // Calculate suspicion scores based on multiple factors
    let mut confidence_scores = vec![0.0; samples.len()];

    for i in 0..samples.len() {
        let layer = result.layers[i];
        let weight = result.selection_scores[i];
        let degree = result.degrees[i];

        // Base suspicion from layer
        let layer_score = match layer {
            Layer::Rare => 0.8,
            Layer::Edge => 0.4,
            Layer::Core => 0.1,
        };

        // Low degree = more suspicious
        let degree_score = if degree == 0 {
            0.9
        } else if degree < 3 {
            0.5
        } else {
            0.2
        };

        // Low weight = more suspicious
        let weight_score = 1.0 - weight.min(1.0);

        // Combined score
        confidence_scores[i] = 0.4 * layer_score + 0.3 * degree_score + 0.3 * weight_score;
    }

    // Collect highly suspicious samples
    let suspicious_threshold = 0.6;
    let suspicious_indices: Vec<usize> = (0..samples.len())
        .filter(|&i| confidence_scores[i] > suspicious_threshold)
        .collect();

    let core = result.layers.iter().filter(|&&l| l == Layer::Core).count();
    let edge = result.layers.iter().filter(|&&l| l == Layer::Edge).count();
    let rare = result.layers.iter().filter(|&&l| l == Layer::Rare).count();

    AdversarialAnalysis {
        suspicious_indices,
        confidence_scores,
        layer_distribution: (core, edge, rare),
    }
}

/// Generate a simple adversarial perturbation
fn add_perturbation(sample: &Sample, epsilon: f64, direction: &[f64]) -> Sample {
    let perturbed_features: Vec<f64> = sample.features.iter()
        .zip(direction.iter())
        .map(|(&f, &d)| f + epsilon * d)
        .collect();

    Sample {
        features: perturbed_features,
        label: sample.label,
        is_adversarial: true,
    }
}

fn main() {
    println!("=== Adversarial Detection with KDF ===\n");

    // Create clean dataset
    let mut samples: Vec<Sample> = vec![
        // Class 0: Dense cluster
        Sample { features: vec![1.0, 0.0, 0.0], label: 0, is_adversarial: false },
        Sample { features: vec![1.0, 0.1, 0.0], label: 0, is_adversarial: false },
        Sample { features: vec![1.0, 0.0, 0.1], label: 0, is_adversarial: false },
        Sample { features: vec![0.9, 0.1, 0.1], label: 0, is_adversarial: false },

        // Class 1: Another cluster
        Sample { features: vec![0.0, 1.0, 0.0], label: 1, is_adversarial: false },
        Sample { features: vec![0.1, 1.0, 0.0], label: 1, is_adversarial: false },
        Sample { features: vec![0.0, 0.9, 0.1], label: 1, is_adversarial: false },

        // Class 2: Sparse cluster
        Sample { features: vec![0.0, 0.0, 1.0], label: 2, is_adversarial: false },
        Sample { features: vec![0.0, 0.2, 0.8], label: 2, is_adversarial: false },
    ];

    // =========================================================================
    // 1. Clean Data Analysis
    // =========================================================================
    println!("--- Clean Data Analysis ---\n");

    let clean_analysis = detect_adversarial(&samples, 0.85);

    println!("Layer distribution (clean): Core={}, Edge={}, Rare={}",
        clean_analysis.layer_distribution.0,
        clean_analysis.layer_distribution.1,
        clean_analysis.layer_distribution.2);

    println!("Suspicious samples (clean): {:?}", clean_analysis.suspicious_indices);
    println!();

    // =========================================================================
    // 2. Inject Adversarial Samples
    // =========================================================================
    println!("--- Injecting Adversarial Samples ---\n");

    // Adversarial: designed to be between class 0 and class 1
    let adv1 = Sample {
        features: vec![0.5, 0.5, 0.0],  // Between class 0 and 1
        label: 0,  // Mislabeled as class 0
        is_adversarial: true,
    };

    // Adversarial: small perturbation from class 0 toward class 1
    let adv2 = add_perturbation(&samples[0], 0.4, &[-0.4, 0.4, 0.0]);

    // Adversarial: outlier disguised as class 2
    let adv3 = Sample {
        features: vec![0.3, 0.3, 0.4],  // Unusual pattern
        label: 2,
        is_adversarial: true,
    };

    samples.push(adv1);
    samples.push(adv2);
    samples.push(adv3);

    println!("Added 3 adversarial samples at indices 9, 10, 11");
    println!();

    // =========================================================================
    // 3. Detect with KDF
    // =========================================================================
    println!("--- KDF Adversarial Detection ---\n");

    let analysis = detect_adversarial(&samples, 0.85);

    println!("Layer distribution: Core={}, Edge={}, Rare={}",
        analysis.layer_distribution.0,
        analysis.layer_distribution.1,
        analysis.layer_distribution.2);
    println!();

    println!("Sample analysis:");
    for (i, sample) in samples.iter().enumerate() {
        let score = analysis.confidence_scores[i];
        let flag = if analysis.suspicious_indices.contains(&i) { "SUSPICIOUS" } else { "" };
        let adv = if sample.is_adversarial { "[ADV]" } else { "     " };

        println!("  {} {:2}: score={:.2} {} {}",
            adv, i, score, flag.to_string(), format!("{:?}", sample.features));
    }
    println!();

    // =========================================================================
    // 4. Evaluation
    // =========================================================================
    println!("--- Detection Evaluation ---\n");

    let true_adversarial: Vec<usize> = samples.iter().enumerate()
        .filter(|(_, s)| s.is_adversarial)
        .map(|(i, _)| i)
        .collect();

    let detected: &Vec<usize> = &analysis.suspicious_indices;

    let true_positives: usize = detected.iter()
        .filter(|&i| true_adversarial.contains(i))
        .count();

    let false_positives: usize = detected.iter()
        .filter(|&i| !true_adversarial.contains(i))
        .count();

    let false_negatives: usize = true_adversarial.iter()
        .filter(|&i| !detected.contains(i))
        .count();

    println!("True Adversarial: {:?}", true_adversarial);
    println!("Detected: {:?}", detected);
    println!();
    println!("True Positives: {}", true_positives);
    println!("False Positives: {}", false_positives);
    println!("False Negatives: {}", false_negatives);

    let precision = if true_positives + false_positives > 0 {
        true_positives as f64 / (true_positives + false_positives) as f64
    } else { 0.0 };

    let recall = if true_positives + false_negatives > 0 {
        true_positives as f64 / (true_positives + false_negatives) as f64
    } else { 0.0 };

    println!("Precision: {:.2}", precision);
    println!("Recall: {:.2}", recall);
    println!();

    // =========================================================================
    // 5. Insights
    // =========================================================================
    println!("=== Insights ===");
    println!("1. Adversarial samples often land in Rare layer");
    println!("2. Samples near decision boundaries have low degrees");
    println!("3. KDF provides unsupervised anomaly detection");
    println!("4. Combine with model confidence for better detection");
}
