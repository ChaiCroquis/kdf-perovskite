//! Incremental KDF example for streaming data

use kdf::{cosine_similarity, IncrementalKdf, KdfParams};

fn main() {
    println!("=== KDF Incremental Processing Example ===\n");

    // Create incremental KDF
    let mut kdf = IncrementalKdf::<Vec<f64>>::new(KdfParams::default(), 0.95);

    println!("--- Adding items incrementally ---\n");

    // Add initial cluster
    for i in 0..5 {
        let item = vec![1.0 + i as f64 * 0.01, 0.9, 0.1, 0.0];
        kdf.add(item, |a, b| cosine_similarity(a, b));
    }
    println!("After adding 5 redundant items:");
    println!("  Total items: {}", kdf.len());
    println!(
        "  Selected: {:?}\n",
        kdf.get_selected(|a, b| cosine_similarity(a, b))
    );

    // Add rare item
    let rare_item = vec![-1.0, 0.0, 0.0, 0.0];
    kdf.add(rare_item, |a, b| cosine_similarity(a, b));
    println!("After adding 1 rare item:");
    println!("  Total items: {}", kdf.len());
    println!(
        "  Selected: {:?}\n",
        kdf.get_selected(|a, b| cosine_similarity(a, b))
    );

    // Add more redundant items
    for i in 0..5 {
        let item = vec![1.0 + i as f64 * 0.01, 0.9, 0.1, 0.0];
        kdf.add(item, |a, b| cosine_similarity(a, b));
    }
    println!("After adding 5 more redundant items:");
    println!("  Total items: {}", kdf.len());
    println!(
        "  Selected: {:?}\n",
        kdf.get_selected(|a, b| cosine_similarity(a, b))
    );

    println!("--- Removing items ---\n");

    // Remove some redundant items
    for _ in 0..3 {
        kdf.remove(0, |a, b| cosine_similarity(a, b));
    }
    println!("After removing 3 items from the front:");
    println!("  Total items: {}", kdf.len());
    println!(
        "  Selected: {:?}\n",
        kdf.get_selected(|a, b| cosine_similarity(a, b))
    );

    println!("--- Window-based processing ---\n");

    // Simulate sliding window
    let window_size = 10;
    let mut window_kdf = IncrementalKdf::<Vec<f64>>::new(KdfParams::default(), 0.95);

    // Fill initial window
    for i in 0..window_size {
        let item = if i % 3 == 0 {
            vec![-1.0 - i as f64 * 0.1, 0.0, 0.0, 0.0] // Rare
        } else {
            vec![1.0, 0.9, 0.1, 0.0] // Redundant
        };
        window_kdf.add(item, |a, b| cosine_similarity(a, b));
    }

    println!("Initial window (size {}):", window_kdf.len());
    println!(
        "  Selected: {:?}",
        window_kdf.get_selected(|a, b| cosine_similarity(a, b))
    );

    // Slide window: remove oldest, add newest
    for slide in 0..3 {
        // Remove oldest
        window_kdf.remove(0, |a, b| cosine_similarity(a, b));

        // Add newest
        let new_item = if (window_size + slide) % 3 == 0 {
            vec![-2.0 - slide as f64 * 0.1, 0.0, 0.0, 0.0] // New rare
        } else {
            vec![1.0, 0.9, 0.1, 0.0] // New redundant
        };
        window_kdf.add(new_item, |a, b| cosine_similarity(a, b));

        println!("\nAfter slide {}:", slide + 1);
        println!("  Window size: {}", window_kdf.len());
        println!(
            "  Selected: {:?}",
            window_kdf.get_selected(|a, b| cosine_similarity(a, b))
        );
    }

    println!("\n=== Summary ===");
    println!("Incremental KDF allows efficient updates without full recomputation.");
    println!("Use cases: streaming data, sliding windows, real-time processing.");
}
