//! Wilcoxon signed-rank test (pure Rust, no dependencies).
//!
//! Tests the null hypothesis that the pairwise differences
//! `x_i - y_i` come from a distribution with zero median.
//! Returns a two-sided p-value using the normal approximation
//! with continuity correction, which is accurate for n >= 10.

use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
pub struct WilcoxonResult {
    pub n_effective: usize,
    pub w_plus: f64,
    pub w_minus: f64,
    pub z: f64,
    pub p_value_two_sided: f64,
    pub significant_at_01: bool,
    pub median_diff: f64,
}

/// Compute Wilcoxon signed-rank test for paired samples x vs y.
pub fn wilcoxon_signed_rank(x: &[f64], y: &[f64]) -> Option<WilcoxonResult> {
    assert_eq!(x.len(), y.len(), "paired samples must have equal length");
    if x.is_empty() { return None; }

    // Compute differences, drop zeros (tied with zero → excluded)
    let mut diffs: Vec<(f64, f64)> = x.iter().zip(y.iter())
        .map(|(a, b)| a - b)
        .filter(|d| d.abs() > 1e-12)
        .map(|d| (d.abs(), d.signum()))
        .collect();
    let n = diffs.len();
    if n == 0 { return None; }

    // Rank by |diff|, averaging ties
    diffs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0f64; n];
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j + 1 < n && (diffs[j + 1].0 - diffs[i].0).abs() < 1e-12 { j += 1; }
        let avg_rank = (i + 1 + j + 1) as f64 / 2.0;
        for r in ranks.iter_mut().take(j + 1).skip(i) { *r = avg_rank; }
        i = j + 1;
    }

    let mut w_plus = 0.0;
    let mut w_minus = 0.0;
    for (rank, (_, sign)) in ranks.iter().zip(diffs.iter()) {
        if *sign > 0.0 { w_plus += rank; }
        else           { w_minus += rank; }
    }

    // Normal approximation with continuity correction
    let n_f = n as f64;
    let mean_w = n_f * (n_f + 1.0) / 4.0;
    let var_w  = n_f * (n_f + 1.0) * (2.0 * n_f + 1.0) / 24.0;
    let z = (w_plus - mean_w - 0.5 * (w_plus - mean_w).signum()) / var_w.sqrt();

    // Two-sided p = 2 * (1 - Φ(|z|))
    let p = 2.0 * (1.0 - standard_normal_cdf(z.abs()));

    // Median difference
    let mut all_diffs: Vec<f64> = x.iter().zip(y.iter()).map(|(a, b)| a - b).collect();
    all_diffs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median_diff = all_diffs[all_diffs.len() / 2];

    Some(WilcoxonResult {
        n_effective: n,
        w_plus,
        w_minus,
        z,
        p_value_two_sided: p,
        significant_at_01: p < 0.01,
        median_diff,
    })
}

/// Standard normal CDF via erf approximation (Abramowitz & Stegun 7.1.26).
fn standard_normal_cdf(x: f64) -> f64 {
    0.5 * (1.0 + erf(x / std::f64::consts::SQRT_2))
}

fn erf(x: f64) -> f64 {
    // A&S 7.1.26 — max error 1.5e-7
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let a1 = 0.254829592;
    let a2 = -0.284496736;
    let a3 = 1.421413741;
    let a4 = -1.453152027;
    let a5 = 1.061405429;
    let p = 0.3275911;
    let t = 1.0 / (1.0 + p * x);
    let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
    sign * y
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strong_positive_effect_is_significant() {
        // KDF 100% vs Random 30% — should be highly significant
        let kdf = vec![1.0; 30];
        let random: Vec<f64> = (0..30).map(|i| 0.25 + (i as f64) * 0.002).collect();
        let r = wilcoxon_signed_rank(&kdf, &random).unwrap();
        assert!(r.significant_at_01, "clear difference must be significant; p={}", r.p_value_two_sided);
        assert!(r.median_diff > 0.6);
    }

    #[test]
    fn zero_effect_not_significant() {
        // Identical samples → no signal → p ~ 1 or NaN → None (all zeros dropped)
        let a = vec![0.5; 20];
        let b = vec![0.5; 20];
        assert!(wilcoxon_signed_rank(&a, &b).is_none());
    }

    #[test]
    fn noisy_equal_means_not_significant() {
        // Same mean, different noise — should not be significant
        let a: Vec<f64> = (0..30).map(|i| 0.5 + ((i % 3) as f64 - 1.0) * 0.01).collect();
        let b: Vec<f64> = (0..30).map(|i| 0.5 + ((i % 4) as f64 - 1.5) * 0.01).collect();
        let r = wilcoxon_signed_rank(&a, &b).unwrap();
        assert!(r.p_value_two_sided > 0.01, "similar samples must not be significant at 0.01, p={}", r.p_value_two_sided);
    }

    #[test]
    fn erf_accuracy() {
        // Reference values
        assert!((erf(0.0) - 0.0).abs() < 1e-6);
        assert!((erf(1.0) - 0.8427007).abs() < 1e-6);
        assert!((erf(2.0) - 0.9953223).abs() < 1e-6);
    }
}
