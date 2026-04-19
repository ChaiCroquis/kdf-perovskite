//! Phase V — First-class multi-modal KDF.
//!
//! Phase 8 demos (D6, D8) showed KDF graph-only fails where text-based
//! signals succeed. Rather than ad-hoc union/hybrid in each demo, this
//! module offers a canonical multi-modal scorer:
//!
//!   FinalScore(n) = α · graph_score(n) + β · text_score(n) + γ · temporal_score(n)
//!
//! with sensible defaults. This becomes the recommended entry point for
//! domains where rareness is partially text-defined (LLM memory, forum
//! dedup, customer support knowledge bases).
//!
//! Patent relevance: Claim 33 ("孤立度指標は、強度、頻度、**接続量、
//! またはこれらの時間的推移の少なくとも一つ** に基づく") allows
//! composite indicators. text/temporal scores fit within this.

use std::collections::HashMap;
use super::Layer;

/// Multi-modal scoring weights.
///
/// Defaults:
///   alpha (graph) = 0.5 — KDF's graph-only score contribution
///   beta  (text)  = 0.3 — text-signal contribution (shingle-based rareness)
///   gamma (time)  = 0.2 — recency / activation contribution
///
/// Callers override via `MultiModalWeights::{graph_heavy, text_heavy, balanced}`.
#[derive(Clone, Debug)]
pub struct MultiModalWeights {
    pub alpha: f64,
    pub beta: f64,
    pub gamma: f64,
}

impl Default for MultiModalWeights {
    fn default() -> Self {
        Self { alpha: 0.5, beta: 0.3, gamma: 0.2 }
    }
}

impl MultiModalWeights {
    pub fn graph_heavy() -> Self { Self { alpha: 0.8, beta: 0.1, gamma: 0.1 } }
    pub fn text_heavy() -> Self { Self { alpha: 0.2, beta: 0.6, gamma: 0.2 } }
    pub fn balanced() -> Self { Self::default() }
    pub fn graph_only() -> Self { Self { alpha: 1.0, beta: 0.0, gamma: 0.0 } }
}

/// Score a node using graph-layer score + optional text/temporal signals.
///
/// This is the canonical "Full KDF" score. Callers provide the per-node
/// layer map (from classifier), shingle-based text rareness, and recency.
pub fn score_multi_modal(
    n_nodes: usize,
    layer_of: &HashMap<u32, Layer>,
    text_rareness: Option<&[f64]>,    // per-node in [0, 1]; None = skip
    temporal_score: Option<&[f64]>,   // per-node in [0, 1]; None = skip
    weights: &MultiModalWeights,
) -> Vec<(u32, f64)> {
    (0..n_nodes as u32)
        .map(|id| {
            let layer_s = match layer_of.get(&id).copied().unwrap_or(Layer::Edge) {
                Layer::Rare => 1.0,
                Layer::Core => 0.67,
                Layer::Edge => 0.33,
                Layer::Garbage => 0.0,
            };
            let text_s = text_rareness
                .and_then(|arr| arr.get(id as usize).copied())
                .unwrap_or(0.0);
            let temp_s = temporal_score
                .and_then(|arr| arr.get(id as usize).copied())
                .unwrap_or(0.0);
            let score = weights.alpha * layer_s
                      + weights.beta * text_s
                      + weights.gamma * temp_s;
            (id, score)
        })
        .collect()
}

/// Select top-K items by multi-modal score.
pub fn select_top_k_multi_modal(
    n_nodes: usize,
    layer_of: &HashMap<u32, Layer>,
    text_rareness: Option<&[f64]>,
    temporal_score: Option<&[f64]>,
    keep: usize,
    weights: &MultiModalWeights,
) -> std::collections::HashSet<u32> {
    let mut scored = score_multi_modal(n_nodes, layer_of, text_rareness, temporal_score, weights);
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_only_recovers_layer_score() {
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Rare);
        layers.insert(1u32, Layer::Edge);
        layers.insert(2u32, Layer::Core);
        layers.insert(3u32, Layer::Garbage);
        let w = MultiModalWeights::graph_only();
        let s = score_multi_modal(4, &layers, None, None, &w);
        // Order: Rare > Core > Edge > Garbage
        assert!(s[0].1 > s[2].1);
        assert!(s[2].1 > s[1].1);
        assert!(s[1].1 > s[3].1);
    }

    #[test]
    fn text_signal_boosts_score() {
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge);
        layers.insert(1u32, Layer::Edge);
        let text_rare = vec![0.0, 1.0]; // node 1 has unique text
        let w = MultiModalWeights::text_heavy();
        let s = score_multi_modal(2, &layers, Some(&text_rare), None, &w);
        assert!(s[1].1 > s[0].1, "text-heavy weight should rank unique-text higher");
    }

    #[test]
    fn weights_affect_selection() {
        let mut layers = HashMap::new();
        layers.insert(0u32, Layer::Edge); // graph=low, text=high
        layers.insert(1u32, Layer::Rare); // graph=high, text=low
        let text = vec![1.0, 0.0];
        let keep = 1;
        let graph_pick = select_top_k_multi_modal(2, &layers, Some(&text), None, keep,
            &MultiModalWeights::graph_heavy());
        let text_pick = select_top_k_multi_modal(2, &layers, Some(&text), None, keep,
            &MultiModalWeights::text_heavy());
        assert!(graph_pick.contains(&1), "graph-heavy picks Rare node");
        assert!(text_pick.contains(&0), "text-heavy picks high-text-rare node");
    }
}
