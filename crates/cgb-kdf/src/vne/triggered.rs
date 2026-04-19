//! VNE-triggered sleep mode integration

use std::collections::HashMap;
use super::types::AnomalyResult;
use super::monitor::VNEMonitor;

/// VNE-Triggered Sleep Mode Integration
///
/// Automatically triggers NREM optimization when VNE anomaly is detected.
///
/// Integration Pattern #1: VNE-Triggered Sleep Mode
pub struct VNETriggeredSleepMode {
    /// VNE monitor
    monitor: VNEMonitor,
    /// Maximum NREM iterations
    pub nrem_max_iterations: u64,
    /// Auto-trigger enabled
    pub auto_trigger: bool,
    /// Statistics
    stats: VNETriggeredStats,
}

/// Statistics for VNE-triggered sleep mode
#[derive(Clone, Debug, Default)]
pub struct VNETriggeredStats {
    /// Number of updates
    pub updates: u64,
    /// Number of anomalies detected
    pub anomalies_detected: u64,
    /// Number of optimizations triggered
    pub optimizations_triggered: u64,
    /// Total entropy reduction
    pub total_entropy_reduction: f64,
}

/// Optimization result from VNE trigger
#[derive(Clone, Debug)]
pub struct OptimizationResult {
    /// Whether optimization was triggered
    pub triggered: bool,
    /// Trigger reason
    pub trigger_reason: String,
    /// VNE z-score that triggered optimization
    pub vne_z_score: f64,
    /// Initial structural entropy
    pub initial_entropy: f64,
    /// Final structural entropy
    pub final_entropy: f64,
    /// Entropy reduction
    pub entropy_reduction: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Iterations performed
    pub iterations: u64,
    /// Optimized partition (node_id -> module_id)
    pub partition: HashMap<String, u32>,
}

impl VNETriggeredSleepMode {
    /// Create a new VNE-triggered sleep mode
    pub fn new(
        max_history: usize,
        anomaly_threshold: f64,
        nrem_max_iterations: u64,
        auto_trigger: bool,
    ) -> Self {
        Self {
            monitor: VNEMonitor::new(max_history, anomaly_threshold),
            nrem_max_iterations,
            auto_trigger,
            stats: VNETriggeredStats::default(),
        }
    }
}

impl Default for VNETriggeredSleepMode {
    fn default() -> Self {
        Self::new(100, 2.0, 1000, true)
    }
}

impl VNETriggeredSleepMode {
    /// Update with graph state and optionally trigger NREM optimization
    ///
    /// Returns (anomaly_result, optional_optimization_result)
    pub fn update(
        &mut self,
        node_count: usize,
        edges: &[(u32, u32, f64)],
        initial_partition: Option<&HashMap<String, u32>>,
    ) -> (AnomalyResult, Option<OptimizationResult>) {
        use crate::sleep_mode::SleepModeOptimizer;

        self.stats.updates += 1;

        // Record VNE and check for anomaly
        let anomaly_result = self.monitor.record(node_count, edges);

        let mut optimization_result = None;

        if anomaly_result.is_anomaly && self.auto_trigger {
            self.stats.anomalies_detected += 1;

            // Build partition if not provided
            let partition: HashMap<String, u32> = match initial_partition {
                Some(p) => p.clone(),
                None => (0..node_count)
                    .map(|i| (i.to_string(), i as u32))
                    .collect(),
            };

            // Convert edges to string-based format for optimizer
            let string_edges: Vec<(String, String, f64)> = edges
                .iter()
                .map(|(u, v, w)| (u.to_string(), v.to_string(), *w))
                .collect();

            // Run NREM optimization
            let mut optimizer = SleepModeOptimizer::new(
                1.0,   // initial_temperature
                0.001, // final_temperature
                self.nrem_max_iterations,
                1000,  // resync_interval
            );

            let nrem_result = optimizer.run_nrem_phase(&string_edges, Some(partition));

            optimization_result = Some(OptimizationResult {
                triggered: true,
                trigger_reason: "vne_anomaly".to_string(),
                vne_z_score: anomaly_result.z_score,
                initial_entropy: nrem_result.initial_entropy,
                final_entropy: nrem_result.final_entropy,
                entropy_reduction: nrem_result.entropy_reduction,
                compression_ratio: nrem_result.compression_ratio,
                iterations: nrem_result.iterations,
                partition: nrem_result.partition,
            });

            self.stats.optimizations_triggered += 1;
            self.stats.total_entropy_reduction += nrem_result.entropy_reduction;
        }

        (anomaly_result, optimization_result)
    }

    /// Force NREM optimization without anomaly trigger
    pub fn force_optimization(
        &mut self,
        node_count: usize,
        edges: &[(u32, u32, f64)],
        initial_partition: Option<&HashMap<String, u32>>,
    ) -> OptimizationResult {
        use crate::sleep_mode::SleepModeOptimizer;

        // Build partition if not provided
        let partition: HashMap<String, u32> = match initial_partition {
            Some(p) => p.clone(),
            None => (0..node_count)
                .map(|i| (i.to_string(), i as u32))
                .collect(),
        };

        // Convert edges to string-based format
        let string_edges: Vec<(String, String, f64)> = edges
            .iter()
            .map(|(u, v, w)| (u.to_string(), v.to_string(), *w))
            .collect();

        // Run NREM optimization
        let mut optimizer = SleepModeOptimizer::new(
            1.0,
            0.001,
            self.nrem_max_iterations,
            1000,
        );

        let nrem_result = optimizer.run_nrem_phase(&string_edges, Some(partition));

        let result = OptimizationResult {
            triggered: true,
            trigger_reason: "forced".to_string(),
            vne_z_score: 0.0,
            initial_entropy: nrem_result.initial_entropy,
            final_entropy: nrem_result.final_entropy,
            entropy_reduction: nrem_result.entropy_reduction,
            compression_ratio: nrem_result.compression_ratio,
            iterations: nrem_result.iterations,
            partition: nrem_result.partition,
        };

        self.stats.optimizations_triggered += 1;
        self.stats.total_entropy_reduction += nrem_result.entropy_reduction;

        result
    }

    /// Get statistics
    pub fn get_statistics(&self) -> VNETriggeredStats {
        self.stats.clone()
    }

    /// Get VNE history size
    pub fn history_size(&self) -> usize {
        self.monitor.get_history().len()
    }

    /// Get last VNE value
    pub fn last_vne(&self) -> Option<f64> {
        self.monitor.get_history().last().copied()
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.monitor.clear();
        self.stats = VNETriggeredStats::default();
    }
}
