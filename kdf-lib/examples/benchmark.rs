//! Benchmark tests for KDF optimizations
//!
//! Tests:
//! 1. Pre-computed decay factors vs naive approach
//! 2. HashSet is_selected() vs Vec contains()
//! 3. Overall process() performance at different scales

use kdf::{Kdf, KdfParams, cosine_similarity, Layer};
use std::time::Instant;

fn main() {
    println!("=== KDF Benchmark Tests ===\n");

    benchmark_decay_computation();
    benchmark_is_selected();
    benchmark_process_scaling();
    benchmark_reason_calls();
}

/// Benchmark: Pre-computed decay factors
fn benchmark_decay_computation() {
    println!("## 1. Decay Computation Optimization");
    println!("   Comparing: pre-computed factors vs naive per-iteration powf\n");

    let n = 1000;
    let iterations = 100;
    let degrees: Vec<usize> = (0..n).map(|i| i % 20).collect();
    let layers: Vec<Layer> = degrees.iter().map(|&d| {
        if d == 0 { Layer::Rare }
        else if d > 10 { Layer::Core }
        else { Layer::Edge }
    }).collect();

    let params = KdfParams::default();

    // Optimized: pre-compute decay factors
    let start = Instant::now();
    for _ in 0..100 {
        let decay_factors: Vec<f64> = degrees.iter().zip(&layers).map(|(&deg, &layer)| {
            let c = deg as f64;
            let alpha = match layer {
                Layer::Core => params.alpha_core,
                Layer::Edge => params.alpha_edge,
                Layer::Rare => params.alpha_rare,
            };
            (1.0 - params.beta * (1.0 + params.gamma * c.powf(alpha))).max(0.0)
        }).collect();

        let mut weights = vec![1.0f64; n];
        for _ in 0..iterations {
            for i in 0..n {
                weights[i] *= decay_factors[i];
            }
        }
        std::hint::black_box(&weights);
    }
    let optimized_time = start.elapsed();

    // Naive: compute powf in every iteration
    let start = Instant::now();
    for _ in 0..100 {
        let mut weights = vec![1.0f64; n];
        for _ in 0..iterations {
            for i in 0..n {
                let c = degrees[i] as f64;
                let alpha = match layers[i] {
                    Layer::Core => params.alpha_core,
                    Layer::Edge => params.alpha_edge,
                    Layer::Rare => params.alpha_rare,
                };
                let decay_rate = params.beta * (1.0 + params.gamma * c.powf(alpha));
                weights[i] *= (1.0 - decay_rate).max(0.0);
            }
        }
        std::hint::black_box(&weights);
    }
    let naive_time = start.elapsed();

    println!("   Items: {}, Iterations: {}", n, iterations);
    println!("   Optimized (pre-computed): {:?}", optimized_time);
    println!("   Naive (per-iteration):    {:?}", naive_time);
    println!("   Speedup: {:.2}x\n", naive_time.as_nanos() as f64 / optimized_time.as_nanos() as f64);
}

/// Benchmark: HashSet is_selected() vs Vec contains()
fn benchmark_is_selected() {
    println!("## 2. is_selected() Optimization (HashSet vs Vec)");
    println!("   Comparing: O(1) HashSet lookup vs O(n) Vec contains\n");

    let sizes = [100, 1000, 5000];

    for &size in &sizes {
        let selected: Vec<usize> = (0..size).step_by(3).collect();
        let selected_set: std::collections::HashSet<usize> = selected.iter().copied().collect();

        let queries: Vec<usize> = (0..size).collect();

        // HashSet lookup
        let start = Instant::now();
        for _ in 0..1000 {
            for &idx in &queries {
                std::hint::black_box(selected_set.contains(&idx));
            }
        }
        let hashset_time = start.elapsed();

        // Vec contains
        let start = Instant::now();
        for _ in 0..1000 {
            for &idx in &queries {
                std::hint::black_box(selected.contains(&idx));
            }
        }
        let vec_time = start.elapsed();

        println!("   Size: {}", size);
        println!("   HashSet: {:?}", hashset_time);
        println!("   Vec:     {:?}", vec_time);
        println!("   Speedup: {:.1}x\n", vec_time.as_nanos() as f64 / hashset_time.as_nanos() as f64);
    }
}

/// Benchmark: process() at different scales
fn benchmark_process_scaling() {
    println!("## 3. process() Scaling Performance");
    println!("   Testing at different data sizes\n");

    let sizes = [100, 500, 1000, 2000];
    let kdf = Kdf::with_defaults();

    for &size in &sizes {
        // Generate test data: 3 clusters + some rare items
        let mut items: Vec<Vec<f64>> = Vec::with_capacity(size);

        for i in 0..size {
            let cluster = i % 4;
            let base = match cluster {
                0 => vec![1.0, 0.0, 0.0],
                1 => vec![0.0, 1.0, 0.0],
                2 => vec![0.0, 0.0, 1.0],
                _ => vec![0.5, 0.5, 0.5 + (i as f64 * 0.01)], // Varied rare
            };
            // Add small noise
            let noisy: Vec<f64> = base.iter().map(|&x| x + (i as f64 * 0.001)).collect();
            items.push(noisy);
        }

        let start = Instant::now();
        let iterations = if size <= 500 { 10 } else { 3 };
        for _ in 0..iterations {
            let result = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
            std::hint::black_box(&result);
        }
        let elapsed = start.elapsed();
        let avg_ms = elapsed.as_millis() as f64 / iterations as f64;

        println!("   Size: {:>4} -> {:>6.1} ms/call ({} selected)", size, avg_ms,
            kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b)).selected.len());
    }
    println!();
}

/// Benchmark: reason() calls (tests is_selected optimization impact)
fn benchmark_reason_calls() {
    println!("## 4. reason() Call Performance");
    println!("   Testing reason() which uses is_selected() internally\n");

    let kdf = Kdf::with_defaults();

    let sizes = [100, 500, 1000];

    for &size in &sizes {
        let items: Vec<Vec<f64>> = (0..size).map(|i| {
            vec![(i as f64).cos(), (i as f64).sin(), i as f64 * 0.01]
        }).collect();

        let result = kdf.process(&items, 0.8, |a, b| cosine_similarity(a, b));

        // Call reason() for all items multiple times
        let start = Instant::now();
        for _ in 0..1000 {
            for i in 0..size {
                std::hint::black_box(result.reason(i));
            }
        }
        let elapsed = start.elapsed();

        println!("   Size: {:>4} -> {:>6.2} µs per reason() call",
            size, elapsed.as_nanos() as f64 / (1000 * size) as f64 / 1000.0);
    }
    println!();
}
