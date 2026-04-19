//! Structural Fingerprint Engine
//!
//! Graph Laplacian Eigenvalue Fingerprint Generator.
//! Distinguishes Truth nodes from Garbage nodes structurally.

#![allow(missing_docs)]

// Module declarations
mod types;
mod precomputed;
mod rng;
mod engine;

// Re-exports
pub use types::{Fingerprint, FingerprintKey, NodeLabel, CacheStats};
pub use precomputed::PrecomputedFingerprint;
pub use engine::StructuralFingerprintEngine;

// Tests
#[cfg(test)]
mod tests;
