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
        "{:>6} | {:>7} | {:>10} | {:>10} | {:>10} | {:>9} | {:>9} | {:>9}",
        "n", "|E|", "S_dense", "S_cheb", "S_slq", "err_cheb", "err_slq", "t_slq_ms",
    );
    println!(
        "{:-<7}+{:-<9}+{:-<12}+{:-<12}+{:-<12}+{:-<11}+{:-<11}+{:-<11}",
        "", "", "", "", "", "", "", ""
    );

    for &n in &[100usize, 500, 1000, 2000, 5000, 10000] {
        let p = 5.0 / (n as f64 - 1.0);
        let edges = er_graph(n, p, 42 + n as u64);
        let m = edges.len();

        let s_dense = if n <= 2000 {
            Some(entropy::von_neumann_entropy_dense(n, &edges).entropy)
        } else {
            None
        };

        let t = Instant::now();
        let s_cheb = sparse::von_neumann_entropy_sparse(n, &edges);
        let t_cheb = t.elapsed().as_secs_f64() * 1000.0;

        let t = Instant::now();
        let s_slq = sparse::von_neumann_entropy_slq(n, &edges);
        let t_slq = t.elapsed().as_secs_f64() * 1000.0;

        let (s_dense_str, err_cheb_str, err_slq_str) = match s_dense {
            Some(sd) => {
                let ec = (sd - s_cheb).abs() / sd.abs().max(1e-12);
                let es = (sd - s_slq).abs() / sd.abs().max(1e-12);
                (
                    format!("{:>10.6}", sd),
                    format!("{:>9.3e}", ec),
                    format!("{:>9.3e}", es),
                )
            }
            None => (
                format!("{:>10}", "—"),
                format!("{:>9}", "—"),
                format!("{:>9}", "—"),
            ),
        };

        println!(
            "{:>6} | {:>7} | {} | {:>10.6} | {:>10.6} | {} | {} | {:>9.2}",
            n, m, s_dense_str, s_cheb, s_slq, err_cheb_str, err_slq_str, t_slq,
        );
        let _ = t_cheb; // kept for potential future timing column
    }
}
