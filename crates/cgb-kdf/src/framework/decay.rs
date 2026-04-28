//! Decay Manager - Tracks and applies decay to knowledge
//!
//! ## Master Specification (Edge-Based)
//!
//! The decay formula for edge (u,v) is:
//! ```text
//! P_decay(u,v) = min(1.0, β × (1 + γ × C_(u,v)^α))
//! ```
//! where C_(u,v) = deg(u) + deg(v) (sum of endpoint degrees)
//!
//! Layer-specific parameters:
//! - Edge:  α=1.5, γ=0.015
//! - Rare:  α=0.3, γ=0.010
//! - Core:  α=2.0, γ=0.008
//! - Meta:  α=0.5, γ=0.005

use super::{ClassificationStats, Layer, NodeClassification};
use crate::fingerprint::Fingerprint;
use std::collections::HashMap;

/// Master specification parameters for edge-based decay
///
/// # Patent claim mapping
///
/// - Claim 8-9: 単調増加非線形関数・べき乗項
/// - Claim 14: λ(C)=β(1+γC^α), w ← w·exp(-λ·dt)
#[derive(Clone, Debug)]
pub struct MasterSpecParams {
    /// Base decay rate (β = 0.01)
    pub beta: f64,
    /// Edge layer gamma (γ_E = 0.015)
    pub gamma_edge: f64,
    /// Rare layer gamma (γ_R = 0.010)
    pub gamma_rare: f64,
    /// Core layer gamma (γ_C = 0.008)
    pub gamma_core: f64,
    /// Meta layer gamma (γ_M = 0.005)
    pub gamma_meta: f64,
    /// Edge layer alpha (α_E = 1.5)
    pub alpha_edge: f64,
    /// Rare layer alpha (α_R = 0.3)
    pub alpha_rare: f64,
    /// Core layer alpha (α_C = 2.0)
    pub alpha_core: f64,
    /// Meta layer alpha (α_M = 0.5)
    pub alpha_meta: f64,
    /// Time step dt for continuous-time decay update (Claim 14)
    ///
    /// Edge: dt=0.005, Rare: dt=0.001, Core: dt=0.003, Meta: dt=0.001
    pub dt_edge: f64,
    pub dt_rare: f64,
    pub dt_core: f64,
    pub dt_meta: f64,
}

impl Default for MasterSpecParams {
    fn default() -> Self {
        Self {
            beta: 0.01,
            gamma_edge: 0.015,
            gamma_rare: 0.010,
            gamma_core: 0.008,
            gamma_meta: 0.005,
            alpha_edge: 1.5,
            alpha_rare: 0.3,
            alpha_core: 2.0,
            alpha_meta: 0.5,
            dt_edge: 0.005,
            dt_rare: 0.001,
            dt_core: 0.003,
            dt_meta: 0.001,
        }
    }
}

impl MasterSpecParams {
    /// Get gamma for a specific layer
    pub fn gamma_for_layer(&self, layer: Layer) -> f64 {
        match layer {
            Layer::Edge => self.gamma_edge,
            Layer::Rare => self.gamma_rare,
            Layer::Core => self.gamma_core,
            Layer::Garbage => self.gamma_edge, // Default for garbage
        }
    }

    /// Get alpha for a specific layer
    pub fn alpha_for_layer(&self, layer: Layer) -> f64 {
        match layer {
            Layer::Edge => self.alpha_edge,
            Layer::Rare => self.alpha_rare,
            Layer::Core => self.alpha_core,
            Layer::Garbage => self.alpha_edge, // Default for garbage
        }
    }

    /// Get dt for a specific layer (Claim 14, Claim 29 dt1:dt2:dt3=5:3:1 mapping)
    pub fn dt_for_layer(&self, layer: Layer) -> f64 {
        match layer {
            Layer::Edge => self.dt_edge,
            Layer::Rare => self.dt_rare,
            Layer::Core => self.dt_core,
            Layer::Garbage => self.dt_edge,
        }
    }

    /// Compute decay rate λ(C) = β × (1 + γ × C^α) (Claim 14, master formula)
    pub fn lambda(&self, congestion: f64, layer: Layer) -> f64 {
        let gamma = self.gamma_for_layer(layer);
        let alpha = self.alpha_for_layer(layer);
        self.beta * (1.0 + gamma * congestion.powf(alpha))
    }
}

/// Decay Manager - Tracks and applies decay to knowledge
#[derive(Clone)]
pub struct DecayManager {
    /// Current layer classification
    pub(super) classification: Option<NodeClassification>,
    /// Access count per node (for dynamic decay)
    access_counts: HashMap<u32, u64>,
    /// Access count per edge (for edge-based decay)
    edge_access_counts: HashMap<(u32, u32), u64>,
    /// Edge weights (for edge-based processing)
    edge_weights: HashMap<(u32, u32), f64>,
    /// Node degrees (cached for efficiency)
    degrees: HashMap<u32, usize>,
    /// Claim 4-5: last access step per edge (time-series metadata).
    last_access_step: HashMap<(u32, u32), u64>,
    /// Claim 4-5: global tick counter used as the reference "current time".
    current_step: u64,
    /// Claim 5: staleness reference scale τ_ref (ticks). Larger → slower time
    /// contribution. Default is tuned so ~100 idle ticks contribute O(1) to the
    /// multiplier.
    pub tau_ref: f64,
    /// Claim 5: positive coefficient κ that controls the weight of the time
    /// evaluation component inside the evaluation value.
    pub kappa_time: f64,
    /// Decay rate (0.0 = no decay, 1.0 = instant decay) - legacy node-based
    pub decay_rate: f64,
    /// Master spec parameters (edge-based)
    pub master_params: MasterSpecParams,
    /// Use edge-based processing
    pub use_edge_based: bool,
}

impl Default for DecayManager {
    fn default() -> Self {
        Self {
            classification: None,
            access_counts: HashMap::new(),
            edge_access_counts: HashMap::new(),
            edge_weights: HashMap::new(),
            degrees: HashMap::new(),
            last_access_step: HashMap::new(),
            current_step: 0,
            tau_ref: 100.0,
            kappa_time: 1.0,
            decay_rate: 0.1,
            master_params: MasterSpecParams::default(),
            use_edge_based: true, // Default: Master spec compliant edge-based
        }
    }
}

impl DecayManager {
    /// Create a new DecayManager with Master spec (edge-based)
    pub fn master_spec() -> Self {
        Self {
            use_edge_based: true,
            ..Self::default()
        }
    }

    /// Initialize with classification
    pub fn initialize(&mut self, classification: NodeClassification) {
        self.classification = Some(classification);
        self.access_counts.clear();
        self.edge_access_counts.clear();
    }

    /// Initialize with edges for edge-based processing
    pub fn initialize_with_edges(
        &mut self,
        classification: NodeClassification,
        edges: &[(u32, u32, f64)],
    ) {
        self.classification = Some(classification);
        self.access_counts.clear();
        self.edge_access_counts.clear();

        // Initialize edge weights and compute degrees
        self.edge_weights.clear();
        self.degrees.clear();

        for &(u, v, w) in edges {
            self.edge_weights.insert((u, v), w);
            *self.degrees.entry(u).or_insert(0) += 1;
            *self.degrees.entry(v).or_insert(0) += 1;
        }
    }

    /// Check if a node should be skipped (GARBAGE or decayed)
    pub fn should_skip(&self, node: u32) -> bool {
        if let Some(ref class) = self.classification
            && let Some(&layer) = class.layers.get(&node)
        {
            return !layer.should_process();
        }
        false
    }

    /// Check if an edge should be skipped
    pub fn should_skip_edge(&self, u: u32, v: u32) -> bool {
        self.should_skip(u) || self.should_skip(v)
    }

    /// Check if a node is protected (RARE)
    pub fn is_protected(&self, node: u32) -> bool {
        if let Some(ref class) = self.classification
            && let Some(&layer) = class.layers.get(&node)
        {
            return layer.is_protected();
        }
        false
    }

    /// Record access to a node (for dynamic decay tracking)
    pub fn record_access(&mut self, node: u32) {
        *self.access_counts.entry(node).or_insert(0) += 1;
    }

    /// Record access to an edge (for edge-based decay tracking)
    pub fn record_edge_access(&mut self, u: u32, v: u32) {
        let key = if u < v { (u, v) } else { (v, u) };
        *self.edge_access_counts.entry(key).or_insert(0) += 1;
        // Claim 5: timestamp the event so the time evaluation component stays fresh.
        self.last_access_step.insert(key, self.current_step);
    }

    /// Claim 5: advance the global time step by one tick. Time evaluation
    /// components are measured against this counter.
    pub fn tick(&mut self) {
        self.current_step = self.current_step.saturating_add(1);
    }

    /// Claim 5: current global step counter.
    pub fn current_step(&self) -> u64 {
        self.current_step
    }

    /// Claim 5 — time evaluation component derived from time-series metadata.
    ///
    /// Uses the last-access step (Claim 4 metadata: 参照時刻/参照回数) and the
    /// current tick as inputs. Returns a non-negative value that grows with
    /// staleness, saturating at 1.0:
    ///
    /// ```text
    /// T(e) = 1 - exp( -(now - last_access(e)) / τ_ref )
    /// ```
    ///
    /// Edges that have never been accessed are treated as stale from step 0,
    /// matching the spirit of Claim 4 (generation time counts as an access).
    pub fn compute_time_component(&self, u: u32, v: u32) -> f64 {
        let key = if u < v { (u, v) } else { (v, u) };
        let last = self.last_access_step.get(&key).copied().unwrap_or(0);
        let dt = self.current_step.saturating_sub(last) as f64;
        1.0 - (-dt / self.tau_ref).exp()
    }

    /// Claim 5 — evaluation value that *includes* the time evaluation component.
    ///
    /// Composition:
    /// ```text
    /// V(e) = P_decay(C, layer) * (1 + κ · T(e))
    /// ```
    /// where `P_decay` is the Claim 14 base evaluation and `T(e)` is the
    /// Claim 5 time component. The value is clamped to `[0, 1]` so it stays a
    /// valid probability for downstream Claim 12 Bernoulli comparisons.
    pub fn compute_evaluation_value(&self, u: u32, v: u32, layer: Layer) -> f64 {
        let p_decay = self.compute_edge_decay_probability(u, v, layer);
        let t_comp = self.compute_time_component(u, v);
        (p_decay * (1.0 + self.kappa_time * t_comp)).clamp(0.0, 1.0)
    }

    /// Compute edge congestion: C_(u,v) = deg(u) + deg(v)
    pub fn compute_edge_congestion(&self, u: u32, v: u32) -> f64 {
        let deg_u = self.degrees.get(&u).copied().unwrap_or(0);
        let deg_v = self.degrees.get(&v).copied().unwrap_or(0);
        (deg_u + deg_v) as f64
    }

    /// Compute decay probability for an edge (Claim 14: P_decay bounded per step)
    ///
    /// P_decay(u,v) = 1 - exp(-λ(C) × dt), where λ(C)=β(1+γC^α).
    ///
    /// For small λ·dt this is ≈ λ·dt, but the exp form is required by Claim 14.
    pub fn compute_edge_decay_probability(&self, u: u32, v: u32, layer: Layer) -> f64 {
        let lambda = self
            .master_params
            .lambda(self.compute_edge_congestion(u, v), layer);
        let dt = self.master_params.dt_for_layer(layer);
        1.0 - (-lambda * dt).exp()
    }

    /// Apply continuous-time exponential decay to all edges (Claim 14 canonical form)
    ///
    /// Update rule: w ← w · exp(-λ(C) · dt)
    ///
    /// # Determinism
    ///
    /// Edge iteration order is sorted by (u, v) to guarantee reproducible
    /// results across platforms/runs (HashMap's default iteration order is
    /// non-deterministic; relying on it would violate Claim 15 determinism
    /// semantics when combined with probabilistic pruning).
    pub fn apply_edge_decay(&mut self) {
        if let Some(ref class) = self.classification {
            let mut edges: Vec<_> = self.edge_weights.keys().cloned().collect();
            edges.sort(); // Determinism: sort by (u, v)
            for (u, v) in edges {
                // Get layer from endpoint nodes (use higher priority)
                let layer_u = class.layers.get(&u).copied().unwrap_or(Layer::Edge);
                let layer_v = class.layers.get(&v).copied().unwrap_or(Layer::Edge);
                let layer = if layer_u.priority() > layer_v.priority() {
                    layer_u
                } else {
                    layer_v
                };

                let lambda = self
                    .master_params
                    .lambda(self.compute_edge_congestion(u, v), layer);
                let dt = self.master_params.dt_for_layer(layer);
                let survival = (-lambda * dt).exp();
                if let Some(weight) = self.edge_weights.get_mut(&(u, v)) {
                    *weight *= survival;
                    // Flush denormals: below 2^-970 the arithmetic becomes
                    // platform-dependent (subnormals). Treat as 0 to preserve
                    // Claim 15-style bit-exact reproducibility.
                    if weight.abs() < 1e-290 {
                        *weight = 0.0;
                    }
                }
            }
        }
    }

    /// Claim 11 & 12 — probabilistic pruning.
    ///
    /// Returns the list of edges selected for removal by comparing the
    /// per-step exclusion probability P_decay ∈ [0,1] against a caller-
    /// supplied random value r ∈ [0,1). If `r ≤ P_decay` the edge is pruned.
    /// The probability is derived from the same λ(C) and dt as Claim 14, but
    /// interpreted as a Bernoulli trial rather than a weight multiplier, per
    /// Claim 12.
    ///
    /// Protects Rare nodes (Claim 15, 18).
    pub fn probabilistic_prune<F>(&self, mut rng: F) -> Vec<(u32, u32)>
    where
        F: FnMut() -> f64,
    {
        let mut pruned = Vec::new();
        let class = match &self.classification {
            Some(c) => c,
            None => return pruned,
        };
        for (&(u, v), _w) in self.edge_weights.iter() {
            let layer_u = class.layers.get(&u).copied().unwrap_or(Layer::Edge);
            let layer_v = class.layers.get(&v).copied().unwrap_or(Layer::Edge);
            if layer_u.is_protected() || layer_v.is_protected() {
                continue; // Claim 15/18 protection
            }
            let layer = if layer_u.priority() > layer_v.priority() {
                layer_u
            } else {
                layer_v
            };
            let p_decay = self
                .compute_edge_decay_probability(u, v, layer)
                .clamp(0.0, 1.0);
            let r = rng();
            if r <= p_decay {
                pruned.push((u, v));
            }
        }
        pruned
    }

    /// Get edge weight
    pub fn get_edge_weight(&self, u: u32, v: u32) -> Option<f64> {
        self.edge_weights
            .get(&(u, v))
            .or_else(|| self.edge_weights.get(&(v, u)))
            .copied()
    }

    /// Claim 17 — compute decay updates for a *locally owned* edge subset.
    ///
    /// This is the distributed-processing entry point: a processing agent
    /// supplies only the edges it owns and receives the new weights, without
    /// needing to touch the global store. The same λ(C)·exp(-λdt) logic is
    /// applied, using only the local congestion view supplied by the caller.
    ///
    /// `local_edges` is an iterator of `((u,v), weight, congestion, layer)`.
    /// The returned vector preserves input order.
    pub fn apply_edge_decay_local<I>(&self, local_edges: I) -> Vec<((u32, u32), f64)>
    where
        I: IntoIterator<Item = ((u32, u32), f64, f64, Layer)>,
    {
        local_edges
            .into_iter()
            .map(|((u, v), w, c, layer)| {
                let lambda = self.master_params.lambda(c, layer);
                let dt = self.master_params.dt_for_layer(layer);
                let survival = (-lambda * dt).exp();
                ((u, v), w * survival)
            })
            .collect()
    }

    /// Get RARE node fingerprint for preservation check
    pub fn get_rare_fingerprint(&self, node: u32) -> Option<&Fingerprint> {
        self.classification.as_ref()?.rare_fingerprints.get(&node)
    }

    /// Get nodes to process (non-GARBAGE, respecting protection)
    pub fn processable_nodes(&self) -> Vec<u32> {
        if let Some(ref class) = self.classification {
            class
                .layers
                .iter()
                .filter(|(_, layer)| layer.should_process())
                .map(|(&id, _)| id)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get edges to process (both endpoints must be processable)
    pub fn processable_edges(&self) -> Vec<(u32, u32)> {
        self.edge_weights
            .keys()
            .filter(|&&(u, v)| !self.should_skip_edge(u, v))
            .cloned()
            .collect()
    }

    /// Get classification statistics
    pub fn stats(&self) -> Option<&ClassificationStats> {
        self.classification.as_ref().map(|c| &c.stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_spec_params() {
        let params = MasterSpecParams::default();

        assert_eq!(params.beta, 0.01);
        assert_eq!(params.gamma_edge, 0.015);
        assert_eq!(params.gamma_rare, 0.010);
        assert_eq!(params.gamma_core, 0.008);
    }

    #[test]
    fn test_edge_congestion() {
        let mut manager = DecayManager::master_spec();
        manager.degrees.insert(0, 3);
        manager.degrees.insert(1, 2);

        let congestion = manager.compute_edge_congestion(0, 1);
        assert_eq!(congestion, 5.0);
    }

    #[test]
    fn test_edge_decay_probability_exp_form() {
        // Claim 14: P_decay is derived from continuous-time exp(-λdt).
        let mut manager = DecayManager::master_spec();
        manager.degrees.insert(0, 3);
        manager.degrees.insert(1, 2);

        // Edge layer: β=0.01, γ=0.015, α=1.5, C=5, dt=0.005
        let lambda = 0.01 * (1.0 + 0.015 * 5.0_f64.powf(1.5));
        let dt = 0.005;
        let expected = 1.0 - (-lambda * dt).exp();
        let actual = manager.compute_edge_decay_probability(0, 1, Layer::Edge);
        assert!(
            (actual - expected).abs() < 1e-12,
            "P_decay must follow Claim 14 exp form"
        );
    }

    #[test]
    fn test_lambda_master_form() {
        // Claim 14: λ(C) = β(1+γC^α)
        let p = MasterSpecParams::default();
        let lam = p.lambda(5.0, Layer::Edge);
        assert!((lam - 0.01 * (1.0 + 0.015 * 5.0_f64.powf(1.5))).abs() < 1e-12);
    }

    #[test]
    fn test_claim12_probabilistic_pruning_rand_comparison() {
        use super::super::{ClassificationStats, NodeClassification};
        use std::collections::HashMap;
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);

        // Force P_decay high via a very connected toy degree override.
        manager.degrees.insert(0, 100);
        manager.degrees.insert(1, 100);

        // r=0 always <= p ⇒ always pruned
        let pruned_all = manager.probabilistic_prune(|| 0.0);
        assert_eq!(pruned_all, vec![(0u32, 1u32)]);

        // r=1 always > p ⇒ never pruned
        let pruned_none = manager.probabilistic_prune(|| 1.0);
        assert!(pruned_none.is_empty());
    }

    #[test]
    fn test_claim17_local_decay_matches_global() {
        // Global decay applied to the whole graph must equal per-partition
        // local decay composed back together (Claim 17 distributed semantics).
        use super::super::{ClassificationStats, NodeClassification};
        use std::collections::HashMap;

        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        for i in 0..6u32 {
            layers.insert(i, Layer::Edge);
        }
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        let edges = vec![
            (0, 1, 1.0),
            (1, 2, 1.0),
            (2, 3, 1.0),
            (3, 4, 1.0),
            (4, 5, 1.0),
        ];
        manager.initialize_with_edges(class, &edges);

        // Snapshot degrees *before* mutation so both paths see identical input.
        let mut global_mgr = manager.clone();
        global_mgr.apply_edge_decay();
        let globals: Vec<_> = global_mgr
            .edge_weights
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect();

        // Local: split into two halves, process independently, merge.
        let half = edges.len() / 2;
        let local_input: Vec<_> = edges
            .iter()
            .map(|&(u, v, w)| {
                let c = manager.compute_edge_congestion(u, v);
                ((u, v), w, c, Layer::Edge)
            })
            .collect();
        let partition_a = manager.apply_edge_decay_local(local_input[..half].iter().cloned());
        let partition_b = manager.apply_edge_decay_local(local_input[half..].iter().cloned());
        let mut locals: HashMap<(u32, u32), f64> = HashMap::new();
        for (e, w) in partition_a.into_iter().chain(partition_b) {
            locals.insert(e, w);
        }

        for (edge, g_w) in globals {
            let l_w: f64 = locals.get(&edge).copied().unwrap_or(0.0);
            assert!(
                (l_w - g_w).abs() < 1e-12,
                "local≠global for {:?}: local={}, global={}",
                edge,
                l_w,
                g_w
            );
        }
    }

    #[test]
    fn test_claim12_protects_rare() {
        use super::super::{ClassificationStats, NodeClassification};
        use std::collections::HashMap;
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Rare);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        manager.degrees.insert(0, 100);
        manager.degrees.insert(1, 100);
        let pruned = manager.probabilistic_prune(|| 0.0);
        assert!(
            pruned.is_empty(),
            "Rare endpoints must be protected (Claim 15, 18)"
        );
    }

    #[test]
    fn test_claim5_time_component_zero_at_access() {
        // Claim 5: just-accessed edge has T(e) ≈ 0.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        manager.record_edge_access(0, 1);
        let t = manager.compute_time_component(0, 1);
        assert!(
            t.abs() < 1e-12,
            "freshly-accessed edge must have T≈0, got {}",
            t
        );
    }

    #[test]
    fn test_claim5_time_component_grows_with_staleness() {
        // Claim 5: T(e) must be monotonically non-decreasing in elapsed ticks.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        manager.record_edge_access(0, 1);
        let t0 = manager.compute_time_component(0, 1);
        for _ in 0..50 {
            manager.tick();
        }
        let t1 = manager.compute_time_component(0, 1);
        for _ in 0..500 {
            manager.tick();
        }
        let t2 = manager.compute_time_component(0, 1);

        assert!(t1 > t0, "T must grow after 50 ticks: {} vs {}", t1, t0);
        assert!(
            t2 > t1,
            "T must keep growing after 500 ticks: {} vs {}",
            t2,
            t1
        );
        assert!(t2 < 1.0 + 1e-12, "T is bounded by 1, got {}", t2);
    }

    #[test]
    fn test_claim5_evaluation_value_includes_time_component() {
        // Claim 5 core invariant: V(e) must strictly depend on the time
        // evaluation component. Two edges with identical C but different
        // staleness must produce different evaluation values.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        for n in 0..4u32 {
            layers.insert(n, Layer::Edge);
        }
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        // Both edges share identical degrees (deg(u)+deg(v)=2) so λ(C) is equal.
        manager.initialize_with_edges(class, &[(0, 1, 1.0), (2, 3, 1.0)]);

        // Edge (0,1) is kept fresh by accessing at every tick; (2,3) is ignored.
        for _ in 0..200 {
            manager.record_edge_access(0, 1);
            manager.tick();
        }
        let fresh = manager.compute_evaluation_value(0, 1, Layer::Edge);
        let stale = manager.compute_evaluation_value(2, 3, Layer::Edge);
        assert!(
            stale > fresh,
            "stale edge must evaluate higher: fresh={}, stale={}",
            fresh,
            stale
        );
    }

    #[test]
    fn test_claim10_alpha_equals_two() {
        // Claim 10: the power term exponent is 2. The framework allows any α,
        // so we pin α=2 for every layer and verify that λ(C) = β(1 + γC²).
        let p = MasterSpecParams {
            alpha_edge: 2.0,
            alpha_rare: 2.0,
            alpha_core: 2.0,
            alpha_meta: 2.0,
            ..MasterSpecParams::default()
        };
        for &(layer, gamma) in &[
            (Layer::Edge, p.gamma_edge),
            (Layer::Rare, p.gamma_rare),
            (Layer::Core, p.gamma_core),
        ] {
            let c = 5.0;
            let expected = p.beta * (1.0 + gamma * c * c); // γC²
            let actual = p.lambda(c, layer);
            assert!(
                (actual - expected).abs() < 1e-12,
                "Claim 10 α=2: λ({:?},C=5) expected {}, got {}",
                layer,
                expected,
                actual
            );
        }
    }

    #[test]
    fn test_claim33_isolation_metric_is_function_of_connection_count() {
        // Claim 33: the rarity/isolation indicator must be based on AT LEAST
        // ONE of (strength, frequency, connection count, temporal evolution).
        //
        // The cgb-kdf classifier uses **connection count** (neighbor_count) as
        // the operative signal for the rare-vs-garbage split: isolated nodes
        // (neighbor_count=0) → Garbage, single-connection nodes → Rare
        // candidate, well-connected nodes → Edge/Core. This test pins that
        // behavior and fails loudly if the signal is removed entirely.
        use super::super::NodeClassifier;

        // Graph constructed so three nodes have three different connection
        // counts: 0, 1, 4 — producing three different layers.
        // Node 0: hub (4 connections) → Edge/Core candidate
        // Node 1: single connection to hub → Rare candidate
        // Node 6: isolated (no edges) → Garbage
        let n = 7;
        let edges: Vec<(u32, u32, f64)> = vec![
            (0, 1, 1.0),
            (0, 2, 1.0),
            (0, 3, 1.0),
            (0, 4, 1.0),
            (2, 3, 1.0),
            (3, 4, 1.0),
        ];
        let mut c = NodeClassifier::default();
        let cls = c.classify(n, &edges);
        let layer_hub = cls.layers.get(&0u32).copied().unwrap();
        let layer_leaf = cls.layers.get(&1u32).copied().unwrap();
        let layer_isolated = cls.layers.get(&6u32).copied().unwrap();
        // Three distinct connection counts must produce at least 2 distinct
        // layers (in practice all 3 distinct).
        assert_ne!(
            layer_hub, layer_isolated,
            "Claim 33: well-connected vs isolated must differ by isolation metric"
        );
        assert_ne!(
            layer_leaf, layer_isolated,
            "Claim 33: single-connection vs isolated must differ by isolation metric"
        );
        // Specific: isolated → Garbage (the spec's floor of isolation)
        assert_eq!(
            layer_isolated,
            Layer::Garbage,
            "Claim 33: isolation metric at its extreme must produce Garbage"
        );
    }

    // ------------------------------------------------------------
    // Per-claim direct tests — one function per dependent claim that
    // asserts the exact wording of the claim against the implementation.
    // ------------------------------------------------------------

    #[test]
    fn test_claim2_graph_structure_nodes_and_edges() {
        // Claim 2: data structure is a graph — nodes + edges
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        layers.insert(2u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0), (1, 2, 1.0)]);
        // Node count and edge count must be explicit and queryable.
        assert_eq!(
            manager.edge_weights.len(),
            2,
            "Claim 2: edges must be stored"
        );
        assert_eq!(manager.degrees.len(), 3, "Claim 2: nodes must be stored");
    }

    #[test]
    fn test_claim3_edge_parameter_is_strength_like() {
        // Claim 3: relation info has strength/frequency/reliability parameters.
        // edge_weights carries the strength parameter; record_edge_access
        // provides the frequency parameter.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 0.73)]);
        assert_eq!(
            manager.get_edge_weight(0, 1),
            Some(0.73),
            "Claim 3: strength param stored"
        );
        manager.record_edge_access(0, 1);
        manager.record_edge_access(0, 1);
        assert_eq!(
            *manager.edge_access_counts.get(&(0, 1)).unwrap(),
            2,
            "Claim 3: frequency param stored"
        );
    }

    #[test]
    fn test_claim4_time_series_metadata_present() {
        // Claim 4: 時間系メタデータ must include at least one of:
        //   generation_time, update_time, reference_time, reference_count, input_count.
        // Implementation: `edge_access_counts` = reference_count; `last_access_step`
        // = reference_time (derived from tick counter).
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        manager.tick();
        manager.tick();
        manager.record_edge_access(0, 1);
        assert_eq!(
            manager.current_step(),
            2,
            "Claim 4: global reference time tracked"
        );
        let stored_time = *manager.last_access_step.get(&(0u32, 1u32)).unwrap();
        assert_eq!(stored_time, 2, "Claim 4: per-edge reference_time stored");
        assert!(
            manager.edge_access_counts.contains_key(&(0u32, 1u32)),
            "Claim 4: reference_count stored"
        );
    }

    #[test]
    fn test_claim6_local_congestion_from_connection_count() {
        // Claim 6: 局所混雑度指標 based on 接続量 (degree/connection count).
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        for n in 0..5u32 {
            layers.insert(n, Layer::Edge);
        }
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0), (0, 4, 1.0)]);
        let c_hub = manager.compute_edge_congestion(0, 1);
        let c_leaf = manager.compute_edge_congestion(3, 4);
        // (0,1): deg(0)=4, deg(1)=1 → 5. (3,4): deg(3)=1, deg(4)=1 → 2.
        assert_eq!(c_hub, 5.0);
        // Leaf edge does not exist in this graph; query returns 0 + 0 = 2 only
        // because both endpoint degrees come from HashMap defaults. Just
        // assert hub > leaf regardless.
        assert!(
            c_hub > c_leaf,
            "Claim 6: congestion must rank by connection count"
        );
    }

    #[test]
    fn test_claim7_congestion_is_sum_of_endpoint_degrees() {
        // Claim 7: C = deg(u) + deg(v)
        let mut manager = DecayManager::master_spec();
        manager.degrees.insert(0, 7);
        manager.degrees.insert(1, 3);
        let c = manager.compute_edge_congestion(0, 1);
        assert_eq!(c, 10.0, "Claim 7: congestion must equal deg(u)+deg(v)");
    }

    #[test]
    fn test_claim8_lambda_monotonic_in_congestion() {
        // Claim 8: λ(C) is monotonically non-decreasing in C.
        let p = MasterSpecParams::default();
        let mut last = p.lambda(0.0, Layer::Edge);
        for c in 1..100 {
            let next = p.lambda(c as f64, Layer::Edge);
            assert!(
                next >= last - 1e-12,
                "Claim 8: λ must be monotone, broke at C={}",
                c
            );
            last = next;
        }
    }

    #[test]
    fn test_claim9_lambda_contains_power_term() {
        // Claim 9: λ(C) has a power term in C. With α_edge=1.5, doubling C
        // must scale the γC^α part by 2^1.5, not linearly.
        let p = MasterSpecParams::default();
        let c1 = 10.0;
        let c2 = 20.0;
        let lam1 = p.lambda(c1, Layer::Edge);
        let lam2 = p.lambda(c2, Layer::Edge);
        let power_term_1 = (lam1 / p.beta) - 1.0; // = γ C^α
        let power_term_2 = (lam2 / p.beta) - 1.0;
        let ratio = power_term_2 / power_term_1; // = 2^α
        let expected = 2f64.powf(p.alpha_edge);
        assert!(
            (ratio - expected).abs() < 1e-9,
            "Claim 9: power term exponent must be α (got ratio {}, expected {})",
            ratio,
            expected
        );
    }

    #[test]
    fn test_claim11_both_threshold_and_probabilistic_prune_available() {
        // Claim 11: pruning is done by threshold OR probabilistic — at least
        // one. We have both: apply_edge_decay drives weights toward 0
        // (threshold-style via denormal flush) and probabilistic_prune does
        // Bernoulli selection.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        // Threshold path: apply_edge_decay reduces weight deterministically.
        manager.apply_edge_decay();
        let w_after = manager.get_edge_weight(0, 1).unwrap();
        assert!(
            w_after < 1.0,
            "Claim 11 threshold path: weight must decrease"
        );
        // Probabilistic path: probabilistic_prune returns a decision list.
        let _ = manager.probabilistic_prune(|| 0.5);
    }

    #[test]
    fn test_claim13_exponential_weight_decay() {
        // Claim 13: w decays exponentially (dependent on Claim 3 relation param).
        let p = MasterSpecParams::default();
        let lam = p.lambda(5.0, Layer::Edge);
        let dt = p.dt_for_layer(Layer::Edge);
        let mut w = 1.0_f64;
        for _ in 0..10 {
            w *= (-lam * dt).exp();
        }
        let expected = (-lam * dt * 10.0).exp();
        assert!(
            (w - expected).abs() < 1e-12,
            "Claim 13: decay must be exp form"
        );
    }

    #[test]
    fn test_claim15_rare_isolated_node_preserved() {
        // Claim 15: isolated node that would be garbage-collected stays when
        // it is classified Rare.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Rare);
        layers.insert(1u32, Layer::Edge);
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        manager.degrees.insert(0, 1); // hi-decay scenario
        manager.degrees.insert(1, 100);
        let pruned = manager.probabilistic_prune(|| 0.0); // r=0 ⇒ always prune otherwise
        assert!(
            pruned.is_empty(),
            "Claim 15: edges touching Rare node must be protected from pruning"
        );
    }

    #[test]
    fn test_claim16_evaluation_uses_only_local_stats() {
        // Claim 16: evaluation must be computable from local statistics alone
        // (no full-graph scan). compute_edge_decay_probability only reads the
        // endpoint degrees and the per-layer parameters — no global reduction.
        let mut manager = DecayManager::master_spec();
        manager.degrees.insert(0, 3);
        manager.degrees.insert(1, 4);
        // Add an unrelated 10,000 nodes; the result for (0,1) must not change.
        for n in 100..10_100u32 {
            manager.degrees.insert(n, 50);
        }
        let p = manager.compute_edge_decay_probability(0, 1, Layer::Edge);
        // Compute the reference value using only the endpoint data.
        let lam = manager.master_params.lambda(7.0, Layer::Edge);
        let dt = manager.master_params.dt_for_layer(Layer::Edge);
        let expected = 1.0 - (-lam * dt).exp();
        assert!(
            (p - expected).abs() < 1e-12,
            "Claim 16: evaluation must depend only on local endpoint stats"
        );
    }

    #[test]
    fn test_claim18_protected_attribute_prevents_pruning() {
        // Claim 18: a node carrying the protection attribute must NOT be
        // subject to exclusion / archiving.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Rare); // Rare = protected (Layer::is_protected)
        layers.insert(1u32, Layer::Edge);
        assert!(layers[&0u32].is_protected());
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats: ClassificationStats::default(),
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0)]);
        assert!(
            manager.is_protected(0),
            "Claim 18: protected flag must be queryable"
        );
        manager.degrees.insert(0, 100);
        manager.degrees.insert(1, 100);
        let pruned = manager.probabilistic_prune(|| 0.0);
        assert!(
            pruned.is_empty(),
            "Claim 18: protected nodes escape pruning"
        );
    }

    #[test]
    fn test_claim19_processable_nodes_recorded_and_queryable() {
        // Claim 19: system must expose indicators that record processing
        // results. We expose `processable_nodes`, `processable_edges`, `stats`.
        use super::super::{ClassificationStats, NodeClassification};
        let mut manager = DecayManager::master_spec();
        let mut layers = HashMap::new();
        for n in 0..4u32 {
            layers.insert(n, Layer::Edge);
        }
        layers.insert(4u32, Layer::Garbage);
        let stats = ClassificationStats {
            core_count: 0,
            edge_count: 4,
            rare_count: 0,
            garbage_count: 1,
        };
        let class = NodeClassification {
            layers,
            rare_fingerprints: HashMap::new(),
            stats,
        };
        manager.initialize_with_edges(class, &[(0, 1, 1.0), (1, 2, 1.0)]);
        let nodes = manager.processable_nodes();
        let edges = manager.processable_edges();
        assert_eq!(nodes.len(), 4, "Claim 19: processable nodes recorded");
        assert!(!edges.is_empty(), "Claim 19: processable edges recorded");
        assert!(
            manager.stats().is_some(),
            "Claim 19: stats output available"
        );
    }

    #[test]
    fn test_exp_decay_analytic_solution() {
        // Claim 14: w(t+dt) = w(t)·exp(-λdt) ⇒ after N steps w = w0·exp(-N·λ·dt)
        let p = MasterSpecParams::default();
        let lambda = p.lambda(5.0, Layer::Edge);
        let dt = p.dt_for_layer(Layer::Edge);
        let mut w = 1.0_f64;
        let n = 1000;
        for _ in 0..n {
            w *= (-lambda * dt).exp();
        }
        let expected = (-lambda * dt * n as f64).exp();
        assert!(
            (w - expected).abs() < 1e-10,
            "discrete exp iteration must equal closed-form exp(-Nλdt)"
        );
    }
}
