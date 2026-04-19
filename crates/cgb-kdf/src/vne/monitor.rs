//! VNE monitoring and anomaly detection

use super::types::AnomalyResult;
use super::entropy::von_neumann_entropy;

/// VNE Monitor for real-time tracking and anomaly detection
pub struct VNEMonitor {
    /// Maximum history size
    pub max_history: usize,
    /// Anomaly threshold (standard deviations)
    pub anomaly_threshold: f64,
    /// VNE history
    history: Vec<f64>,
}

impl VNEMonitor {
    /// Create a new VNE monitor
    pub fn new(max_history: usize, anomaly_threshold: f64) -> Self {
        Self {
            max_history,
            anomaly_threshold,
            history: Vec::new(),
        }
    }
}

impl Default for VNEMonitor {
    fn default() -> Self {
        Self::new(100, 2.0)
    }
}

impl VNEMonitor {
    /// Record VNE from graph and check for anomaly
    pub fn record(&mut self, node_count: usize, edges: &[(u32, u32, f64)]) -> AnomalyResult {
        let vne = von_neumann_entropy(node_count, edges);
        self.record_value(vne)
    }

    /// Record VNE value directly
    pub fn record_value(&mut self, vne: f64) -> AnomalyResult {
        let (mean, std_dev) = self.compute_stats();

        // Compute z-score
        let z_score = if std_dev > 1e-10 {
            (vne - mean) / std_dev
        } else {
            0.0
        };

        // Anomaly detection (need at least 3 history points)
        let is_anomaly = self.history.len() >= 3 && z_score.abs() > self.anomaly_threshold;

        // Update history
        if self.history.len() >= self.max_history {
            self.history.remove(0);
        }
        self.history.push(vne);

        AnomalyResult {
            vne,
            mean,
            std_dev,
            z_score,
            is_anomaly,
            history_size: self.history.len(),
        }
    }

    /// Compute mean and standard deviation of history
    fn compute_stats(&self) -> (f64, f64) {
        if self.history.is_empty() {
            return (0.0, 0.0);
        }

        let n = self.history.len() as f64;
        let mean: f64 = self.history.iter().sum::<f64>() / n;

        if self.history.len() < 2 {
            return (mean, 0.0);
        }

        let variance: f64 = self.history.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / (n - 1.0);

        (mean, variance.sqrt())
    }

    /// Get history
    pub fn get_history(&self) -> &[f64] {
        &self.history
    }

    /// Clear history
    pub fn clear(&mut self) {
        self.history.clear();
    }

    /// Check if latest state is anomaly
    pub fn check_anomaly(&self, z_threshold: Option<f64>) -> Option<AnomalyResult> {
        if self.history.is_empty() {
            return None;
        }

        let threshold = z_threshold.unwrap_or(self.anomaly_threshold);
        let (mean, std_dev) = self.compute_stats();
        let latest_vne = *self.history.last().unwrap();

        let z_score = if std_dev > 1e-10 {
            (latest_vne - mean) / std_dev
        } else {
            0.0
        };

        if z_score.abs() > threshold {
            Some(AnomalyResult {
                vne: latest_vne,
                mean,
                std_dev,
                z_score,
                is_anomaly: true,
                history_size: self.history.len(),
            })
        } else {
            None
        }
    }
}
