//! Causal Discovery and Transfer Entropy Module
//!
//! Transfer entropy-based causal discovery integrated with KDF
//! for intelligent edge pruning and knowledge consolidation.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    CausalKDF_V3                              │
//! ├─────────────────────────────────────────────────────────────┤
//! │  Awake Mode:     Screening (Gaussian TE) - Edge Layer       │
//! │  Sleep Mode:     DeepProbe (Symbolic) + Validation (KSG)    │
//! │  Integration:    Analogy-guided candidate generation        │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Reference
//!
//! Python implementations:
//! - python/kdf/causal_kdf.py
//! - python/kdf/causal_partition.py

// Module declarations
pub mod engine;
pub mod estimators;
pub mod kdf_v3;
pub mod nrem;
pub mod partition;
pub mod types;

#[cfg(test)]
mod tests;

// Re-export public API
pub use engine::{BatchStats, CausalEngine};
pub use estimators::{GaussianEstimator, KsgEstimator, SymbolicEstimator};
pub use kdf_v3::{CausalKdfStats, CausalKdfV3, SleepCycleResult};
pub use nrem::{CausalEnhancedNREMOptimizer, CausalNREMResult, CausalNREMStats};
pub use partition::{CausalCluster, CausalPartitionBuilder};
pub use types::{CausalLink, TeResult, TeStrategy};
