//! Transition control between management regions (Claim 23-26)
//!
//! # Claim mapping
//!
//! | Claim | Requirement | Implementation |
//! |-------|-------------|----------------|
//! | 23 | Transition control means between Region 1 and Region 2 | [`TransitionController::step`] |
//! | 24 | Transition score from ≥2 of {connectivity, activation, semantic importance} | [`TransitionScore`] |
//! | 25 | Activation: increases on event, decays in time | [`ActivationScore`] |
//! | 26 | Semantic importance from reference set or external model | [`SemanticImportance`] |

use std::collections::HashMap;
use super::region::RegionKind;

/// Activation score with event-based increment and time decay (Claim 25).
#[derive(Clone, Debug)]
pub struct ActivationScore {
    /// Per-node activation level, A(t)
    pub levels: HashMap<u32, f64>,
    /// Exponential decay rate λ_a per tick (Claim 25: 時間経過に応じて減少)
    pub decay_rate: f64,
    /// Event increment δ_a (Claim 25: 参照/更新/入力イベントで増加)
    pub event_increment: f64,
}

impl Default for ActivationScore {
    fn default() -> Self {
        Self {
            levels: HashMap::new(),
            decay_rate: 0.10,
            event_increment: 1.0,
        }
    }
}

impl ActivationScore {
    /// Claim 25: record a reference/update/input event for the node.
    pub fn record_event(&mut self, node: u32) {
        let entry = self.levels.entry(node).or_insert(0.0);
        *entry += self.event_increment;
    }

    /// Claim 25: advance time by one tick, applying exponential decay.
    pub fn advance_tick(&mut self) {
        let survival = (-self.decay_rate).exp();
        for v in self.levels.values_mut() {
            *v *= survival;
        }
    }

    pub fn get(&self, node: u32) -> f64 {
        self.levels.get(&node).copied().unwrap_or(0.0)
    }
}

/// Semantic importance (Claim 26): derivable either from a reference set of
/// nodes in the data structure or from an external model.
#[derive(Clone, Debug, Default)]
pub struct SemanticImportance {
    /// Reference set (基準集合, Claim 26)
    pub reference_set: std::collections::HashSet<u32>,
    /// Optional external-model scores, keyed by node id.
    pub external_scores: HashMap<u32, f64>,
}

impl SemanticImportance {
    /// Claim 26: score from connection rate to the reference set OR an
    /// external model, whichever is populated. If both populated, they sum.
    pub fn score(&self, node: u32, neighbors: &[u32]) -> f64 {
        let ref_score = if self.reference_set.is_empty() || neighbors.is_empty() {
            0.0
        } else {
            let hits = neighbors.iter().filter(|n| self.reference_set.contains(n)).count();
            hits as f64 / neighbors.len() as f64
        };
        let ext_score = self.external_scores.get(&node).copied().unwrap_or(0.0);
        ref_score + ext_score
    }
}

/// Transition score S_t (Claim 24).
///
/// S_t = w_c · connectivity + w_a · activation + w_s · semantic_importance
/// where at least two of the three weights must be non-zero (Claim 24: 少なくとも二つ).
#[derive(Clone, Debug)]
pub struct TransitionScore {
    pub w_connectivity: f64,
    pub w_activation: f64,
    pub w_semantic: f64,
}

impl Default for TransitionScore {
    fn default() -> Self {
        // Default: all three active (full compliance with Claim 24's "at least two").
        Self { w_connectivity: 0.4, w_activation: 0.3, w_semantic: 0.3 }
    }
}

impl TransitionScore {
    /// Claim 24 precondition: at least two non-zero weights.
    pub fn is_claim24_valid(&self) -> bool {
        let n = [self.w_connectivity, self.w_activation, self.w_semantic]
            .iter()
            .filter(|w| **w > 0.0)
            .count();
        n >= 2
    }

    pub fn compute(&self, connectivity: f64, activation: f64, semantic: f64) -> f64 {
        self.w_connectivity * connectivity + self.w_activation * activation + self.w_semantic * semantic
    }
}

/// Transition controller (Claim 23).
#[derive(Clone, Debug, Default)]
pub struct TransitionController {
    pub score_config: TransitionScore,
    pub activation: ActivationScore,
    pub semantic: SemanticImportance,
    /// Threshold for promotion ShortTerm → LongTerm
    pub promote_threshold: f64,
    /// Threshold for demotion LongTerm → ShortTerm
    pub demote_threshold: f64,
}

impl TransitionController {
    pub fn new() -> Self {
        Self {
            promote_threshold: 0.7,
            demote_threshold: 0.2,
            ..Default::default()
        }
    }

    /// Decide the target region for a node given its metrics (Claim 23).
    pub fn target_region(
        &self,
        current: RegionKind,
        connectivity: f64,
        activation: f64,
        semantic: f64,
    ) -> RegionKind {
        let s = self.score_config.compute(connectivity, activation, semantic);
        match current {
            RegionKind::ShortTerm if s >= self.promote_threshold => RegionKind::LongTerm,
            RegionKind::LongTerm if s < self.demote_threshold => RegionKind::ShortTerm,
            other => other,
        }
    }

    /// Claim 23 entry point: evaluate `node` and return `Some(new_region)` on transition.
    pub fn step(
        &self,
        node: u32,
        current: RegionKind,
        connectivity: f64,
        neighbors: &[u32],
    ) -> Option<RegionKind> {
        let a = self.activation.get(node);
        let s = self.semantic.score(node, neighbors);
        let new_region = self.target_region(current, connectivity, a, s);
        if new_region != current { Some(new_region) } else { None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claim25_activation_event_increment() {
        let mut a = ActivationScore::default();
        a.record_event(1);
        assert_eq!(a.get(1), 1.0);
        a.record_event(1);
        assert_eq!(a.get(1), 2.0);
    }

    #[test]
    fn test_claim25_activation_time_decay() {
        let mut a = ActivationScore::default();
        a.record_event(1); // A=1.0
        let before = a.get(1);
        a.advance_tick();
        let after = a.get(1);
        assert!(after < before, "Claim 25: must decay in time");
        let expected = (-a.decay_rate).exp();
        assert!((after - expected).abs() < 1e-12);
    }

    #[test]
    fn test_claim26_semantic_importance_reference_set() {
        let mut s = SemanticImportance::default();
        s.reference_set.insert(10);
        s.reference_set.insert(20);
        // Node 1 connected to [10, 20, 99] — 2 of 3 in reference set
        let score = s.score(1, &[10, 20, 99]);
        assert!((score - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn test_claim26_semantic_importance_external_model() {
        let mut s = SemanticImportance::default();
        s.external_scores.insert(1, 0.8);
        assert!((s.score(1, &[]) - 0.8).abs() < 1e-12);
    }

    #[test]
    fn test_claim24_weight_count_validation() {
        let mut s = TransitionScore::default();
        assert!(s.is_claim24_valid());
        s.w_semantic = 0.0;
        assert!(s.is_claim24_valid(), "2 of 3 weights is still compliant");
        s.w_activation = 0.0;
        assert!(!s.is_claim24_valid(), "only 1 weight non-zero violates Claim 24");
    }

    #[test]
    fn test_claim23_promotion_on_high_score() {
        let mut tc = TransitionController::new();
        tc.activation.record_event(1);
        tc.activation.record_event(1);
        tc.activation.record_event(1); // A=3.0
        let new_region = tc.step(1, RegionKind::ShortTerm, /*connectivity*/ 2.0, &[]);
        assert_eq!(new_region, Some(RegionKind::LongTerm));
    }

    #[test]
    fn test_claim23_no_transition_when_stable() {
        let tc = TransitionController::new();
        let new_region = tc.step(1, RegionKind::LongTerm, 1.0, &[]);
        assert_eq!(new_region, None, "stable node stays in region");
    }
}
