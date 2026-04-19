//! Von Neumann Entropy (VNE) Integration Module
//!
//! VNE-based graph complexity measurement and anomaly detection.
//! Provides integration triggers for Sleep Mode (structural entropy minimization).
//!
//! # Features
//!
//! - VNE calculation from Laplacian matrix eigenvalues
//! - Anomaly detection from VNE time series changes
//! - Sleep Mode trigger on VNE anomaly
//!
//! # Reference
//!
//! Python implementation: python/kdf/vne_integration.py

// Submodules
pub mod types;
pub mod matrix;
pub mod entropy;
pub mod monitor;
pub mod triggered;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use types::{VNEResult, AnomalyResult, ChangeDetection};
pub use matrix::{laplacian_matrix, density_matrix};
pub use entropy::{von_neumann_entropy, von_neumann_entropy_detailed, detect_change};
pub use monitor::VNEMonitor;
pub use triggered::{VNETriggeredSleepMode, VNETriggeredStats, OptimizationResult};
