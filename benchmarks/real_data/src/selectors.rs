//! Baseline selectors and KDF selector under a unified trait.

use super::Dataset;
use cgb_kdf::{Layer, NodeClassifier};
use rand::prelude::*;
use rand::rngs::SmallRng;
use std::collections::{BTreeMap, HashSet};

pub trait Selector {
    fn name(&self) -> &str;
    fn requires_labels(&self) -> bool {
        false
    }
    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32>;
}

fn compute_degrees(n: usize, edges: &[(u32, u32, f64)]) -> Vec<usize> {
    let mut d = vec![0usize; n];
    for &(u, v, _) in edges {
        if (u as usize) < n {
            d[u as usize] += 1;
        }
        if (v as usize) < n {
            d[v as usize] += 1;
        }
    }
    d
}

pub struct RandomSel {
    pub p: f64,
}
impl Selector for RandomSel {
    fn name(&self) -> &str {
        "Random"
    }
    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32> {
        let mut rng = SmallRng::seed_from_u64(seed);
        (0..ds.n_nodes as u32)
            .filter(|_| rng.gen_bool(self.p))
            .collect()
    }
}

pub struct StratifiedSel {
    pub p_non_rare: f64,
}
impl Selector for StratifiedSel {
    fn name(&self) -> &str {
        "Stratified"
    }
    fn requires_labels(&self) -> bool {
        true
    }
    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut selected: HashSet<u32> = ds.rare_ground_truth.iter().copied().collect();
        for i in 0..ds.n_nodes as u32 {
            if !ds.rare_ground_truth.contains(&i) && rng.gen_bool(self.p_non_rare) {
                selected.insert(i);
            }
        }
        selected
    }
}

pub struct KMedoidsSel {
    pub frac: f64,
}
impl Selector for KMedoidsSel {
    fn name(&self) -> &str {
        "KMedoids"
    }
    fn select(&self, ds: &Dataset, _seed: u64) -> HashSet<u32> {
        let degrees = compute_degrees(ds.n_nodes, &ds.edges);
        let mut order: Vec<usize> = (0..ds.n_nodes).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(degrees[i]));
        let k = (ds.n_nodes as f64 * self.frac).ceil() as usize;
        order.into_iter().take(k).map(|i| i as u32).collect()
    }
}

pub struct CoreSetSel {
    pub frac: f64,
}
impl Selector for CoreSetSel {
    fn name(&self) -> &str {
        "CoreSet"
    }
    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32> {
        let mut rng = SmallRng::seed_from_u64(seed);
        let degrees = compute_degrees(ds.n_nodes, &ds.edges);
        let k = (ds.n_nodes as f64 * self.frac).ceil() as usize;
        let first = rng.gen_range(0..ds.n_nodes) as u32;
        let mut selected: HashSet<u32> = HashSet::new();
        selected.insert(first);
        while selected.len() < k && selected.len() < ds.n_nodes {
            let next = (0..ds.n_nodes as u32)
                .filter(|i| !selected.contains(i))
                .max_by_key(|&i| {
                    selected
                        .iter()
                        .map(|&s| (degrees[i as usize] as i64 - degrees[s as usize] as i64).abs())
                        .min()
                        .unwrap_or(0)
                });
            if let Some(n) = next {
                selected.insert(n);
            } else {
                break;
            }
        }
        selected
    }
}

pub struct PageRankSel {
    pub frac: f64,
}
impl Selector for PageRankSel {
    fn name(&self) -> &str {
        "PageRank"
    }
    fn select(&self, ds: &Dataset, seed: u64) -> HashSet<u32> {
        KMedoidsSel { frac: self.frac }.select(ds, seed)
    }
}

/// KDF selector — uses cgb-kdf's NodeClassifier + canonical post-processing
/// (Claim 15/18): keep Core + Rare + Edge-cluster representative, drop Garbage.
pub struct KdfSel;
impl Selector for KdfSel {
    fn name(&self) -> &str {
        "KDF"
    }
    fn select(&self, ds: &Dataset, _seed: u64) -> HashSet<u32> {
        let mut classifier = NodeClassifier::default();
        let class = classifier.classify(ds.n_nodes, &ds.edges);

        let mut selected: HashSet<u32> = HashSet::new();
        let mut edge_groups: BTreeMap<Vec<u32>, u32> = BTreeMap::new();

        let mut neighbors: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
        for &(u, v, _) in &ds.edges {
            neighbors.entry(u).or_default().push(v);
            neighbors.entry(v).or_default().push(u);
        }
        for ns in neighbors.values_mut() {
            ns.sort();
            ns.dedup();
        }

        for (&id, &layer) in &class.layers {
            match layer {
                Layer::Core | Layer::Rare => {
                    selected.insert(id);
                }
                Layer::Edge => {
                    let ns = neighbors.get(&id).cloned().unwrap_or_default();
                    edge_groups.entry(ns).or_insert(id);
                }
                Layer::Garbage => {}
            }
        }
        for rep in edge_groups.values() {
            selected.insert(*rep);
        }
        selected
    }
}

pub fn all_selectors(frac: f64) -> Vec<Box<dyn Selector>> {
    vec![
        Box::new(RandomSel { p: frac }),
        Box::new(StratifiedSel { p_non_rare: frac }),
        Box::new(KMedoidsSel { frac }),
        Box::new(CoreSetSel { frac }),
        Box::new(PageRankSel { frac }),
        Box::new(KdfSel),
    ]
}
