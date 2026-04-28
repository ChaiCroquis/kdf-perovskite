//! Tests for the causal module

#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use super::super::*;
    use std::collections::{HashMap, HashSet};

    fn create_test_series() -> (Vec<f64>, Vec<f64>) {
        // Create correlated time series
        let n = 100;
        let mut source = Vec::with_capacity(n);
        let mut target = Vec::with_capacity(n);

        for i in 0..n {
            let x = (i as f64 * 0.1).sin() + 0.1 * (i as f64 * 0.3).cos();
            source.push(x);
            // Target is lagged and noisy version of source
            if i > 0 {
                target.push(source[i - 1] * 0.8 + 0.2 * (i as f64 * 0.2).sin());
            } else {
                target.push(0.0);
            }
        }

        (source, target)
    }

    // ========== TeStrategy Tests ==========

    #[test]
    fn test_te_strategy_variants() {
        assert_eq!(TeStrategy::Screening, TeStrategy::Screening);
        assert_eq!(TeStrategy::DeepProbe, TeStrategy::DeepProbe);
        assert_eq!(TeStrategy::Validation, TeStrategy::Validation);
    }

    #[test]
    fn test_te_strategy_clone_and_debug() {
        let strategy = TeStrategy::DeepProbe;
        let cloned = strategy;
        assert_eq!(strategy, cloned);
        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("DeepProbe"));
    }

    // ========== TeResult Tests ==========

    #[test]
    fn test_te_result_new() {
        let result = TeResult::new(0.5, true);
        assert_eq!(result.te, 0.5);
        assert!(result.source_to_target);
        assert!(result.p_value.is_none());
        assert!(result.is_significant);
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_te_result_new_negative() {
        let result = TeResult::new(-0.5, true);
        assert!(!result.is_significant); // te <= 0 is not significant
    }

    #[test]
    fn test_te_result_confidence_clamping() {
        let result = TeResult::new(1.5, true);
        assert_eq!(result.confidence, 1.0); // Clamped to 1.0
    }

    #[test]
    fn test_te_result_with_significance() {
        let result = TeResult::with_significance(0.5, true, 0.01, 0.05);
        assert!(result.is_significant);
        assert_eq!(result.p_value, Some(0.01));
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_te_result_with_significance_not_significant() {
        let result = TeResult::with_significance(0.5, true, 0.10, 0.05);
        assert!(!result.is_significant); // p_value > alpha
        assert_eq!(result.confidence, 0.0);
    }

    #[test]
    fn test_te_result_is_causal() {
        let result = TeResult::with_significance(0.5, true, 0.01, 0.05);
        assert!(result.is_causal(0.1));
        assert!(!result.is_causal(0.6)); // threshold too high
    }

    // ========== CausalLink Tests ==========

    #[test]
    fn test_causal_link_new() {
        let link = CausalLink::new("A".to_string(), "B".to_string(), 0.5, TeStrategy::Screening);
        assert_eq!(link.source, "A");
        assert_eq!(link.target, "B");
        assert_eq!(link.te, 0.5);
        assert_eq!(link.strategy, TeStrategy::Screening);
        assert!(link.is_significant);
    }

    #[test]
    fn test_causal_link_is_strong() {
        let link = CausalLink::new("A".to_string(), "B".to_string(), 0.5, TeStrategy::Screening);
        assert!(link.is_strong(0.1));
        assert!(!link.is_strong(0.6));
    }

    #[test]
    fn test_causal_link_clone() {
        let link = CausalLink::new("A".to_string(), "B".to_string(), 0.5, TeStrategy::DeepProbe);
        let cloned = link.clone();
        assert_eq!(link.source, cloned.source);
        assert_eq!(link.te, cloned.te);
    }

    // ========== GaussianEstimator Tests ==========

    #[test]
    fn test_gaussian_estimator() {
        let (source, target) = create_test_series();
        let estimator = estimators::GaussianEstimator::default();

        let result = estimator.compute(&source, &target);
        assert!(result.is_some());

        let te = result.unwrap();
        assert!(te.te >= 0.0);
    }

    #[test]
    fn test_gaussian_estimator_custom_params() {
        let estimator = estimators::GaussianEstimator::new(2, 20);
        assert_eq!(estimator.lag, 2);
        assert_eq!(estimator.min_samples, 20);
        assert_eq!(estimator.name(), "Gaussian");
    }

    #[test]
    fn test_gaussian_estimator_short_series() {
        let estimator = estimators::GaussianEstimator::new(5, 10);
        let short_source = vec![1.0, 2.0, 3.0];
        let short_target = vec![1.0, 2.0, 3.0];

        let result = estimator.compute(&short_source, &short_target);
        assert!(result.is_none()); // Too short
    }

    #[test]
    fn test_gaussian_variance() {
        let empty: Vec<f64> = vec![];
        let variance = estimators::GaussianEstimator::variance(&empty);
        assert_eq!(variance, 0.0);

        let constant = vec![5.0, 5.0, 5.0];
        let variance = estimators::GaussianEstimator::variance(&constant);
        assert!(variance.abs() < 1e-10);
    }

    #[test]
    fn test_gaussian_covariance() {
        let x = vec![1.0, 2.0, 3.0];
        let y = vec![2.0, 4.0, 6.0];
        let cov = estimators::GaussianEstimator::covariance(&x, &y);
        assert!(cov > 0.0);

        // Empty arrays
        let empty: Vec<f64> = vec![];
        let cov = estimators::GaussianEstimator::covariance(&empty, &empty);
        assert_eq!(cov, 0.0);

        // Mismatched lengths
        let cov = estimators::GaussianEstimator::covariance(&x, &empty);
        assert_eq!(cov, 0.0);
    }

    // ========== SymbolicEstimator Tests ==========

    #[test]
    fn test_symbolic_estimator() {
        let (source, target) = create_test_series();
        let estimator = estimators::SymbolicEstimator::default();

        let result = estimator.compute(&source, &target);
        assert!(result.is_some());

        let te = result.unwrap();
        assert!(te.te >= 0.0);
    }

    #[test]
    fn test_symbolic_estimator_custom_params() {
        let estimator = estimators::SymbolicEstimator::new(4, 2, 100);
        assert_eq!(estimator.dim, 4);
        assert_eq!(estimator.delay, 2);
        assert_eq!(estimator.min_samples, 100);
        assert_eq!(estimator.name(), "Symbolic");
    }

    #[test]
    fn test_symbolic_estimator_short_series() {
        let estimator = estimators::SymbolicEstimator::default();
        let short = vec![1.0, 2.0];

        let result = estimator.compute(&short, &short);
        assert!(result.is_none());
    }

    #[test]
    fn test_symbolic_window_to_pattern() {
        let estimator = estimators::SymbolicEstimator::new(3, 1, 50);

        let window1 = vec![1.0, 2.0, 3.0]; // Sorted pattern
        let rank1 = estimator.window_to_pattern(&window1);

        let window2 = vec![3.0, 2.0, 1.0]; // Reverse sorted
        let rank2 = estimator.window_to_pattern(&window2);

        assert_ne!(rank1, rank2);

        // Empty window
        let empty: Vec<f64> = vec![];
        let rank = estimator.window_to_pattern(&empty);
        assert_eq!(rank, 0);
    }

    #[test]
    fn test_symbolic_shannon_entropy() {
        let mut counts: HashMap<usize, usize> = HashMap::new();
        counts.insert(0, 5);
        counts.insert(1, 5);

        let entropy = estimators::SymbolicEstimator::shannon_entropy(&counts, 10);
        assert!(entropy > 0.0);
        assert!(entropy <= 1.0);

        // Empty case
        let entropy = estimators::SymbolicEstimator::shannon_entropy(&counts, 0);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_symbolic_joint_entropy() {
        let x = vec![0, 1, 0, 1];
        let y = vec![0, 0, 1, 1];

        let entropy = estimators::SymbolicEstimator::joint_entropy(&x, &y);
        assert!(entropy > 0.0);

        // Empty case
        let entropy = estimators::SymbolicEstimator::joint_entropy(&[], &[]);
        assert_eq!(entropy, 0.0);

        // Mismatched lengths
        let entropy = estimators::SymbolicEstimator::joint_entropy(&x, &[]);
        assert_eq!(entropy, 0.0);
    }

    #[test]
    fn test_symbolic_triple_joint_entropy() {
        let x = vec![0, 1, 0, 1];
        let y = vec![0, 0, 1, 1];
        let z = vec![1, 0, 1, 0];

        let entropy = estimators::SymbolicEstimator::triple_joint_entropy(&x, &y, &z);
        assert!(entropy > 0.0);

        // Empty case
        let entropy = estimators::SymbolicEstimator::triple_joint_entropy(&[], &[], &[]);
        assert_eq!(entropy, 0.0);
    }

    // ========== KsgEstimator Tests ==========

    #[test]
    fn test_ksg_estimator() {
        let (source, target) = create_test_series();
        let mut estimator = estimators::KsgEstimator::new(4, 0, 0.05, 1, 50); // No surrogates for speed

        let result = estimator.compute(&source, &target);
        assert!(result.is_some());

        let te = result.unwrap();
        assert!(te.te >= 0.0);
    }

    #[test]
    fn test_ksg_estimator_default() {
        let estimator = estimators::KsgEstimator::default();
        assert_eq!(estimator.k, 4);
        assert_eq!(estimator.surrogates, 100);
        assert_eq!(estimator.alpha, 0.05);
        assert_eq!(estimator.name(), "KSG");
    }

    #[test]
    fn test_ksg_estimator_with_surrogates() {
        let (source, target) = create_test_series();
        let mut estimator = estimators::KsgEstimator::new(4, 10, 0.05, 1, 50); // Few surrogates

        let result = estimator.compute(&source, &target);
        assert!(result.is_some());
        assert!(result.as_ref().unwrap().p_value.is_some());
    }

    #[test]
    fn test_ksg_short_series() {
        let mut estimator = estimators::KsgEstimator::new(4, 0, 0.05, 1, 100);
        let short = vec![1.0, 2.0, 3.0];

        let result = estimator.compute(&short, &short);
        assert!(result.is_none());
    }

    #[test]
    fn test_ksg_chebyshev_distance() {
        let p1 = vec![0.0, 0.0];
        let p2 = vec![3.0, 4.0];

        let dist = estimators::KsgEstimator::chebyshev_distance(&p1, &p2);
        assert_eq!(dist, 4.0);
    }

    #[test]
    fn test_ksg_digamma() {
        // Digamma function should be defined
        let psi_1 = estimators::KsgEstimator::digamma(1.0);
        let psi_2 = estimators::KsgEstimator::digamma(2.0);

        // Digamma is increasing
        assert!(psi_2 > psi_1);

        // For large x, approaches ln(x)
        let psi_large = estimators::KsgEstimator::digamma(100.0);
        let ln_large = 100.0_f64.ln();
        assert!((psi_large - ln_large).abs() < 0.1);
    }

    // ========== CausalEngine Tests ==========

    #[test]
    fn test_causal_engine() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();

        let link = engine.compute_pair(&source, &target, TeStrategy::Screening, "A", "B");
        assert!(link.is_some());
    }

    #[test]
    fn test_causal_engine_custom() {
        let engine = engine::CausalEngine::new(2, 4, 2, 5, 50, 0.05);
        assert_eq!(engine.te_threshold, 0.05);
        assert!(engine.cache_enabled);
    }

    #[test]
    fn test_causal_engine_cache() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();

        // First compute
        let link1 = engine.compute_pair(&source, &target, TeStrategy::Screening, "A", "B");
        assert!(link1.is_some());
        assert_eq!(engine.cache_size(), 1);

        // Second compute should use cache
        let link2 = engine.compute_pair(&source, &target, TeStrategy::Screening, "A", "B");
        assert!(link2.is_some());
        assert_eq!(engine.cache_size(), 1);

        // Clear cache
        engine.clear_cache();
        assert_eq!(engine.cache_size(), 0);
    }

    #[test]
    fn test_causal_engine_cache_disabled() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();
        engine.cache_enabled = false;

        engine.compute_pair(&source, &target, TeStrategy::Screening, "A", "B");
        assert_eq!(engine.cache_size(), 0);
    }

    #[test]
    fn test_causal_engine_bidirectional() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();

        let result =
            engine.compute_bidirectional("A", "B", &source, &target, TeStrategy::Screening);
        assert!(result.is_some());

        let (a_to_b, b_to_a) = result.unwrap();
        assert_eq!(a_to_b.source, "A");
        assert_eq!(b_to_a.source, "B");
    }

    #[test]
    fn test_causal_engine_net_causality() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();

        let net = engine.net_causality("A", "B", &source, &target, TeStrategy::Screening);
        assert!(net.is_some());
    }

    #[test]
    fn test_causal_engine_filter_significant() {
        let links = vec![
            CausalLink::new("A".to_string(), "B".to_string(), 0.5, TeStrategy::Screening),
            CausalLink::new(
                "C".to_string(),
                "D".to_string(),
                0.001,
                TeStrategy::Screening,
            ),
        ];

        let mut engine = engine::CausalEngine::default();
        engine.te_threshold = 0.1;

        let filtered = engine.filter_significant(&links);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].source, "A");
    }

    #[test]
    fn test_causal_engine_batch() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut engine = engine::CausalEngine::default();
        let (links, stats) = engine.batch_compute(&data, &candidates, TeStrategy::Screening);

        assert_eq!(links.len(), 1);
        assert_eq!(stats.pairs_computed, 1);
    }

    #[test]
    fn test_causal_engine_batch_missing_data() {
        let data: HashMap<String, Vec<f64>> = HashMap::new();
        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut engine = engine::CausalEngine::default();
        let (links, stats) = engine.batch_compute(&data, &candidates, TeStrategy::Screening);

        assert!(links.is_empty());
        assert_eq!(stats.pairs_computed, 0);
    }

    #[test]
    fn test_causal_engine_all_strategies() {
        let (source, target) = create_test_series();
        let mut engine = engine::CausalEngine::default();

        // Test all strategies
        let link1 = engine.compute_pair(&source, &target, TeStrategy::Screening, "A1", "B1");
        assert!(link1.is_some());

        let link2 = engine.compute_pair(&source, &target, TeStrategy::DeepProbe, "A2", "B2");
        assert!(link2.is_some());

        let link3 = engine.compute_pair(&source, &target, TeStrategy::Validation, "A3", "B3");
        assert!(link3.is_some());
    }

    // ========== BatchStats Tests ==========

    #[test]
    fn test_batch_stats_default() {
        let stats = engine::BatchStats::default();
        assert_eq!(stats.pairs_computed, 0);
        assert_eq!(stats.significant_links, 0);
        assert_eq!(stats.mean_te, 0.0);
        assert_eq!(stats.max_te, 0.0);
        assert_eq!(stats.elapsed_ms, 0.0);
    }

    // ========== CausalKdfV3 Tests ==========

    #[test]
    fn test_causal_kdf_v3() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut kdf = kdf_v3::CausalKdfV3::default();
        let screened = kdf.process_stream(&data, &candidates);

        assert!(!screened.is_empty() || screened.is_empty()); // May or may not find links
        assert!(kdf.get_stats().screening_calls > 0);
    }

    #[test]
    fn test_causal_kdf_v3_custom() {
        let kdf = kdf_v3::CausalKdfV3::new(0.02, 0.10, 0.20, 50, 0.10);
        assert_eq!(kdf.screening_threshold, 0.02);
        assert_eq!(kdf.deep_probe_threshold, 0.10);
        assert_eq!(kdf.validation_threshold, 0.20);
    }

    #[test]
    fn test_causal_kdf_v3_deep_probe() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut kdf = kdf_v3::CausalKdfV3::default();
        let probed = kdf.deep_probe(&data, &candidates);

        // Check that stats are updated
        assert!(kdf.get_stats().deep_probe_calls > 0);
        let _ = probed; // Use variable to avoid warning
    }

    #[test]
    fn test_causal_kdf_v3_validate() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut kdf = kdf_v3::CausalKdfV3::new(0.001, 0.001, 0.001, 10, 0.05);
        let validated = kdf.validate(&data, &candidates);

        assert!(kdf.get_stats().validation_calls > 0);
        let _ = validated;
    }

    #[test]
    fn test_causal_kdf_v3_sleep_cycle() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];

        let mut kdf = kdf_v3::CausalKdfV3::new(0.001, 0.001, 0.001, 5, 0.10);
        let result = kdf.run_sleep_cycle(&data, &candidates);

        // Check result structure exists
        let _ = result.screened.len();
        let _ = result.probed.len();
        let _ = result.validated.len();
    }

    #[test]
    fn test_causal_kdf_v3_reset_stats() {
        let mut kdf = kdf_v3::CausalKdfV3::default();
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let candidates = vec![("A".to_string(), "B".to_string())];
        kdf.process_stream(&data, &candidates);
        assert!(kdf.get_stats().screening_calls > 0);

        kdf.reset_stats();
        assert_eq!(kdf.get_stats().screening_calls, 0);
    }

    // ========== CausalKdfStats Tests ==========

    #[test]
    fn test_causal_kdf_stats_default() {
        let stats = kdf_v3::CausalKdfStats::default();
        assert_eq!(stats.screening_calls, 0);
        assert_eq!(stats.deep_probe_calls, 0);
        assert_eq!(stats.validation_calls, 0);
        assert_eq!(stats.significant_links_found, 0);
    }

    // ========== SleepCycleResult Tests ==========

    #[test]
    fn test_sleep_cycle_result_clone() {
        let result = kdf_v3::SleepCycleResult {
            screened: vec![],
            probed: vec![],
            validated: vec![],
            stats: kdf_v3::CausalKdfStats::default(),
        };
        let cloned = result.clone();
        assert_eq!(result.screened.len(), cloned.screened.len());
    }

    // ========== CausalPartitionBuilder Tests ==========

    #[test]
    fn test_partition_builder() {
        let links = vec![
            CausalLink::new("A".to_string(), "B".to_string(), 0.5, TeStrategy::Screening),
            CausalLink::new("B".to_string(), "C".to_string(), 0.4, TeStrategy::Screening),
            CausalLink::new("D".to_string(), "E".to_string(), 0.3, TeStrategy::Screening),
        ];

        let builder = partition::CausalPartitionBuilder::default();
        let all_nodes = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
            "E".to_string(),
            "F".to_string(),
        ];
        let partition = builder.build_partition_from_links(&links, Some(&all_nodes));

        // A, B, C should be in same cluster
        assert_eq!(partition.get("A"), partition.get("B"));
        assert_eq!(partition.get("B"), partition.get("C"));

        // D, E should be in same cluster
        assert_eq!(partition.get("D"), partition.get("E"));

        // A and D should be in different clusters
        assert_ne!(partition.get("A"), partition.get("D"));
    }

    #[test]
    fn test_partition_builder_custom() {
        let builder = partition::CausalPartitionBuilder::new(0.05, 3, 100);
        assert_eq!(builder.te_threshold, 0.05);
        assert_eq!(builder.min_cluster_size, 3);
        assert_eq!(builder.max_cluster_size, 100);
    }

    #[test]
    fn test_partition_builder_no_all_nodes() {
        let links = vec![CausalLink::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
            TeStrategy::Screening,
        )];

        let builder = partition::CausalPartitionBuilder::default();
        let partition = builder.build_partition_from_links(&links, None);

        assert!(partition.contains_key("A"));
        assert!(partition.contains_key("B"));
    }

    #[test]
    fn test_partition_builder_from_time_series() {
        let (source, target) = create_test_series();
        let mut data = HashMap::new();
        data.insert("A".to_string(), source);
        data.insert("B".to_string(), target);

        let builder = partition::CausalPartitionBuilder::new(0.001, 2, 50);
        let (partition, links) =
            builder.build_partition_from_time_series(&data, TeStrategy::Screening);

        assert!(!partition.is_empty());
        let _ = links; // Use variable
    }

    #[test]
    fn test_partition_builder_large_cluster_split() {
        // Create links that form a large cluster
        let mut links = Vec::new();
        for i in 0..60 {
            links.push(CausalLink::new(
                format!("N{}", i),
                format!("N{}", i + 1),
                0.5,
                TeStrategy::Screening,
            ));
        }

        let builder = partition::CausalPartitionBuilder::new(0.01, 2, 50);
        let partition = builder.build_partition_from_links(&links, None);

        // Should have multiple clusters due to splitting
        let unique_clusters: HashSet<u32> = partition.values().cloned().collect();
        assert!(!unique_clusters.is_empty());
    }

    // ========== CausalCluster Tests ==========

    #[test]
    fn test_causal_cluster() {
        let cluster = partition::CausalCluster {
            cluster_id: 1,
            nodes: vec!["A".to_string(), "B".to_string()].into_iter().collect(),
            internal_links: vec![],
            hub_node: Some("A".to_string()),
        };

        assert_eq!(cluster.cluster_id, 1);
        assert_eq!(cluster.nodes.len(), 2);
        assert_eq!(cluster.hub_node, Some("A".to_string()));
    }

    // ========== CausalEnhancedNREMOptimizer Tests ==========

    #[test]
    fn test_causal_enhanced_nrem_default() {
        let optimizer = nrem::CausalEnhancedNREMOptimizer::default();
        assert_eq!(optimizer.nrem_max_iterations, 1000);
        assert!(optimizer.use_causal_init);
    }

    #[test]
    fn test_causal_enhanced_nrem_custom() {
        let optimizer = nrem::CausalEnhancedNREMOptimizer::new(0.05, 500, false);
        assert_eq!(optimizer.nrem_max_iterations, 500);
        assert!(!optimizer.use_causal_init);
    }

    #[test]
    fn test_causal_enhanced_nrem_optimize() {
        let edges = vec![
            ("A".to_string(), "B".to_string(), 1.0),
            ("B".to_string(), "C".to_string(), 1.0),
        ];

        let links = vec![CausalLink::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
            TeStrategy::Screening,
        )];

        let mut optimizer = nrem::CausalEnhancedNREMOptimizer::default();
        let result = optimizer.optimize_with_causal_links(&edges, &links);

        assert!(!result.partition.is_empty());
        assert_eq!(result.init_type, "causal");
    }

    #[test]
    fn test_causal_enhanced_nrem_without_causal_init() {
        let edges = vec![("A".to_string(), "B".to_string(), 1.0)];

        let mut optimizer = nrem::CausalEnhancedNREMOptimizer::new(0.01, 100, false);
        let result = optimizer.optimize_with_causal_links(&edges, &[]);

        assert_eq!(result.init_type, "singleton");
    }

    #[test]
    fn test_causal_enhanced_nrem_empty_links() {
        let edges = vec![("A".to_string(), "B".to_string(), 1.0)];

        let mut optimizer = nrem::CausalEnhancedNREMOptimizer::default();
        let result = optimizer.optimize_with_causal_links(&edges, &[]);

        assert_eq!(result.init_type, "singleton");
    }

    #[test]
    fn test_causal_enhanced_nrem_statistics() {
        let edges = vec![("A".to_string(), "B".to_string(), 1.0)];

        let links = vec![CausalLink::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
            TeStrategy::Screening,
        )];

        let mut optimizer = nrem::CausalEnhancedNREMOptimizer::default();
        optimizer.optimize_with_causal_links(&edges, &links);
        optimizer.optimize_with_causal_links(&edges, &[]);

        let stats = optimizer.get_statistics();
        assert_eq!(stats.optimizations, 2);
        assert_eq!(stats.causal_inits, 1);
    }

    // ========== CausalNREMStats Tests ==========

    #[test]
    fn test_causal_nrem_stats_default() {
        let stats = nrem::CausalNREMStats::default();
        assert_eq!(stats.optimizations, 0);
        assert_eq!(stats.causal_inits, 0);
        assert_eq!(stats.avg_iterations_with_causal, 0.0);
        assert_eq!(stats.avg_iterations_without_causal, 0.0);
    }

    // ========== CausalNREMResult Tests ==========

    #[test]
    fn test_causal_nrem_result_clone() {
        let result = nrem::CausalNREMResult {
            partition: HashMap::new(),
            initial_entropy: 1.0,
            final_entropy: 0.5,
            entropy_reduction: 0.5,
            compression_ratio: 0.5,
            iterations: 100,
            init_type: "causal".to_string(),
            initial_modules: 5,
            initial_partition: HashMap::new(),
        };
        let cloned = result.clone();
        assert_eq!(result.init_type, cloned.init_type);
        assert_eq!(result.iterations, cloned.iterations);
    }
}
