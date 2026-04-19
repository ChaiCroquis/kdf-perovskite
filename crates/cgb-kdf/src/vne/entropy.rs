//! VNE calculation and change detection

use nalgebra::SymmetricEigen;
use super::matrix::{laplacian_matrix, density_matrix};
use super::types::{VNEResult, ChangeDetection};

/// Compute Von Neumann Entropy from edge list
///
/// VNE = -Σ λᵢ * ln(λᵢ) where λᵢ > 0
///
/// # Arguments
/// * `node_count` - Number of nodes
/// * `edges` - Edge list as (from, to, weight)
pub fn von_neumann_entropy(node_count: usize, edges: &[(u32, u32, f64)]) -> f64 {
    von_neumann_entropy_detailed(node_count, edges).entropy
}

/// Compute detailed VNE result including eigenvalues and spectral gap
pub fn von_neumann_entropy_detailed(node_count: usize, edges: &[(u32, u32, f64)]) -> VNEResult {
    if node_count == 0 {
        return VNEResult::empty();
    }

    // Compute density matrix
    let lap = laplacian_matrix(node_count, edges);
    let rho = density_matrix(&lap);

    if rho.trace().abs() < 1e-10 {
        return VNEResult {
            entropy: 0.0,
            eigenvalues: vec![0.0; node_count],
            spectral_gap: 0.0,
            num_components: node_count,
        };
    }

    // Compute eigenvalues (symmetric matrix)
    let eigen = SymmetricEigen::new(rho);
    let mut eigenvalues: Vec<f64> = eigen.eigenvalues.iter().cloned().collect();
    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // Count zero eigenvalues (connected components)
    let epsilon = 1e-10;
    let num_components = eigenvalues.iter().filter(|&&ev| ev.abs() < epsilon).count();

    // Spectral gap
    let spectral_gap = if eigenvalues.len() >= 2 {
        eigenvalues[1] - eigenvalues[0]
    } else {
        0.0
    };

    // VNE calculation: S = -Σ λᵢ * ln(λᵢ)
    let mut entropy = 0.0;
    for &ev in &eigenvalues {
        if ev > epsilon {
            entropy -= ev * ev.ln();
        }
    }

    VNEResult {
        entropy,
        eigenvalues,
        spectral_gap,
        num_components,
    }
}

/// Detect VNE change between two graphs
///
/// # Arguments
/// * `node_count1` - Node count for first graph
/// * `edges1` - Edges for first graph
/// * `node_count2` - Node count for second graph
/// * `edges2` - Edges for second graph
/// * `threshold` - Relative change threshold for significance
pub fn detect_change(
    node_count1: usize,
    edges1: &[(u32, u32, f64)],
    node_count2: usize,
    edges2: &[(u32, u32, f64)],
    threshold: f64,
) -> ChangeDetection {
    let vne1 = von_neumann_entropy_detailed(node_count1, edges1);
    let vne2 = von_neumann_entropy_detailed(node_count2, edges2);

    let absolute_change = (vne2.entropy - vne1.entropy).abs();

    let relative_change = if vne1.entropy.abs() > 1e-10 {
        absolute_change / vne1.entropy
    } else if vne2.entropy.abs() > 1e-10 {
        f64::INFINITY
    } else {
        0.0
    };

    let is_significant = relative_change > threshold;

    ChangeDetection {
        vne_before: vne1.entropy,
        vne_after: vne2.entropy,
        absolute_change,
        relative_change,
        is_significant,
        spectral_gap_before: vne1.spectral_gap,
        spectral_gap_after: vne2.spectral_gap,
    }
}
