//! Symbolic Transfer Entropy Estimator

use super::super::types::TeResult;
use std::collections::HashMap;

/// Symbolic Transfer Entropy Estimator
///
/// Non-linear estimator using permutation patterns.
/// Complexity: O(N log N)
pub struct SymbolicEstimator {
    /// Embedding dimension
    pub dim: usize,
    /// Time delay
    pub delay: usize,
    /// Minimum samples
    pub min_samples: usize,
}

impl SymbolicEstimator {
    /// Create a new Symbolic estimator
    pub fn new(dim: usize, delay: usize, min_samples: usize) -> Self {
        Self {
            dim,
            delay,
            min_samples,
        }
    }
}

impl Default for SymbolicEstimator {
    fn default() -> Self {
        Self::new(3, 1, 50)
    }
}

impl SymbolicEstimator {
    /// Convert a window to permutation pattern rank
    pub(crate) fn window_to_pattern(&self, window: &[f64]) -> usize {
        let n = window.len();
        if n == 0 {
            return 0;
        }

        // Create indices sorted by values
        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|&a, &b| {
            window[a]
                .partial_cmp(&window[b])
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Compute pattern rank
        let mut rank = 0;
        let mut factorial = 1;
        for i in 0..n {
            let count = (i + 1..n).filter(|&j| indices[j] < indices[i]).count();
            rank += count * factorial;
            factorial *= n - i;
        }

        rank
    }

    /// Symbolize a time series
    fn symbolize(&self, series: &[f64]) -> Vec<usize> {
        let n = series.len();
        let window_len = (self.dim - 1) * self.delay + 1;

        if n < window_len {
            return Vec::new();
        }

        let num_patterns = n - window_len + 1;
        let mut patterns = Vec::with_capacity(num_patterns);

        for i in 0..num_patterns {
            let window: Vec<f64> = (0..self.dim).map(|j| series[i + j * self.delay]).collect();
            patterns.push(self.window_to_pattern(&window));
        }

        patterns
    }

    /// Compute Shannon entropy
    pub(crate) fn shannon_entropy(counts: &HashMap<usize, usize>, total: usize) -> f64 {
        if total == 0 {
            return 0.0;
        }

        let total_f = total as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total_f;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Compute joint entropy of two symbol sequences
    pub(crate) fn joint_entropy(x: &[usize], y: &[usize]) -> f64 {
        if x.len() != y.len() || x.is_empty() {
            return 0.0;
        }

        let mut counts: HashMap<(usize, usize), usize> = HashMap::new();
        for (&xi, &yi) in x.iter().zip(y.iter()) {
            *counts.entry((xi, yi)).or_insert(0) += 1;
        }

        let total = x.len() as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Compute triple joint entropy
    pub(crate) fn triple_joint_entropy(x: &[usize], y: &[usize], z: &[usize]) -> f64 {
        if x.len() != y.len() || x.len() != z.len() || x.is_empty() {
            return 0.0;
        }

        let mut counts: HashMap<(usize, usize, usize), usize> = HashMap::new();
        for i in 0..x.len() {
            *counts.entry((x[i], y[i], z[i])).or_insert(0) += 1;
        }

        let total = x.len() as f64;
        let mut entropy = 0.0;
        for &count in counts.values() {
            let p = count as f64 / total;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Compute transfer entropy
    pub fn compute(&self, source: &[f64], target: &[f64]) -> Option<TeResult> {
        let n = source.len().min(target.len());
        if n < self.min_samples {
            return None;
        }

        // Symbolize
        let source_symbols = self.symbolize(source);
        let target_symbols = self.symbolize(target);

        let sym_len = source_symbols.len().min(target_symbols.len());
        if sym_len < 3 {
            return None;
        }

        // Create lagged versions
        let y_t: Vec<usize> = target_symbols[1..sym_len].to_vec();
        let y_past: Vec<usize> = target_symbols[0..sym_len - 1].to_vec();
        let x_past: Vec<usize> = source_symbols[0..sym_len - 1].to_vec();

        // Compute entropies
        let mut y_past_counts: HashMap<usize, usize> = HashMap::new();
        for &y in &y_past {
            *y_past_counts.entry(y).or_insert(0) += 1;
        }

        let h_y_past = Self::shannon_entropy(&y_past_counts, y_past.len());
        let h_y_t_y_past = Self::joint_entropy(&y_t, &y_past);
        let h_y_past_x_past = Self::joint_entropy(&y_past, &x_past);
        let h_y_t_y_past_x_past = Self::triple_joint_entropy(&y_t, &y_past, &x_past);

        // TE = H(Y_t, Y_{t-1}) - H(Y_{t-1}) - H(Y_t, Y_{t-1}, X_{t-1}) + H(Y_{t-1}, X_{t-1})
        let te = h_y_t_y_past - h_y_past - h_y_t_y_past_x_past + h_y_past_x_past;

        Some(TeResult::new(te.max(0.0), true))
    }

    /// Get estimator name
    pub fn name(&self) -> &str {
        "Symbolic"
    }
}
