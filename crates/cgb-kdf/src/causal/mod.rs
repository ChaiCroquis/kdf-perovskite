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
pub mod types;
pub mod estimators;
pub mod engine;
pub mod kdf_v3;
pub mod partition;
pub mod nrem;

#[cfg(test)]
mod tests;

// Re-export public API
pub use types::{TeStrategy, TeResult, CausalLink};
pub use estimators::{GaussianEstimator, SymbolicEstimator, KsgEstimator};
pub use engine::{CausalEngine, BatchStats};
pub use kdf_v3::{CausalKdfV3, CausalKdfStats, SleepCycleResult};
pub use partition::{CausalPartitionBuilder, CausalCluster};
pub use nrem::{CausalEnhancedNREMOptimizer, CausalNREMStats, CausalNREMResult};
