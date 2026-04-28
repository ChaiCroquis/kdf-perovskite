//! Phase 7 scaling benchmark — measure wall-clock vs n for each method.
//! Tests the "O(n log n)" claim from the patent/marketing.
//!
//! CoreSet is excluded because its farthest-first heuristic is O(n²),
//! which is expected to scale quadratically.

use adversarial_bench as adv;
use real_data_bench::selectors::{KMedoidsSel, KdfSel, RandomSel, Selector};
use std::time::Instant;

fn main() {
    let sizes = [500_usize, 1_000, 2_000, 5_000, 10_000, 20_000, 50_000];
    let seed = 42;
    let selectors: Vec<(String, Box<dyn Selector>)> = vec![
        ("Random".to_string(), Box::new(RandomSel { p: 0.30 })),
        ("KMedoids".to_string(), Box::new(KMedoidsSel { frac: 0.30 })),
        ("KDF".to_string(), Box::new(KdfSel)),
    ];

    println!(
        "| n | method | build_ms | select_ms | total_ms | selected | ns/node | ns/(n·log2 n) |"
    );
    println!("|---:|---|---:|---:|---:|---:|---:|---:|");

    for &n in &sizes {
        let build_start = Instant::now();
        let ds = adv::high_degree_rare(n, 1, seed);
        let build_ms = build_start.elapsed().as_secs_f64() * 1000.0;

        for (name, sel) in &selectors {
            let start = Instant::now();
            let selected = sel.select(&ds, seed);
            let sel_ms = start.elapsed().as_secs_f64() * 1000.0;
            let n_f = n as f64;
            let ns_per_node = (sel_ms * 1e6) / n_f;
            let ns_per_nlogn = (sel_ms * 1e6) / (n_f * n_f.log2());
            println!(
                "| {} | {} | {:.2} | {:.2} | {:.2} | {} | {:.1} | {:.2} |",
                n,
                name,
                build_ms,
                sel_ms,
                build_ms + sel_ms,
                selected.len(),
                ns_per_node,
                ns_per_nlogn
            );
        }
    }

    println!("\nInterpretation:");
    println!("- If O(n log n), then ns/(n·log2 n) should be ~constant across n.");
    println!("- If O(n), then ns/node should be ~constant, and ns/(n·log2 n) decreasing.");
    println!("- If O(n²), then ns/node grows linearly with n.");
}
