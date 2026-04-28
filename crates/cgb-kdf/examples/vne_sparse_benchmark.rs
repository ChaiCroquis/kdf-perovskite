//! Benchmark: dense vs sparse VNE estimation.
//!
//! Run: `cargo run --release -p cgb-kdf --example vne_sparse_benchmark`
//!
//! Reports timing and precision for n ∈ {100, 500, 1000, 2000, 5000, 10000} on
//! Erdős–Rényi graphs with average degree ≈ 5. The dense path is skipped above
//! n = 2000 because `SymmetricEigen` becomes prohibitive.

use cgb_kdf::vne::{entropy, sparse};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use std::time::Instant;

fn er_graph(n: usize, p: f64, seed: u64) -> Vec<(u32, u32, f64)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut edges = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            if rng.r#gen::<f64>() < p {
                edges.push((i as u32, j as u32, 1.0));
            }
        }
    }
    edges
}

fn main() {
    println!(
        "{:>6} | {:>7} | {:>14} | {:>14} | {:>10} | {:>10} | {:>10}",
        "n", "|E|", "S_dense", "S_sparse", "rel_err", "t_dense_ms", "t_sparse_ms",
    );
    println!(
        "{:-<7}+{:-<9}+{:-<16}+{:-<16}+{:-<12}+{:-<12}+{:-<12}",
        "", "", "", "", "", "", ""
    );

    for &n in &[100usize, 500, 1000, 2000, 5000, 10000] {
        // Average degree ≈ 5: p = 5 / (n - 1)
        let p = 5.0 / (n as f64 - 1.0);
        let edges = er_graph(n, p, 42 + n as u64);
        let m = edges.len();

        let (s_dense, t_dense_ms) = if n <= 2000 {
            let t = Instant::now();
            let result = entropy::von_neumann_entropy_dense(n, &edges);
            (
                Some(result.entropy),
                Some(t.elapsed().as_secs_f64() * 1000.0),
            )
        } else {
            (None, None)
        };

        let t = Instant::now();
        let s_sparse = sparse::von_neumann_entropy_sparse(n, &edges);
        let t_sparse_ms = t.elapsed().as_secs_f64() * 1000.0;

        let (s_dense_str, rel_err_str, t_dense_str) = match (s_dense, t_dense_ms) {
            (Some(sd), Some(td)) => {
                let rel = (sd - s_sparse).abs() / sd.abs().max(1e-12);
                (
                    format!("{:>14.6}", sd),
                    format!("{:>10.3e}", rel),
                    format!("{:>10.2}", td),
                )
            }
            _ => (
                format!("{:>14}", "—"),
                format!("{:>10}", "—"),
                format!("{:>10}", "—"),
            ),
        };

        println!(
            "{:>6} | {:>7} | {} | {:>14.6} | {} | {} | {:>10.2}",
            n, m, s_dense_str, s_sparse, rel_err_str, t_dense_str, t_sparse_ms,
        );
    }
}
