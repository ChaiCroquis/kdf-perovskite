//! Structural fingerprint engine implementation

use std::collections::HashMap;
use nalgebra::DMatrix;

use super::types::{Fingerprint, FingerprintKey, NodeLabel, CacheStats, hash_to_seed};
use super::precomputed::PrecomputedFingerprint;
use super::rng::SimpleRng;

/// Structural Fingerprint Engine
///
/// Generates fingerprints based on Graph Laplacian eigenvalues.
pub struct StructuralFingerprintEngine {
    /// Dimension of fingerprint vectors
    pub fingerprint_dim: usize,
    /// Systematic similarity weight (0.5-0.9, optimal: 0.7)
    pub w_sys: f64,
    /// Relational similarity weight (0.1-0.3, optimal: 0.2)
    pub w_rel: f64,
    /// Attribute similarity weight (0.05-0.2, optimal: 0.1)
    pub w_attr: f64,
    /// Fingerprint cache
    cache: HashMap<FingerprintKey, Fingerprint>,
    /// Statistics
    stats: CacheStats,
}

impl StructuralFingerprintEngine {
    /// Create a new fingerprint engine
    ///
    /// # Arguments
    /// * `fingerprint_dim` - Dimension of fingerprint vectors (default: 32)
    /// * `w_sys` - Systematic similarity weight (0.5-0.9, optimal: 0.7)
    /// * `w_rel` - Relational similarity weight (0.1-0.3, optimal: 0.2)
    /// * `w_attr` - Attribute similarity weight (0.05-0.2, optimal: 0.1)
    pub fn new(fingerprint_dim: usize, w_sys: f64, w_rel: f64, w_attr: f64) -> Self {
        Self {
            fingerprint_dim,
            w_sys,
            w_rel,
            w_attr,
            cache: HashMap::new(),
            stats: CacheStats::default(),
        }
    }
}

impl Default for StructuralFingerprintEngine {
    fn default() -> Self {
        Self::new(32, 0.7, 0.2, 0.1)
    }
}

impl StructuralFingerprintEngine {

    /// Compute fingerprint for a node
    ///
    /// # Arguments
    /// * `node_id` - Node identifier
    /// * `label` - Node label
    /// * `ego_laplacian` - Optional ego graph Laplacian matrix
    pub fn compute_fingerprint(
        &mut self,
        node_id: &str,
        label: &NodeLabel,
        ego_laplacian: Option<&DMatrix<f64>>,
    ) -> Fingerprint {
        let cache_key = FingerprintKey {
            node_id: node_id.to_string(),
            label: label.as_str().to_string(),
        };

        // Check cache
        if let Some(fp) = self.cache.get(&cache_key) {
            self.stats.cache_hits += 1;
            return fp.clone();
        }

        self.stats.total_computations += 1;

        // Generate deterministic seed from node ID
        let seed = hash_to_seed(node_id);
        let mut rng = SimpleRng::new(seed);

        // Generate base fingerprint
        let base = if let Some(laplacian) = ego_laplacian {
            self.eigenvalue_fingerprint(laplacian, &mut rng)
        } else {
            self.random_fingerprint(&mut rng)
        };

        // Apply label-specific pattern
        let fp = self.apply_label_pattern(&base, label, &mut rng);

        // Cache and return
        self.cache.insert(cache_key, fp.clone());
        fp
    }

    /// Compute fingerprint from adjacency list (ego graph)
    pub fn compute_from_ego_graph(
        &mut self,
        node_id: &str,
        label: &NodeLabel,
        neighbors: &[(String, Vec<String>)], // (node, its_neighbors)
    ) -> Fingerprint {
        if neighbors.len() < 2 {
            return self.compute_fingerprint(node_id, label, None);
        }

        // Build Laplacian matrix from ego graph
        let laplacian = self.build_laplacian(neighbors);
        self.compute_fingerprint(node_id, label, Some(&laplacian))
    }

    /// Build Laplacian matrix from adjacency
    fn build_laplacian(&self, neighbors: &[(String, Vec<String>)]) -> DMatrix<f64> {
        let n = neighbors.len();
        let mut adj = DMatrix::<f64>::zeros(n, n);

        // Create node index mapping
        let node_idx: HashMap<&str, usize> = neighbors
            .iter()
            .enumerate()
            .map(|(i, (id, _))| (id.as_str(), i))
            .collect();

        // Build adjacency matrix
        for (i, (_, node_neighbors)) in neighbors.iter().enumerate() {
            for neighbor in node_neighbors {
                if let Some(&j) = node_idx.get(neighbor.as_str()) {
                    adj[(i, j)] = 1.0;
                    adj[(j, i)] = 1.0;
                }
            }
        }

        // Compute Laplacian: L = D - A
        let mut laplacian = DMatrix::<f64>::zeros(n, n);
        for i in 0..n {
            let degree: f64 = (0..n).map(|j| adj[(i, j)]).sum();
            laplacian[(i, i)] = degree;
            for j in 0..n {
                if i != j {
                    laplacian[(i, j)] = -adj[(i, j)];
                }
            }
        }

        laplacian
    }

    /// Generate fingerprint from Laplacian eigenvalues
    fn eigenvalue_fingerprint(&self, laplacian: &DMatrix<f64>, rng: &mut SimpleRng) -> Fingerprint {
        // Compute eigenvalues using symmetric eigendecomposition
        let eigen = laplacian.clone().symmetric_eigen();
        let mut eigenvalues: Vec<f64> = eigen.eigenvalues.iter().cloned().collect();
        eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Resize to fingerprint dimension
        self.resize_to_dim(&eigenvalues, rng)
    }

    /// Generate random fingerprint
    fn random_fingerprint(&self, rng: &mut SimpleRng) -> Fingerprint {
        (0..self.fingerprint_dim).map(|_| rng.next_f64()).collect()
    }

    /// Resize array to fixed fingerprint dimension
    fn resize_to_dim(&self, arr: &[f64], rng: &mut SimpleRng) -> Fingerprint {
        if arr.is_empty() {
            return self.random_fingerprint(rng);
        }

        let n = arr.len();
        if n >= self.fingerprint_dim {
            // Downsample
            (0..self.fingerprint_dim)
                .map(|i| {
                    let idx = (i * (n - 1)) / (self.fingerprint_dim - 1).max(1);
                    arr[idx.min(n - 1)]
                })
                .collect()
        } else {
            // Linear interpolation
            (0..self.fingerprint_dim)
                .map(|i| {
                    let t = i as f64 / (self.fingerprint_dim - 1).max(1) as f64;
                    let pos = t * (n - 1) as f64;
                    let idx0 = pos.floor() as usize;
                    let idx1 = (idx0 + 1).min(n - 1);
                    let frac = pos - idx0 as f64;
                    arr[idx0] * (1.0 - frac) + arr[idx1] * frac
                })
                .collect()
        }
    }

    /// Apply label-specific structural patterns
    fn apply_label_pattern(
        &self,
        base: &[f64],
        label: &NodeLabel,
        rng: &mut SimpleRng,
    ) -> Fingerprint {
        let mut fp: Vec<f64> = base.to_vec();

        match label {
            NodeLabel::IsolatedTruth | NodeLabel::Normal => {
                // STRUCTURED pattern: coherent, monotonic, smooth
                fp.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                // Compress to [0.3, 0.7]
                for v in &mut fp {
                    *v = 0.3 + 0.4 * *v;
                }

                // Smooth
                for i in 1..fp.len() {
                    fp[i] = 0.7 * fp[i] + 0.3 * fp[i - 1];
                }
            }
            NodeLabel::Garbage => {
                // CHAOTIC pattern: random, jagged, anti-correlated
                // Shuffle
                for i in (1..fp.len()).rev() {
                    let j = rng.next_usize() % (i + 1);
                    fp.swap(i, j);
                }

                // Anti-correlation
                let mut i = 0;
                while i + 1 < fp.len() {
                    fp[i + 1] = 1.0 - fp[i];
                    i += 2;
                }

                // Add noise
                let freq = rng.next_f64() * 10.0;
                for (i, v) in fp.iter_mut().enumerate() {
                    *v += 0.1 * (i as f64 * freq).sin();
                }
            }
            NodeLabel::Unknown => {
                // Keep as-is
            }
        }

        // Clip to [0, 1]
        fp.iter().map(|&v| v.clamp(0.0, 1.0)).collect()
    }

    /// Fast Euclidean distance for screening (O(d))
    pub fn quick_distance(&self, fp1: &Fingerprint, fp2: &Fingerprint) -> f64 {
        if fp1.len() != fp2.len() {
            return f64::MAX;
        }

        fp1.iter()
            .zip(fp2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt()
    }

    /// Full similarity computation with multiple metrics
    ///
    /// Components:
    /// 1. Cosine similarity (semantic alignment)
    /// 2. Structure similarity (sorted pattern match)
    /// 3. Gradient sign match (trend alignment)
    pub fn full_similarity(&self, fp1: &Fingerprint, fp2: &Fingerprint) -> f64 {
        if fp1.len() != fp2.len() || fp1.is_empty() {
            return 0.0;
        }

        // Cosine similarity
        let dot: f64 = fp1.iter().zip(fp2.iter()).map(|(a, b)| a * b).sum();
        let n1: f64 = fp1.iter().map(|a| a.powi(2)).sum::<f64>().sqrt();
        let n2: f64 = fp2.iter().map(|b| b.powi(2)).sum::<f64>().sqrt();

        let cos_sim = if n1 == 0.0 || n2 == 0.0 {
            0.0
        } else {
            dot / (n1 * n2)
        };

        // Structure similarity
        let mut sorted1 = fp1.clone();
        let mut sorted2 = fp2.clone();
        sorted1.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        sorted2.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let struct_diff: f64 = sorted1
            .iter()
            .zip(sorted2.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
            / fp1.len() as f64;
        let struct_sim = 1.0 - struct_diff;

        // Gradient sign match
        let grad1: Vec<f64> = fp1.windows(2).map(|w| w[1] - w[0]).collect();
        let grad2: Vec<f64> = fp2.windows(2).map(|w| w[1] - w[0]).collect();

        let sign_matches: usize = grad1
            .iter()
            .zip(grad2.iter())
            .filter(|(a, b)| a.signum() == b.signum())
            .count();
        let sign_match = if grad1.is_empty() {
            0.0
        } else {
            sign_matches as f64 / grad1.len() as f64
        };

        // Weighted combination
        0.40 * cos_sim.max(0.0) + 0.35 * struct_sim + 0.25 * sign_match
    }

    /// Fast similarity using precomputed fingerprints with early termination
    ///
    /// This is significantly faster than `full_similarity` when computing
    /// pairwise similarities because:
    /// 1. Norm, sorted values, and gradient signs are precomputed
    /// 2. Early termination if cosine similarity is too low
    ///
    /// # Arguments
    /// * `pfp1` - Precomputed fingerprint 1
    /// * `pfp2` - Precomputed fingerprint 2
    /// * `threshold` - Early termination threshold (skip if unlikely to exceed)
    ///
    /// # Returns
    /// Similarity score in [0, 1], or 0.0 if early terminated
    pub fn fast_similarity(
        &self,
        pfp1: &PrecomputedFingerprint,
        pfp2: &PrecomputedFingerprint,
        threshold: f64,
    ) -> f64 {
        // Early termination: zero norm
        if pfp1.is_zero_norm() || pfp2.is_zero_norm() {
            return 0.0;
        }

        // Early termination: dimension mismatch
        if pfp1.raw.len() != pfp2.raw.len() || pfp1.raw.is_empty() {
            return 0.0;
        }

        // Cosine similarity (using precomputed norms)
        let dot: f64 = pfp1.raw.iter().zip(pfp2.raw.iter()).map(|(a, b)| a * b).sum();
        let cos_sim = dot / (pfp1.norm * pfp2.norm);

        // Early termination: if cosine similarity is too low, final result can't exceed threshold
        // Since final = 0.40*cos + 0.35*struct + 0.25*sign, and struct/sign are at most 1.0,
        // max possible = 0.40*cos + 0.35 + 0.25 = 0.40*cos + 0.60
        // If 0.40*cos + 0.60 < threshold, skip
        if 0.40 * cos_sim.max(0.0) + 0.60 < threshold {
            return 0.0;
        }

        // Structure similarity (using precomputed sorted values - no clone/sort needed!)
        let struct_diff: f64 = pfp1
            .sorted
            .iter()
            .zip(pfp2.sorted.iter())
            .map(|(a, b)| (a - b).abs())
            .sum::<f64>()
            / pfp1.raw.len() as f64;
        let struct_sim = 1.0 - struct_diff;

        // Gradient sign match (using precomputed gradient signs - no windows/map needed!)
        let sign_matches = pfp1
            .gradient_signs
            .iter()
            .zip(pfp2.gradient_signs.iter())
            .filter(|(a, b)| **a == **b)
            .count();
        let sign_match = if pfp1.gradient_signs.is_empty() {
            0.0
        } else {
            sign_matches as f64 / pfp1.gradient_signs.len() as f64
        };

        // Weighted combination
        0.40 * cos_sim.max(0.0) + 0.35 * struct_sim + 0.25 * sign_match
    }

    /// Batch precompute fingerprints for efficient pairwise comparison
    ///
    /// Use this before computing many pairwise similarities.
    pub fn precompute_batch(&self, fingerprints: &[Fingerprint]) -> Vec<PrecomputedFingerprint> {
        fingerprints
            .iter()
            .map(PrecomputedFingerprint::from_fingerprint)
            .collect()
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            total_computations: self.stats.total_computations,
            cache_hits: self.stats.cache_hits,
            cache_size: self.cache.len(),
        }
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
        self.stats = CacheStats::default();
    }
}
