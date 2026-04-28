//! Causal KDF - Integrating Layer Structure with Causal Relationships
//!
//! This example demonstrates how KDF layers can inform causal analysis:
//! - Rare samples may represent intervention effects
//! - Core samples represent baseline behavior
//! - Layer transitions under intervention indicate causal relationships
//!
//! Run: cargo run --example kdf_causal

use kdf::{cosine_similarity, Kdf, Layer};

/// Sample with features and treatment indicator
#[derive(Clone)]
struct CausalSample {
    features: Vec<f64>,
    treated: bool,
    outcome: f64,
}

fn main() {
    println!("=== Causal KDF Demo ===\n");

    let kdf = Kdf::with_defaults();

    // =========================================================================
    // 1. Control vs Treatment Groups
    // =========================================================================
    println!("--- Control vs Treatment Analysis ---\n");

    // Control group (no treatment)
    let control: Vec<CausalSample> = vec![
        CausalSample {
            features: vec![1.0, 0.5, 0.3],
            treated: false,
            outcome: 10.0,
        },
        CausalSample {
            features: vec![1.0, 0.6, 0.3],
            treated: false,
            outcome: 11.0,
        },
        CausalSample {
            features: vec![1.0, 0.5, 0.4],
            treated: false,
            outcome: 10.5,
        },
        CausalSample {
            features: vec![0.5, 0.5, 0.5],
            treated: false,
            outcome: 8.0,
        },
        CausalSample {
            features: vec![0.5, 0.6, 0.5],
            treated: false,
            outcome: 8.5,
        },
    ];

    // Treatment group (with intervention)
    let treatment: Vec<CausalSample> = vec![
        CausalSample {
            features: vec![1.0, 0.5, 0.3],
            treated: true,
            outcome: 15.0,
        }, // Effect!
        CausalSample {
            features: vec![1.0, 0.6, 0.3],
            treated: true,
            outcome: 16.0,
        }, // Effect!
        CausalSample {
            features: vec![1.0, 0.5, 0.4],
            treated: true,
            outcome: 15.5,
        }, // Effect!
        CausalSample {
            features: vec![0.5, 0.5, 0.5],
            treated: true,
            outcome: 8.5,
        }, // No effect
        CausalSample {
            features: vec![0.5, 0.6, 0.5],
            treated: true,
            outcome: 9.0,
        }, // Slight effect
    ];

    // Analyze control group
    let control_result = kdf.process(&control, 0.9, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    // Analyze treatment group
    let treatment_result = kdf.process(&treatment, 0.9, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    println!("Control group layers: {:?}", control_result.layers);
    println!("Treatment group layers: {:?}", treatment_result.layers);

    // Identify samples with different layer status under treatment
    println!("\nLayer changes under treatment:");
    for i in 0..control.len() {
        if control_result.layers[i] != treatment_result.layers[i] {
            println!(
                "  Sample {}: {:?} -> {:?} (outcome: {:.1} -> {:.1})",
                i,
                control_result.layers[i],
                treatment_result.layers[i],
                control[i].outcome,
                treatment[i].outcome
            );
        }
    }
    println!();

    // =========================================================================
    // 2. Heterogeneous Treatment Effects
    // =========================================================================
    println!("--- Heterogeneous Treatment Effects ---\n");

    // Combine all samples
    let all_samples: Vec<CausalSample> = control.iter().chain(treatment.iter()).cloned().collect();

    let combined_result = kdf.process(&all_samples, 0.9, |a, b| {
        cosine_similarity(&a.features, &b.features)
    });

    // Find rare samples in combined analysis
    let rare_in_combined = combined_result.rare_items();
    println!("Rare samples in combined analysis: {:?}", rare_in_combined);

    // These might represent unusual treatment effects
    for &idx in &rare_in_combined {
        let sample = &all_samples[idx];
        println!(
            "  Index {}: treated={}, outcome={:.1}, features={:?}",
            idx, sample.treated, sample.outcome, sample.features
        );
    }
    println!();

    // =========================================================================
    // 3. Causal Discovery Hints
    // =========================================================================
    println!("--- Causal Discovery Insights ---\n");

    // Group by layer and calculate average treatment effect
    let mut core_control = Vec::new();
    let mut core_treated = Vec::new();

    for i in 0..control.len() {
        if control_result.layers[i] == Layer::Core {
            core_control.push(control[i].outcome);
            core_treated.push(treatment[i].outcome);
        }
    }

    if !core_control.is_empty() {
        let avg_control: f64 = core_control.iter().sum::<f64>() / core_control.len() as f64;
        let avg_treated: f64 = core_treated.iter().sum::<f64>() / core_treated.len() as f64;
        let ate = avg_treated - avg_control;

        println!("Average Treatment Effect (Core samples):");
        println!("  Control mean: {:.2}", avg_control);
        println!("  Treated mean: {:.2}", avg_treated);
        println!("  ATE: {:.2}", ate);
    }

    let mut rare_control = Vec::new();
    let mut rare_treated = Vec::new();

    for i in 0..control.len() {
        if control_result.layers[i] == Layer::Rare {
            rare_control.push(control[i].outcome);
            rare_treated.push(treatment[i].outcome);
        }
    }

    if !rare_control.is_empty() {
        let avg_control: f64 = rare_control.iter().sum::<f64>() / rare_control.len() as f64;
        let avg_treated: f64 = rare_treated.iter().sum::<f64>() / rare_treated.len() as f64;
        let ate = avg_treated - avg_control;

        println!("\nAverage Treatment Effect (Rare samples):");
        println!("  Control mean: {:.2}", avg_control);
        println!("  Treated mean: {:.2}", avg_treated);
        println!("  ATE: {:.2}", ate);
    }
    println!();

    println!("=== Summary ===");
    println!("Causal KDF insights:");
    println!("1. Layer changes under treatment indicate causal sensitivity");
    println!("2. Rare samples may represent unusual responders");
    println!("3. Treatment effects can vary by layer (heterogeneity)");
    println!("4. KDF helps identify subgroups for targeted interventions");
}
