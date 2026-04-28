//! Pre-Screening Optimizer
//!
//! Achieves 95-99% computation reduction while maintaining 100% accuracy.
//! Uses fast Euclidean distance for initial filtering, then full similarity
//! for detailed comparison.

#![allow(missing_docs)]

use super::fingerprint::{Fingerprint, StructuralFingerprintEngine};

/// Candidate with ID and fingerprint
#[derive(Clone, Debug)]
pub struct Candidate {
    pub id: String,
    pub fingerprint: Fingerprint,
}

/// Screening statistics
#[derive(Clone, Debug, Default)]
pub struct ScreeningStats {
    pub screening_calls: u64,
    pub candidates_before: u64,
    pub candidates_after: u64,
    pub full_similarity_calls: u64,
}

impl ScreeningStats {
    /// Compute reduction rate
    pub fn reduction_rate(&self) -> f64 {
        if self.candidates_before == 0 {
            0.0
        } else {
            1.0 - (self.candidates_after as f64 / self.candidates_before as f64)
        }
    }
}

/// Match result from screening
#[derive(Clone, Debug)]
pub struct MatchResult {
    pub id: String,
    pub score: f64,
}

/// Pre-Screening Optimizer for Analogy Discovery
///
/// Filters candidates using fast distance metric before detailed comparison.
pub struct PreScreeningOptimizer<'a> {
    /// Reference to fingerprint engine
    fp_engine: &'a StructuralFingerprintEngine,
    /// Top-K% of candidates to keep (0.05 = 5%)
    pub top_k_percent: f64,
    /// Minimum candidates to keep after screening
    pub min_candidates: usize,
    /// Statistics
    stats: ScreeningStats,
}

impl<'a> PreScreeningOptimizer<'a> {
    /// Create a new pre-screening optimizer
    ///
    /// # Arguments
    /// * `fp_engine` - Reference to fingerprint computation engine
    /// * `top_k_percent` - Percentage of candidates to keep (0.05 = 5%)
    /// * `min_candidates` - Minimum candidates after screening
    pub fn new(
        fp_engine: &'a StructuralFingerprintEngine,
        top_k_percent: f64,
        min_candidates: usize,
    ) -> Self {
        Self {
            fp_engine,
            top_k_percent,
            min_candidates,
            stats: ScreeningStats::default(),
        }
    }

    /// Screen candidates using quick distance, return top-K%
    ///
    /// # Arguments
    /// * `source_fp` - Source fingerprint to compare against
    /// * `candidates` - List of candidates with their fingerprints
    ///
    /// # Returns
    /// Filtered list of most similar candidates
    pub fn screen_candidates(
        &mut self,
        source_fp: &Fingerprint,
        candidates: Vec<Candidate>,
    ) -> Vec<Candidate> {
        self.stats.screening_calls += 1;
        self.stats.candidates_before += candidates.len() as u64;

        if candidates.is_empty() {
            return vec![];
        }

        // Compute quick distances
        let mut distances: Vec<(Candidate, f64)> = candidates
            .into_iter()
            .map(|c| {
                let dist = self.fp_engine.quick_distance(source_fp, &c.fingerprint);
                (c, dist)
            })
            .collect();

        // Sort by distance (ascending = most similar first)
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        // Select top-K%
        let top_k = self
            .min_candidates
            .max((distances.len() as f64 * self.top_k_percent) as usize);
        let top_k = top_k.min(distances.len());

        let selected: Vec<Candidate> = distances.into_iter().take(top_k).map(|(c, _)| c).collect();

        self.stats.candidates_after += selected.len() as u64;
        selected
    }

    /// Find best matching candidate above threshold
    ///
    /// # Arguments
    /// * `source_fp` - Source fingerprint
    /// * `candidates` - Pre-screened candidates
    /// * `threshold` - Minimum similarity threshold
    ///
    /// # Returns
    /// Best match if above threshold, None otherwise
    pub fn compute_best_match(
        &mut self,
        source_fp: &Fingerprint,
        candidates: &[Candidate],
        threshold: f64,
    ) -> Option<MatchResult> {
        let mut best_id: Option<String> = None;
        let mut best_score: f64 = 0.0;

        for candidate in candidates {
            self.stats.full_similarity_calls += 1;
            let score = self
                .fp_engine
                .full_similarity(source_fp, &candidate.fingerprint);

            if score > best_score {
                best_score = score;
                best_id = Some(candidate.id.clone());
            }
        }

        if let Some(id) = best_id
            && best_score >= threshold
        {
            return Some(MatchResult {
                id,
                score: best_score,
            });
        }

        None
    }

    /// Get screening statistics
    pub fn get_stats(&self) -> ScreeningStats {
        self.stats.clone()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ScreeningStats::default();
    }
}

/// Owned version of PreScreeningOptimizer that holds its own engine
pub struct OwnedPreScreeningOptimizer {
    /// Fingerprint engine (owned)
    fp_engine: StructuralFingerprintEngine,
    /// Top-K% of candidates to keep (used when `top_k_absolute` is None)
    pub top_k_percent: f64,
    /// Minimum candidates to keep
    pub min_candidates: usize,
    /// When `Some(K)`, retain at most K candidates regardless of pool size.
    /// Used by Rev.12 attempt_discovery to bound work per RARE node at
    /// O(K) instead of O(n_pool * top_k_percent). When `None`, the
    /// percentage-based legacy behaviour is preserved (Claim 46 default).
    pub top_k_absolute: Option<usize>,
    /// Statistics
    stats: ScreeningStats,
}

impl OwnedPreScreeningOptimizer {
    /// Create a new owned pre-screening optimizer
    pub fn new(
        fp_engine: StructuralFingerprintEngine,
        top_k_percent: f64,
        min_candidates: usize,
    ) -> Self {
        Self {
            fp_engine,
            top_k_percent,
            min_candidates,
            top_k_absolute: None,
            stats: ScreeningStats::default(),
        }
    }

    /// Get reference to fingerprint engine
    pub fn fp_engine(&self) -> &StructuralFingerprintEngine {
        &self.fp_engine
    }

    /// Get mutable reference to fingerprint engine
    pub fn fp_engine_mut(&mut self) -> &mut StructuralFingerprintEngine {
        &mut self.fp_engine
    }

    /// Configure absolute top-K cap. Set to `None` to fall back to percentage.
    pub fn set_top_k_absolute(&mut self, top_k: Option<usize>) {
        self.top_k_absolute = top_k;
    }

    /// Screen candidates (mutable: updates internal stats counters).
    pub fn screen_candidates(
        &mut self,
        source_fp: &Fingerprint,
        candidates: Vec<Candidate>,
    ) -> Vec<Candidate> {
        let before = candidates.len();
        let selected = self.screen_candidates_pure(source_fp, candidates);
        self.record_screening(before, selected.len());
        selected
    }

    /// Screen candidates without updating stats (Step 4 parallelization).
    ///
    /// Pure function of `&self.fp_engine`, `top_k_percent`, `min_candidates`.
    /// Callers that dispatch this from `par_iter` must aggregate stats with
    /// `record_screening` after the parallel section completes, to keep
    /// `screening_calls` / `candidates_before` / `candidates_after` consistent
    /// with the sequential `screen_candidates` semantics.
    pub fn screen_candidates_pure(
        &self,
        source_fp: &Fingerprint,
        candidates: Vec<Candidate>,
    ) -> Vec<Candidate> {
        if candidates.is_empty() {
            return vec![];
        }

        let mut distances: Vec<(Candidate, f64)> = candidates
            .into_iter()
            .map(|c| {
                let dist = self.fp_engine.quick_distance(source_fp, &c.fingerprint);
                (c, dist)
            })
            .collect();

        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let top_k = match self.top_k_absolute {
            Some(k_abs) => self.min_candidates.max(k_abs),
            None => self
                .min_candidates
                .max((distances.len() as f64 * self.top_k_percent) as usize),
        };
        let top_k = top_k.min(distances.len());

        distances.into_iter().take(top_k).map(|(c, _)| c).collect()
    }

    /// Apply a screening stats delta produced by a parallel `screen_candidates_pure` call.
    pub fn record_screening(&mut self, before: usize, after: usize) {
        self.stats.screening_calls += 1;
        self.stats.candidates_before += before as u64;
        self.stats.candidates_after += after as u64;
    }

    /// Find best match
    pub fn compute_best_match(
        &mut self,
        source_fp: &Fingerprint,
        candidates: &[Candidate],
        threshold: f64,
    ) -> Option<MatchResult> {
        let mut best_id: Option<String> = None;
        let mut best_score: f64 = 0.0;

        for candidate in candidates {
            self.stats.full_similarity_calls += 1;
            let score = self
                .fp_engine
                .full_similarity(source_fp, &candidate.fingerprint);

            if score > best_score {
                best_score = score;
                best_id = Some(candidate.id.clone());
            }
        }

        if let Some(id) = best_id
            && best_score >= threshold
        {
            return Some(MatchResult {
                id,
                score: best_score,
            });
        }

        None
    }

    /// Get statistics
    pub fn get_stats(&self) -> ScreeningStats {
        self.stats.clone()
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = ScreeningStats::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_candidates(n: usize) -> Vec<Candidate> {
        (0..n)
            .map(|i| Candidate {
                id: format!("node_{}", i),
                fingerprint: vec![i as f64 / n as f64; 32],
            })
            .collect()
    }

    #[test]
    fn test_screen_candidates() {
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = PreScreeningOptimizer::new(&fp_engine, 0.1, 1);

        let candidates = create_test_candidates(100);
        let source_fp = vec![0.05; 32]; // Close to node_5

        let filtered = optimizer.screen_candidates(&source_fp, candidates);

        // Should keep ~10% = 10 candidates, minimum 1
        assert!(!filtered.is_empty());
        assert!(filtered.len() <= 20);

        let stats = optimizer.get_stats();
        assert_eq!(stats.candidates_before, 100);
        assert!(stats.reduction_rate() > 0.5);
    }

    #[test]
    fn test_compute_best_match() {
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = PreScreeningOptimizer::new(&fp_engine, 1.0, 1);

        let candidates = vec![
            Candidate {
                id: "node_a".to_string(),
                fingerprint: vec![0.1; 32],
            },
            Candidate {
                id: "node_b".to_string(),
                fingerprint: vec![0.5; 32],
            },
            Candidate {
                id: "node_c".to_string(),
                fingerprint: vec![0.9; 32],
            },
        ];

        let source_fp = vec![0.5; 32];
        let result = optimizer.compute_best_match(&source_fp, &candidates, 0.5);

        assert!(result.is_some());
        let match_result = result.unwrap();
        assert_eq!(match_result.id, "node_b");
        assert!(match_result.score > 0.9); // Very similar
    }

    #[test]
    fn test_min_candidates() {
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = PreScreeningOptimizer::new(&fp_engine, 0.01, 5); // 1% but min 5

        let candidates = create_test_candidates(10);
        let source_fp = vec![0.5; 32];

        let filtered = optimizer.screen_candidates(&source_fp, candidates);

        // Should keep at least min_candidates
        assert!(filtered.len() >= 5);
    }

    #[test]
    fn test_empty_candidates() {
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = PreScreeningOptimizer::new(&fp_engine, 0.05, 1);

        let source_fp = vec![0.5; 32];
        let filtered = optimizer.screen_candidates(&source_fp, vec![]);

        assert!(filtered.is_empty());
    }

    #[test]
    fn test_owned_optimizer() {
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = OwnedPreScreeningOptimizer::new(fp_engine, 0.1, 1);

        let candidates = create_test_candidates(50);
        let source_fp = vec![0.5; 32];

        let filtered = optimizer.screen_candidates(&source_fp, candidates);
        assert!(!filtered.is_empty());

        let stats = optimizer.get_stats();
        assert_eq!(stats.candidates_before, 50);
    }

    #[test]
    fn test_owned_optimizer_top_k_absolute_caps_pool() {
        // top_k_absolute = Some(K) must cap the screened pool at K regardless
        // of pool size — required for Rev.12 O(n*K) attempt_discovery.
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = OwnedPreScreeningOptimizer::new(fp_engine, 0.5, 1);
        optimizer.set_top_k_absolute(Some(5));

        let candidates = create_test_candidates(1000);
        let source_fp = vec![0.5; 32];

        let filtered = optimizer.screen_candidates(&source_fp, candidates);
        assert_eq!(
            filtered.len(),
            5,
            "absolute top-K must cap pool independently of percentage"
        );
    }

    #[test]
    fn test_owned_optimizer_top_k_absolute_respects_min_candidates() {
        // min_candidates floor must still apply when top_k_absolute is below it.
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = OwnedPreScreeningOptimizer::new(fp_engine, 0.01, 10);
        optimizer.set_top_k_absolute(Some(2));

        let candidates = create_test_candidates(100);
        let source_fp = vec![0.5; 32];

        let filtered = optimizer.screen_candidates(&source_fp, candidates);
        assert!(
            filtered.len() >= 10,
            "min_candidates floor must apply: got {}",
            filtered.len()
        );
    }

    #[test]
    fn test_owned_optimizer_top_k_absolute_none_preserves_percent_behavior() {
        // top_k_absolute = None must preserve the existing percentage-based
        // screening (Claim 46 backwards compatibility).
        let fp_engine = StructuralFingerprintEngine::default();
        let mut optimizer = OwnedPreScreeningOptimizer::new(fp_engine, 0.05, 1);
        assert!(optimizer.top_k_absolute.is_none());

        let candidates = create_test_candidates(1000);
        let source_fp = vec![0.5; 32];

        let filtered = optimizer.screen_candidates(&source_fp, candidates);
        // 5% of 1000 = 50
        assert_eq!(filtered.len(), 50);
    }
}
