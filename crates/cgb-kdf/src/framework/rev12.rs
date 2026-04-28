//! KDF Rev.12 Implementation - Analogy Discovery Mechanism

use super::{ClassificationStats, DecayManager, Layer, NodeClassifier};
use crate::analogy::{AnalogyComputeResult, AnalogyDiscoveryEngine, NodeFeatures};
use crate::fingerprint::{Fingerprint, NodeLabel};
use rayon::prelude::*;
use std::collections::HashMap;

/// Outcome of a parallel-friendly `KdfProcessorRev12::compute_discovery` call.
///
/// Returned from the read-only path that runs inside `process_review_cycle`'s
/// `par_iter`. The caller applies side effects (rev12 stats, analogy_engine
/// stats / history, rare_states mutation, phase transitions) sequentially in
/// `active_nodes` order, mirroring the sequential `attempt_discovery` exactly.
///
/// `compute_discovery` returns `Some(...)` whenever the rare_node is in
/// `rare_states`. Inside that, the two cases are:
/// 1. `invoked_analogy = false` — no candidates available (CORE/EDGE empty),
///    sequential `attempt_discovery` did not call `find_analogy` either.
///    Equivalent to "skip analogy_engine stats bump, just wait_count++".
/// 2. `invoked_analogy = true` — candidates non-empty; `compute_analogy` was
///    invoked and the result (or `None` if source not registered in analogy
///    engine) is in `analogy_result`. Caller bumps `analogy_engine`'s
///    `discovery_attempts` and applies the result if present.
#[derive(Clone, Debug)]
pub struct DiscoveryOutcome {
    pub invoked_analogy: bool,
    pub analogy_result: Option<AnalogyComputeResult>,
}

impl DiscoveryOutcome {
    /// Returns `Some((target, score))` iff the best mapping is within the
    /// Claim 47–48 sandwich band `[θ_L, θ_U]`. Mirrors the spoke_up check in
    /// the sequential `attempt_discovery`.
    pub fn spoke_up(&self, theta_l: f64, theta_u: f64) -> Option<(u32, f64)> {
        let mapping = self.analogy_result.as_ref()?.best_mapping.as_ref()?;
        let s = mapping.overall_score;
        if s >= theta_l && s <= theta_u {
            mapping.target_node.parse().ok().map(|tgt| (tgt, s))
        } else {
            None
        }
    }
}

/// Review phase for RARE nodes in Rev.12
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReviewPhase {
    /// Phase 1: Initial waiting period (T_wait1)
    Phase1,
    /// Phase 2: Extended waiting period (T_wait2)
    Phase2,
    /// Review complete - node has been processed
    Complete,
}

/// State of a RARE node under Rev.12 review
#[derive(Clone, Debug)]
pub struct RareNodeState {
    /// Node ID
    pub node_id: u32,
    /// Whether the node found a structural analogy (spoke_up)
    pub spoke_up: bool,
    /// Current review phase
    pub phase: ReviewPhase,
    /// Number of review cycles elapsed
    pub wait_count: u64,
    /// Analogy target node (if spoke_up)
    pub analogy_target: Option<u32>,
    /// Analogy score (if spoke_up)
    pub analogy_score: f64,
    /// Fingerprint for structure preservation
    pub fingerprint: Fingerprint,
}

impl RareNodeState {
    /// Create a new RARE node state
    pub fn new(node_id: u32, fingerprint: Fingerprint) -> Self {
        Self {
            node_id,
            spoke_up: false,
            phase: ReviewPhase::Phase1,
            wait_count: 0,
            analogy_target: None,
            analogy_score: 0.0,
            fingerprint,
        }
    }

    /// Check if node has exceeded waiting periods
    pub fn is_expired(&self, t_wait1: u64, t_wait2: u64) -> bool {
        match self.phase {
            ReviewPhase::Phase1 => self.wait_count >= t_wait1,
            ReviewPhase::Phase2 => self.wait_count >= t_wait2,
            ReviewPhase::Complete => false,
        }
    }
}

/// Rev.12 processing statistics
#[derive(Clone, Debug, Default)]
pub struct Rev12Stats {
    /// Number of RARE nodes that spoke_up (found analogy)
    pub spoke_up_count: u64,
    /// Number of RARE nodes demoted to GARBAGE
    pub demoted_count: u64,
    /// Total analogy discovery attempts
    pub discovery_attempts: u64,
    /// Successful analogy discoveries
    pub successful_discoveries: u64,
    /// Nodes promoted from RARE to CORE-candidate
    pub promoted_count: u64,
}

impl Rev12Stats {
    /// Spoke-up rate (successful discoveries / attempts)
    pub fn spoke_up_rate(&self) -> f64 {
        if self.discovery_attempts == 0 {
            0.0
        } else {
            self.successful_discoveries as f64 / self.discovery_attempts as f64
        }
    }
}

/// KDF Processor Rev.12 - With Analogy Discovery Mechanism
///
/// Implements the complete KDF Rev.12 specification:
/// - Two-phase review (T_wait1, T_wait2)
/// - Analogy discovery for RARE nodes (θ_disc = 0.75)
/// - spoke_up flag for truth vs garbage discrimination
/// - Structure-mapping based integration
pub struct KdfProcessorRev12 {
    /// Base classifier
    classifier: NodeClassifier,
    /// Decay manager
    decay_manager: DecayManager,
    /// Analogy discovery engine
    analogy_engine: AnalogyDiscoveryEngine,
    /// RARE node states under review
    pub(crate) rare_states: HashMap<u32, RareNodeState>,
    /// Phase 1 waiting period (cycles before Phase 2)
    pub t_wait1: u64,
    /// Phase 2 waiting period (cycles before GARBAGE demotion)
    pub t_wait2: u64,
    /// Discovery lower threshold θ_L (Claim 46: ∈[0.70, 0.80], Claim 48: =0.70)
    pub discovery_threshold: f64,
    /// Discovery upper threshold θ_U (Claim 47-48: > θ_L, canonical =0.80)
    pub discovery_threshold_upper: f64,
    /// Rev.12 statistics
    stats: Rev12Stats,
    /// Neighbors map for quick lookup
    neighbors: HashMap<u32, Vec<u32>>,
}

/// Lower bound of the multi-stage review period (Claim 39: 30 ≤ t_wait ≤ 70)
pub const T_WAIT_MIN: u64 = 30;
/// Upper bound of the multi-stage review period (Claim 39)
pub const T_WAIT_MAX: u64 = 70;
/// Canonical default matching Master Formulas §9.1 / Claim 39 mid-range
pub const T_WAIT_DEFAULT: u64 = 50;
/// Canonical discovery threshold (Claim 46: θ_L ∈ [0.70, 0.80])
pub const DISCOVERY_THRESHOLD_DEFAULT: f64 = 0.75;
/// Canonical upper-bound discovery threshold (Claim 48: θ_U = 0.80)
pub const DISCOVERY_THRESHOLD_UPPER_DEFAULT: f64 = 0.80;

impl Default for KdfProcessorRev12 {
    fn default() -> Self {
        Self {
            classifier: NodeClassifier::default(),
            decay_manager: DecayManager::default(),
            analogy_engine: AnalogyDiscoveryEngine::default(),
            rare_states: HashMap::new(),
            // Claim 39: multi-stage review periods must satisfy 30 ≤ t_wait ≤ 70.
            // Claim 45 further requires t_wait1 == t_wait2 in the canonical form.
            t_wait1: T_WAIT_DEFAULT,
            t_wait2: T_WAIT_DEFAULT,
            discovery_threshold: DISCOVERY_THRESHOLD_DEFAULT,
            discovery_threshold_upper: DISCOVERY_THRESHOLD_UPPER_DEFAULT,
            stats: Rev12Stats::default(),
            neighbors: HashMap::new(),
        }
    }
}

/// Errors returned when constructing a Claim-compliant Rev.12 processor.
#[derive(Debug, Clone, PartialEq)]
pub enum Rev12Error {
    /// Claim 39 violation: t_wait outside [30, 70].
    TwaitOutOfRange { value: u64 },
    /// Claim 46 violation: theta_L outside [0.70, 0.80].
    ThetaLowerOutOfRange { value: f64 },
    /// Claim 47 violation: theta_U <= theta_L.
    ThetaUpperNotAbove { theta_l: f64, theta_u: f64 },
}

impl std::fmt::Display for Rev12Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TwaitOutOfRange { value } => {
                write!(
                    f,
                    "Claim 39 violation: t_wait={} outside [{}, {}]",
                    value, T_WAIT_MIN, T_WAIT_MAX
                )
            }
            Self::ThetaLowerOutOfRange { value } => {
                write!(
                    f,
                    "Claim 46 violation: theta_L={} outside [0.70, 0.80]",
                    value
                )
            }
            Self::ThetaUpperNotAbove { theta_l, theta_u } => {
                write!(
                    f,
                    "Claim 47 violation: theta_U={} must be > theta_L={}",
                    theta_u, theta_l
                )
            }
        }
    }
}

impl std::error::Error for Rev12Error {}

impl KdfProcessorRev12 {
    /// Create a new Rev.12 processor with custom parameters.
    ///
    /// Returns `Err` if `t_wait1` / `t_wait2` violate Claim 39 (30–70) or
    /// `discovery_threshold` violates Claim 46 ([0.70, 0.80]).
    pub fn new(t_wait1: u64, t_wait2: u64, discovery_threshold: f64) -> Result<Self, Rev12Error> {
        Self::with_upper_threshold(
            t_wait1,
            t_wait2,
            discovery_threshold,
            DISCOVERY_THRESHOLD_UPPER_DEFAULT,
        )
    }

    /// Create a processor bypassing Claim-range validation (**internal test only**).
    ///
    /// Useful for fast unit tests where a full 30-cycle review would be noise.
    /// Never use in production paths: produced instances are not Claim-compliant.
    #[doc(hidden)]
    pub fn new_unchecked_for_tests(t_wait1: u64, t_wait2: u64, discovery_threshold: f64) -> Self {
        Self {
            t_wait1,
            t_wait2,
            discovery_threshold,
            discovery_threshold_upper: DISCOVERY_THRESHOLD_UPPER_DEFAULT
                .max(discovery_threshold + 1e-6),
            analogy_engine: AnalogyDiscoveryEngine::new(
                0.1,
                0.2,
                0.7,
                discovery_threshold,
                32,
                true,
                0.05,
            ),
            ..Default::default()
        }
    }

    /// Create a new Rev.12 processor with an explicit upper-bound threshold (Claim 47-48).
    pub fn with_upper_threshold(
        t_wait1: u64,
        t_wait2: u64,
        theta_l: f64,
        theta_u: f64,
    ) -> Result<Self, Rev12Error> {
        if !(T_WAIT_MIN..=T_WAIT_MAX).contains(&t_wait1) {
            return Err(Rev12Error::TwaitOutOfRange { value: t_wait1 });
        }
        if !(T_WAIT_MIN..=T_WAIT_MAX).contains(&t_wait2) {
            return Err(Rev12Error::TwaitOutOfRange { value: t_wait2 });
        }
        if !(0.70..=0.80).contains(&theta_l) {
            return Err(Rev12Error::ThetaLowerOutOfRange { value: theta_l });
        }
        if theta_u <= theta_l {
            return Err(Rev12Error::ThetaUpperNotAbove { theta_l, theta_u });
        }
        Ok(Self {
            t_wait1,
            t_wait2,
            discovery_threshold: theta_l,
            discovery_threshold_upper: theta_u,
            analogy_engine: AnalogyDiscoveryEngine::new(
                0.1, // attribute_weight  (Claim 44: 7:2:1)
                0.2, // relational_weight
                0.7, // systematic_weight
                theta_l, 32,   // fingerprint_dim
                true, // screening_enabled (Claim 46)
                0.05, // top_k_percent
            ),
            ..Default::default()
        })
    }

    /// Initialize processor with graph data
    pub fn initialize(&mut self, node_count: usize, edges: &[(u32, u32, f64)]) {
        // Classify nodes
        let classification = self.classifier.classify(node_count, edges);

        // Build neighbors map
        self.neighbors.clear();
        for &(from, to, _) in edges {
            self.neighbors.entry(from).or_default().push(to);
            self.neighbors.entry(to).or_default().push(from);
        }

        // Initialize RARE node states
        self.rare_states.clear();
        for (&node, &layer) in &classification.layers {
            if layer == Layer::Rare
                && let Some(fp) = classification.rare_fingerprints.get(&node)
            {
                self.rare_states
                    .insert(node, RareNodeState::new(node, fp.clone()));
            }
        }

        // Register non-GARBAGE nodes with analogy engine
        // GARBAGE nodes are skipped as they don't participate in analogy discovery
        for node in 0..node_count {
            let node_id = node as u32;
            let layer = classification
                .layers
                .get(&node_id)
                .copied()
                .unwrap_or(Layer::Edge);

            // Skip GARBAGE nodes - they don't participate in analogy discovery
            if layer == Layer::Garbage {
                continue;
            }

            let label = match layer {
                Layer::Core | Layer::Edge => NodeLabel::Normal,
                Layer::Rare => NodeLabel::IsolatedTruth,
                Layer::Garbage => unreachable!(), // already skipped above
            };

            let degree = self.neighbors.get(&node_id).map(|n| n.len()).unwrap_or(0) as u32;
            let mut features = NodeFeatures::new(node_id.to_string());
            features.degree = degree;

            self.analogy_engine
                .register_node(&node_id.to_string(), features, &label);
        }

        self.decay_manager.initialize(classification);
    }

    /// Pure discovery computation (Step 4 parallelization entry point).
    ///
    /// Read-only over `&self`: gathers candidates, calls
    /// `analogy_engine.compute_analogy`, and packages the result into a
    /// `DiscoveryOutcome` that the caller applies sequentially. Returns
    /// `None` only when `rare_node` is not in `rare_states` (sentinel
    /// matching the early-return in the sequential `attempt_discovery`).
    pub fn compute_discovery(&self, rare_node: u32) -> Option<DiscoveryOutcome> {
        if !self.rare_states.contains_key(&rare_node) {
            return None;
        }

        // Get candidate nodes (CORE and EDGE layers).
        let candidates: Vec<String> = self
            .decay_manager
            .classification
            .as_ref()
            .map(|c| {
                c.layers
                    .iter()
                    .filter(|&(_, &layer)| layer == Layer::Core || layer == Layer::Edge)
                    .map(|(&id, _)| id.to_string())
                    .collect()
            })
            .unwrap_or_default();

        if candidates.is_empty() {
            return Some(DiscoveryOutcome {
                invoked_analogy: false,
                analogy_result: None,
            });
        }

        let analogy_result = self
            .analogy_engine
            .compute_analogy(&rare_node.to_string(), &candidates);

        Some(DiscoveryOutcome {
            invoked_analogy: true,
            analogy_result,
        })
    }

    /// Apply the side effects of a `DiscoveryOutcome` to the rev12 + analogy
    /// engine state. Mirrors the mutation phase of the sequential
    /// `attempt_discovery` and is shared between `attempt_discovery` and the
    /// parallel `process_review_cycle`.
    ///
    /// Returns `true` when the rare node spoke_up (Claim 47–48 band match).
    fn apply_discovery_outcome(&mut self, rare_node: u32, outcome: &DiscoveryOutcome) -> bool {
        if outcome.invoked_analogy {
            self.analogy_engine.record_discovery_attempt();
            if let Some(ref result) = outcome.analogy_result {
                self.analogy_engine.apply_compute_result(result);
            }
        }

        let band = outcome.spoke_up(self.discovery_threshold, self.discovery_threshold_upper);
        let state = self
            .rare_states
            .get_mut(&rare_node)
            .expect("apply_discovery_outcome requires rare_node in rare_states");

        if let Some((target, score)) = band {
            // Claim 47-48 sandwich match: spoke_up.
            state.spoke_up = true;
            state.analogy_target = Some(target);
            state.analogy_score = score;
            state.phase = ReviewPhase::Complete;
            self.stats.spoke_up_count += 1;
            self.stats.successful_discoveries += 1;
            true
        } else {
            state.wait_count += 1;
            false
        }
    }

    /// Attempt analogy discovery for a RARE node.
    ///
    /// Returns true if analogy found (spoke_up), false otherwise.
    ///
    /// Equivalent to: pure `compute_discovery` followed by sequential
    /// `apply_discovery_outcome`. Kept for API compatibility — the parallel
    /// `process_review_cycle` does not call this; it dispatches
    /// `compute_discovery` via `par_iter` and aggregates outcomes itself.
    pub fn attempt_discovery(&mut self, rare_node: u32) -> bool {
        self.stats.discovery_attempts += 1;
        let outcome = match self.compute_discovery(rare_node) {
            Some(o) => o,
            None => return false,
        };
        self.apply_discovery_outcome(rare_node, &outcome)
    }

    /// Process review cycle for all RARE nodes.
    ///
    /// Returns list of (node, action) where action is "promote" or "demote".
    ///
    /// # Parallelism (rayon, Step 4)
    ///
    /// Each RARE node's `compute_discovery` is independent and read-only over
    /// `&self`, so the heavy O(N_candidates) per-node work runs through
    /// `par_iter`. Outcomes are reassembled in sorted-node-id order — strictly
    /// more deterministic than the previous `HashMap::keys()` iteration —
    /// and applied sequentially via `apply_discovery_outcome` so that
    /// `discovery_history.push`, stats counters, and phase transitions all
    /// run in a fixed order. Counters are commutative; the deterministic
    /// `discovery_history` order is a strict improvement over the previous
    /// behavior.
    pub fn process_review_cycle(&mut self) -> Vec<(u32, &'static str)> {
        let mut actions = Vec::new();

        // Active nodes in deterministic (sorted) order.
        let mut active_nodes: Vec<u32> = self
            .rare_states
            .iter()
            .filter(|(_, s)| s.phase != ReviewPhase::Complete)
            .map(|(&n, _)| n)
            .collect();
        active_nodes.sort();

        // Parallel pure compute: per-node, read-only, heavy O(N_candidates).
        let outcomes: Vec<(u32, DiscoveryOutcome)> = active_nodes
            .par_iter()
            .filter_map(|&node| self.compute_discovery(node).map(|o| (node, o)))
            .collect();

        // Sequential mutation phase (deterministic order).
        for (node, outcome) in outcomes {
            self.stats.discovery_attempts += 1;
            let found = self.apply_discovery_outcome(node, &outcome);

            if found {
                actions.push((node, "promote"));
                self.stats.promoted_count += 1;
            } else {
                let state = self.rare_states.get_mut(&node).unwrap();
                match state.phase {
                    ReviewPhase::Phase1 if state.wait_count >= self.t_wait1 => {
                        state.phase = ReviewPhase::Phase2;
                        state.wait_count = 0;
                    }
                    ReviewPhase::Phase2 if state.wait_count >= self.t_wait2 => {
                        state.phase = ReviewPhase::Complete;
                        actions.push((node, "demote"));
                        self.stats.demoted_count += 1;
                    }
                    _ => {}
                }
            }
        }

        actions
    }

    /// Apply promotion: RARE → CORE-candidate
    ///
    /// The RARE node integrates with its analogy target
    pub fn apply_promotion(&mut self, node: u32) {
        if let Some(ref mut classification) = self.decay_manager.classification {
            // Change layer to Edge (will behave like CORE candidate)
            classification.layers.insert(node, Layer::Edge);
            classification.stats.rare_count = classification.stats.rare_count.saturating_sub(1);
            classification.stats.edge_count += 1;

            // Remove fingerprint (no longer needs protection)
            classification.rare_fingerprints.remove(&node);
        }
    }

    /// Apply demotion: RARE → GARBAGE
    pub fn apply_demotion(&mut self, node: u32) {
        if let Some(ref mut classification) = self.decay_manager.classification {
            classification.layers.insert(node, Layer::Garbage);
            classification.stats.rare_count = classification.stats.rare_count.saturating_sub(1);
            classification.stats.garbage_count += 1;
            classification.rare_fingerprints.remove(&node);
        }
    }

    /// Get RARE node state
    pub fn get_rare_state(&self, node: u32) -> Option<&RareNodeState> {
        self.rare_states.get(&node)
    }

    /// Get all RARE nodes with spoke_up=true and their analogy targets
    pub fn get_spoke_up_nodes(&self) -> Vec<(u32, u32, f64)> {
        self.rare_states
            .iter()
            .filter(|(_, state)| state.spoke_up)
            .filter_map(|(&node, state)| {
                state
                    .analogy_target
                    .map(|target| (node, target, state.analogy_score))
            })
            .collect()
    }

    /// Check if a node should be skipped
    pub fn should_skip(&self, node: u32) -> bool {
        self.decay_manager.should_skip(node)
    }

    /// Check if a node is protected (RARE under review)
    pub fn is_protected(&self, node: u32) -> bool {
        // In Rev.12, RARE nodes are protected during review
        if let Some(state) = self.rare_states.get(&node) {
            state.phase != ReviewPhase::Complete
        } else {
            self.decay_manager.is_protected(node)
        }
    }

    /// Get nodes to process in optimal order
    pub fn processing_order(&self) -> Vec<u32> {
        if let Some(ref class) = self.decay_manager.classification {
            self.classifier.processing_order(class)
        } else {
            Vec::new()
        }
    }

    /// Get the layer of a node (current, after any promotions/demotions)
    pub fn get_layer(&self, node: u32) -> Option<Layer> {
        self.decay_manager
            .classification
            .as_ref()?
            .layers
            .get(&node)
            .copied()
    }

    /// Get classification statistics
    pub fn classification_stats(&self) -> Option<&ClassificationStats> {
        self.decay_manager.stats()
    }

    /// Get Rev.12 statistics
    pub fn rev12_stats(&self) -> &Rev12Stats {
        &self.stats
    }

    /// Get neighbor of a RARE node (for module placement)
    ///
    /// If spoke_up, returns the analogy target.
    /// Otherwise, returns the direct neighbor.
    pub fn get_rare_neighbor(&self, node: u32) -> Option<u32> {
        // First check if spoke_up - use analogy target
        if let Some(state) = self.rare_states.get(&node)
            && state.spoke_up
        {
            return state.analogy_target;
        }
        // Otherwise, use direct neighbor
        self.neighbors.get(&node).and_then(|n| n.first().copied())
    }

    /// Get direct neighbor of a node (ignores spoke_up status)
    ///
    /// Always returns the first direct neighbor from the graph structure.
    /// Used for semantic preservation where physical connection matters.
    pub fn get_direct_neighbor(&self, node: u32) -> Option<u32> {
        self.neighbors.get(&node).and_then(|n| n.first().copied())
    }

    /// Get all nodes that were originally classified as RARE
    pub fn get_original_rare_nodes(&self) -> Vec<u32> {
        self.rare_states.keys().copied().collect()
    }
}
