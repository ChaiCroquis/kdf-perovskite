//! Differential Privacy KDF
//!
//! This example demonstrates how to apply KDF with differential privacy:
//! - Add noise to preserve individual privacy
//! - Maintain utility while protecting sensitive data
//! - Layer-aware noise calibration
//!
//! Run: cargo run --example kdf_differential_privacy

use kdf::{cosine_similarity, Kdf, Layer};

/// Add Laplace noise for differential privacy
fn laplace_noise(scale: f64) -> f64 {
    // Simple Laplace noise using uniform random
    // In production, use a proper crypto-secure random
    let u: f64 = rand_simple() - 0.5;
    -scale * u.signum() * (1.0 - 2.0 * u.abs()).ln()
}

/// Simple pseudo-random (for demo purposes)
fn rand_simple() -> f64 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        (SEED as f64 / u64::MAX as f64)
    }
}

/// Add noise to vector (local DP)
fn add_noise_to_vector(vec: &[f64], epsilon: f64) -> Vec<f64> {
    let sensitivity = 1.0;  // Assuming normalized vectors
    let scale = sensitivity / epsilon;

    vec.iter()
        .map(|&v| v + laplace_noise(scale))
        .collect()
}

/// Private sample for DP analysis
#[derive(Clone)]
struct PrivateSample {
    original: Vec<f64>,
    noisy: Vec<f64>,
    sensitive: bool,  // Is this sensitive data?
}

fn main() {
    println!("=== Differential Privacy KDF Demo ===\n");

    let kdf = Kdf::with_defaults();

    // =========================================================================
    // 1. Create dataset with sensitive records
    // =========================================================================
    println!("--- Dataset with Sensitive Records ---\n");

    let epsilon = 1.0;  // Privacy budget

    let mut samples: Vec<PrivateSample> = vec![
        // Regular samples (cluster)
        PrivateSample { original: vec![1.0, 0.0, 0.0], noisy: vec![], sensitive: false },
        PrivateSample { original: vec![1.0, 0.1, 0.0], noisy: vec![], sensitive: false },
        PrivateSample { original: vec![1.0, 0.0, 0.1], noisy: vec![], sensitive: false },

        // Another cluster
        PrivateSample { original: vec![0.0, 1.0, 0.0], noisy: vec![], sensitive: false },
        PrivateSample { original: vec![0.1, 1.0, 0.0], noisy: vec![], sensitive: false },

        // SENSITIVE records (need extra protection)
        PrivateSample { original: vec![0.5, 0.5, 0.0], noisy: vec![], sensitive: true },
        PrivateSample { original: vec![0.0, 0.0, 1.0], noisy: vec![], sensitive: true },
    ];

    // Add noise to all samples
    for sample in &mut samples {
        sample.noisy = add_noise_to_vector(&sample.original, epsilon);
    }

    println!("Epsilon (privacy budget): {}", epsilon);
    println!("Total samples: {}, Sensitive: {}",
        samples.len(),
        samples.iter().filter(|s| s.sensitive).count());
    println!();

    // =========================================================================
    // 2. KDF on Original Data
    // =========================================================================
    println!("--- KDF on Original (Non-Private) ---\n");

    let originals: Vec<Vec<f64>> = samples.iter()
        .map(|s| s.original.clone())
        .collect();

    let result_original = kdf.process(&originals, 0.85, |a, b| {
        cosine_similarity(a, b)
    });

    println!("Layers: {:?}", result_original.layers);
    println!("Selected: {:?}", result_original.selected);
    println!("Rare: {:?}", result_original.rare_items());
    println!();

    // =========================================================================
    // 3. KDF on Noisy Data (Private)
    // =========================================================================
    println!("--- KDF on Noisy (Private) Data ---\n");

    let noisy: Vec<Vec<f64>> = samples.iter()
        .map(|s| s.noisy.clone())
        .collect();

    let result_noisy = kdf.process(&noisy, 0.85, |a, b| {
        cosine_similarity(a, b)
    });

    println!("Layers: {:?}", result_noisy.layers);
    println!("Selected: {:?}", result_noisy.selected);
    println!("Rare: {:?}", result_noisy.rare_items());
    println!();

    // =========================================================================
    // 4. Privacy-Utility Analysis
    // =========================================================================
    println!("--- Privacy-Utility Analysis ---\n");

    // Check if layer classifications are stable under noise
    let mut layer_matches = 0;
    for i in 0..samples.len() {
        if result_original.layers[i] == result_noisy.layers[i] {
            layer_matches += 1;
        }
    }

    let layer_stability = layer_matches as f64 / samples.len() as f64;
    println!("Layer stability: {:.1}% ({}/{})",
        layer_stability * 100.0, layer_matches, samples.len());

    // Check if sensitive samples are protected
    println!("\nSensitive sample analysis:");
    for (i, sample) in samples.iter().enumerate() {
        if sample.sensitive {
            let orig_layer = result_original.layers[i];
            let noisy_layer = result_noisy.layers[i];
            let protected = orig_layer != noisy_layer;

            println!("  Sample {}: {:?} -> {:?} {}",
                i,
                orig_layer,
                noisy_layer,
                if protected { "(layer changed - more privacy)" } else { "(stable)" });
        }
    }
    println!();

    // =========================================================================
    // 5. Layer-Aware Noise Calibration
    // =========================================================================
    println!("--- Layer-Aware Privacy (Conceptual) ---\n");

    println!("Insight: Rare samples may need MORE noise because:");
    println!("  - They are unique and potentially identifiable");
    println!("  - Removing them would significantly change results");
    println!("  - They have high sensitivity in DP terms");
    println!();

    println!("Recommended approach:");
    println!("  - Core samples: Lower noise (many similar samples)");
    println!("  - Edge samples: Moderate noise");
    println!("  - Rare samples: Higher noise (unique, identifiable)");
    println!();

    // Demonstrate layer-aware epsilon
    let mut layer_epsilon = vec![0.0; samples.len()];
    for i in 0..samples.len() {
        layer_epsilon[i] = match result_original.layers[i] {
            Layer::Core => epsilon * 1.5,  // Less noise needed
            Layer::Edge => epsilon * 1.0,  // Standard noise
            Layer::Rare => epsilon * 0.5,  // More noise needed (smaller epsilon)
        };
    }

    println!("Layer-calibrated epsilon values:");
    for (i, &eps) in layer_epsilon.iter().enumerate() {
        let layer = result_original.layers[i];
        let sens = if samples[i].sensitive { "[SENS]" } else { "      " };
        println!("  {} Sample {}: {:?} -> epsilon={:.2}", sens, i, layer, eps);
    }
    println!();

    println!("=== Summary ===");
    println!("Differential Privacy + KDF:");
    println!("1. Add noise to protect individual records");
    println!("2. KDF layers can guide noise calibration");
    println!("3. Rare samples need more protection (higher sensitivity)");
    println!("4. Balance privacy budget across layers for optimal utility");
}
