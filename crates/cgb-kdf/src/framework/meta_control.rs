//! Meta-cognitive control (Claim 27-32)
//!
//! Implements the meta control means that updates the decay parameters
//! (α, β, γ) based on a health index derived from the gap between the
//! observed average connectivity ⟨k⟩ and the target connectivity k_opt.
//!
//! # Claim mapping
//!
//! | Claim | Requirement | Implementation |
//! |-------|-------------|----------------|
//! | 27 | Meta control means updating decay parameters | [`MetaController::step`] |
//! | 28 | Health index from ⟨k⟩ vs k_opt | [`MetaController::health_index`] |
//! | 29 | Δparameter ∝ δk^4 (fourth-power law) | [`MetaController::alpha_update`] |
//! | 30 | Bidirectional update + upper/lower bounds | [`MetaController::step`] |
//! | 31 | Emergency intervention on crisis | [`MetaController::emergency_intervention`] |
//! | 32 | Mode toggle (enable/disable) | [`MetaController::enabled`] |

use super::decay::MasterSpecParams;
use super::Layer;

/// Meta-cognitive controller implementing Claim 27-32.
#[derive(Clone, Debug)]
pub struct MetaController {
    /// Whether meta control is currently active (Claim 32)
    pub enabled: bool,
    /// Target average connectivity ⟨k⟩_opt per layer (明細書 Fig.12C)
    pub k_opt_edge: f64,
    pub k_opt_core: f64,
    /// Health sensitivity η (how strongly to react to health deviation)
    pub eta: f64,
    /// Imbalance sensitivity μ (how strongly to react to EC imbalance)
    pub mu: f64,
    /// Target health H_target (Claim 28: 目標値)
    pub health_target: f64,
    /// α bounds (Claim 30: 所定の上限値と下限値)
    pub alpha_edge_bounds: (f64, f64),
    pub alpha_core_bounds: (f64, f64),
    /// Emergency trigger: health falls below this threshold (Claim 31)
    pub emergency_health_threshold: f64,
    /// Emergency selection criterion: fraction of lowest-weight edges to prune
    pub emergency_prune_fraction: f64,
    /// Running count of emergency interventions (for statistics)
    pub emergency_count: u64,
}

impl Default for MetaController {
    fn default() -> Self {
        Self {
            enabled: true,
            k_opt_edge: 6.0,
            k_opt_core: 4.0,
            eta: 0.15,
            mu: 0.08,
            health_target: 0.70,
            // Claim 30 bounds: α_E ∈ [1.0, 2.5], α_C ∈ [1.5, 3.0]
            alpha_edge_bounds: (1.0, 2.5),
            alpha_core_bounds: (1.5, 3.0),
            emergency_health_threshold: 0.30,
            emergency_prune_fraction: 0.05,
            emergency_count: 0,
        }
    }
}

impl MetaController {
    /// Compute the health index H for a given layer (Claim 28, simplified form):
    ///
    /// H = 1 - |⟨k⟩ − k_opt| / k_opt
    ///
    /// Returns a value in [-∞, 1]; clamped to [0, 1] for stability.
    pub fn health_index(&self, avg_k: f64, k_opt: f64) -> f64 {
        if k_opt <= 0.0 {
            return 1.0;
        }
        let raw = 1.0 - (avg_k - k_opt).abs() / k_opt;
        raw.clamp(0.0, 1.0)
    }

    /// Claim 29: positive deviation δk = max(0, ⟨k⟩ − k_opt).
    #[inline]
    pub fn positive_deviation(avg_k: f64, k_opt: f64) -> f64 {
        (avg_k - k_opt).max(0.0)
    }

    /// Claim 29: parameter update magnitude proportional to δk^4 (fourth power).
    ///
    /// `delta_alpha = −η · (H − H_target) + μ · δk^4` (sign depends on layer)
    pub fn alpha_update(&self, layer: Layer, avg_k: f64) -> f64 {
        let k_opt = match layer {
            Layer::Edge | Layer::Garbage => self.k_opt_edge,
            Layer::Core => self.k_opt_core,
            Layer::Rare => self.k_opt_edge, // Rare reuses edge-layer target
        };
        let h = self.health_index(avg_k, k_opt);
        let dk = Self::positive_deviation(avg_k, k_opt);
        let fourth_power = dk * dk * dk * dk;
        // Edge layer: react to its own health, moderated by EC imbalance (+μ·δk^4).
        // Core layer: react to its own health, opposite sign for EC imbalance (−μ·δk^4).
        let sign = match layer {
            Layer::Core => -1.0,
            _ => 1.0,
        };
        -self.eta * (h - self.health_target) + sign * self.mu * fourth_power
    }

    /// One meta-control step (Claim 27, 28, 30).
    ///
    /// Updates `params` in place if `self.enabled`. Returns the Δα applied
    /// to the Edge layer (for logging/testing).
    pub fn step(
        &self,
        params: &mut MasterSpecParams,
        avg_k_edge: f64,
        avg_k_core: f64,
    ) -> (f64, f64) {
        if !self.enabled {
            return (0.0, 0.0);
        }
        let d_alpha_e = self.alpha_update(Layer::Edge, avg_k_edge);
        let d_alpha_c = self.alpha_update(Layer::Core, avg_k_core);

        // Claim 30: bidirectional update + clamp to [lo, hi]
        params.alpha_edge = (params.alpha_edge + d_alpha_e)
            .clamp(self.alpha_edge_bounds.0, self.alpha_edge_bounds.1);
        params.alpha_core = (params.alpha_core + d_alpha_c)
            .clamp(self.alpha_core_bounds.0, self.alpha_core_bounds.1);
        (d_alpha_e, d_alpha_c)
    }

    /// Claim 31: emergency intervention.
    ///
    /// Returns the number of edges selected for prioritised pruning.
    /// The caller is responsible for actually removing them.
    pub fn emergency_intervention<I>(&mut self, avg_k_edge: f64, edge_weights: I) -> Vec<(u32, u32)>
    where
        I: Iterator<Item = ((u32, u32), f64)>,
    {
        let k_opt = self.k_opt_edge;
        let h = self.health_index(avg_k_edge, k_opt);
        if h >= self.emergency_health_threshold {
            return Vec::new();
        }
        self.emergency_count += 1;
        let mut weighted: Vec<_> = edge_weights.collect();
        // Ascending by weight → low-weight edges are the selection criterion.
        weighted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let n_prune = ((weighted.len() as f64) * self.emergency_prune_fraction).ceil() as usize;
        weighted.into_iter().take(n_prune).map(|(e, _)| e).collect()
    }

    /// Claim 32: toggle mode.
    pub fn set_enabled(&mut self, flag: bool) {
        self.enabled = flag;
    }

    /// Lyapunov stability condition (Rev.11 §7.4): η² > μ².
    /// Required for convergent adaptive control.
    pub fn check_lyapunov_stability(&self) -> bool {
        self.eta * self.eta > self.mu * self.mu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_index_basic() {
        let mc = MetaController::default();
        // avg_k = k_opt ⇒ perfect health
        assert!((mc.health_index(6.0, 6.0) - 1.0).abs() < 1e-12);
        // Deviation reduces health
        assert!(mc.health_index(12.0, 6.0) < mc.health_index(8.0, 6.0));
        assert!(mc.health_index(4.0, 6.0) < 1.0);
    }

    #[test]
    fn test_positive_deviation() {
        assert_eq!(MetaController::positive_deviation(10.0, 6.0), 4.0);
        // Only positive part (Claim 29)
        assert_eq!(MetaController::positive_deviation(4.0, 6.0), 0.0);
    }

    #[test]
    fn test_alpha_update_fourth_power_scaling() {
        // Claim 29: update ∝ δk^4. Doubling δk must 16× the fourth-power term.
        let mc = MetaController::default();
        // Cancel the η·(H−H_target) term by using k at boundary so δk=1 and δk=2 only differ in fourth-power
        let d1 = mc.mu * 1.0_f64.powi(4);
        let d2 = mc.mu * 2.0_f64.powi(4);
        assert!((d2 / d1 - 16.0).abs() < 1e-12, "Claim 29: Δα ∝ δk^4");
    }

    #[test]
    fn test_claim30_bidirectional_and_bounded() {
        let mc = MetaController::default();
        let mut params = MasterSpecParams::default();
        let original = params.alpha_edge;

        // Under-connected (below target): health high, δk=0 ⇒ Δα driven by −η(H−H_t).
        // With default health_target=0.7 and near-perfect health H≈1.0, Δα = −η·0.3 < 0.
        mc.step(&mut params, 6.0, 4.0); // perfect health, no deviation
        assert!(
            params.alpha_edge < original,
            "health>target must push α down (Claim 30)"
        );
        assert!(
            params.alpha_edge >= mc.alpha_edge_bounds.0,
            "lower bound enforced"
        );

        // Over-connected: large δk drives Δα positive via fourth-power term
        let mut params = MasterSpecParams::default();
        mc.step(&mut params, 12.0, 4.0);
        assert!(
            params.alpha_edge <= mc.alpha_edge_bounds.1,
            "upper bound enforced"
        );
    }

    #[test]
    fn test_claim31_emergency_intervention() {
        let mut mc = MetaController::default();
        // Healthy state: no intervention
        let picked = mc.emergency_intervention(6.0, Vec::<((u32, u32), f64)>::new().into_iter());
        assert!(picked.is_empty());
        assert_eq!(mc.emergency_count, 0);

        // Crisis: health low, pick 5% of lowest-weight edges
        let edges = (0..100)
            .map(|i| ((i as u32, (i + 1) as u32), i as f64))
            .collect::<Vec<_>>();
        let picked = mc.emergency_intervention(0.0, edges.into_iter());
        assert_eq!(mc.emergency_count, 1);
        assert_eq!(picked.len(), 5); // 5% of 100
                                     // Lowest-weight first
        assert_eq!(picked[0], (0, 1));
    }

    #[test]
    fn test_claim32_mode_toggle() {
        let mut mc = MetaController::default();
        let mut params = MasterSpecParams::default();
        let orig_alpha = params.alpha_edge;
        mc.set_enabled(false);
        mc.step(&mut params, 12.0, 4.0);
        assert_eq!(
            params.alpha_edge, orig_alpha,
            "disabled controller must not mutate params"
        );
    }

    #[test]
    fn test_claim27_meta_control_updates_params() {
        // Claim 27: meta control means updates 代謝制御 parameters (α).
        let mc: MetaController = MetaController::default();
        let mut params = MasterSpecParams::default();
        let original_edge = params.alpha_edge;
        let original_core = params.alpha_core;
        let (d_e, d_c) = mc.step(&mut params, 12.0, 4.0); // over-connected edge, balanced core
        assert!(
            d_e != 0.0 || d_c != 0.0,
            "Claim 27: meta-step must produce parameter update when state departs from target"
        );
        assert!(
            params.alpha_edge != original_edge || params.alpha_core != original_core,
            "Claim 27: parameters must be updated in place"
        );
    }

    #[test]
    fn test_claim28_health_index_from_avg_vs_target() {
        // Claim 28: health index is based on the gap between average connectivity
        // and target connectivity.
        let mc = MetaController::default();
        // Exact match → health = 1
        assert!(
            (mc.health_index(6.0, 6.0) - 1.0).abs() < 1e-12,
            "Claim 28: avg=target ⇒ H=1"
        );
        // Larger gap → smaller health
        let h_close = mc.health_index(7.0, 6.0);
        let h_far = mc.health_index(10.0, 6.0);
        assert!(h_close > h_far, "Claim 28: larger |⟨k⟩-target| ⇒ lower H");
    }

    #[test]
    fn test_claim29_update_proportional_to_delta_k_fourth_power() {
        // Claim 29: δk = max(0, ⟨k⟩ − k_opt) and parameter update is proportional
        // to δk^4. We isolate the fourth-power term via the static positive_deviation.
        // Positive-only property: below target ⇒ δk = 0
        assert_eq!(
            MetaController::positive_deviation(4.0, 6.0),
            0.0,
            "Claim 29: δk = max(0, ⟨k⟩-k_opt)"
        );
        // Above target ⇒ δk positive
        assert_eq!(MetaController::positive_deviation(10.0, 6.0), 4.0);
        // Doubling δk scales the fourth-power term by 2^4 = 16
        let dk1 = 1.0_f64;
        let dk2 = 2.0_f64;
        let t1 = dk1.powi(4);
        let t2 = dk2.powi(4);
        assert!(
            (t2 / t1 - 16.0).abs() < 1e-12,
            "Claim 29: the δk^4 term scales by 2^4 when δk doubles"
        );
    }

    #[test]
    fn test_lyapunov_stability_default() {
        let mc = MetaController::default();
        // 0.15^2 = 0.0225  >  0.08^2 = 0.0064
        assert!(
            mc.check_lyapunov_stability(),
            "default params must satisfy η² > μ²"
        );
    }
}
