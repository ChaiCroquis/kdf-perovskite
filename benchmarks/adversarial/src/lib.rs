//! Phase 6 adversarial data generators.
//!
//! Each generator is designed to **stress a known KDF assumption**. The
//! benchmark runner applies all 6 methods to each adversarial dataset
//! and reports where KDF breaks (or holds).

pub mod solutions;

use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::Dataset;
use std::collections::HashSet;

/// (A) High-degree rare: break classifier assumption `neighbor_count == 1`.
pub fn high_degree_rare(n: usize, rare_degree: usize, seed: u64) -> Dataset {
    let mut rng = SmallRng::seed_from_u64(seed);
    let n_hubs = (n / 50).max(2);
    let n_rare = (n / 20).max(1);

    let mut edges = Vec::new();
    let mut rare = HashSet::new();

    // Dense hubs
    for u in 0..n_hubs as u32 {
        for v in (u + 1)..n_hubs as u32 {
            edges.push((u, v, 1.0));
        }
    }

    // Rare nodes with rare_degree connections — defeats deg=1 heuristic
    for i in 0..n_rare {
        let id = (n_hubs + i) as u32;
        let mut used = HashSet::new();
        for _ in 0..rare_degree {
            let h = rng.gen_range(0..n_hubs) as u32;
            if used.insert(h) {
                edges.push((id, h, 1.0));
            }
        }
        rare.insert(id);
    }

    // Tail filler
    for i in (n_hubs + n_rare)..n {
        let id = i as u32;
        for _ in 0..3 {
            let h = rng.gen_range(0..n_hubs) as u32;
            edges.push((id, h, 1.0));
        }
    }

    Dataset {
        name: format!("Adv_A_HighDegRare_deg{}", rare_degree),
        n_nodes: n,
        edges,
        rare_ground_truth: rare,
        description: format!(
            "Adversarial (A): rare nodes have degree {} (defeats deg-1 heuristic)",
            rare_degree
        ),
    }
}

/// (B) Structurally isolated rare: same degree as tail but disconnected
/// from the main component; only reachable via shared attribute similarity.
pub fn structurally_isolated(n: usize, rare_degree: usize, seed: u64) -> Dataset {
    let mut rng = SmallRng::seed_from_u64(seed);
    let n_hubs = (n / 50).max(2);
    let n_rare = (n / 20).max(1);

    let mut edges = Vec::new();
    let mut rare = HashSet::new();

    // Main component (hubs + tail)
    for u in 0..n_hubs as u32 {
        for v in (u + 1)..n_hubs as u32 {
            edges.push((u, v, 1.0));
        }
    }
    for i in (n_hubs + n_rare)..n {
        let id = i as u32;
        for _ in 0..3 {
            let h = rng.gen_range(0..n_hubs) as u32;
            edges.push((id, h, 1.0));
        }
    }

    // Rare island: n_rare nodes connected only among themselves (isolated subgraph)
    let rare_start = n_hubs;
    for i in 0..n_rare {
        let id = (rare_start + i) as u32;
        rare.insert(id);
        // Each rare connects to rare_degree other rares
        for _ in 0..rare_degree {
            let other_idx = rng.gen_range(0..n_rare);
            if other_idx != i {
                edges.push((id, (rare_start + other_idx) as u32, 1.0));
            }
        }
    }

    Dataset {
        name: format!("Adv_B_Isolated_deg{}", rare_degree),
        n_nodes: n,
        edges,
        rare_ground_truth: rare,
        description: format!(
            "Adversarial (B): rare forms an isolated component of degree {}",
            rare_degree
        ),
    }
}

/// (C) Zero redundancy: every edge unique, no compression gain.
pub fn zero_redundancy(n: usize, seed: u64) -> Dataset {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut edges = Vec::new();
    let mut seen = HashSet::new();
    // Random sparse graph with no duplicates
    while edges.len() < 2 * n {
        let u = rng.gen_range(0..n) as u32;
        let v = rng.gen_range(0..n) as u32;
        if u == v {
            continue;
        }
        let key = if u < v { (u, v) } else { (v, u) };
        if seen.insert(key) {
            edges.push((u, v, 1.0));
        }
    }
    // Rare = nodes with degree 1 (naturally occurring in sparse random graph)
    let mut deg = vec![0u32; n];
    for &(u, v, _) in &edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
    }
    let rare: HashSet<u32> = deg
        .iter()
        .enumerate()
        .filter(|(_, &d)| d == 1)
        .map(|(i, _)| i as u32)
        .collect();
    Dataset {
        name: "Adv_C_ZeroRedundancy".to_string(),
        n_nodes: n,
        edges,
        rare_ground_truth: rare,
        description: "Adversarial (C): sparse random graph, no duplicate clusters".to_string(),
    }
}

/// (D) Noisy edges: 10% random edges added to a clean graph.
pub fn noisy_edges(n: usize, noise_rate: f64, seed: u64) -> Dataset {
    let mut base = high_degree_rare(n, 1, seed);
    let mut rng = SmallRng::seed_from_u64(seed.wrapping_add(1));
    let n_noise = (base.edges.len() as f64 * noise_rate).ceil() as usize;
    for _ in 0..n_noise {
        let u = rng.gen_range(0..n) as u32;
        let v = rng.gen_range(0..n) as u32;
        if u != v {
            base.edges.push((u, v, 0.5));
        }
    }
    base.name = format!("Adv_D_Noise{}pct", (noise_rate * 100.0) as usize);
    base.description = format!(
        "Adversarial (D): high-deg-rare graph + {}% random noise edges",
        noise_rate * 100.0
    );
    base
}

/// (E) Temporal evolution: snapshot at `step` of an evolving graph.
/// Returns a sequence of datasets; caller picks which step to bench.
pub fn temporal_snapshots(n: usize, n_steps: usize, seed: u64) -> Vec<Dataset> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut snapshots = Vec::new();
    let n_rare = (n / 20).max(1);
    let n_hubs = (n / 50).max(2);

    // Initialize with hubs
    for u in 0..n_hubs as u32 {
        for v in (u + 1)..n_hubs as u32 {
            edges.push((u, v, 1.0));
        }
    }

    for step in 0..n_steps {
        // Add tail+rare nodes incrementally
        let n_new = (n - n_hubs) / n_steps;
        let start = n_hubs + step * n_new;
        let end = (start + n_new).min(n);
        for id in start..end {
            let degree = if id < n_hubs + n_rare { 1 } else { 3 };
            for _ in 0..degree {
                let h = rng.gen_range(0..n_hubs) as u32;
                edges.push((id as u32, h, 1.0));
            }
        }
        // Also random edge decay (30% of oldest edges removed per step)
        let keep = (edges.len() as f64 * 0.85) as usize;
        if edges.len() > keep {
            edges.drain(..edges.len() - keep);
        }

        let rare: HashSet<u32> = ((n_hubs as u32)..((n_hubs + n_rare) as u32)).collect();
        snapshots.push(Dataset {
            name: format!("Adv_E_Temporal_t{}", step),
            n_nodes: end,
            edges: edges.clone(),
            rare_ground_truth: rare,
            description: format!(
                "Adversarial (E): temporal snapshot at t={}/{}",
                step, n_steps
            ),
        });
    }
    snapshots
}

/// (F) Large scale: n=100,000 (small regime). Caller can pass larger `n`.
pub fn large_scale(n: usize, seed: u64) -> Dataset {
    high_degree_rare(n, 1, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_six_generators_produce_rare_truth() {
        let a = high_degree_rare(200, 3, 42);
        assert!(a.n_rare() > 0, "A must have rare truth");

        let b = structurally_isolated(200, 3, 42);
        assert!(b.n_rare() > 0);

        let c = zero_redundancy(200, 42);
        assert!(!c.edges.is_empty());

        let d = noisy_edges(200, 0.10, 42);
        assert!(d.n_rare() > 0);

        let e = temporal_snapshots(200, 5, 42);
        assert_eq!(e.len(), 5);

        let f = large_scale(500, 42);
        assert!(f.n_rare() > 0);
    }
}
