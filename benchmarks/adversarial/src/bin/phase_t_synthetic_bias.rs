//! Phase T — Synthetic bias detection metric.
//!
//! Problem (F-025): results on synthetic benchmarks can have OPPOSITE sign
//! from real data. We need a pre-hoc metric that flags "this dataset may
//! over/under-state KDF's advantage".
//!
//! Candidate indicators:
//!   I1: Degree distribution tail heaviness (Kolmogorov-Smirnov vs power-law)
//!   I2: Clustering coefficient ratio (local vs global)
//!   I3: Rare-truth concentration (do rare ground-truth items share structure?)
//!   I4: Boundary-of-Rare sharpness (is there a clear deg==1 cliff?)
//!
//! For each, we compute on both the known synthetic (high_degree_rare)
//! and the real (FB15K-237, NASA log) datasets, and see whether a given
//! indicator value PREDICTS the sign flip we observed in F-025.

use adversarial_bench as adv;
use real_data_bench::{public_datasets, Dataset};
use std::collections::{HashMap, HashSet};

fn compute_degrees(ds: &Dataset) -> Vec<u32> {
    let mut d = vec![0u32; ds.n_nodes];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < ds.n_nodes { d[u as usize] += 1; }
        if (v as usize) < ds.n_nodes { d[v as usize] += 1; }
    }
    d
}

/// Indicator I1: fraction of nodes with deg==1 out of non-zero-degree.
/// HIGH → synthetic-like (deg==1 heavy), LOW → real-like.
fn indicator_deg1_ratio(deg: &[u32]) -> f64 {
    let nonzero = deg.iter().filter(|&&d| d > 0).count();
    if nonzero == 0 { return 0.0; }
    let deg1 = deg.iter().filter(|&&d| d == 1).count();
    deg1 as f64 / nonzero as f64
}

/// Indicator I2: power-law fit quality (Kolmogorov-Smirnov vs idealized Zipf).
/// LOW value (good fit to power-law) → realistic long-tail; HIGH → synthetic.
fn indicator_powerlaw_deviation(deg: &[u32]) -> f64 {
    let mut d_sorted: Vec<u32> = deg.iter().filter(|&&x| x > 0).copied().collect();
    d_sorted.sort_unstable_by(|a, b| b.cmp(a));
    if d_sorted.len() < 10 { return 1.0; }
    // Fit: rank k → expected deg = d_max / k
    let d_max = d_sorted[0] as f64;
    let ks: f64 = d_sorted.iter().enumerate().map(|(k, &d)| {
        let expected = d_max / (k as f64 + 1.0);
        ((d as f64 - expected) / expected).abs()
    }).take(100).sum::<f64>() / 100.0;
    ks
}

/// Indicator I3: rare ground-truth concentration — do rare items cluster
/// in degree space? Compute mean degree of rare items vs overall.
/// RATIO close to 1 (rare deg = overall mean) → **hard for KDF**.
/// RATIO far from 1 → structural signal is strong.
fn indicator_rare_deg_signal(deg: &[u32], rare: &HashSet<u32>) -> f64 {
    if rare.is_empty() { return 1.0; }
    let rare_mean: f64 = rare.iter()
        .filter(|&&id| (id as usize) < deg.len())
        .map(|&id| deg[id as usize] as f64)
        .sum::<f64>() / rare.len() as f64;
    let overall_mean: f64 = deg.iter().map(|&d| d as f64).sum::<f64>() / deg.len().max(1) as f64;
    if overall_mean == 0.0 { 1.0 } else { (rare_mean / overall_mean - 1.0).abs() }
}

/// Indicator I4: Deg==1 cliff — how many rare-truth items have exactly deg==1?
/// HIGH → synthetic bias in KDF's favor (classifier's Rare rule matches truth).
fn indicator_rare_deg1_rate(deg: &[u32], rare: &HashSet<u32>) -> f64 {
    if rare.is_empty() { return 0.0; }
    let d1 = rare.iter().filter(|&&id| (id as usize) < deg.len() && deg[id as usize] == 1).count();
    d1 as f64 / rare.len() as f64
}

fn report_dataset(name: &str, ds: &Dataset, synthetic: bool) {
    let deg = compute_degrees(ds);
    let i1 = indicator_deg1_ratio(&deg);
    let i2 = indicator_powerlaw_deviation(&deg);
    let i3 = indicator_rare_deg_signal(&deg, &ds.rare_ground_truth);
    let i4 = indicator_rare_deg1_rate(&deg, &ds.rare_ground_truth);

    let bias_score = i1 * 0.3 + i4 * 0.7; // deg1 prevalence + rare-deg1 match
    let flag = if bias_score > 0.5 { "⚠️ HIGH bias toward KDF" }
               else if bias_score > 0.2 { "◐ moderate bias" }
               else { "✓ low bias" };

    println!(
        "| {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {} |",
        name,
        if synthetic { "synth" } else { "REAL " },
        i1, i2, i3, i4, bias_score, flag
    );
}

fn main() {
    println!("# Phase T — Synthetic-bias detection metric\n");
    println!("Indicators (higher = more bias toward KDF's favor):");
    println!("- I1: fraction of deg==1 nodes (KDF's default Rare rule)");
    println!("- I2: power-law fit deviation (lower = more realistic)");
    println!("- I3: rare-degree signal strength (rare_mean_deg vs overall)");
    println!("- I4: rare-truth-at-deg==1 rate (KDF's alignment with truth)");
    println!("- bias_score = 0.3·I1 + 0.7·I4\n");
    println!("| Dataset | Type | I1 | I2 | I3 | I4 | bias_score | flag |");
    println!("|---|---|---:|---:|---:|---:|---:|---|");

    // Synthetic
    let s1 = adv::high_degree_rare(500, 1, 42);
    report_dataset("A_deg1(500)", &s1, true);
    let s3 = adv::high_degree_rare(500, 3, 42);
    report_dataset("A_deg3(500)", &s3, true);
    let bn = adv::structurally_isolated(500, 2, 42);
    report_dataset("B_isolated(500)", &bn, true);

    // Real
    if let Some(fb) = public_datasets::load_fb15k_237(200) {
        report_dataset("FB15K-237(real)", &fb, false);
    }
    let rare_codes = [400u16, 401, 403, 404, 500, 502, 503, 504].into_iter().collect();
    if let Some(nasa) = public_datasets::load_nasa_log(&rare_codes) {
        report_dataset("NASA-HTTP(real)", &nasa, false);
    }

    // Verify: for datasets where KDF won on synthetic but lost on real,
    // bias_score should flag HIGH on synthetic, LOW on real.
    println!("\n## Interpretation");
    println!("If synthetic A_deg1 has high bias_score AND real FB15K/NASA have low,");
    println!("the metric is predictive. Use it pre-hoc to flag 'synthetic biases in KDF's favor'.");
}
