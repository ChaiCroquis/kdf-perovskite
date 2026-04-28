//! Causal KDF V3 - Transfer Entropy Integration with KDF

use super::engine::CausalEngine;
use super::types::{CausalLink, TeStrategy};
use std::collections::HashMap;

/// Causal KDF V3 - Transfer Entropy Integration with KDF
///
/// Integrates transfer entropy-based causal discovery with KDF layers
/// for intelligent edge management and knowledge consolidation.
///
/// Layer Integration:
/// - Edge Layer: Use Screening (Gaussian TE) for fast filtering
/// - Rare Layer: Use DeepProbe (Symbolic TE) for isolated truth detection
/// - Sleep Mode: Use Validation (KSG TE) for final verification
pub struct CausalKdfV3 {
    /// Causal engine
    engine: CausalEngine,
    /// Screening threshold
    pub screening_threshold: f64,
    /// Deep probe threshold
    pub deep_probe_threshold: f64,
    /// Validation threshold
    pub validation_threshold: f64,
    /// Statistics
    stats: CausalKdfStats,
}

/// Statistics for CausalKdfV3
#[derive(Clone, Debug, Default)]
pub struct CausalKdfStats {
    /// Number of screening calls
    pub screening_calls: usize,
    /// Number of deep probe calls
    pub deep_probe_calls: usize,
    /// Number of validation calls
    pub validation_calls: usize,
    /// Number of significant links found
    pub significant_links_found: usize,
    /// Number of false positives filtered
    pub false_positives_filtered: usize,
    /// Number of isolated truths detected
    pub isolated_truths_detected: usize,
}

impl CausalKdfV3 {
    /// Create a new CausalKdfV3
    pub fn new(
        screening_threshold: f64,
        deep_probe_threshold: f64,
        validation_threshold: f64,
        ksg_surrogates: usize,
        _alpha: f64,
    ) -> Self {
        // Create engine with appropriate parameters
        let engine = CausalEngine::new(
            1,                   // gaussian_lag
            3,                   // symbolic_dim
            1,                   // symbolic_delay
            4,                   // ksg_k
            ksg_surrogates,      // ksg_surrogates
            screening_threshold, // te_threshold
        );

        Self {
            engine,
            screening_threshold,
            deep_probe_threshold,
            validation_threshold,
            stats: CausalKdfStats::default(),
        }
    }
}

impl Default for CausalKdfV3 {
    fn default() -> Self {
        Self::new(0.01, 0.05, 0.1, 100, 0.05)
    }
}

impl CausalKdfV3 {
    /// Awake Mode: Process incoming data stream with Screening strategy
    ///
    /// Fast O(N) computation for Edge layer filtering.
    pub fn process_stream(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        candidates: &[(String, String)],
    ) -> Vec<CausalLink> {
        let (links, batch_stats) =
            self.engine
                .batch_compute(data, candidates, TeStrategy::Screening);

        self.stats.screening_calls += batch_stats.pairs_computed;

        // Filter by screening threshold
        let significant: Vec<CausalLink> = links
            .into_iter()
            .filter(|link| link.te >= self.screening_threshold)
            .collect();
        self.stats.significant_links_found += significant.len();

        significant
    }

    /// Rare Layer: Deep probe for isolated truth detection
    ///
    /// Uses Symbolic TE (O(N log N)) for noise-resistant estimation.
    pub fn deep_probe(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        candidates: &[(String, String)],
    ) -> Vec<CausalLink> {
        let (links, batch_stats) =
            self.engine
                .batch_compute(data, candidates, TeStrategy::DeepProbe);

        self.stats.deep_probe_calls += batch_stats.pairs_computed;

        // Filter by deep probe threshold
        let significant: Vec<CausalLink> = links
            .into_iter()
            .filter(|link| {
                if link.te >= self.deep_probe_threshold {
                    // Detect potential isolated truths
                    if link.te >= self.deep_probe_threshold * 2.0 {
                        self.stats.isolated_truths_detected += 1;
                    }
                    true
                } else {
                    false
                }
            })
            .collect();

        significant
    }

    /// Sleep Mode: Final validation with statistical significance testing
    ///
    /// Uses KSG TE (O(S*N²)) with surrogate testing.
    pub fn validate(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        candidates: &[(String, String)],
    ) -> Vec<CausalLink> {
        let (links, batch_stats) =
            self.engine
                .batch_compute(data, candidates, TeStrategy::Validation);

        self.stats.validation_calls += batch_stats.pairs_computed;

        // Filter by validation threshold and significance
        let mut validated = Vec::new();
        for link in links {
            if link.te >= self.validation_threshold && link.is_significant {
                validated.push(link);
            } else if link.te >= self.screening_threshold && !link.is_significant {
                self.stats.false_positives_filtered += 1;
            }
        }

        validated
    }

    /// Run full sleep cycle
    ///
    /// 1. Screening (Edge layer)
    /// 2. Deep Probe (Rare layer candidates)
    /// 3. Validation (Sleep mode final check)
    pub fn run_sleep_cycle(
        &mut self,
        data: &HashMap<String, Vec<f64>>,
        screening_candidates: &[(String, String)],
    ) -> SleepCycleResult {
        // Phase 1: Screening
        let screened = self.process_stream(data, screening_candidates);
        let screened_pairs: Vec<(String, String)> = screened
            .iter()
            .map(|link| (link.source.clone(), link.target.clone()))
            .collect();

        // Phase 2: Deep Probe on screened candidates
        let probed = self.deep_probe(data, &screened_pairs);
        let probed_pairs: Vec<(String, String)> = probed
            .iter()
            .map(|link| (link.source.clone(), link.target.clone()))
            .collect();

        // Phase 3: Validation on probed candidates
        let validated = self.validate(data, &probed_pairs);

        SleepCycleResult {
            screened,
            probed,
            validated,
            stats: self.stats.clone(),
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> CausalKdfStats {
        self.stats.clone()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = CausalKdfStats::default();
    }
}

/// Result of a sleep cycle
#[derive(Clone, Debug)]
pub struct SleepCycleResult {
    /// Links from screening phase
    pub screened: Vec<CausalLink>,
    /// Links from deep probe phase
    pub probed: Vec<CausalLink>,
    /// Links from validation phase
    pub validated: Vec<CausalLink>,
    /// Statistics
    pub stats: CausalKdfStats,
}
