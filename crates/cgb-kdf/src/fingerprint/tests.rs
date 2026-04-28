//! Tests for fingerprint module

use super::*;
use nalgebra::DMatrix;

#[test]
fn test_fingerprint_basic() {
    let mut engine = StructuralFingerprintEngine::default();
    let fp = engine.compute_fingerprint("node1", &NodeLabel::Normal, None);
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_fingerprint_cache() {
    let mut engine = StructuralFingerprintEngine::default();

    let fp1 = engine.compute_fingerprint("node1", &NodeLabel::Normal, None);
    let fp2 = engine.compute_fingerprint("node1", &NodeLabel::Normal, None);

    assert_eq!(fp1, fp2);

    let stats = engine.get_cache_stats();
    assert_eq!(stats.cache_hits, 1);
    assert_eq!(stats.total_computations, 1);
}

#[test]
fn test_fingerprint_label_difference() {
    let mut engine = StructuralFingerprintEngine::default();

    let fp_normal = engine.compute_fingerprint("node1", &NodeLabel::Normal, None);
    engine.clear_cache();
    let fp_garbage = engine.compute_fingerprint("node1", &NodeLabel::Garbage, None);

    // Should be different patterns
    let sim = engine.full_similarity(&fp_normal, &fp_garbage);
    assert!(sim < 0.9); // Different patterns should have lower similarity
}

#[test]
fn test_quick_distance() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.0, 0.0, 0.0];
    let fp2 = vec![1.0, 0.0, 0.0];

    let dist = engine.quick_distance(&fp1, &fp2);
    assert!((dist - 1.0).abs() < 1e-10);
}

#[test]
fn test_full_similarity_identical() {
    let engine = StructuralFingerprintEngine::default();

    let fp = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let sim = engine.full_similarity(&fp, &fp);

    assert!((sim - 1.0).abs() < 1e-10);
}

#[test]
fn test_deterministic_fingerprint() {
    let mut engine1 = StructuralFingerprintEngine::default();
    let mut engine2 = StructuralFingerprintEngine::default();

    let fp1 = engine1.compute_fingerprint("test_node", &NodeLabel::Normal, None);
    let fp2 = engine2.compute_fingerprint("test_node", &NodeLabel::Normal, None);

    assert_eq!(fp1, fp2); // Same node ID should produce same fingerprint
}

// ========== NodeLabel Tests ==========

#[test]
fn test_node_label_from_str() {
    assert_eq!(
        "isolated_truth".parse::<NodeLabel>().unwrap(),
        NodeLabel::IsolatedTruth
    );
    assert_eq!(
        "ISOLATED_TRUTH".parse::<NodeLabel>().unwrap(),
        NodeLabel::IsolatedTruth
    );
    assert_eq!("normal".parse::<NodeLabel>().unwrap(), NodeLabel::Normal);
    assert_eq!("garbage".parse::<NodeLabel>().unwrap(), NodeLabel::Garbage);
    assert_eq!("unknown".parse::<NodeLabel>().unwrap(), NodeLabel::Unknown);
    assert_eq!(
        "something_else".parse::<NodeLabel>().unwrap(),
        NodeLabel::Unknown
    );
}

#[test]
fn test_node_label_as_str() {
    assert_eq!(NodeLabel::IsolatedTruth.as_str(), "isolated_truth");
    assert_eq!(NodeLabel::Normal.as_str(), "normal");
    assert_eq!(NodeLabel::Garbage.as_str(), "garbage");
    assert_eq!(NodeLabel::Unknown.as_str(), "unknown");
}

#[test]
fn test_node_label_clone() {
    let label = NodeLabel::IsolatedTruth;
    let cloned = label.clone();
    assert_eq!(label, cloned);
}

#[test]
fn test_node_label_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(NodeLabel::Normal);
    set.insert(NodeLabel::Garbage);
    assert_eq!(set.len(), 2);
    assert!(set.contains(&NodeLabel::Normal));
}

// ========== CacheStats Tests ==========

#[test]
fn test_cache_stats_hit_rate_zero() {
    let stats = CacheStats::default();
    assert_eq!(stats.hit_rate(), 0.0);
}

#[test]
fn test_cache_stats_hit_rate_partial() {
    let stats = CacheStats {
        total_computations: 4,
        cache_hits: 6,
        cache_size: 10,
    };
    // total = 4 + 6 = 10, hits = 6, rate = 0.6
    assert!((stats.hit_rate() - 0.6).abs() < 1e-10);
}

#[test]
fn test_cache_stats_hit_rate_all_hits() {
    let stats = CacheStats {
        total_computations: 0,
        cache_hits: 10,
        cache_size: 5,
    };
    assert!((stats.hit_rate() - 1.0).abs() < 1e-10);
}

#[test]
fn test_cache_stats_clone() {
    let stats = CacheStats {
        total_computations: 5,
        cache_hits: 3,
        cache_size: 2,
    };
    let cloned = stats.clone();
    assert_eq!(stats.total_computations, cloned.total_computations);
}

// ========== FingerprintKey Tests ==========

#[test]
fn test_fingerprint_key_hash() {
    use std::collections::HashSet;
    let key1 = FingerprintKey {
        node_id: "node1".to_string(),
        label: "normal".to_string(),
    };
    let key2 = FingerprintKey {
        node_id: "node2".to_string(),
        label: "normal".to_string(),
    };

    let mut set = HashSet::new();
    set.insert(key1.clone());
    set.insert(key2);
    assert_eq!(set.len(), 2);
}

#[test]
fn test_fingerprint_key_clone() {
    let key = FingerprintKey {
        node_id: "test".to_string(),
        label: "normal".to_string(),
    };
    let cloned = key.clone();
    assert_eq!(key.node_id, cloned.node_id);
}

// ========== StructuralFingerprintEngine Tests ==========

#[test]
fn test_engine_new_custom() {
    let engine = StructuralFingerprintEngine::new(64, 0.8, 0.15, 0.05);
    assert_eq!(engine.fingerprint_dim, 64);
    assert_eq!(engine.w_sys, 0.8);
    assert_eq!(engine.w_rel, 0.15);
    assert_eq!(engine.w_attr, 0.05);
}

#[test]
fn test_engine_clear_cache() {
    let mut engine = StructuralFingerprintEngine::default();

    engine.compute_fingerprint("node1", &NodeLabel::Normal, None);
    engine.compute_fingerprint("node2", &NodeLabel::Normal, None);

    let stats = engine.get_cache_stats();
    assert_eq!(stats.cache_size, 2);

    engine.clear_cache();

    let stats = engine.get_cache_stats();
    assert_eq!(stats.cache_size, 0);
    assert_eq!(stats.total_computations, 0);
}

#[test]
fn test_fingerprint_isolated_truth() {
    let mut engine = StructuralFingerprintEngine::default();
    let fp = engine.compute_fingerprint("test", &NodeLabel::IsolatedTruth, None);
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_fingerprint_garbage() {
    let mut engine = StructuralFingerprintEngine::default();
    let fp = engine.compute_fingerprint("test", &NodeLabel::Garbage, None);
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_fingerprint_unknown() {
    let mut engine = StructuralFingerprintEngine::default();
    let fp = engine.compute_fingerprint("test", &NodeLabel::Unknown, None);
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_compute_from_ego_graph_small() {
    let mut engine = StructuralFingerprintEngine::default();

    // Empty/small ego graph should still work
    let neighbors: Vec<(String, Vec<String>)> = vec![];
    let fp = engine.compute_from_ego_graph("center", &NodeLabel::Normal, &neighbors);
    assert_eq!(fp.len(), 32);
}

#[test]
fn test_compute_from_ego_graph_single_neighbor() {
    let mut engine = StructuralFingerprintEngine::default();

    // Single neighbor (too small for Laplacian)
    let neighbors = vec![("center".to_string(), vec!["neighbor".to_string()])];
    let fp = engine.compute_from_ego_graph("center", &NodeLabel::Normal, &neighbors);
    assert_eq!(fp.len(), 32);
}

#[test]
fn test_compute_from_ego_graph_proper() {
    let mut engine = StructuralFingerprintEngine::default();

    let neighbors = vec![
        ("a".to_string(), vec!["b".to_string(), "c".to_string()]),
        ("b".to_string(), vec!["a".to_string(), "c".to_string()]),
        ("c".to_string(), vec!["a".to_string(), "b".to_string()]),
    ];

    let fp = engine.compute_from_ego_graph("a", &NodeLabel::Normal, &neighbors);
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_compute_from_ego_graph_star() {
    let mut engine = StructuralFingerprintEngine::default();

    // Star topology: center connected to many leaves
    let neighbors = vec![
        (
            "center".to_string(),
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ],
        ),
        ("a".to_string(), vec!["center".to_string()]),
        ("b".to_string(), vec!["center".to_string()]),
        ("c".to_string(), vec!["center".to_string()]),
        ("d".to_string(), vec!["center".to_string()]),
    ];

    let fp = engine.compute_from_ego_graph("center", &NodeLabel::Normal, &neighbors);
    assert_eq!(fp.len(), 32);
}

#[test]
fn test_compute_with_laplacian() {
    let mut engine = StructuralFingerprintEngine::default();

    // Create a simple 3x3 Laplacian matrix
    let laplacian =
        DMatrix::from_row_slice(3, 3, &[2.0, -1.0, -1.0, -1.0, 2.0, -1.0, -1.0, -1.0, 2.0]);

    let fp = engine.compute_fingerprint("node", &NodeLabel::Normal, Some(&laplacian));
    assert_eq!(fp.len(), 32);
    assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
}

#[test]
fn test_quick_distance_empty() {
    let engine = StructuralFingerprintEngine::default();

    let fp1: Vec<f64> = vec![];
    let fp2: Vec<f64> = vec![];

    let dist = engine.quick_distance(&fp1, &fp2);
    assert_eq!(dist, 0.0);
}

#[test]
fn test_quick_distance_mismatched_length() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![1.0, 2.0];
    let fp2 = vec![1.0, 2.0, 3.0];

    let dist = engine.quick_distance(&fp1, &fp2);
    assert_eq!(dist, f64::MAX);
}

#[test]
fn test_quick_distance_zero() {
    let engine = StructuralFingerprintEngine::default();

    let fp = vec![0.5, 0.5, 0.5];
    let dist = engine.quick_distance(&fp, &fp);
    assert!((dist - 0.0).abs() < 1e-10);
}

#[test]
fn test_full_similarity_empty() {
    let engine = StructuralFingerprintEngine::default();

    let fp1: Vec<f64> = vec![];
    let fp2: Vec<f64> = vec![];

    let sim = engine.full_similarity(&fp1, &fp2);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_full_similarity_mismatched_length() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![1.0, 2.0];
    let fp2 = vec![1.0, 2.0, 3.0];

    let sim = engine.full_similarity(&fp1, &fp2);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_full_similarity_orthogonal() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![1.0, 0.0, 0.0];
    let fp2 = vec![0.0, 1.0, 0.0];

    let sim = engine.full_similarity(&fp1, &fp2);
    // Cosine = 0, but struct and sign may contribute
    assert!((0.0..=1.0).contains(&sim));
}

#[test]
fn test_full_similarity_opposite() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.0, 0.2, 0.4, 0.6, 0.8];
    let fp2 = vec![0.8, 0.6, 0.4, 0.2, 0.0];

    let sim = engine.full_similarity(&fp1, &fp2);
    // Opposite trends should have lower similarity
    assert!(sim < 1.0);
}

#[test]
fn test_full_similarity_zero_norm() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.0, 0.0, 0.0];
    let fp2 = vec![1.0, 2.0, 3.0];

    let sim = engine.full_similarity(&fp1, &fp2);
    // Zero norm for fp1 should handle gracefully
    // The result may include struct_sim and sign_match contributions
    // Just verify it's a valid number
    assert!(!sim.is_nan());
}

#[test]
fn test_different_node_ids_different_fingerprints() {
    let mut engine = StructuralFingerprintEngine::default();

    let fp1 = engine.compute_fingerprint("node_alpha", &NodeLabel::Normal, None);
    engine.clear_cache();
    let fp2 = engine.compute_fingerprint("node_beta", &NodeLabel::Normal, None);

    // Different node IDs should produce different fingerprints
    assert_ne!(fp1, fp2);
}

#[test]
fn test_fingerprint_consistency_across_labels() {
    let mut engine = StructuralFingerprintEngine::default();

    // All labels should produce valid fingerprints
    for label in [
        NodeLabel::Normal,
        NodeLabel::IsolatedTruth,
        NodeLabel::Garbage,
        NodeLabel::Unknown,
    ] {
        let fp = engine.compute_fingerprint(&format!("node_{:?}", label), &label, None);
        assert_eq!(fp.len(), 32);
        assert!(fp.iter().all(|&v| (0.0..=1.0).contains(&v)));
    }
}

#[test]
fn test_cache_works_correctly() {
    let mut engine = StructuralFingerprintEngine::default();

    // First computation
    let fp1 = engine.compute_fingerprint("cached_node", &NodeLabel::Normal, None);
    assert_eq!(engine.get_cache_stats().total_computations, 1);
    assert_eq!(engine.get_cache_stats().cache_hits, 0);

    // Second call should hit cache
    let fp2 = engine.compute_fingerprint("cached_node", &NodeLabel::Normal, None);
    assert_eq!(engine.get_cache_stats().total_computations, 1);
    assert_eq!(engine.get_cache_stats().cache_hits, 1);

    // Fingerprints should be identical
    assert_eq!(fp1, fp2);
}

#[test]
fn test_fingerprint_large_laplacian() {
    let mut engine = StructuralFingerprintEngine::default();

    // Large Laplacian (larger than fingerprint dimension)
    let n = 64;
    let mut data = vec![0.0; n * n];
    for i in 0..n {
        data[i * n + i] = 2.0;
        if i > 0 {
            data[i * n + (i - 1)] = -1.0;
            data[(i - 1) * n + i] = -1.0;
        }
    }
    let laplacian = DMatrix::from_row_slice(n, n, &data);

    let fp = engine.compute_fingerprint("large", &NodeLabel::Normal, Some(&laplacian));
    assert_eq!(fp.len(), 32);
}

#[test]
fn test_fingerprint_small_laplacian() {
    let mut engine = StructuralFingerprintEngine::default();

    // Small Laplacian (smaller than fingerprint dimension)
    let laplacian = DMatrix::from_row_slice(2, 2, &[1.0, -1.0, -1.0, 1.0]);

    let fp = engine.compute_fingerprint("small", &NodeLabel::Normal, Some(&laplacian));
    assert_eq!(fp.len(), 32);
}

// ========== SimpleRng Tests ==========

#[test]
fn test_simple_rng_deterministic() {
    use crate::fingerprint::rng::SimpleRng;

    let mut rng1 = SimpleRng::new(12345);
    let mut rng2 = SimpleRng::new(12345);

    for _ in 0..100 {
        assert_eq!(rng1.next_f64(), rng2.next_f64());
    }
}

#[test]
fn test_simple_rng_different_seeds() {
    use crate::fingerprint::rng::SimpleRng;

    let mut rng1 = SimpleRng::new(12345);
    let mut rng2 = SimpleRng::new(54321);

    // Should produce different sequences
    let mut all_same = true;
    for _ in 0..10 {
        if rng1.next_f64() != rng2.next_f64() {
            all_same = false;
            break;
        }
    }
    assert!(!all_same);
}

#[test]
fn test_simple_rng_f64_range() {
    use crate::fingerprint::rng::SimpleRng;

    let mut rng = SimpleRng::new(42);

    for _ in 0..100 {
        let val = rng.next_f64();
        assert!((0.0..=1.0).contains(&val));
    }
}

#[test]
fn test_simple_rng_usize() {
    use crate::fingerprint::rng::SimpleRng;

    let mut rng = SimpleRng::new(42);

    for _ in 0..100 {
        let val = rng.next_usize();
        // Should be valid usize
        let _ = val;
    }
}

// ========== PrecomputedFingerprint Tests ==========

#[test]
fn test_precomputed_fingerprint_creation() {
    let fp = vec![0.3, 0.1, 0.4, 0.1, 0.5];
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    assert_eq!(pfp.raw, fp);
    assert!(pfp.norm > 0.0);
    assert_eq!(pfp.sorted.len(), fp.len());
    assert_eq!(pfp.gradient_signs.len(), fp.len() - 1);
}

#[test]
fn test_precomputed_fingerprint_norm() {
    let fp = vec![3.0, 4.0]; // 3² + 4² = 25, √25 = 5
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    assert!((pfp.norm - 5.0).abs() < 1e-10);
}

#[test]
fn test_precomputed_fingerprint_sorted() {
    let fp = vec![0.5, 0.1, 0.9, 0.3];
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    // sorted should be [0.1, 0.3, 0.5, 0.9]
    assert!((pfp.sorted[0] - 0.1).abs() < 1e-10);
    assert!((pfp.sorted[1] - 0.3).abs() < 1e-10);
    assert!((pfp.sorted[2] - 0.5).abs() < 1e-10);
    assert!((pfp.sorted[3] - 0.9).abs() < 1e-10);
}

#[test]
fn test_precomputed_fingerprint_gradient_signs() {
    let fp = vec![0.1, 0.5, 0.3, 0.3]; // +, -, 0
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    assert_eq!(pfp.gradient_signs.len(), 3);
    assert_eq!(pfp.gradient_signs[0], 1); // 0.5 - 0.1 > 0
    assert_eq!(pfp.gradient_signs[1], -1); // 0.3 - 0.5 < 0
    assert_eq!(pfp.gradient_signs[2], 0); // 0.3 - 0.3 = 0
}

#[test]
fn test_precomputed_fingerprint_zero_norm() {
    let fp = vec![0.0, 0.0, 0.0];
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    assert!(pfp.is_zero_norm());
}

// ========== fast_similarity Tests ==========

#[test]
fn test_fast_similarity_identical() {
    let engine = StructuralFingerprintEngine::default();
    let fp = vec![0.3, 0.1, 0.4, 0.1, 0.5];
    let pfp = PrecomputedFingerprint::from_fingerprint(&fp);

    let sim = engine.fast_similarity(&pfp, &pfp, 0.0);
    assert!((sim - 1.0).abs() < 1e-10);
}

#[test]
fn test_fast_similarity_matches_full_similarity() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
    let fp2 = vec![0.2, 0.3, 0.1, 0.5, 0.4];

    let pfp1 = PrecomputedFingerprint::from_fingerprint(&fp1);
    let pfp2 = PrecomputedFingerprint::from_fingerprint(&fp2);

    let fast_sim = engine.fast_similarity(&pfp1, &pfp2, 0.0);
    let full_sim = engine.full_similarity(&fp1, &fp2);

    // Should produce same result
    assert!((fast_sim - full_sim).abs() < 1e-10);
}

#[test]
fn test_fast_similarity_early_termination() {
    let engine = StructuralFingerprintEngine::default();

    // Orthogonal vectors with cosine = 0
    let fp1 = vec![1.0, 0.0, 0.0, 0.0];
    let fp2 = vec![0.0, 1.0, 0.0, 0.0];

    let pfp1 = PrecomputedFingerprint::from_fingerprint(&fp1);
    let pfp2 = PrecomputedFingerprint::from_fingerprint(&fp2);

    // With high threshold, early termination should kick in
    let sim = engine.fast_similarity(&pfp1, &pfp2, 0.9);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_fast_similarity_zero_norm() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.0, 0.0, 0.0];
    let fp2 = vec![1.0, 2.0, 3.0];

    let pfp1 = PrecomputedFingerprint::from_fingerprint(&fp1);
    let pfp2 = PrecomputedFingerprint::from_fingerprint(&fp2);

    let sim = engine.fast_similarity(&pfp1, &pfp2, 0.0);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_fast_similarity_dimension_mismatch() {
    let engine = StructuralFingerprintEngine::default();

    let fp1 = vec![0.1, 0.2, 0.3];
    let fp2 = vec![0.1, 0.2, 0.3, 0.4];

    let pfp1 = PrecomputedFingerprint::from_fingerprint(&fp1);
    let pfp2 = PrecomputedFingerprint::from_fingerprint(&fp2);

    let sim = engine.fast_similarity(&pfp1, &pfp2, 0.0);
    assert_eq!(sim, 0.0);
}

#[test]
fn test_precompute_batch() {
    let engine = StructuralFingerprintEngine::default();

    let fingerprints = vec![
        vec![0.1, 0.2, 0.3],
        vec![0.4, 0.5, 0.6],
        vec![0.7, 0.8, 0.9],
    ];

    let precomputed = engine.precompute_batch(&fingerprints);

    assert_eq!(precomputed.len(), 3);
    for (i, pfp) in precomputed.iter().enumerate() {
        assert_eq!(pfp.raw, fingerprints[i]);
        assert!(pfp.norm > 0.0);
    }
}
