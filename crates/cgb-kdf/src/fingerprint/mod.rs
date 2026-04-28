//! Structural Fingerprint Engine
//!
//! Graph Laplacian Eigenvalue Fingerprint Generator.
//! Distinguishes Truth nodes from Garbage nodes structurally.

#![allow(missing_docs)]

// Module declarations
mod engine;
mod precomputed;
pub(crate) mod rng;
mod types;

// Re-exports
pub use engine::StructuralFingerprintEngine;
pub use precomputed::PrecomputedFingerprint;
pub use types::{CacheStats, Fingerprint, FingerprintKey, NodeLabel};

// Tests
#[cfg(test)]
mod tests;
