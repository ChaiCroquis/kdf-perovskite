//! # bias-detector
//!
//! Pre-hoc detection of "synthetic dataset bias" — does a benchmark's
//! structural shape favor graph-based selection methods over baselines?
//!
//! Origin: KDF project Phase T (F-030). 5/5 datasets predicted correctly
//! on first test. Spun out here as a **standalone, zero-dependency** crate
//! so other projects (not just KDF) can use it for reproducibility hygiene.
//!
//! ## Typical usage
//!
//! ```
//! use bias_detector::BiasReport;
//!
//! let degrees: Vec<u32> = vec![1, 1, 2, 3, 1, 5, 1];
//! let rare_ids: &[u32] = &[0, 1, 4, 6]; // ground-truth rare nodes (if known)
//! let report = BiasReport::compute(&degrees, rare_ids);
//!
//! if report.bias_score > 0.5 {
//!     println!("⚠️ Benchmark is biased toward structure-based methods");
//! }
//! ```
//!
//! ## Indicators
//!
//! | Symbol | Meaning |
//! |---|---|
//! | I1 | Fraction of nodes with degree == 1 |
//! | I2 | Power-law fit deviation (lower = more realistic) |
//! | I3 | Rare-truth degree signal (signed deviation from mean) |
//! | I4 | Fraction of rare-truth at degree == 1 |
//! | bias_score | 0.3·I1 + 0.7·I4 (primary flag) |
//!
//! ## Thresholds
//!
//! - `bias_score > 0.5`: **HIGH bias** toward structure-exploiting methods
//! - `0.2 < bias_score ≤ 0.5`: **MODERATE bias**
//! - `bias_score ≤ 0.2`: **LOW bias** — benchmark is relatively realistic

use std::collections::HashSet;

/// Per-benchmark bias analysis.
#[derive(Debug, Clone)]
pub struct BiasReport {
    /// I1: fraction of non-zero-degree nodes with degree exactly 1
    pub deg1_ratio: f64,
    /// I2: max-normalized deviation from ideal power law (d_k ∝ 1/k)
    pub powerlaw_deviation: f64,
    /// I3: |rare_mean_deg / overall_mean_deg - 1|
    pub rare_deg_signal: f64,
    /// I4: fraction of rare-truth items with degree == 1
    pub rare_deg1_rate: f64,
    /// Composite flag: 0.3·I1 + 0.7·I4
    pub bias_score: f64,
    pub bias_level: BiasLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiasLevel {
    Low,
    Moderate,
    High,
}

impl BiasReport {
    /// Compute all indicators from node degrees + optional rare ground truth.
    ///
    /// If `rare_ids` is empty, I3/I4 default to 0.0 and bias_score is driven
    /// purely by I1 (structural deg==1 prevalence).
    pub fn compute(degrees: &[u32], rare_ids: &[u32]) -> Self {
        let deg1_ratio = indicator_deg1_ratio(degrees);
        let powerlaw_deviation = indicator_powerlaw_deviation(degrees);
        let rare_set: HashSet<u32> = rare_ids.iter().copied().collect();
        let rare_deg_signal = indicator_rare_deg_signal(degrees, &rare_set);
        let rare_deg1_rate = indicator_rare_deg1_rate(degrees, &rare_set);

        let bias_score = 0.3 * deg1_ratio + 0.7 * rare_deg1_rate;
        let bias_level = if bias_score > 0.5 {
            BiasLevel::High
        } else if bias_score > 0.2 {
            BiasLevel::Moderate
        } else {
            BiasLevel::Low
        };

        BiasReport {
            deg1_ratio,
            powerlaw_deviation,
            rare_deg_signal,
            rare_deg1_rate,
            bias_score,
            bias_level,
        }
    }

    /// Human-readable summary.
    pub fn summary(&self) -> String {
        let flag = match self.bias_level {
            BiasLevel::High => "⚠️ HIGH bias toward structure-based methods",
            BiasLevel::Moderate => "◐ Moderate bias",
            BiasLevel::Low => "✓ Low bias (realistic)",
        };
        format!(
            "bias_score = {:.3} [{}]\n  I1 (deg==1 fraction):  {:.3}\n  I2 (power-law dev):    {:.3}\n  I3 (rare-deg signal):  {:.3}\n  I4 (rare-at-deg==1):   {:.3}",
            self.bias_score,
            flag,
            self.deg1_ratio,
            self.powerlaw_deviation,
            self.rare_deg_signal,
            self.rare_deg1_rate,
        )
    }
}

// ============================================================================
// Indicator implementations
// ============================================================================

fn indicator_deg1_ratio(deg: &[u32]) -> f64 {
    let nonzero = deg.iter().filter(|&&d| d > 0).count();
    if nonzero == 0 {
        return 0.0;
    }
    let d1 = deg.iter().filter(|&&d| d == 1).count();
    d1 as f64 / nonzero as f64
}

fn indicator_powerlaw_deviation(deg: &[u32]) -> f64 {
    let mut d_sorted: Vec<u32> = deg.iter().filter(|&&x| x > 0).copied().collect();
    d_sorted.sort_unstable_by(|a, b| b.cmp(a));
    if d_sorted.len() < 10 {
        return 1.0;
    }
    let d_max = d_sorted[0] as f64;
    let n_eval = d_sorted.len().min(100);
    let ks: f64 = d_sorted
        .iter()
        .enumerate()
        .take(n_eval)
        .map(|(k, &d)| {
            let expected = d_max / (k as f64 + 1.0);
            ((d as f64 - expected) / expected).abs()
        })
        .sum::<f64>()
        / n_eval as f64;
    ks
}

fn indicator_rare_deg_signal(deg: &[u32], rare: &HashSet<u32>) -> f64 {
    if rare.is_empty() {
        return 0.0;
    }
    let rare_mean: f64 = rare
        .iter()
        .filter(|&&id| (id as usize) < deg.len())
        .map(|&id| deg[id as usize] as f64)
        .sum::<f64>()
        / rare.len() as f64;
    let overall_mean: f64 = deg.iter().map(|&d| d as f64).sum::<f64>() / deg.len().max(1) as f64;
    if overall_mean == 0.0 {
        0.0
    } else {
        (rare_mean / overall_mean - 1.0).abs()
    }
}

fn indicator_rare_deg1_rate(deg: &[u32], rare: &HashSet<u32>) -> f64 {
    if rare.is_empty() {
        return 0.0;
    }
    let d1 = rare
        .iter()
        .filter(|&&id| (id as usize) < deg.len() && deg[id as usize] == 1)
        .count();
    d1 as f64 / rare.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highly_biased_case_flags_as_high() {
        // Star graph: 1 hub + 100 deg==1 leaves, all leaves marked as rare
        let mut degrees = vec![100u32];
        degrees.extend(std::iter::repeat_n(1u32, 100));
        let rare: Vec<u32> = (1..=100u32).collect();
        let r = BiasReport::compute(&degrees, &rare);
        assert_eq!(r.bias_level, BiasLevel::High);
        assert!(
            r.bias_score > 0.9,
            "star-graph bias_score should be very high, got {}",
            r.bias_score
        );
        assert!(r.deg1_ratio > 0.99);
        assert!(r.rare_deg1_rate > 0.99);
    }

    #[test]
    fn realistic_case_flags_as_low() {
        // Heavy-tailed non-deg1 distribution, rare items at moderate degree
        let degrees: Vec<u32> = (0..1000)
            .map(|i| {
                // Zipf-like: degrees 2..50 weighted
                2 + (i as u32 * 47 / 1000)
            })
            .collect();
        let rare: Vec<u32> = (100..150u32).collect(); // mid-degree nodes
        let r = BiasReport::compute(&degrees, &rare);
        assert_eq!(r.bias_level, BiasLevel::Low);
        assert!(
            r.bias_score < 0.2,
            "realistic bias_score should be low, got {}",
            r.bias_score
        );
    }

    #[test]
    fn empty_rare_defaults_to_i1_only() {
        let degrees: Vec<u32> = vec![1, 1, 1, 5, 5, 5];
        let r = BiasReport::compute(&degrees, &[]);
        // I1 = 3/6 = 0.5, I4 = 0; bias_score = 0.3 * 0.5 = 0.15
        assert!((r.bias_score - 0.15).abs() < 0.01);
        assert_eq!(r.bias_level, BiasLevel::Low);
    }

    #[test]
    fn moderate_case() {
        // 30% deg==1, 30% of rare at deg==1
        let mut degrees = vec![1u32; 30];
        degrees.extend(std::iter::repeat_n(3u32, 70));
        let rare: Vec<u32> = (0..10u32).collect();
        let r = BiasReport::compute(&degrees, &rare);
        // I1 = 0.3, I4 = 1.0 (all rare are in first 10 which are deg=1)
        // bias_score = 0.3*0.3 + 0.7*1.0 = 0.79 → High
        assert_eq!(r.bias_level, BiasLevel::High);
    }

    #[test]
    fn summary_contains_level_emoji() {
        let degrees = vec![1u32; 100];
        let rare: Vec<u32> = (0..50).collect();
        let r = BiasReport::compute(&degrees, &rare);
        let s = r.summary();
        assert!(s.contains("bias_score"));
        assert!(s.contains("HIGH") || s.contains("Moderate") || s.contains("Low"));
    }
}
