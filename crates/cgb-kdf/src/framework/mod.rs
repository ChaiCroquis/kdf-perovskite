//! KDF Framework - Unified Entry Point
//!
//! Knowledge Decay Framework as a cohesive system:
//! - Node classification (CORE/EDGE/RARE/GARBAGE)
//! - Decay management (skip GARBAGE, protect RARE)
//! - Layer-based processing strategies
//!
//! # KDF Concept
//!
//! - **CORE**: Central, stable knowledge - full processing

#![allow(missing_docs)]
//! - **EDGE**: Active, frequently accessed - fast processing
//! - **RARE**: Isolated truths - protect with fingerprint
//! - **GARBAGE**: Noise/artifacts - skip processing
//!
//! # KDF Rev.12 Specification
//!
//! Rev.12 introduces the analogy discovery mechanism for RARE nodes:
//! - **Two-phase review**: T_wait1 (Phase 1), T_wait2 (Phase 2) waiting periods
//! - **spoke_up flag**: Tracks if RARE node found structural analogy (θ >= 0.75)
//! - **If spoke_up**: RARE node integrates with analogy target (becomes CORE candidate)
//! - **If not spoke_up after T_wait2**: Node demotes to GARBAGE

use std::collections::HashMap;

mod classifier;
pub mod classifier_fast;
mod decay;
pub mod invariants;
mod meta_control;
pub mod multimodal;
mod processor;
mod region;
pub mod rev12;
mod transition;

#[cfg(test)]
mod tests;

// Re-exports
pub use classifier::NodeClassifier;
pub use classifier_fast::FastNodeClassifier;
pub use decay::{DecayManager, MasterSpecParams};
pub use meta_control::MetaController;
pub use processor::KdfProcessor;
pub use region::{HierarchicalRegionManager, RegionConfig, RegionKind};
pub use rev12::{
    DISCOVERY_THRESHOLD_DEFAULT, DISCOVERY_THRESHOLD_UPPER_DEFAULT, KdfProcessorRev12,
    RareNodeState, Rev12Error, Rev12Stats, ReviewPhase, SHORTLIST_TOP_K_DEFAULT, T_WAIT_DEFAULT,
    T_WAIT_MAX, T_WAIT_MIN,
};
pub use transition::{ActivationScore, SemanticImportance, TransitionController, TransitionScore};

/// KDF Layer classification
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Layer {
    /// Central nodes with high connectivity - full processing
    Core,
    /// Boundary nodes with moderate connectivity - standard processing
    Edge,
    /// Isolated but important nodes - protect with fingerprint
    Rare,
    /// Noise/artifacts - skip processing
    Garbage,
}

impl Layer {
    /// Should this layer be processed in optimization?
    pub fn should_process(&self) -> bool {
        match self {
            Layer::Core | Layer::Edge | Layer::Rare => true,
            Layer::Garbage => false,
        }
    }

    /// Should this layer be protected from modification?
    pub fn is_protected(&self) -> bool {
        matches!(self, Layer::Rare)
    }

    /// Processing priority (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            Layer::Core => 3,
            Layer::Edge => 2,
            Layer::Rare => 1,
            Layer::Garbage => 0,
        }
    }
}

/// Node classification result
#[derive(Clone, Debug)]
pub struct NodeClassification {
    /// Layer assignment for each node
    pub layers: HashMap<u32, Layer>,
    /// Fingerprints for RARE layer nodes (for preservation)
    pub rare_fingerprints: HashMap<u32, super::fingerprint::Fingerprint>,
    /// Statistics
    pub stats: ClassificationStats,
}

/// Classification statistics
#[derive(Clone, Debug, Default)]
pub struct ClassificationStats {
    pub core_count: usize,
    pub edge_count: usize,
    pub rare_count: usize,
    pub garbage_count: usize,
}

impl ClassificationStats {
    /// Total nodes classified
    pub fn total(&self) -> usize {
        self.core_count + self.edge_count + self.rare_count + self.garbage_count
    }

    /// Percentage that will be processed (non-GARBAGE)
    pub fn processing_rate(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 1.0;
        }
        (total - self.garbage_count) as f64 / total as f64
    }

    /// Percentage skipped (GARBAGE)
    pub fn skip_rate(&self) -> f64 {
        1.0 - self.processing_rate()
    }
}
