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

/// Hutchinson probe-vector count (used by both Chebyshev and SLQ paths).
pub const VNE_HUTCHINSON_SAMPLES: usize = 30;

/// Chebyshev polynomial degree (number of terms = degree + 1).
pub const VNE_CHEBYSHEV_DEGREE: usize = 80;

/// Lanczos depth per probe vector for the SLQ estimator.
///
/// 30 is the empirical sweet spot: bumping `k` to 60 produces identical Ritz
/// sets (β_j drops below the 1e-12 deflation threshold once the extremes have
/// converged), so the extra cost buys no precision. See
/// [`von_neumann_entropy_slq`] doc-comment for the precision comparison
/// against the Chebyshev path.
pub const VNE_SLQ_LANCZOS_DEGREE: usize = 30;

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

/// Lanczos iteration count for spectral-gap estimation. Empirically 30 is
/// sufficient for the second-smallest eigenvalue of `ρ = L/tr(L)` on graphs
/// up to n ≈ 10⁴; raise if convergence diagnostics warrant.
pub const VNE_LANCZOS_ITERATIONS: usize = 30;

/// Apply `ρ = L / tr_l` to a dense vector `v` (writing into `out`).
fn apply_rho(lap: &CsrMatrix<f64>, tr_l: f64, v: &[f64], out: &mut [f64]) {
    let row_offsets = lap.row_offsets();
    let col_indices = lap.col_indices();
    let values = lap.values();
    let n = v.len();
    for i in 0..n {
        let mut s = 0.0;
        for k in row_offsets[i]..row_offsets[i + 1] {
            s += values[k] * v[col_indices[k]];
        }
        out[i] = s / tr_l;
    }
}

/// Estimate the spectral gap `λ₂(ρ) − λ₁(ρ)` of the normalized Laplacian.
///
/// `λ₁` is always 0 for graph Laplacians; for connected graphs the gap equals
/// the (normalized) Fiedler value, and for graphs with `c ≥ 2` connected
/// components the multiplicity of the zero eigenvalue is `c`, hence the gap
/// is 0. We exploit this:
///
/// - **Disconnected case** (`num_components ≥ 2`): return 0 directly.
/// - **Connected case**: Lanczos with explicit deflation against the kernel
///   vector `v₀ = ones/√n` (which spans `ker(L)` for connected graphs).
///   Starting from a random vector orthogonalised against `v₀`, the Krylov
///   subspace `K(ρ, q₁)` lies entirely in `span{v₀}^⊥`, so the smallest
///   Ritz value of the resulting `K × K` tridiagonal `T` approximates `λ₂(ρ)`.
///
/// Full reorthogonalisation (Modified Gram-Schmidt against all prior basis
/// vectors plus `v₀`) is used for numerical stability — Lanczos is famously
/// fragile in finite precision. Cost is `O(K · |E| + K² · n)`, dominated by
/// the matvec for typical sparse graphs.
///
/// Determinism: ChaCha8Rng seeded with `VNE_RNG_SEED ^ 0xCAFE_BABE` (distinct
/// from the entropy estimator's RNG so the two paths do not share probe state).
pub fn spectral_gap_sparse(
    node_count: usize,
    edges: &[(u32, u32, f64)],
    num_components: usize,
) -> f64 {
    if num_components != 1 || node_count < 2 || edges.is_empty() {
        return 0.0;
    }

    let lap = laplacian_csr(node_count, edges);
    let tr_l = csr_trace(&lap);
    if tr_l.abs() < 1e-15 {
        return 0.0;
    }

    let n = node_count;
    let v0_factor = 1.0 / (n as f64).sqrt();

    // Helper: orthogonalise `w` against v₀ = ones/√n.
    let orth_v0 = |w: &mut [f64]| {
        let dot: f64 = w.iter().sum::<f64>() * v0_factor;
        let coeff = dot * v0_factor;
        for x in w.iter_mut() {
            *x -= coeff;
        }
    };

    // Initial Lanczos vector: random Gaussian-ish, projected onto span{v₀}^⊥.
    let mut rng = ChaCha8Rng::seed_from_u64(VNE_RNG_SEED ^ 0xCAFE_BABE);
    let mut q1: Vec<f64> = (0..n)
        .map(|_| rng.gen_bool(0.5) as i32 as f64 * 2.0 - 1.0)
        .collect();
    orth_v0(&mut q1);
    let q1_norm = q1.iter().map(|x| x * x).sum::<f64>().sqrt();
    if q1_norm < 1e-15 {
        return 0.0;
    }
    for x in q1.iter_mut() {
        *x /= q1_norm;
    }

    let max_iter = VNE_LANCZOS_ITERATIONS.min(n - 1);
    let mut alpha: Vec<f64> = Vec::with_capacity(max_iter);
    let mut beta: Vec<f64> = Vec::with_capacity(max_iter);
    let mut basis: Vec<Vec<f64>> = Vec::with_capacity(max_iter + 1);
    basis.push(q1);

    let mut z = vec![0.0f64; n];

    for j in 0..max_iter {
        // z ← ρ · basis[j]
        apply_rho(&lap, tr_l, &basis[j], &mut z);

        // α_j = ⟨basis[j], z⟩
        let alpha_j: f64 = basis[j].iter().zip(z.iter()).map(|(v, w)| v * w).sum();
        alpha.push(alpha_j);

        // z ← z − α_j basis[j] − β_{j-1} basis[j-1]
        for i in 0..n {
            z[i] -= alpha_j * basis[j][i];
        }
        if j > 0 {
            let bm1 = beta[j - 1];
            for i in 0..n {
                z[i] -= bm1 * basis[j - 1][i];
            }
        }

        // Full reorthogonalisation: against v₀ first, then all prior basis vectors.
        orth_v0(&mut z);
        for v_prev in basis.iter() {
            let coeff: f64 = v_prev.iter().zip(z.iter()).map(|(v, w)| v * w).sum();
            for i in 0..n {
                z[i] -= coeff * v_prev[i];
            }
        }

        let beta_j: f64 = z.iter().map(|x| x * x).sum::<f64>().sqrt();
        if beta_j < 1e-12 {
            break; // Krylov subspace exhausted (rare for our use case).
        }
        beta.push(beta_j);

        let mut q_next = vec![0.0f64; n];
        for i in 0..n {
            q_next[i] = z[i] / beta_j;
        }
        basis.push(q_next);
    }

    let k = alpha.len();
    if k == 0 {
        return 0.0;
    }

    // Diagonalise the small dense tridiagonal T_k via SymmetricEigen.
    use nalgebra::{DMatrix, SymmetricEigen};
    let mut t_mat = DMatrix::zeros(k, k);
    for i in 0..k {
        t_mat[(i, i)] = alpha[i];
        if i + 1 < k {
            t_mat[(i, i + 1)] = beta[i];
            t_mat[(i + 1, i)] = beta[i];
        }
    }

    let eig = SymmetricEigen::new(t_mat);
    let smallest = eig
        .eigenvalues
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    // Ritz values are an approximation; clamp to the mathematically valid range.
    smallest.max(0.0)
}

/// Stochastic Lanczos Quadrature estimator for the von Neumann entropy.
///
/// # Status: alternative path, not the default
///
/// **Empirically**, SLQ does **not** improve precision over the Chebyshev
/// path in [`von_neumann_entropy_sparse`] for this problem. Measured on
/// Erdős–Rényi graphs (avg deg 5, n ∈ {100..2000}):
///
/// | n | Chebyshev rel.err | SLQ rel.err |
/// |---:|---:|---:|
/// | 100  | 1.11e-2 | 1.78e-2 |
/// | 500  | 3.96e-3 | 9.70e-3 |
/// | 1000 | 3.60e-4 | 2.59e-3 |
/// | 2000 | 2.29e-3 | 2.89e-3 |
///
/// Why: graph Laplacian density matrices have most spectral mass clustered
/// near 0, where Lanczos converges to the *extremal* (largest) eigenvalues
/// fastest — interior small eigenvalues, which dominate `−x ln x`, are
/// captured slowly. Pushing `k` from 30 → 60 produces near-identical Ritz
/// sets because `β_j` collapses below 1e-12 once the extremes are caught.
/// The Chebyshev path's K=80 polynomial fit covers `[0,1]` more uniformly.
///
/// SLQ is kept as a comparison baseline for future research (e.g. larger
/// `m`, restarted Lanczos, or different `f`); the production dispatch in
/// [`super::entropy`] continues to call [`von_neumann_entropy_sparse`].
///
/// # Algorithm
///
/// For each Rademacher probe `z_i`:
///   1. Normalise `q_1 = z_i / ||z_i||`
///   2. Run `k` Lanczos steps on `ρ` (full Modified-GS reorthogonalisation)
///   3. Eigendecompose the resulting tridiagonal `T_k = Q diag(θ_j) Q^T`
///   4. Add `||z_i||² Σ_j (Q[0,j])² f(θ_j)` to the running sum (`f(0):=0`)
///
/// Final estimate is the average over `m = VNE_HUTCHINSON_SAMPLES` probes.
///
/// # Reference
///
/// Ubaru, Chen, Saad. "Fast estimation of `tr(f(A))` via Stochastic Lanczos
/// Quadrature." SIAM J. Matrix Anal. Appl., 2017.
pub fn von_neumann_entropy_slq(node_count: usize, edges: &[(u32, u32, f64)]) -> f64 {
    if node_count == 0 || edges.is_empty() {
        return 0.0;
    }

    let lap = laplacian_csr(node_count, edges);
    let tr_l = csr_trace(&lap);
    if tr_l.abs() < 1e-15 {
        return 0.0;
    }

    let n = node_count;
    let m = VNE_HUTCHINSON_SAMPLES;
    let k = VNE_SLQ_LANCZOS_DEGREE.min(n.saturating_sub(1).max(1));
    let mut rng = ChaCha8Rng::seed_from_u64(VNE_RNG_SEED ^ 0x00C0_FFEE);

    let mut alpha = vec![0.0f64; k];
    let mut beta = vec![0.0f64; k.saturating_sub(1)];
    let mut total = 0.0f64;

    for _ in 0..m {
        // Rademacher probe z, ||z||² = n.
        let z: Vec<f64> = (0..n)
            .map(|_| if rng.gen_bool(0.5) { 1.0 } else { -1.0 })
            .collect();
        let z_norm_sq = n as f64;
        let inv_zn = 1.0 / z_norm_sq.sqrt();

        // q_1 = z / ||z||
        let mut basis: Vec<Vec<f64>> = Vec::with_capacity(k);
        let mut q1 = vec![0.0f64; n];
        for i in 0..n {
            q1[i] = z[i] * inv_zn;
        }
        basis.push(q1);

        let mut w = vec![0.0f64; n];
        let mut actual_k = 0usize;

        for j in 0..k {
            // w = ρ · basis[j]
            apply_rho(&lap, tr_l, &basis[j], &mut w);

            // α_j = ⟨basis[j], w⟩
            let alpha_j: f64 = basis[j].iter().zip(w.iter()).map(|(b, x)| b * x).sum();
            alpha[j] = alpha_j;

            // w ← w − α_j basis[j] − β_{j-1} basis[j-1]
            for i in 0..n {
                w[i] -= alpha_j * basis[j][i];
            }
            if j > 0 {
                let b_prev = beta[j - 1];
                let prev = &basis[j - 1];
                for i in 0..n {
                    w[i] -= b_prev * prev[i];
                }
            }

            // Modified Gram-Schmidt full reorthogonalisation against all prior basis
            // vectors. Without this, vanilla Lanczos loses orthogonality after
            // ~15-20 steps and produces spurious Ritz pairs that pollute SLQ.
            for v_prev in basis.iter() {
                let coeff: f64 = v_prev.iter().zip(w.iter()).map(|(v, x)| v * x).sum();
                for i in 0..n {
                    w[i] -= coeff * v_prev[i];
                }
            }

            actual_k = j + 1;

            if j + 1 < k {
                let bj = w.iter().map(|x| x * x).sum::<f64>().sqrt();
                if bj < 1e-12 {
                    break;
                }
                beta[j] = bj;
                let inv_bj = 1.0 / bj;
                let mut q_next = vec![0.0f64; n];
                for i in 0..n {
                    q_next[i] = w[i] * inv_bj;
                }
                basis.push(q_next);
            }
        }

        // Diagonalise T_{actual_k}
        if actual_k == 0 {
            continue;
        }
        use nalgebra::{DMatrix, SymmetricEigen};
        let mut t_mat = DMatrix::zeros(actual_k, actual_k);
        for i in 0..actual_k {
            t_mat[(i, i)] = alpha[i];
            if i + 1 < actual_k {
                t_mat[(i, i + 1)] = beta[i];
                t_mat[(i + 1, i)] = beta[i];
            }
        }
        let eig = SymmetricEigen::new(t_mat);

        // Σ_j (Q[0,j])² f(θ_j) with f(0):=0.
        let mut sample = 0.0f64;
        for j in 0..actual_k {
            let theta = eig.eigenvalues[j];
            if theta > 0.0 {
                let q0j = eig.eigenvectors[(0, j)];
                sample += q0j * q0j * (-theta * theta.ln());
            }
        }
        total += z_norm_sq * sample;
    }

    let result = total / m as f64;
    result.max(0.0)
}
