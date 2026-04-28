//! Sparse approximation of Von Neumann Entropy for large graphs (n >= 1000).
//!
//! # Background
//!
//! The dense path in [`super::entropy`] computes ρ = L / tr(L) as an
//! `nalgebra::DMatrix` and feeds it to `SymmetricEigen`, which is O(n³).
//! At n = 10⁴ that is ~10¹² floating-point ops — infeasible for the Claim 27-32
//! "health metric / δk⁴ / emergency intervention" path that needs to run
//! online on production-scale graphs.
//!
//! # Algorithm
//!
//! `S = -Σ ρᵢ ln(ρᵢ)` is approximated as `tr(f(ρ))` with `f(x) = -x ln(x)` via:
//!
//! 1. **CSR Laplacian** — build `L = D - W` from the edge list directly in
//!    sparse form (`O(|E|)` time/space, never materialising the n×n dense matrix).
//! 2. **Affine remap** — `ρ` has eigenvalues in `[0, 1]`; we work with
//!    `M = (2/tr(L)) · L − I` whose eigenvalues lie in `[-1, 1]` (the natural
//!    domain for Chebyshev polynomials).
//! 3. **Chebyshev expansion** — `f((y+1)/2) ≈ c₀/2 + Σ cₖ Tₖ(y)`. Coefficients
//!    are computed once via Chebyshev-Gauss quadrature.
//! 4. **Hutchinson estimator** — `tr(f(M)) ≈ (1/m) Σ z_iᵀ f(M) z_i` for random
//!    Rademacher vectors `z_i`. Each `f(M) z_i` is built via the three-term
//!    recurrence `T_{k+1}(M) z = 2 M Tₖ(M) z − T_{k-1}(M) z`, requiring just
//!    `K` sparse matrix-vector products.
//!
//! Total cost: `O(|E| · K · m)`. At n = 10⁴, |E| ≈ 10n, K = 80, m = 30:
//! ~2.4 × 10⁷ ops vs ~10¹² for dense — three orders of magnitude.
//!
//! # Determinism
//!
//! Hutchinson is stochastic, but we seed `ChaCha8Rng` with the fixed constant
//! [`VNE_RNG_SEED`]. Identical input → identical output across runs/platforms.
//! Bit-exact equality with the dense path is **not** preserved (sparse is a
//! tolerance-bounded approximation), so the dispatch in [`super::entropy`]
//! routes `n < 1000` to dense to keep Claim 15 bit-exactness for that regime.
//!
//! # Reference
//!
//! Han, Avron, Kasiviswanathan, et al. "Approximating Spectral Sums of
//! Large-scale Matrices using Stochastic Chebyshev Approximations." 2017.

use nalgebra_sparse::{CooMatrix, CsrMatrix};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

/// Hutchinson probe-vector count.
pub const VNE_HUTCHINSON_SAMPLES: usize = 30;

/// Chebyshev polynomial degree (number of terms = degree + 1).
pub const VNE_CHEBYSHEV_DEGREE: usize = 80;

/// Deterministic RNG seed: ASCII "VNEHCH08" interpreted as little-endian u64.
const VNE_RNG_SEED: u64 = 0x3830_4843_4845_4E56;

/// Build the sparse graph Laplacian `L = D - W` (CSR format) from an edge list.
///
/// Each edge `(u, v, w)` adds `-w` to off-diagonal entries `(u, v)` and `(v, u)`
/// and `+w` to the diagonal entries `(u, u)` and `(v, v)`. Self-loops (`u == v`)
/// are ignored, matching the standard graph-Laplacian definition; out-of-range
/// indices are silently dropped, matching the dense path's behaviour.
pub fn laplacian_csr(node_count: usize, edges: &[(u32, u32, f64)]) -> CsrMatrix<f64> {
    let mut coo = CooMatrix::new(node_count, node_count);
    let mut diag = vec![0.0f64; node_count];

    for &(u, v, w) in edges {
        let i = u as usize;
        let j = v as usize;
        if i < node_count && j < node_count && i != j {
            coo.push(i, j, -w);
            coo.push(j, i, -w);
            diag[i] += w;
            diag[j] += w;
        }
    }

    for (i, &d) in diag.iter().enumerate() {
        if d != 0.0 {
            coo.push(i, i, d);
        }
    }

    CsrMatrix::from(&coo)
}

/// Trace of a CSR matrix (sum of diagonal entries).
fn csr_trace(m: &CsrMatrix<f64>) -> f64 {
    let row_offsets = m.row_offsets();
    let col_indices = m.col_indices();
    let values = m.values();
    let mut tr = 0.0f64;
    for i in 0..m.nrows() {
        for k in row_offsets[i]..row_offsets[i + 1] {
            if col_indices[k] == i {
                tr += values[k];
                break;
            }
        }
    }
    tr
}

/// Chebyshev coefficients `c_0..c_K` for `g(y) = f((y+1)/2)` where
/// `f(x) = -x ln(x)` on `[0, 1]` with `f(0) := 0`.
///
/// Reconstruction: `g(y) ≈ c_0/2 · T_0(y) + Σ_{k=1}^K c_k Tₖ(y)`.
fn chebyshev_coeffs(degree: usize) -> Vec<f64> {
    let n = degree + 1;
    let n_f = n as f64;
    let pi = std::f64::consts::PI;

    let g_vals: Vec<f64> = (0..n)
        .map(|j| {
            let y = ((j as f64 + 0.5) * pi / n_f).cos();
            let x = (y + 1.0) * 0.5;
            if x <= 0.0 { 0.0 } else { -x * x.ln() }
        })
        .collect();

    let mut c = vec![0.0f64; n];
    for k in 0..n {
        let mut s = 0.0;
        for j in 0..n {
            s += g_vals[j] * (((j as f64 + 0.5) * (k as f64) * pi) / n_f).cos();
        }
        c[k] = (2.0 / n_f) * s;
    }
    c
}

/// Compute `out = M · v` where `M = (2/tr_L) · L − I`.
fn apply_m(lap: &CsrMatrix<f64>, tr_l: f64, v: &[f64], out: &mut [f64]) {
    let row_offsets = lap.row_offsets();
    let col_indices = lap.col_indices();
    let values = lap.values();
    let scale = 2.0 / tr_l;
    let n = v.len();

    for i in 0..n {
        let mut s = 0.0;
        for k in row_offsets[i]..row_offsets[i + 1] {
            s += values[k] * v[col_indices[k]];
        }
        out[i] = scale * s - v[i];
    }
}

#[inline]
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Sparse VNE estimator: `S ≈ tr(f(ρ))` with `f(x) = -x ln(x)`.
///
/// Returns 0 for empty graphs or graphs with no edges. Determinism is
/// maintained via the fixed seed [`VNE_RNG_SEED`] (see module docs).
pub fn von_neumann_entropy_sparse(node_count: usize, edges: &[(u32, u32, f64)]) -> f64 {
    if node_count == 0 || edges.is_empty() {
        return 0.0;
    }

    let lap = laplacian_csr(node_count, edges);
    let tr_l = csr_trace(&lap);
    if tr_l.abs() < 1e-15 {
        return 0.0;
    }

    let coeffs = chebyshev_coeffs(VNE_CHEBYSHEV_DEGREE);
    let mut rng = ChaCha8Rng::seed_from_u64(VNE_RNG_SEED);
    let mut sum_estimate = 0.0;

    let mut t_prev = vec![0.0f64; node_count];
    let mut t_curr = vec![0.0f64; node_count];
    let mut t_next = vec![0.0f64; node_count];

    for _ in 0..VNE_HUTCHINSON_SAMPLES {
        let z: Vec<f64> = (0..node_count)
            .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
            .collect();

        // T_0(M) z = z, T_1(M) z = M z
        t_prev.copy_from_slice(&z);
        apply_m(&lap, tr_l, &z, &mut t_curr);

        // c_0/2 · z·T_0 z  +  c_1 · z·T_1 z  +  Σ_{k=2}^K c_k · z·T_k z
        let mut acc = 0.5 * coeffs[0] * dot(&z, &t_prev) + coeffs[1] * dot(&z, &t_curr);

        for k in 2..=VNE_CHEBYSHEV_DEGREE {
            apply_m(&lap, tr_l, &t_curr, &mut t_next);
            for i in 0..node_count {
                t_next[i] = 2.0 * t_next[i] - t_prev[i];
            }
            acc += coeffs[k] * dot(&z, &t_next);

            std::mem::swap(&mut t_prev, &mut t_curr);
            std::mem::swap(&mut t_curr, &mut t_next);
        }

        sum_estimate += acc;
    }

    let result = sum_estimate / VNE_HUTCHINSON_SAMPLES as f64;
    // Numerical noise can push the estimate slightly negative for graphs
    // whose true entropy is near zero; clamp to the mathematical range.
    result.max(0.0)
}

/// Count connected components via iterative union-find with path compression.
///
/// Used to populate `VNEResult.num_components` on the sparse path, where the
/// dense `SymmetricEigen` is unavailable.
pub fn count_components(node_count: usize, edges: &[(u32, u32, f64)]) -> usize {
    if node_count == 0 {
        return 0;
    }

    let mut parent: Vec<usize> = (0..node_count).collect();
    let mut rank = vec![0u32; node_count];

    fn find(parent: &mut [usize], x: usize) -> usize {
        let mut root = x;
        while parent[root] != root {
            root = parent[root];
        }
        let mut cur = x;
        while parent[cur] != root {
            let next = parent[cur];
            parent[cur] = root;
            cur = next;
        }
        root
    }

    for &(u, v, _) in edges {
        let i = u as usize;
        let j = v as usize;
        if i < node_count && j < node_count {
            let ri = find(&mut parent, i);
            let rj = find(&mut parent, j);
            if ri != rj {
                use std::cmp::Ordering;
                match rank[ri].cmp(&rank[rj]) {
                    Ordering::Less => parent[ri] = rj,
                    Ordering::Greater => parent[rj] = ri,
                    Ordering::Equal => {
                        parent[rj] = ri;
                        rank[ri] += 1;
                    }
                }
            }
        }
    }

    let mut roots = std::collections::HashSet::new();
    for i in 0..node_count {
        roots.insert(find(&mut parent, i));
    }
    roots.len()
}
