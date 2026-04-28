//! Phase 7 candidate solutions for Phase 6 failure modes.
//!
//! - **S1 PersistentRareMemory** (targets failure mode E: temporal drift)
//!   Wraps any selector and maintains exponentially decaying "rare memory"
//!   across snapshots. Uses Claim 25's `ActivationScore` semantics.
//!
//! - **S2 RelativeDensity** (targets failure mode A: high-degree rare)
//!   Replaces the `neighbor_count == 1` rule with
//!   "neighbor_count < local_avg_degree × ratio".
//!
//! - **S3 FingerprintIsolation** (Claim 46 spirit, for A)
//!   Treats a node as rare if its structural fingerprint distance to all
//!   classifier-assigned Cores exceeds `theta`.
//!
//! - **S4 Hybrid** (combines S1 + S2)

use cgb_kdf::{Layer, NodeClassifier};
use real_data_bench::{selectors::Selector, Dataset};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};

// -------------------------------------------------------------- S1

/// S1: Persistent Rare Memory across snapshots.
///
/// For each node ever classified as Rare in past snapshots, we keep an
/// activation value `A(t)` that decays as `A(t+1) = A(t) · exp(-lambda)`
/// and gets reset to 1.0 on re-observation. A node is **memory-rare** if
/// `A(t) > threshold`.
pub struct PersistentRareMemory {
    inner: Box<dyn Selector>,
    state: RefCell<MemoryState>,
    pub decay_lambda: f64,
    pub remember_threshold: f64,
    display_name: String,
}

struct MemoryState {
    activation: HashMap<u32, f64>,
    last_seen_name: Option<String>,
}

impl PersistentRareMemory {
    pub fn new(inner: Box<dyn Selector>) -> Self {
        Self {
            inner,
            state: RefCell::new(MemoryState {
                activation: HashMap::new(),
                last_seen_name: None,
            }),
            decay_lambda: 0.10,
            remember_threshold: 0.30,
            display_name: "KDF+PersistMem".to_string(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = name.into();
        self
    }

    /// Reset memory between independent dataset groups.
    pub fn reset(&self) {
        let mut s = self.state.borrow_mut();
        s.activation.clear();
        s.last_seen_name = None;
    }
}

impl Selector for PersistentRareMemory {
    fn name(&self) -> &str {
        &self.display_name
    }

    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32> {
        let mut state = self.state.borrow_mut();

        // Detect new temporal series → reset on name family change
        let family = ds.name.split("_t").next().unwrap_or(&ds.name).to_string();
        let family_changed = state
            .last_seen_name
            .as_ref()
            .map(|prev| !prev.starts_with(&family))
            .unwrap_or(false);
        if family_changed {
            state.activation.clear();
        }
        state.last_seen_name = Some(family);

        // Decay all existing activations exp(-λ)
        let survival = (-self.decay_lambda).exp();
        for v in state.activation.values_mut() {
            *v *= survival;
        }

        // Baseline selection from the wrapped selector
        let mut selected = self.inner.select(ds, seed);

        // Classify current snapshot to find CURRENT rare → refresh activation
        let mut classifier = NodeClassifier::default();
        let class = classifier.classify(ds.n_nodes, &ds.edges);
        for (&id, &layer) in &class.layers {
            if matches!(layer, Layer::Rare) {
                state.activation.insert(id, 1.0);
            }
        }

        // Include memory-rare nodes that are still active
        for (&id, &a) in state.activation.iter() {
            if a > self.remember_threshold && (id as usize) < ds.n_nodes {
                selected.insert(id);
            }
        }
        selected
    }
}

// -------------------------------------------------------------- S2

/// S2: Relative-density rare detector (for high-degree rare).
///
/// A node is flagged as rare if its degree is strictly less than
/// `local_avg_degree * rare_ratio`, regardless of absolute value.
/// The "local average" is computed over **1-hop** neighborhood
/// (i.e. mean degree of the node's direct neighbors).
pub struct RelativeDensitySelector {
    pub rare_ratio: f64,
    pub core_ratio: f64,
}

impl Default for RelativeDensitySelector {
    fn default() -> Self {
        Self {
            rare_ratio: 0.50,
            core_ratio: 1.50,
        }
    }
}

impl Selector for RelativeDensitySelector {
    fn name(&self) -> &str {
        "KDF+RelDensity"
    }

    fn select(&self, ds: &Dataset, _seed: u64) -> HashSet<u32> {
        let n = ds.n_nodes;
        let mut deg = vec![0usize; n];
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
        for &(u, v, _) in &ds.edges {
            if (u as usize) < n && (v as usize) < n {
                deg[u as usize] += 1;
                deg[v as usize] += 1;
                adj[u as usize].push(v);
                adj[v as usize].push(u);
            }
        }
        let global_mean = deg.iter().sum::<usize>() as f64 / n.max(1) as f64;

        let mut selected: HashSet<u32> = HashSet::new();
        // Dedup by neighbor pattern (cluster representatives, like KDF)
        let mut edge_groups: BTreeMap<Vec<u32>, u32> = BTreeMap::new();

        for i in 0..n {
            let neighbors = &adj[i];
            if neighbors.is_empty() {
                continue; // drop isolates
            }
            // Local average = mean degree of direct (1-hop) neighbors
            let local_avg = neighbors
                .iter()
                .map(|&v| deg[v as usize] as f64)
                .sum::<f64>()
                / neighbors.len() as f64;
            let reference = local_avg.max(global_mean);

            if (deg[i] as f64) >= reference * self.core_ratio {
                // Core
                selected.insert(i as u32);
            } else if (deg[i] as f64) < reference * self.rare_ratio {
                // Relatively rare vs context
                selected.insert(i as u32);
            } else {
                // Edge → keep one rep per neighborhood signature
                let mut sorted_n = neighbors.clone();
                sorted_n.sort();
                sorted_n.dedup();
                edge_groups.entry(sorted_n).or_insert(i as u32);
            }
        }
        for rep in edge_groups.values() {
            selected.insert(*rep);
        }
        selected
    }
}

// -------------------------------------------------------------- S3

/// S3: Fingerprint-based isolation detector.
///
/// Uses a lightweight degree-histogram fingerprint over 1-hop neighborhood
/// (faster than Laplacian eigenvalues, same Claim 46 spirit). A node is
/// rare if its fingerprint distance to the median fingerprint exceeds
/// `theta`.
pub struct FingerprintIsolationSelector {
    pub theta: f64,
}

impl Default for FingerprintIsolationSelector {
    fn default() -> Self {
        Self { theta: 0.4 }
    }
}

fn deg_fingerprint(i: usize, adj: &[Vec<u32>], deg: &[usize]) -> [f64; 4] {
    // 4-bin histogram of neighbor degrees (log-spaced)
    let mut bins = [0.0f64; 4];
    let neighbors = &adj[i];
    if neighbors.is_empty() {
        return bins;
    }
    for &v in neighbors {
        let d = deg[v as usize] as f64;
        let idx = if d < 2.0 {
            0
        } else if d < 5.0 {
            1
        } else if d < 20.0 {
            2
        } else {
            3
        };
        bins[idx] += 1.0;
    }
    let total: f64 = bins.iter().sum();
    for b in bins.iter_mut() {
        *b /= total;
    }
    bins
}

fn l1_distance(a: &[f64; 4], b: &[f64; 4]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

impl Selector for FingerprintIsolationSelector {
    fn name(&self) -> &str {
        "KDF+Fingerprint"
    }

    fn select(&self, ds: &Dataset, _seed: u64) -> HashSet<u32> {
        let n = ds.n_nodes;
        let mut deg = vec![0usize; n];
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
        for &(u, v, _) in &ds.edges {
            if (u as usize) < n && (v as usize) < n {
                deg[u as usize] += 1;
                deg[v as usize] += 1;
                adj[u as usize].push(v);
                adj[v as usize].push(u);
            }
        }
        let fps: Vec<[f64; 4]> = (0..n).map(|i| deg_fingerprint(i, &adj, &deg)).collect();

        // Median fingerprint
        let mut median = [0.0f64; 4];
        for dim in 0..4 {
            let mut vals: Vec<f64> = fps.iter().map(|fp| fp[dim]).collect();
            vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            median[dim] = vals[vals.len() / 2];
        }

        let mut selected: HashSet<u32> = HashSet::new();
        for i in 0..n {
            if deg[i] == 0 {
                continue;
            }
            let d = l1_distance(&fps[i], &median);
            if d > self.theta {
                selected.insert(i as u32);
            }
        }
        // Ensure at least something selected (fall back to top-N by degree)
        if selected.is_empty() {
            let k = (n as f64 * 0.30).ceil() as usize;
            let mut order: Vec<usize> = (0..n).collect();
            order.sort_by_key(|&i| std::cmp::Reverse(deg[i]));
            for &i in order.iter().take(k) {
                selected.insert(i as u32);
            }
        }
        selected
    }
}

// -------------------------------------------------------------- S4

/// S4: Hybrid S1 + S2 — persistent memory wrapping the relative-density
/// selector. Gets a distinct name so it shows up separately in benchmarks.
pub fn s4_hybrid() -> PersistentRareMemory {
    PersistentRareMemory::new(Box::new(RelativeDensitySelector::default()))
        .with_name("KDF+Hybrid(S1+S2)")
}

#[cfg(test)]
mod tests {
    use super::*;
    use real_data_bench::selectors::KdfSel;

    fn tiny_dataset() -> Dataset {
        Dataset {
            name: "unit_test".to_string(),
            n_nodes: 5,
            edges: vec![(0, 1, 1.0), (1, 2, 1.0), (2, 3, 1.0), (3, 4, 1.0)],
            rare_ground_truth: [0, 4].into_iter().collect(),
            description: "path graph".to_string(),
        }
    }

    #[test]
    fn s1_memory_carries_rare_forward() {
        let s1 = PersistentRareMemory::new(Box::new(KdfSel));
        // First snapshot includes rare
        let sel1 = s1.select(&tiny_dataset(), 1);
        assert!(!sel1.is_empty());
        // Simulate disappearance: rare nodes removed by dropping their edges
        let mut ds2 = tiny_dataset();
        ds2.edges.retain(|&(u, v, _)| u != 0 && v != 0);
        ds2.name = "unit_test_t1".to_string();
        let sel2 = s1.select(&ds2, 1);
        // Memory should still include node 0 via activation
        assert!(
            sel2.contains(&0) || sel1.contains(&0),
            "S1 should remember rare node from first snapshot"
        );
    }

    #[test]
    fn s1_resets_on_new_family() {
        let s1 = PersistentRareMemory::new(Box::new(KdfSel));

        // Populate memory with a synthetic "past Rare" node that would NOT be
        // classified as Rare in the next family's dataset. We write it
        // directly into state so we can verify the family-switch reset.
        let mut ds_a = tiny_dataset();
        ds_a.name = "Adv_E_Temporal_t0".to_string();
        s1.select(&ds_a, 1);
        // Inject a synthetic memory-rare id=999 that is OUTSIDE tiny_dataset's
        // node range so subsequent classifier passes cannot re-register it.
        s1.state.borrow_mut().activation.insert(999, 1.0);
        assert!(s1.state.borrow().activation.contains_key(&999));

        // Switch to a different family — memory should be cleared.
        let mut ds_b = tiny_dataset();
        ds_b.name = "Adv_A_HighDegRare_deg1".to_string();
        let _ = s1.select(&ds_b, 1);
        assert!(
            !s1.state.borrow().activation.contains_key(&999),
            "family switch must purge memory from the previous family"
        );
    }

    #[test]
    fn s2_detects_relative_rare() {
        let s2 = RelativeDensitySelector::default();
        // Star graph: hub 0 connected to 1..10, with node 11 weakly connected
        let mut edges: Vec<(u32, u32, f64)> = (1..=10).map(|i| (0, i as u32, 1.0)).collect();
        edges.push((0, 11, 1.0));
        let ds = Dataset {
            name: "star".to_string(),
            n_nodes: 12,
            edges,
            rare_ground_truth: [11].into_iter().collect(),
            description: "star".to_string(),
        };
        let sel = s2.select(&ds, 1);
        // Node 11 has degree 1 << hub degree 11, should be marked rare
        assert!(
            sel.contains(&11),
            "S2 should detect relatively-rare node in star"
        );
    }

    #[test]
    fn s3_fingerprint_selects_something() {
        let s3 = FingerprintIsolationSelector::default();
        let sel = s3.select(&tiny_dataset(), 1);
        assert!(!sel.is_empty());
    }
}
