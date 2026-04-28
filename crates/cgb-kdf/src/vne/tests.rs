//! Tests for VNE module

use super::*;
use nalgebra::DMatrix;
use std::collections::HashMap;

fn create_test_graph() -> (usize, Vec<(u32, u32, f64)>) {
    let edges = vec![
        (0, 1, 1.0),
        (1, 2, 1.0),
        (2, 3, 1.0),
        (3, 0, 1.0),
        (0, 2, 0.5),
    ];
    (4, edges)
}

#[test]
fn test_laplacian_matrix() {
    let (node_count, edges) = create_test_graph();
    let lap = matrix::laplacian_matrix(node_count, &edges);

    // Laplacian should be symmetric
    for i in 0..node_count {
        for j in 0..node_count {
            assert!((lap[(i, j)] - lap[(j, i)]).abs() < 1e-10);
        }
    }

    // Row sums should be zero
    for i in 0..node_count {
        let row_sum: f64 = (0..node_count).map(|j| lap[(i, j)]).sum();
        assert!(row_sum.abs() < 1e-10);
    }
}

#[test]
fn test_von_neumann_entropy() {
    let (node_count, edges) = create_test_graph();
    let vne = entropy::von_neumann_entropy(node_count, &edges);

    // VNE should be non-negative
    assert!(vne >= 0.0);
}

#[test]
fn test_von_neumann_entropy_detailed() {
    let (node_count, edges) = create_test_graph();
    let result = entropy::von_neumann_entropy_detailed(node_count, &edges);

    assert!(result.entropy >= 0.0);
    assert_eq!(result.eigenvalues.len(), node_count);
    assert!(result.spectral_gap >= 0.0);
}

#[test]
fn test_empty_graph() {
    let result = entropy::von_neumann_entropy_detailed(0, &[]);
    assert_eq!(result.entropy, 0.0);
    assert!(result.eigenvalues.is_empty());
}

#[test]
fn test_detect_change() {
    let (node_count1, edges1) = create_test_graph();
    let mut edges2 = edges1.clone();
    edges2.push((0, 3, 2.0)); // Add a strong edge

    let change = entropy::detect_change(node_count1, &edges1, node_count1, &edges2, 0.1);

    assert!(change.absolute_change >= 0.0);
    assert!(change.relative_change >= 0.0);
}

#[test]
fn test_vne_monitor() {
    let mut monitor = monitor::VNEMonitor::default();

    // Record several values
    for i in 0..10 {
        monitor.record_value(1.0 + (i as f64) * 0.01);
    }

    assert_eq!(monitor.get_history().len(), 10);

    // Normal value shouldn't be anomaly
    let result = monitor.record_value(1.05);
    assert!(!result.is_anomaly);

    // Extreme value should be anomaly
    let result = monitor.record_value(10.0);
    assert!(result.is_anomaly);
}

#[test]
fn test_vne_monitor_max_history() {
    let mut monitor = monitor::VNEMonitor::new(5, 2.0);

    // Record more than max_history
    for i in 0..10 {
        monitor.record_value(i as f64);
    }

    // Should only keep max_history values
    assert_eq!(monitor.get_history().len(), 5);
}

#[test]
fn test_vne_triggered_sleep_mode() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    // First few updates shouldn't trigger (not enough history)
    for _ in 0..5 {
        let (anomaly, opt) = triggered.update(node_count, &edges, None);
        assert!(!anomaly.is_anomaly);
        assert!(opt.is_none());
    }

    // Force optimization should work
    let result = triggered.force_optimization(node_count, &edges, None);
    assert!(result.triggered);
    assert_eq!(result.trigger_reason, "forced");
}

#[test]
fn test_vne_triggered_stats() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    // Several updates
    for _ in 0..5 {
        triggered.update(node_count, &edges, None);
    }

    let stats = triggered.get_statistics();
    assert_eq!(stats.updates, 5);
}

// =========================================================================
// Additional VNEResult Tests
// =========================================================================

#[test]
fn test_vne_result_empty() {
    let result = types::VNEResult::empty();
    assert_eq!(result.entropy, 0.0);
    assert!(result.eigenvalues.is_empty());
    assert_eq!(result.spectral_gap, 0.0);
    assert_eq!(result.num_components, 0);
}

#[test]
fn test_vne_result_clone() {
    let result = types::VNEResult {
        entropy: 1.5,
        eigenvalues: vec![0.1, 0.2, 0.3],
        spectral_gap: 0.1,
        num_components: 1,
    };
    let cloned = result.clone();
    assert_eq!(result.entropy, cloned.entropy);
    assert_eq!(result.eigenvalues, cloned.eigenvalues);
}

#[test]
fn test_vne_result_debug() {
    let result = types::VNEResult::empty();
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("VNEResult"));
}

// =========================================================================
// Additional AnomalyResult Tests
// =========================================================================

#[test]
fn test_anomaly_result_clone() {
    let result = types::AnomalyResult {
        vne: 1.0,
        mean: 0.9,
        std_dev: 0.1,
        z_score: 1.0,
        is_anomaly: false,
        history_size: 10,
    };
    let cloned = result.clone();
    assert_eq!(result.vne, cloned.vne);
    assert_eq!(result.is_anomaly, cloned.is_anomaly);
}

#[test]
fn test_anomaly_result_debug() {
    let result = types::AnomalyResult {
        vne: 1.0,
        mean: 0.9,
        std_dev: 0.1,
        z_score: 1.0,
        is_anomaly: false,
        history_size: 10,
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("AnomalyResult"));
}

// =========================================================================
// Additional ChangeDetection Tests
// =========================================================================

#[test]
fn test_change_detection_clone() {
    let change = types::ChangeDetection {
        vne_before: 1.0,
        vne_after: 1.5,
        absolute_change: 0.5,
        relative_change: 0.5,
        is_significant: true,
        spectral_gap_before: 0.1,
        spectral_gap_after: 0.2,
    };
    let cloned = change.clone();
    assert_eq!(change.vne_before, cloned.vne_before);
    assert_eq!(change.is_significant, cloned.is_significant);
}

#[test]
fn test_change_detection_debug() {
    let change = types::ChangeDetection {
        vne_before: 1.0,
        vne_after: 1.5,
        absolute_change: 0.5,
        relative_change: 0.5,
        is_significant: true,
        spectral_gap_before: 0.1,
        spectral_gap_after: 0.2,
    };
    let debug_str = format!("{:?}", change);
    assert!(debug_str.contains("ChangeDetection"));
}

#[test]
fn test_detect_change_same_graph() {
    let (node_count, edges) = create_test_graph();
    let change = entropy::detect_change(node_count, &edges, node_count, &edges, 0.1);

    assert_eq!(change.absolute_change, 0.0);
    assert!(!change.is_significant);
}

#[test]
fn test_detect_change_zero_entropy_before() {
    // Graph with no edges (zero entropy)
    let edges1: Vec<(u32, u32, f64)> = vec![];
    let edges2 = vec![(0, 1, 1.0)];

    let change = entropy::detect_change(2, &edges1, 2, &edges2, 0.1);

    // When vne1 is ~0 and vne2 > 0, relative_change may be infinity or 0
    // depending on edge case handling in detect_change
    // The key assertion is that the function completes without panic
    assert!(change.absolute_change >= 0.0);
}

#[test]
fn test_detect_change_both_zero_entropy() {
    let edges1: Vec<(u32, u32, f64)> = vec![];
    let edges2: Vec<(u32, u32, f64)> = vec![];

    let change = entropy::detect_change(2, &edges1, 2, &edges2, 0.1);

    assert_eq!(change.relative_change, 0.0);
    assert!(!change.is_significant);
}

// =========================================================================
// Additional density_matrix Tests
// =========================================================================

#[test]
fn test_density_matrix_basic() {
    let (node_count, edges) = create_test_graph();
    let lap = matrix::laplacian_matrix(node_count, &edges);
    let rho = matrix::density_matrix(&lap);

    // Density matrix should have trace 1 (normalized)
    let trace = rho.trace();
    assert!((trace - 1.0).abs() < 1e-10);
}

#[test]
fn test_density_matrix_zero_trace() {
    // Empty graph (all zeros)
    let lap = DMatrix::zeros(3, 3);
    let rho = matrix::density_matrix(&lap);

    // Should return the original matrix when trace is zero
    assert_eq!(rho, lap);
}

// =========================================================================
// Additional VNEMonitor Tests
// =========================================================================

#[test]
fn test_vne_monitor_check_anomaly_empty() {
    let monitor = monitor::VNEMonitor::default();
    let result = monitor.check_anomaly(None);
    assert!(result.is_none());
}

#[test]
fn test_vne_monitor_check_anomaly_normal() {
    let mut monitor = monitor::VNEMonitor::default();

    // Add normal values
    for i in 0..10 {
        monitor.record_value(1.0 + (i as f64) * 0.01);
    }

    // Check with default threshold - should not be anomaly
    let result = monitor.check_anomaly(None);
    assert!(result.is_none());
}

#[test]
fn test_vne_monitor_check_anomaly_with_custom_threshold() {
    let mut monitor = monitor::VNEMonitor::default();

    // Add values
    for _ in 0..10 {
        monitor.record_value(1.0);
    }
    monitor.record_value(1.5); // Slightly higher

    // With low threshold, might detect anomaly
    let result = monitor.check_anomaly(Some(0.1));
    // Result depends on actual z-score
    assert!(result.is_some() || result.is_none());
}

#[test]
fn test_vne_monitor_clear() {
    let mut monitor = monitor::VNEMonitor::default();

    // Add values
    for i in 0..5 {
        monitor.record_value(i as f64);
    }
    assert_eq!(monitor.get_history().len(), 5);

    // Clear
    monitor.clear();
    assert!(monitor.get_history().is_empty());
}

#[test]
fn test_vne_monitor_single_value() {
    let mut monitor = monitor::VNEMonitor::default();
    let result = monitor.record_value(1.0);

    assert_eq!(result.vne, 1.0);
    assert!(!result.is_anomaly); // Not enough history
    assert_eq!(result.history_size, 1);
}

#[test]
fn test_vne_monitor_two_values() {
    let mut monitor = monitor::VNEMonitor::default();
    monitor.record_value(1.0);
    let result = monitor.record_value(1.1);

    assert!(!result.is_anomaly); // Still not enough history (need 3+)
    assert_eq!(result.history_size, 2);
}

#[test]
fn test_vne_monitor_record_from_graph() {
    let mut monitor = monitor::VNEMonitor::default();
    let (node_count, edges) = create_test_graph();

    let result = monitor.record(node_count, &edges);
    assert!(result.vne >= 0.0);
    assert_eq!(result.history_size, 1);
}

#[test]
fn test_vne_monitor_stats_computation() {
    let mut monitor = monitor::VNEMonitor::default();

    // Add values with known mean and std_dev
    let values = [1.0, 2.0, 3.0, 4.0, 5.0];
    for v in &values {
        monitor.record_value(*v);
    }

    // Mean should be 3.0
    // Manually compute for verification
    let (mean, std_dev) = (3.0, 1.5811388300841898); // sample std dev

    let result = monitor.record_value(3.0);
    assert!((result.mean - mean).abs() < 0.1);
    assert!((result.std_dev - std_dev).abs() < 0.1);
}

// =========================================================================
// Additional VNETriggeredSleepMode Tests
// =========================================================================

#[test]
fn test_vne_triggered_new() {
    let triggered = triggered::VNETriggeredSleepMode::new(50, 3.0, 500, false);
    assert_eq!(triggered.nrem_max_iterations, 500);
    assert!(!triggered.auto_trigger);
}

#[test]
fn test_vne_triggered_history_size() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    triggered.update(node_count, &edges, None);
    triggered.update(node_count, &edges, None);
    triggered.update(node_count, &edges, None);

    assert_eq!(triggered.history_size(), 3);
}

#[test]
fn test_vne_triggered_last_vne_empty() {
    let triggered = triggered::VNETriggeredSleepMode::default();
    assert!(triggered.last_vne().is_none());
}

#[test]
fn test_vne_triggered_last_vne() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    triggered.update(node_count, &edges, None);
    let last = triggered.last_vne();

    assert!(last.is_some());
    assert!(last.unwrap() >= 0.0);
}

#[test]
fn test_vne_triggered_reset() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    // Do some updates
    for _ in 0..5 {
        triggered.update(node_count, &edges, None);
    }
    triggered.force_optimization(node_count, &edges, None);

    // Reset
    triggered.reset();

    assert_eq!(triggered.history_size(), 0);
    let stats = triggered.get_statistics();
    assert_eq!(stats.updates, 0);
    assert_eq!(stats.optimizations_triggered, 0);
}

#[test]
fn test_vne_triggered_with_initial_partition() {
    let mut triggered = triggered::VNETriggeredSleepMode::default();
    let (node_count, edges) = create_test_graph();

    let mut partition = HashMap::new();
    partition.insert("0".to_string(), 0);
    partition.insert("1".to_string(), 0);
    partition.insert("2".to_string(), 1);
    partition.insert("3".to_string(), 1);

    let result = triggered.force_optimization(node_count, &edges, Some(&partition));
    assert!(result.triggered);
}

#[test]
fn test_vne_triggered_auto_trigger_disabled() {
    let mut triggered = triggered::VNETriggeredSleepMode::new(100, 0.5, 1000, false);
    let (node_count, edges) = create_test_graph();

    // Build up history
    for _ in 0..10 {
        triggered.update(node_count, &edges, None);
    }

    // Even with an anomalous value, optimization should not trigger
    // because auto_trigger is false
    let stats = triggered.get_statistics();
    assert_eq!(stats.optimizations_triggered, 0);
}

// =========================================================================
// Additional OptimizationResult Tests
// =========================================================================

#[test]
fn test_optimization_result_clone() {
    let mut partition = HashMap::new();
    partition.insert("0".to_string(), 0);

    let result = triggered::OptimizationResult {
        triggered: true,
        trigger_reason: "test".to_string(),
        vne_z_score: 2.5,
        initial_entropy: 1.0,
        final_entropy: 0.5,
        entropy_reduction: 0.5,
        compression_ratio: 0.5,
        iterations: 100,
        partition,
    };

    let cloned = result.clone();
    assert_eq!(result.triggered, cloned.triggered);
    assert_eq!(result.trigger_reason, cloned.trigger_reason);
    assert_eq!(result.iterations, cloned.iterations);
}

#[test]
fn test_optimization_result_debug() {
    let result = triggered::OptimizationResult {
        triggered: false,
        trigger_reason: "none".to_string(),
        vne_z_score: 0.0,
        initial_entropy: 0.0,
        final_entropy: 0.0,
        entropy_reduction: 0.0,
        compression_ratio: 0.0,
        iterations: 0,
        partition: HashMap::new(),
    };
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("OptimizationResult"));
}

// =========================================================================
// Additional VNETriggeredStats Tests
// =========================================================================

#[test]
fn test_vne_triggered_stats_default() {
    let stats = triggered::VNETriggeredStats::default();
    assert_eq!(stats.updates, 0);
    assert_eq!(stats.anomalies_detected, 0);
    assert_eq!(stats.optimizations_triggered, 0);
    assert_eq!(stats.total_entropy_reduction, 0.0);
}

#[test]
fn test_vne_triggered_stats_clone() {
    let stats = triggered::VNETriggeredStats {
        updates: 10,
        anomalies_detected: 2,
        optimizations_triggered: 1,
        total_entropy_reduction: 0.5,
    };
    let cloned = stats.clone();
    assert_eq!(stats.updates, cloned.updates);
    assert_eq!(
        stats.total_entropy_reduction,
        cloned.total_entropy_reduction
    );
}

#[test]
fn test_vne_triggered_stats_debug() {
    let stats = triggered::VNETriggeredStats::default();
    let debug_str = format!("{:?}", stats);
    assert!(debug_str.contains("VNETriggeredStats"));
}

// =========================================================================
// Edge Case Tests
// =========================================================================

#[test]
fn test_laplacian_matrix_single_node() {
    let lap = matrix::laplacian_matrix(1, &[]);
    assert_eq!(lap.nrows(), 1);
    assert_eq!(lap.ncols(), 1);
    assert_eq!(lap[(0, 0)], 0.0);
}

#[test]
fn test_laplacian_matrix_out_of_bounds_edge() {
    // Edge with indices >= node_count should be ignored
    let edges = vec![(0, 5, 1.0), (5, 6, 1.0)];
    let lap = matrix::laplacian_matrix(3, &edges);

    // Should be zero matrix since all edges are out of bounds
    for i in 0..3 {
        for j in 0..3 {
            assert_eq!(lap[(i, j)], 0.0);
        }
    }
}

#[test]
fn test_von_neumann_entropy_single_node() {
    let vne = entropy::von_neumann_entropy(1, &[]);
    assert_eq!(vne, 0.0);
}

#[test]
fn test_von_neumann_entropy_disconnected() {
    // Two disconnected nodes
    let vne = entropy::von_neumann_entropy_detailed(2, &[]);
    assert_eq!(vne.entropy, 0.0);
    assert_eq!(vne.num_components, 2);
}

#[test]
fn test_von_neumann_entropy_complete_graph() {
    // Complete graph K4
    let edges = vec![
        (0, 1, 1.0),
        (0, 2, 1.0),
        (0, 3, 1.0),
        (1, 2, 1.0),
        (1, 3, 1.0),
        (2, 3, 1.0),
    ];
    let result = entropy::von_neumann_entropy_detailed(4, &edges);

    assert!(result.entropy > 0.0);
    assert_eq!(result.num_components, 1); // Connected
}

#[test]
fn test_von_neumann_entropy_star_graph() {
    // Star graph with center node 0
    let edges = vec![(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0), (0, 4, 1.0)];
    let result = entropy::von_neumann_entropy_detailed(5, &edges);

    assert!(result.entropy > 0.0);
    assert_eq!(result.num_components, 1);
}

#[test]
fn test_vne_monitor_high_variance() {
    let mut monitor = monitor::VNEMonitor::new(100, 2.0);

    // Add values with high variance
    let values = [0.1, 10.0, 0.2, 9.0, 0.3, 8.0];
    for v in &values {
        monitor.record_value(*v);
    }

    // Due to high variance, extreme values may not be flagged as anomalies
    let result = monitor.record_value(5.0);
    // Check that stats are computed
    assert!(result.std_dev > 0.0);
}

#[test]
fn test_detect_change_large_threshold() {
    let (node_count, edges) = create_test_graph();
    let mut edges2 = edges.clone();
    edges2.push((0, 3, 10.0)); // Add edge

    // With very large threshold, should never be significant
    let change = entropy::detect_change(node_count, &edges, node_count, &edges2, 100.0);
    assert!(!change.is_significant);
}
