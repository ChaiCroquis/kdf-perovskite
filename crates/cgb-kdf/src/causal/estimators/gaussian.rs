//! Gaussian Transfer Entropy Estimator

use super::super::types::TeResult;

/// Gaussian-based Transfer Entropy Estimator
///
/// Fast linear estimator for Edge layer screening.
/// Complexity: O(N)
pub struct GaussianEstimator {
    /// Time lag
    pub lag: usize,
    /// Minimum samples required
    pub min_samples: usize,
}

impl GaussianEstimator {
    /// Create a new Gaussian estimator
    pub fn new(lag: usize, min_samples: usize) -> Self {
        Self { lag, min_samples }
    }
}

impl Default for GaussianEstimator {
    fn default() -> Self {
        Self::new(1, 10)
    }
}

impl GaussianEstimator {
    /// Compute variance
    pub(crate) fn variance(data: &[f64]) -> f64 {
        if data.is_empty() {
            return 0.0;
        }
        let n = data.len() as f64;
        let mean: f64 = data.iter().sum::<f64>() / n;
        data.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / n
    }

    /// Compute covariance
    pub(crate) fn covariance(x: &[f64], y: &[f64]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }
        let n = x.len() as f64;
        let mean_x: f64 = x.iter().sum::<f64>() / n;
        let mean_y: f64 = y.iter().sum::<f64>() / n;
        x.iter()
            .zip(y.iter())
            .map(|(&xi, &yi)| (xi - mean_x) * (yi - mean_y))
            .sum::<f64>() / n
    }

    /// Compute residual variance Var(target | predictor)
    fn residual_variance(&self, target: &[f64], predictor: &[f64]) -> f64 {
        if target.len() != predictor.len() || target.is_empty() {
            return 0.0;
        }

        let cov = Self::covariance(target, predictor);
        let var_predictor = Self::variance(predictor);

        if var_predictor < 1e-10 {
            return Self::variance(target);
        }

        let beta = cov / var_predictor;
        let var_target = Self::variance(target);
        let residual = var_target - beta.powi(2) * var_predictor;

        residual.max(1e-10)
    }

    /// Compute joint residual variance Var(target | predictor1, predictor2)
    fn joint_residual_variance(
        &self,
        target: &[f64],
        predictor1: &[f64],
        predictor2: &[f64],
    ) -> f64 {
        if target.len() != predictor1.len() || target.len() != predictor2.len() || target.is_empty() {
            return 0.0;
        }

        let n = target.len() as f64;

        // Compute statistics
        let sum_t: f64 = target.iter().sum();
        let sum_p1: f64 = predictor1.iter().sum();
        let sum_p2: f64 = predictor2.iter().sum();
        let sum_p1_t: f64 = predictor1.iter().zip(target.iter()).map(|(&p1, &t)| p1 * t).sum();
        let sum_p2_t: f64 = predictor2.iter().zip(target.iter()).map(|(&p2, &t)| p2 * t).sum();
        let sum_p1_p1: f64 = predictor1.iter().map(|&p1| p1 * p1).sum();
        let sum_p2_p2: f64 = predictor2.iter().map(|&p2| p2 * p2).sum();
        let sum_p1_p2: f64 = predictor1.iter().zip(predictor2.iter()).map(|(&p1, &p2)| p1 * p2).sum();

        let mean_t = sum_t / n;
        let mean_p1 = sum_p1 / n;
        let mean_p2 = sum_p2 / n;

        // Centered covariances
        let cov_p1_t = sum_p1_t / n - mean_p1 * mean_t;
        let cov_p2_t = sum_p2_t / n - mean_p2 * mean_t;
        let var_p1 = sum_p1_p1 / n - mean_p1 * mean_p1;
        let var_p2 = sum_p2_p2 / n - mean_p2 * mean_p2;
        let cov_p1_p2 = sum_p1_p2 / n - mean_p1 * mean_p2;
        let var_t = Self::variance(target);

        // Solve 2x2 system
        let det = var_p1 * var_p2 - cov_p1_p2 * cov_p1_p2;
        if det.abs() < 1e-10 {
            return var_t;
        }

        let beta1 = (var_p2 * cov_p1_t - cov_p1_p2 * cov_p2_t) / det;
        let beta2 = (var_p1 * cov_p2_t - cov_p1_p2 * cov_p1_t) / det;

        let residual = var_t - beta1 * cov_p1_t - beta2 * cov_p2_t - beta1 * beta2 * cov_p1_p2;

        residual.max(1e-10)
    }

    /// Compute transfer entropy from source to target
    pub fn compute(&self, source: &[f64], target: &[f64]) -> Option<TeResult> {
        let n = source.len().min(target.len());
        if n < self.min_samples + self.lag {
            return None;
        }

        let effective_len = n - self.lag;

        // Build lagged vectors
        let target_current: Vec<f64> = target[self.lag..n].to_vec();
        let target_past: Vec<f64> = target[0..effective_len].to_vec();
        let source_past: Vec<f64> = source[0..effective_len].to_vec();

        // Var(Y_t | Y_{t-k})
        let var_reduced = self.residual_variance(&target_current, &target_past);

        // Var(Y_t | Y_{t-k}, X_{t-k})
        let var_full = self.joint_residual_variance(&target_current, &target_past, &source_past);

        if var_full < 1e-10 {
            return Some(TeResult::new(0.0, true));
        }

        // TE = 0.5 * ln(var_reduced / var_full) / ln(2)
        let te = 0.5 * (var_reduced / var_full).ln() / 2.0_f64.ln();

        Some(TeResult::new(te.max(0.0), true))
    }

    /// Get estimator name
    pub fn name(&self) -> &str {
        "Gaussian"
    }
}
