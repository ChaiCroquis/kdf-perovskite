//! KSG Transfer Entropy Estimator

use super::super::types::TeResult;

/// KSG Transfer Entropy Estimator
///
/// High-accuracy k-nearest neighbor estimator with surrogate testing.
/// Complexity: O(S * N²) where S is surrogate count.
pub struct KsgEstimator {
    /// Number of nearest neighbors
    pub k: usize,
    /// Number of surrogates for significance testing
    pub surrogates: usize,
    /// Significance level
    pub alpha: f64,
    /// Time lag
    pub lag: usize,
    /// Minimum samples
    pub min_samples: usize,
    /// RNG state for surrogate generation
    rng_state: u64,
}

impl KsgEstimator {
    /// Create a new KSG estimator
    pub fn new(k: usize, surrogates: usize, alpha: f64, lag: usize, min_samples: usize) -> Self {
        Self {
            k,
            surrogates,
            alpha,
            lag,
            min_samples,
            rng_state: 42,
        }
    }
}

impl Default for KsgEstimator {
    fn default() -> Self {
        Self::new(4, 100, 0.05, 1, 50)
    }
}

impl KsgEstimator {
    /// Simple xorshift RNG
    fn next_rand(&mut self) -> f64 {
        self.rng_state ^= self.rng_state << 13;
        self.rng_state ^= self.rng_state >> 7;
        self.rng_state ^= self.rng_state << 17;
        (self.rng_state as f64) / (u64::MAX as f64)
    }

    /// Chebyshev (L-infinity) distance
    pub(crate) fn chebyshev_distance(p1: &[f64], p2: &[f64]) -> f64 {
        p1.iter()
            .zip(p2.iter())
            .map(|(&a, &b)| (a - b).abs())
            .fold(0.0_f64, |a, b| a.max(b))
    }

    /// Find k-th neighbor distance
    fn kth_neighbor_distance(&self, point: &[f64], all_points: &[Vec<f64>]) -> f64 {
        let mut distances: Vec<f64> = all_points
            .iter()
            .map(|p| Self::chebyshev_distance(point, p))
            .filter(|&d| d > 1e-10)
            .collect();
        distances.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        if self.k <= distances.len() {
            distances[self.k - 1]
        } else if !distances.is_empty() {
            *distances.last().unwrap()
        } else {
            1.0
        }
    }

    /// Count points within epsilon distance
    fn count_within_epsilon(&self, point: &[f64], all_points: &[Vec<f64>], epsilon: f64) -> usize {
        all_points
            .iter()
            .filter(|p| Self::chebyshev_distance(point, p) < epsilon)
            .count()
            .saturating_sub(1) // Exclude self
    }

    /// Digamma function approximation
    pub(crate) fn digamma(x: f64) -> f64 {
        if x < 6.0 {
            Self::digamma(x + 1.0) - 1.0 / x
        } else {
            let x2 = x * x;
            x.ln() - 0.5 / x - 1.0 / (12.0 * x2) + 1.0 / (120.0 * x2 * x2)
        }
    }

    /// Compute KSG-based transfer entropy
    fn compute_ksg_te(&self, source: &[f64], target: &[f64]) -> Option<f64> {
        let n = source.len().min(target.len());
        if n < self.min_samples + self.lag {
            return None;
        }

        let effective_n = n - self.lag;

        // Build joint space points
        let mut joint_points: Vec<Vec<f64>> = Vec::with_capacity(effective_n);
        let mut y_ypast_points: Vec<Vec<f64>> = Vec::with_capacity(effective_n);
        let mut ypast_xpast_points: Vec<Vec<f64>> = Vec::with_capacity(effective_n);
        let mut ypast_points: Vec<Vec<f64>> = Vec::with_capacity(effective_n);

        for i in 0..effective_n {
            let y_t = target[i + self.lag];
            let y_past = target[i];
            let x_past = source[i];

            joint_points.push(vec![y_t, y_past, x_past]);
            y_ypast_points.push(vec![y_t, y_past]);
            ypast_xpast_points.push(vec![y_past, x_past]);
            ypast_points.push(vec![y_past]);
        }

        // KSG estimator
        let psi_k = Self::digamma(self.k as f64);

        let mut sum_psi = 0.0;
        for i in 0..effective_n {
            let epsilon = self.kth_neighbor_distance(&joint_points[i], &joint_points);

            let n_y_ypast = self.count_within_epsilon(&y_ypast_points[i], &y_ypast_points, epsilon);
            let n_ypast_xpast = self.count_within_epsilon(&ypast_xpast_points[i], &ypast_xpast_points, epsilon);
            let n_ypast = self.count_within_epsilon(&ypast_points[i], &ypast_points, epsilon);

            sum_psi += Self::digamma((n_y_ypast + 1) as f64)
                + Self::digamma((n_ypast_xpast + 1) as f64)
                - Self::digamma((n_ypast + 1) as f64);
        }

        let te = psi_k - sum_psi / effective_n as f64;
        Some(te.max(0.0))
    }

    /// Generate surrogate time series (Fisher-Yates shuffle)
    fn generate_surrogate(&mut self, data: &[f64]) -> Vec<f64> {
        let n = data.len();
        if n < 4 {
            return data.to_vec();
        }

        let mut surrogate = data.to_vec();

        for i in (1..n).rev() {
            let j = (self.next_rand() * (i + 1) as f64) as usize;
            surrogate.swap(i, j);
        }

        surrogate
    }

    /// Surrogate test for significance
    fn surrogate_test(&mut self, source: &[f64], target: &[f64], observed_te: f64) -> (f64, bool) {
        let mut greater_count = 0;

        for _ in 0..self.surrogates {
            let surrogate = self.generate_surrogate(source);
            if let Some(surrogate_te) = self.compute_ksg_te(&surrogate, target) {
                if surrogate_te >= observed_te {
                    greater_count += 1;
                }
            }
        }

        let p_value = (greater_count + 1) as f64 / (self.surrogates + 1) as f64;
        let is_significant = p_value < self.alpha;

        (p_value, is_significant)
    }

    /// Compute transfer entropy with optional significance testing
    pub fn compute(&mut self, source: &[f64], target: &[f64]) -> Option<TeResult> {
        let te = self.compute_ksg_te(source, target)?;

        if self.surrogates > 0 {
            let (p_value, _) = self.surrogate_test(source, target, te);
            Some(TeResult::with_significance(te, true, p_value, self.alpha))
        } else {
            Some(TeResult::new(te, true))
        }
    }

    /// Get estimator name
    pub fn name(&self) -> &str {
        "KSG"
    }
}
