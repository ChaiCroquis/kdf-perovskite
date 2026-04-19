//! Shared utilities for KDF showcase demos.
//!
//! Every demo produces:
//! - A `ThreeAxisReport` summarising "KDF wins / ties / loses" vs baselines
//! - A JSON dump for the Python visualizer
//! - A Markdown report rendered from a common template

pub mod report;
pub mod visualizer;

use serde::Serialize;
use std::collections::BTreeMap;

/// A single metric value with uncertainty info.
#[derive(Serialize, Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub higher_is_better: bool,
    pub mean: f64,
    pub stderr: f64,
    /// Which of the three axes does this metric belong to?
    pub axis: Axis,
}

#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Axis {
    /// Where KDF claims to outperform baselines (its unique-mechanism niche).
    KdfStrength,
    /// Where KDF should be roughly equal to baselines (no regression).
    Tie,
    /// Where KDF expectedly trades off; transparency axis.
    KdfWeakness,
}

/// A run of one method (KDF or a baseline) on one dataset, with its metric values.
#[derive(Serialize, Debug, Clone)]
pub struct MethodResult {
    pub method: String,
    pub requires_labels: bool,
    pub metrics: BTreeMap<String, f64>,
    pub wall_ms: f64,
    pub notes: String,
}

/// A demo's full report.
#[derive(Serialize, Debug, Clone)]
pub struct DemoReport {
    pub demo_id: String,
    pub title: String,
    pub dataset_name: String,
    pub n_items: usize,
    pub patent_section: String,
    pub metric_definitions: Vec<Metric>,
    pub method_results: Vec<MethodResult>,
    /// Per-"method/metric" raw trial values for Wilcoxon tests downstream.
    /// Key format: "METHOD/METRIC" (slash-separated). JSON-safe (no tuple keys).
    pub raw_trials: BTreeMap<String, Vec<f64>>,
    pub conclusion: Conclusion,
}

#[derive(Serialize, Debug, Clone)]
pub struct Conclusion {
    pub kdf_recommended_for: Vec<String>,
    pub kdf_not_recommended_for: Vec<String>,
    pub honest_limits: Vec<String>,
}

impl DemoReport {
    /// Serialize to JSON file (used as input for the Python visualizer).
    pub fn write_json(&self, path: &std::path::Path) -> std::io::Result<()> {
        let j = serde_json::to_string_pretty(self).map_err(std::io::Error::other)?;
        std::fs::write(path, j)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_roundtrip() {
        let m = Metric {
            name: "rare_recall".to_string(),
            higher_is_better: true,
            mean: 0.863,
            stderr: 0.0,
            axis: Axis::KdfStrength,
        };
        let j = serde_json::to_string(&m).unwrap();
        assert!(j.contains("KdfStrength"));
    }
}
