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
pub mod entropy;
pub mod matrix;
pub mod monitor;
pub mod triggered;
pub mod types;

#[cfg(test)]
mod tests;

// Re-exports for convenience
pub use entropy::{detect_change, von_neumann_entropy, von_neumann_entropy_detailed};
pub use matrix::{density_matrix, laplacian_matrix};
pub use monitor::VNEMonitor;
pub use triggered::{OptimizationResult, VNETriggeredSleepMode, VNETriggeredStats};
pub use types::{AnomalyResult, ChangeDetection, VNEResult};
