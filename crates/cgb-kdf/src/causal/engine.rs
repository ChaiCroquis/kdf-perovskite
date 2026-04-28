//! Causal Discovery Engine

use super::estimators::{GaussianEstimator, KsgEstimator, SymbolicEstimator};
use super::types::{CausalLink, TeStrategy};
use rayon::prelude::*;
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

    /// Batch compute TE for multiple pairs.
    ///
    /// # Parallelism (rayon)
    ///
    /// For `Screening` (Gaussian) and `DeepProbe` (Symbolic) strategies, the
    /// per-pair TE computations are pure functions of `&self.gaussian` /
    /// `&self.symbolic` and are dispatched via `par_iter`. Cache hits are
    /// resolved sequentially before parallel dispatch, and cache writes happen
    /// sequentially after the parallel section, so the cache is always
    /// consistent.
    ///
    /// For `Validation` (KSG), the per-pair compute is `&mut self.ksg` because
    /// of the estimator's internal surrogate-test RNG state. Parallelizing
    /// would either change the RNG sequence (breaking determinism across
    /// thread counts) or require per-thread RNG forks (changing observable
    /// behavior). Validation therefore stays sequential — its workloads are
    /// also dominated by KSG-internal nested loops, not pair-level dispatch.
    ///
    /// Output `links` order matches the order pairs appear in `candidates`
    /// (cache hits and parallel-computed misses are reassembled by index), so
    /// downstream `mean_te += link.te` accumulation is bit-exact reproducible.
    pub fn batch_compute(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        candidates: &[(String, String)],
        strategy: TeStrategy,
    ) -> (Vec<CausalLink>, BatchStats) {
        use std::time::Instant;
        let start = Instant::now();

        // Slot per candidate; order-preserving reassembly.
        let mut slots: Vec<Option<CausalLink>> = vec![None; candidates.len()];
        // Indices that need fresh computation (cache miss + data present).
        let mut to_compute: Vec<usize> = Vec::new();

        for (i, (source_id, target_id)) in candidates.iter().enumerate() {
            // Skip pairs missing time-series data — same semantics as the
            // sequential version (silent drop, no stats counted).
            if !data.contains_key(source_id) || !data.contains_key(target_id) {
                continue;
            }
            if self.cache_enabled
                && let Some(cached) = self.cache.get(&(source_id.clone(), target_id.clone()))
            {
                slots[i] = Some(cached.clone());
            } else {
                to_compute.push(i);
            }
        }

        // Compute misses. Screening / DeepProbe go through rayon; Validation
        // stays sequential (see doc comment for why).
        let computed: Vec<(usize, CausalLink)> = match strategy {
            TeStrategy::Screening => {
                let estimator = &self.gaussian;
                to_compute
                    .par_iter()
                    .filter_map(|&i| {
                        let (source_id, target_id) = &candidates[i];
                        let source = data.get(source_id)?;
                        let target = data.get(target_id)?;
                        let result = estimator.compute(source, target)?;
                        Some((
                            i,
                            CausalLink {
                                source: source_id.clone(),
                                target: target_id.clone(),
                                te: result.te,
                                strategy,
                                p_value: result.p_value,
                                is_significant: result.is_significant,
                                confidence: result.confidence,
                            },
                        ))
                    })
                    .collect()
            }
            TeStrategy::DeepProbe => {
                let estimator = &self.symbolic;
                to_compute
                    .par_iter()
                    .filter_map(|&i| {
                        let (source_id, target_id) = &candidates[i];
                        let source = data.get(source_id)?;
                        let target = data.get(target_id)?;
                        let result = estimator.compute(source, target)?;
                        Some((
                            i,
                            CausalLink {
                                source: source_id.clone(),
                                target: target_id.clone(),
                                te: result.te,
                                strategy,
                                p_value: result.p_value,
                                is_significant: result.is_significant,
                                confidence: result.confidence,
                            },
                        ))
                    })
                    .collect()
            }
            TeStrategy::Validation => {
                // KsgEstimator::compute is &mut self due to internal RNG used
                // by surrogate_test; keep sequential to preserve determinism.
                let mut out = Vec::with_capacity(to_compute.len());
                for &i in &to_compute {
                    let (source_id, target_id) = &candidates[i];
                    let (source, target) = match (data.get(source_id), data.get(target_id)) {
                        (Some(s), Some(t)) => (s, t),
                        _ => continue,
                    };
                    if let Some(result) = self.ksg.compute(source, target) {
                        out.push((
                            i,
                            CausalLink {
                                source: source_id.clone(),
                                target: target_id.clone(),
                                te: result.te,
                                strategy,
                                p_value: result.p_value,
                                is_significant: result.is_significant,
                                confidence: result.confidence,
                            },
                        ));
                    }
                }
                out
            }
        };

        // Cache update + slot fill (sequential, deterministic order).
        for (i, link) in &computed {
            if self.cache_enabled {
                self.cache
                    .insert((link.source.clone(), link.target.clone()), link.clone());
            }
            slots[*i] = Some(link.clone());
        }

        // Reassemble in candidates order so the f64 accumulator below is bit-exact
        // reproducible (sequential version's invariant).
        let mut links = Vec::with_capacity(slots.len());
        let mut stats = BatchStats::default();
        for slot in slots.into_iter().flatten() {
            stats.pairs_computed += 1;
            stats.mean_te += slot.te;
            if slot.te > stats.max_te {
                stats.max_te = slot.te;
            }
            if slot.is_significant {
                stats.significant_links += 1;
            }
            links.push(slot);
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
