//! Causal Discovery Engine

use super::estimators::{GaussianEstimator, KsgEstimator, SymbolicEstimator};
use super::types::{CausalLink, TeStrategy};
use std::collections::HashMap;

/// Statistics for batch computation
#[derive(Clone, Debug, Default)]
pub struct BatchStats {
    /// Number of pairs computed
    pub pairs_computed: usize,
    /// Number of significant links found
    pub significant_links: usize,
    /// Mean TE value
    pub mean_te: f64,
    /// Maximum TE value
    pub max_te: f64,
    /// Elapsed time in milliseconds
    pub elapsed_ms: f64,
}

/// Causal Discovery Engine
///
/// Computes transfer entropy between time series pairs using
/// strategy-appropriate estimators integrated with KDF layers.
pub struct CausalEngine {
    /// Gaussian estimator for screening
    gaussian: GaussianEstimator,
    /// Symbolic estimator for deep probe
    symbolic: SymbolicEstimator,
    /// KSG estimator for validation
    ksg: KsgEstimator,
    /// TE threshold for significance
    pub te_threshold: f64,
    /// Cache for computed results
    cache: HashMap<(String, String), CausalLink>,
    /// Cache enabled flag
    pub cache_enabled: bool,
}

impl CausalEngine {
    /// Create a new causal engine
    pub fn new(
        gaussian_lag: usize,
        symbolic_dim: usize,
        symbolic_delay: usize,
        ksg_k: usize,
        ksg_surrogates: usize,
        te_threshold: f64,
    ) -> Self {
        Self {
            gaussian: GaussianEstimator::new(gaussian_lag, 10),
            symbolic: SymbolicEstimator::new(symbolic_dim, symbolic_delay, 50),
            ksg: KsgEstimator::new(ksg_k, ksg_surrogates, 0.05, 1, 50),
            te_threshold,
            cache: HashMap::new(),
            cache_enabled: true,
        }
    }
}

impl Default for CausalEngine {
    fn default() -> Self {
        Self::new(1, 3, 1, 4, 100, 0.01)
    }
}

impl CausalEngine {
    /// Compute transfer entropy for a single pair
    pub fn compute_pair(
        &mut self,
        source: &[f64],
        target: &[f64],
        strategy: TeStrategy,
        source_id: &str,
        target_id: &str,
    ) -> Option<CausalLink> {
        // Check cache
        let cache_key = (source_id.to_string(), target_id.to_string());
        if self.cache_enabled
            && let Some(cached) = self.cache.get(&cache_key)
        {
            return Some(cached.clone());
        }

        // Compute based on strategy
        let result = match strategy {
            TeStrategy::Screening => self.gaussian.compute(source, target),
            TeStrategy::DeepProbe => self.symbolic.compute(source, target),
            TeStrategy::Validation => self.ksg.compute(source, target),
        }?;

        let link = CausalLink {
            source: source_id.to_string(),
            target: target_id.to_string(),
            te: result.te,
            strategy,
            p_value: result.p_value,
            is_significant: result.is_significant,
            confidence: result.confidence,
        };

        // Cache result
        if self.cache_enabled {
            self.cache.insert(cache_key, link.clone());
        }

        Some(link)
    }

    /// Compute bidirectional TE for a pair
    pub fn compute_bidirectional(
        &mut self,
        id_a: &str,
        id_b: &str,
        series_a: &[f64],
        series_b: &[f64],
        strategy: TeStrategy,
    ) -> Option<(CausalLink, CausalLink)> {
        let a_to_b = self.compute_pair(series_a, series_b, strategy, id_a, id_b)?;
        let b_to_a = self.compute_pair(series_b, series_a, strategy, id_b, id_a)?;
        Some((a_to_b, b_to_a))
    }

    /// Batch compute TE for multiple pairs
    pub fn batch_compute(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        candidates: &[(String, String)],
        strategy: TeStrategy,
    ) -> (Vec<CausalLink>, BatchStats) {
        use std::time::Instant;
        let start = Instant::now();

        let mut links = Vec::new();
        let mut stats = BatchStats::default();

        for (source_id, target_id) in candidates {
            let source = match data.get(source_id) {
                Some(s) => s,
                None => continue,
            };
            let target = match data.get(target_id) {
                Some(t) => t,
                None => continue,
            };

            if let Some(link) = self.compute_pair(source, target, strategy, source_id, target_id) {
                stats.pairs_computed += 1;
                stats.mean_te += link.te;
                if link.te > stats.max_te {
                    stats.max_te = link.te;
                }
                if link.is_significant {
                    stats.significant_links += 1;
                }
                links.push(link);
            }
        }

        if stats.pairs_computed > 0 {
            stats.mean_te /= stats.pairs_computed as f64;
        }

        stats.elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        (links, stats)
    }

    /// Compute net causality (difference in bidirectional TE)
    pub fn net_causality(
        &mut self,
        id_a: &str,
        id_b: &str,
        series_a: &[f64],
        series_b: &[f64],
        strategy: TeStrategy,
    ) -> Option<f64> {
        let (a_to_b, b_to_a) =
            self.compute_bidirectional(id_a, id_b, series_a, series_b, strategy)?;
        Some(a_to_b.te - b_to_a.te)
    }

    /// Filter links by threshold
    pub fn filter_significant(&self, links: &[CausalLink]) -> Vec<CausalLink> {
        links
            .iter()
            .filter(|link| link.is_strong(self.te_threshold))
            .cloned()
            .collect()
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}
