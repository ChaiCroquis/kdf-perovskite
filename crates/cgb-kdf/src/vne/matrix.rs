//! Matrix computation functions for VNE

use nalgebra::DMatrix;

/// Compute Laplacian matrix from edge list
///
/// L = D - A where D is degree diagonal and A is adjacency matrix
///
/// # Arguments
/// * `node_count` - Number of nodes
/// * `edges` - Edge list as (from, to, weight)
pub fn laplacian_matrix(node_count: usize, edges: &[(u32, u32, f64)]) -> DMatrix<f64> {
    let mut laplacian = DMatrix::zeros(node_count, node_count);

    for &(u, v, weight) in edges {
        let i = u as usize;
        let j = v as usize;
        if i < node_count && j < node_count {
            laplacian[(i, j)] = -weight;
            laplacian[(j, i)] = -weight;
            laplacian[(i, i)] += weight;
            laplacian[(j, j)] += weight;
        }
    }

    laplacian
}

/// Compute density matrix from Laplacian
///
/// ρ = L / tr(L)
pub fn density_matrix(laplacian: &DMatrix<f64>) -> DMatrix<f64> {
    let trace = laplacian.trace();
    if trace.abs() < 1e-10 {
        return laplacian.clone();
    }
    laplacian / trace
}
