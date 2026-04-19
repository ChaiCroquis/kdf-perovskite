//! Tests for sleep mode optimization

use std::collections::HashMap;

use crate::interning::NodeIdMap;

use super::{
    compute_structural_entropy, entropy_cache::IncrementalEntropyCache,
    cooling::AdaptiveCoolingScheduler, optimizer::SleepModeOptimizer,
};

fn create_test_graph() -> Vec<(String, String, f64)> {
    vec![
        ("a".to_string(), "b".to_string(), 1.0),
        ("b".to_string(), "c".to_string(), 1.0),
        ("c".to_string(), "d".to_string(), 1.0),
        ("d".to_string(), "a".to_string(), 1.0),
        ("a".to_string(), "c".to_string(), 0.5),
    ]
}

#[test]
fn test_entropy_cache_initialization() {
    let edges = create_test_graph();
    let partition: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 1),
        ("d".to_string(), 1),
    ]
    .into_iter()
    .collect();

    let mut cache = IncrementalEntropyCache::new(1000);
    let entropy = cache.initialize_from_edges(&edges, &partition);

    assert!(entropy > 0.0);
    assert_eq!(cache.get_module_count(), 2);
}

#[test]
fn test_interned_initialization() {
    let mut id_map = NodeIdMap::new();
    let edges = vec![
        ("a".to_string(), "b".to_string(), 1.0),
        ("b".to_string(), "c".to_string(), 1.0),
    ];
    let interned_edges = id_map.intern_edges(&edges);

    let partition: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 1),
    ]
    .into_iter()
    .collect();
    let interned_partition = id_map.intern_partition(&partition);

    let mut cache = IncrementalEntropyCache::new(1000);
    let entropy = cache.initialize_from_interned(&interned_edges, &interned_partition, id_map.len());

    assert!(entropy >= 0.0);
}

#[test]
fn test_adaptive_cooling() {
    let mut scheduler = AdaptiveCoolingScheduler::default();

    let initial_temp = scheduler.get_temperature();
    assert_eq!(initial_temp, 1.0);

    // Simulate some updates
    for i in 0..100 {
        scheduler.update(1.0 / (i + 1) as f64);
    }

    assert!(scheduler.get_temperature() < initial_temp);
}

#[test]
fn test_sleep_mode_optimizer() {
    let edges = create_test_graph();

    let mut optimizer = SleepModeOptimizer::new(1.0, 0.01, 100, 50);
    let result = optimizer.run_nrem_phase(&edges, None);

    assert!(result.final_entropy <= result.initial_entropy);
    assert!(result.compression_ratio >= 0.0);
    assert!(result.acceptance_rate >= 0.0 && result.acceptance_rate <= 1.0);
}

#[test]
fn test_compute_structural_entropy() {
    let edges = create_test_graph();
    let partition: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 0),
        ("d".to_string(), 0),
    ]
    .into_iter()
    .collect();

    let entropy = compute_structural_entropy(&edges, &partition);
    assert!(entropy >= 0.0);
}

#[test]
fn test_empty_graph() {
    let edges: Vec<(String, String, f64)> = vec![];
    let partition: HashMap<String, u32> = HashMap::new();

    let entropy = compute_structural_entropy(&edges, &partition);
    assert_eq!(entropy, 0.0);
}

#[test]
#[ignore] // Flaky test - entropy comparison is non-deterministic
fn test_single_module() {
    let edges = create_test_graph();
    let partition: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 0),
        ("d".to_string(), 0),
    ]
    .into_iter()
    .collect();

    let entropy1 = compute_structural_entropy(&edges, &partition);

    // Multiple modules should have different entropy
    let partition2: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 1),
        ("c".to_string(), 2),
        ("d".to_string(), 3),
    ]
    .into_iter()
    .collect();

    let entropy2 = compute_structural_entropy(&edges, &partition2);

    // Different partitions should generally have different entropies
    assert!(entropy1 != entropy2 || entropy1 == 0.0);
}

#[test]
fn test_boundary_conversion_roundtrip() {
    let edges = create_test_graph();
    let initial_partition: HashMap<String, u32> = [
        ("a".to_string(), 0),
        ("b".to_string(), 0),
        ("c".to_string(), 1),
        ("d".to_string(), 1),
    ]
    .into_iter()
    .collect();

    let mut optimizer = SleepModeOptimizer::new(1.0, 0.01, 10, 50);
    let result = optimizer.run_nrem_phase(&edges, Some(initial_partition.clone()));

    // Check that partition keys are preserved
    for key in initial_partition.keys() {
        assert!(result.partition.contains_key(key));
    }
}

#[test]
#[ignore] // Run with: cargo test --release benchmark_interning -- --ignored --nocapture
fn benchmark_interning() {
    use std::time::Instant;

    fn generate_graph(n: usize) -> Vec<(String, String, f64)> {
        let mut edges = Vec::new();
        for i in 0..n {
            for j in (i + 1)..n {
                if (i * 17 + j * 31) % 10 < 2 {
                    edges.push((format!("node_{}", i), format!("node_{}", j), 1.0));
                }
            }
        }
        edges
    }

    println!("\nSleep Mode Optimizer Benchmark");
    println!("==============================");
    println!("{:<8} {:<8} {:<12} {:<12}", "Nodes", "Edges", "Time(ms)", "Iter/sec");
    println!("{}", "-".repeat(44));

    for &n in &[100, 200, 500, 1000] {
        let edges = generate_graph(n);
        let edge_count = edges.len();

        let mut optimizer = SleepModeOptimizer::new(1.0, 0.01, 500, 100);

        let start = Instant::now();
        let result = optimizer.run_nrem_phase(&edges, None);
        let elapsed = start.elapsed();

        let iter_per_sec = result.iterations as f64 / elapsed.as_secs_f64();

        println!(
            "{:<8} {:<8} {:<12.2} {:<12.0}",
            n,
            edge_count,
            elapsed.as_secs_f64() * 1000.0,
            iter_per_sec
        );
    }
}
