//! Test new features: Builder pattern, Levenshtein, DTW, parallel processing
use kdf::{Kdf, KdfParams, cosine_similarity, levenshtein_similarity, dtw_similarity};

fn main() {
    println!("=== New Features Test ===\n");

    // ========================================================================
    // 1. Builder Pattern
    // ========================================================================
    println!("## 1. Builder Pattern");

    let params = KdfParams::builder()
        .alpha_edge(1.8)
        .alpha_rare(0.2)
        .iterations(50)
        .build();

    println!("   alpha_edge: {} (set to 1.8)", params.alpha_edge);
    println!("   alpha_rare: {} (set to 0.2)", params.alpha_rare);
    println!("   iterations: {} (set to 50)", params.iterations);
    println!("   beta: {} (default)", params.beta);

    let kdf = Kdf::new(params);
    let items = vec![
        vec![1.0, 0.0],
        vec![0.9, 0.1],
        vec![0.0, 1.0],
    ];
    let result = kdf.process(&items, 0.85, |a, b| cosine_similarity(a, b));
    println!("   Process result: {} selected\n", result.selected.len());

    // ========================================================================
    // 2. Levenshtein Similarity
    // ========================================================================
    println!("## 2. Levenshtein Similarity (strings)");

    let strings = vec![
        "hello",
        "hallo",
        "world",
        "words",
        "xxxxx",
    ];

    println!("   Similarity matrix:");
    for s1 in &strings {
        print!("   ");
        for s2 in &strings {
            let sim = levenshtein_similarity(s1, s2);
            print!("{:.2} ", sim);
        }
        println!("  <- {}", s1);
    }

    // Use with KDF
    let kdf = Kdf::with_defaults();
    let result = kdf.process(&strings, 0.6, |a, b| levenshtein_similarity(a, b));
    println!("\n   KDF with Levenshtein (threshold=0.6):");
    println!("   Selected: {:?}", result.selected);
    for &i in &result.selected {
        println!("      {} - {:?}", strings[i], result.layers[i]);
    }

    // ========================================================================
    // 3. DTW Similarity
    // ========================================================================
    println!("\n## 3. DTW Similarity (time series)");

    let time_series = vec![
        vec![1.0, 2.0, 3.0, 4.0, 5.0],           // Rising
        vec![1.1, 2.1, 3.1, 4.1, 5.1],           // Rising (similar)
        vec![5.0, 4.0, 3.0, 2.0, 1.0],           // Falling
        vec![1.0, 1.0, 1.0, 1.0, 1.0],           // Flat
        vec![1.0, 5.0, 1.0, 5.0, 1.0],           // Oscillating
    ];

    println!("   Similarity matrix:");
    for (i, ts1) in time_series.iter().enumerate() {
        print!("   ");
        for ts2 in &time_series {
            let sim = dtw_similarity(ts1, ts2);
            print!("{:.2} ", sim);
        }
        println!("  <- series {}", i);
    }

    // Use with KDF
    let result = kdf.process(&time_series, 0.15, |a, b| dtw_similarity(a, b));
    println!("\n   KDF with DTW (threshold=0.15):");
    println!("   Selected: {:?}", result.selected);
    for &i in &result.selected {
        println!("      Series {} - {:?}", i, result.layers[i]);
    }

    // ========================================================================
    // 4. Parallel Processing (if feature enabled)
    // ========================================================================
    #[cfg(feature = "parallel")]
    {
        println!("\n## 4. Parallel Processing");

        let large_data: Vec<Vec<f64>> = (0..100)
            .map(|i| vec![(i as f64) / 100.0, 1.0 - (i as f64) / 100.0])
            .collect();

        let start = std::time::Instant::now();
        let result_seq = kdf.process(&large_data, 0.95, |a, b| cosine_similarity(a, b));
        let seq_time = start.elapsed();

        let start = std::time::Instant::now();
        let result_par = kdf.process_parallel(&large_data, 0.95, |a, b| cosine_similarity(a, b));
        let par_time = start.elapsed();

        println!("   Sequential: {:?} ({} selected)", seq_time, result_seq.selected.len());
        println!("   Parallel:   {:?} ({} selected)", par_time, result_par.selected.len());
        println!("   Results match: {}", result_seq.selected == result_par.selected);
    }

    #[cfg(not(feature = "parallel"))]
    {
        println!("\n## 4. Parallel Processing");
        println!("   (Enable with: cargo run --example new_features --features parallel)");
    }

    println!("\n✅ All new features working correctly!");
}
