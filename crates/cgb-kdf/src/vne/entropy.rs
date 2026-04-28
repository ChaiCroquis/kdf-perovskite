//! VNE calculation and change detection

use super::matrix::{density_matrix, laplacian_matrix};
use super::sparse;
use super::types::{ChangeDetection, VNEResult};
use nalgebra::SymmetricEigen;

/// Switch to sparse Hutchinson + Chebyshev approximation at or above this size.
///
/// Below the threshold the dense `SymmetricEigen` path is used so that small
/// graphs preserve bit-exact reproducibility (Claim 15). At and above it, the
/// dense path's O(n³) cost (~10¹² ops at n = 10⁴) becomes infeasible online,
/// so we trade bit-exactness for tolerance-bounded determinism (seeded RNG).
pub const SPARSE_THRESHOLD: usize = 1000;

/// Compute Von Neumann Entropy from edge list
///
/// VNE = -Σ λᵢ * ln(λᵢ) where λᵢ > 0
///
/// # Dispatch
/// `node_count < `[`SPARSE_THRESHOLD`]: dense `SymmetricEigen` (bit-exact).
/// Otherwise: [`sparse::von_neumann_entropy_sparse`] (deterministic, ≈1e-2 rel. err).
///
/// # Arguments
/// * `node_count` - Number of nodes
/// * `edges` - Edge list as (from, to, weight)
pub fn von_neumann_entropy(node_count: usize, edges: &[(u32, u32, f64)]) -> f64 {
    if node_count < SPARSE_THRESHOLD {
        von_neumann_entropy_dense(node_count, edges).entropy
    } else {
        sparse::von_neumann_entropy_sparse(node_count, edges)
    }
}

/// Compute detailed VNE result including eigenvalues and spectral gap.
///
/// On the sparse path (`node_count >= `[`SPARSE_THRESHOLD`]) eigenvalues and
/// `spectral_gap` are not computed — those fields are left empty / zero.
/// `num_components` is recovered via union-find on the edge list.
pub fn von_neumann_entropy_detailed(node_count: usize, edges: &[(u32, u32, f64)]) -> VNEResult {
    if node_count < SPARSE_THRESHOLD {
        von_neumann_entropy_dense(node_count, edges)
    } else {
        VNEResult {
            entropy: sparse::von_neumann_entropy_sparse(node_count, edges),
            eigenvalues: Vec::new(),
            spectral_gap: 0.0,
            num_components: sparse::count_components(node_count, edges),
        }
    }
}

/// Dense VNE via full eigendecomposition. O(n³) — caller is responsible for
/// invoking only on small graphs (see [`SPARSE_THRESHOLD`]). Exposed so the
/// precision test and benchmark example can compare against it directly,
/// bypassing the dispatch in [`von_neumann_entropy_detailed`].
pub fn von_neumann_entropy_dense(node_count: usize, edges: &[(u32, u32, f64)]) -> VNEResult {
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
