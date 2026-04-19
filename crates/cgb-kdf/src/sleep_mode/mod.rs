//! Sleep Mode Optimizer
//!
//! Implements NREM/REM phases for knowledge consolidation using
//! structural entropy minimization and simulated annealing.
//!
//! # Design: Boundary Conversion Pattern
//!
//! External API uses `String` for user convenience.
//! Internal computation uses `u32` IDs for performance:
//! - O(1) comparison instead of O(n) string comparison
//! - Direct indexing instead of HashMap lookup
//! - No string cloning during optimization loop

use std::collections::HashMap;

mod context;
mod cooling;
mod entropy_cache;
mod optimizer;
mod rng;

#[cfg(test)]
mod tests;

// Re-export public types
pub use context::{ModuleStats, NodeMoveContext};
pub use cooling::AdaptiveCoolingScheduler;
pub use entropy_cache::IncrementalEntropyCache;
pub use optimizer::{NREMResult, SleepModeOptimizer};

/// Compute structural entropy directly (for verification)
pub fn compute_structural_entropy(
    edges: &[(String, String, f64)],
    partition: &HashMap<String, u32>,
) -> f64 {
    let mut cache = IncrementalEntropyCache::new(u64::MAX);
    cache.initialize_from_edges(edges, partition)
}
