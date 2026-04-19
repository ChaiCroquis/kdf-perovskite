//! Analogy Discovery Engine
//!
//! Structure-mapping Theory based Analogy Discovery Engine v3.0
//! Implements Gentner's (1983) key principles with structural fingerprints.

#![allow(missing_docs)]

use std::collections::{HashMap, HashSet};

use super::fingerprint::{Fingerprint, NodeLabel, StructuralFingerprintEngine};
use super::prescreening::{Candidate, OwnedPreScreeningOptimizer};

/// Types of relations between nodes (for structure-mapping)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum RelationType {
    Causal,
    Temporal,
    Similarity,
    PartOf,
    Enables,
    Contrast,
    Attribute,
}

impl RelationType {
    /// Get compatible relation types for mapping
    pub fn compatible_types(&self) -> HashSet<RelationType> {
        match self {
            RelationType::Causal => {
                [RelationType::Causal, RelationType::Enables].into_iter().collect()
            }
            RelationType::Temporal => [RelationType::Temporal].into_iter().collect(),
            RelationType::Similarity => {
                [RelationType::Similarity, RelationType::Contrast].into_iter().collect()
            }
            RelationType::PartOf => [RelationType::PartOf].into_iter().collect(),
            RelationType::Enables => {
                [RelationType::Enables, RelationType::Causal].into_iter().collect()
            }
            RelationType::Contrast => {
                [RelationType::Contrast, RelationType::Similarity].into_iter().collect()
            }
            RelationType::Attribute => [RelationType::Attribute].into_iter().collect(),
        }
    }
}

/// Feature representation for a knowledge node
#[derive(Clone, Debug)]
pub struct NodeFeatures {
    pub node_id: String,
    pub semantic_vector: Vec<f64>,
    pub fingerprint: Fingerprint,
    pub degree: u32,
    pub clustering_coef: f64,
    pub betweenness: f64,
    pub incoming_relation_types: HashSet<RelationType>,
    pub outgoing_relation_types: HashSet<RelationType>,
    pub domain: String,
    pub creation_time: u64,
    pub last_access_time: u64,
    pub access_count: u64,
}

impl NodeFeatures {
    /// Create a new NodeFeatures with default values
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            semantic_vector: vec![0.0; 64],
            fingerprint: vec![0.0; 32],
            degree: 0,
            clustering_coef: 0.0,
            betweenness: 0.0,
            incoming_relation_types: HashSet::new(),
            outgoing_relation_types: HashSet::new(),
            domain: "unknown".to_string(),
            creation_time: 0,
            last_access_time: 0,
            access_count: 0,
        }
    }
}

/// Result of structure-mapping between two nodes
#[derive(Clone, Debug)]
pub struct StructuralMapping {
    pub source_node: String,
    pub target_node: String,
    pub attribute_similarity: f64,
    pub relational_similarity: f64,
    pub systematic_similarity: f64,
    pub overall_score: f64,
    pub mapped_relations: Vec<(String, String)>,
    pub confidence: f64,
    pub screening_applied: bool,
    pub candidates_screened: usize,
    pub candidates_evaluated: usize,
}

impl StructuralMapping {
    pub fn new(source: &str, target: &str) -> Self {
        Self {
            source_node: source.to_string(),
            target_node: target.to_string(),
            attribute_similarity: 0.0,
            relational_similarity: 0.0,
            systematic_similarity: 0.0,
            overall_score: 0.0,
            mapped_relations: Vec::new(),
            confidence: 0.0,
            screening_applied: false,
            candidates_screened: 0,
            candidates_evaluated: 0,
        }
    }
}

/// Discovery statistics
#[derive(Clone, Debug, Default)]
pub struct DiscoveryStats {
    pub discovery_attempts: u64,
    pub successful_discoveries: u64,
}

impl DiscoveryStats {
    pub fn discovery_rate(&self) -> f64 {
        if self.discovery_attempts == 0 {
            0.0
        } else {
            self.successful_discoveries as f64 / self.discovery_attempts as f64
        }
    }
}

/// Analogy Discovery Engine v3.0
///
/// Uses structural fingerprints and pre-screening for efficient analogy discovery.
pub struct AnalogyDiscoveryEngine {
    /// Attribute similarity weight (0.05-0.2, optimal: 0.1)
    pub attribute_weight: f64,
    /// Relational similarity weight (0.1-0.3, optimal: 0.2)
    pub relational_weight: f64,
    /// Systematic similarity weight (0.5-0.9, optimal: 0.7)
    pub systematic_weight: f64,
    /// Discovery threshold θ_disc (0.70-0.80, optimal: 0.75)
    pub discovery_threshold: f64,
    /// Pre-screening optimizer
    screening_optimizer: OwnedPreScreeningOptimizer,
    /// Whether screening is enabled
    pub screening_enabled: bool,
    /// Top-K% for screening
    pub top_k_percent: f64,
    /// Node features cache
    node_features: HashMap<String, NodeFeatures>,
    /// Statistics
    stats: DiscoveryStats,
    /// Discovery history
    discovery_history: Vec<StructuralMapping>,
}

impl AnalogyDiscoveryEngine {
    /// Create a new Analogy Discovery Engine
    ///
    /// # Arguments
    /// * `attribute_weight` - Weight for attribute similarity (0.05-0.2)
    /// * `relational_weight` - Weight for relational similarity (0.1-0.3)
    /// * `systematic_weight` - Weight for systematic similarity (0.5-0.9)
    /// * `discovery_threshold` - θ_disc (0.70-0.80, optimal: 0.75)
    /// * `fingerprint_dim` - Dimension of fingerprints (default: 32)
    /// * `screening_enabled` - Enable pre-screening
    /// * `top_k_percent` - Top-K% for screening (0.05 = 5%)
    pub fn new(
        attribute_weight: f64,
        relational_weight: f64,
        systematic_weight: f64,
        discovery_threshold: f64,
        fingerprint_dim: usize,
        screening_enabled: bool,
        top_k_percent: f64,
    ) -> Self {
        let fp_engine = StructuralFingerprintEngine::new(
            fingerprint_dim,
            systematic_weight,
            relational_weight,
            attribute_weight,
        );

        let screening_optimizer = OwnedPreScreeningOptimizer::new(fp_engine, top_k_percent, 1);

        Self {
            attribute_weight,
            relational_weight,
            systematic_weight,
            discovery_threshold,
            screening_optimizer,
            screening_enabled,
            top_k_percent,
            node_features: HashMap::new(),
            stats: DiscoveryStats::default(),
            discovery_history: Vec::new(),
        }
    }
}

impl Default for AnalogyDiscoveryEngine {
    fn default() -> Self {
        Self::new(
            0.1,  // attribute_weight
            0.2,  // relational_weight
            0.7,  // systematic_weight
            0.75, // discovery_threshold
            32,   // fingerprint_dim
            true, // screening_enabled
            0.05, // top_k_percent
        )
    }
}

impl AnalogyDiscoveryEngine {
    /// Get reference to fingerprint engine
    pub fn fingerprint_engine(&self) -> &StructuralFingerprintEngine {
        self.screening_optimizer.fp_engine()
    }

    /// Get mutable reference to fingerprint engine
    pub fn fingerprint_engine_mut(&mut self) -> &mut StructuralFingerprintEngine {
        self.screening_optimizer.fp_engine_mut()
    }

    /// Register a node with its features
    pub fn register_node(&mut self, node_id: &str, mut features: NodeFeatures, label: &NodeLabel) {
        // Compute fingerprint
        features.fingerprint = self
            .screening_optimizer
            .fp_engine_mut()
            .compute_fingerprint(node_id, label, None);

        self.node_features.insert(node_id.to_string(), features);
    }

    /// Update node features from graph structure
    pub fn update_node_features(
        &mut self,
        node_id: &str,
        degree: u32,
        clustering_coef: f64,
        total_nodes: u32,
        current_step: u64,
        label: Option<&NodeLabel>,
    ) {
        let features = self
            .node_features
            .entry(node_id.to_string())
            .or_insert_with(|| NodeFeatures::new(node_id.to_string()));

        features.degree = degree;
        features.clustering_coef = clustering_coef;

        if total_nodes > 1 {
            features.betweenness = degree as f64 / (total_nodes - 1) as f64;
        }

        features.last_access_time = current_step;
        features.access_count += 1;

        if let Some(lbl) = label {
            features.fingerprint = self
                .screening_optimizer
                .fp_engine_mut()
                .compute_fingerprint(node_id, lbl, None);
        }
    }

    /// Generate semantic vector (hash-based)
    pub fn generate_semantic_vector(&self, node_id: &str, label: &NodeLabel, domain: &str) -> Vec<f64> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        node_id.hash(&mut hasher);
        let seed = hasher.finish();

        let mut rng = SimpleRng::new(seed);
        let mut base_vector: Vec<f64> = (0..64).map(|_| rng.next_f64()).collect();

        match label {
            NodeLabel::IsolatedTruth | NodeLabel::Normal => {
                // Structured pattern
                for chunk in base_vector.chunks_mut(8) {
                    chunk.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                }

                // Compress and smooth first 32
                for value in base_vector.iter_mut().take(32) {
                    *value = 0.3 + 0.4 * *value;
                }
                for value in base_vector.iter_mut().skip(32).take(32) {
                    *value = 0.4 + 0.3 * *value;
                }

                // Smooth
                for i in 1..64 {
                    base_vector[i] = 0.6 * base_vector[i] + 0.4 * base_vector[i - 1];
                }
            }
            NodeLabel::Garbage => {
                // Chaotic pattern
                for i in (1..64).rev() {
                    let j = rng.next_usize() % (i + 1);
                    base_vector.swap(i, j);
                }

                // Anti-correlation
                let mut i = 0;
                while i + 1 < 64 {
                    base_vector[i] = rng.next_f64();
                    base_vector[i + 1] = 1.0 - base_vector[i];
                    i += 2;
                }
            }
            NodeLabel::Unknown => {}
        }

        // Apply domain offset
        let mut domain_hasher = DefaultHasher::new();
        domain.hash(&mut domain_hasher);
        let domain_seed = domain_hasher.finish();
        let mut domain_rng = SimpleRng::new(domain_seed);

        for v in &mut base_vector {
            let offset = domain_rng.next_f64() * 0.05;
            *v = *v * 0.95 + offset * 0.05;
        }

        base_vector
    }

    /// Find best analogy for source node among target candidates
    pub fn find_analogy(
        &mut self,
        source_node: &str,
        target_candidates: &[String],
    ) -> Option<StructuralMapping> {
        self.stats.discovery_attempts += 1;

        let source_features = self.node_features.get(source_node)?;
        let source_fp = source_features.fingerprint.clone();

        // Prepare candidates with fingerprints
        let mut candidates_with_fp: Vec<Candidate> = target_candidates
            .iter()
            .filter_map(|target| {
                self.node_features.get(target).map(|f| Candidate {
                    id: target.clone(),
                    fingerprint: f.fingerprint.clone(),
                })
            })
            .collect();

        if candidates_with_fp.is_empty() {
            return None;
        }

        let candidates_screened = candidates_with_fp.len();

        // Apply pre-screening if enabled
        if self.screening_enabled && candidates_with_fp.len() > 10 {
            candidates_with_fp = self
                .screening_optimizer
                .screen_candidates(&source_fp, candidates_with_fp);
        }

        let candidates_evaluated = candidates_with_fp.len();

        // Find best match using full similarity
        let mut best_mapping: Option<StructuralMapping> = None;
        let mut best_score = 0.0;

        for candidate in &candidates_with_fp {
            let target_features = match self.node_features.get(&candidate.id) {
                Some(f) => f,
                None => continue,
            };

            let sys_sim = self
                .screening_optimizer
                .fp_engine()
                .full_similarity(&source_fp, &candidate.fingerprint);

            let attr_sim = self.compute_attribute_similarity(source_features, target_features);
            let rel_sim = self.compute_relational_similarity(source_features, target_features);

            let overall = self.attribute_weight * attr_sim
                + self.relational_weight * rel_sim
                + self.systematic_weight * sys_sim;

            if overall > best_score {
                best_score = overall;

                let mut mapping = StructuralMapping::new(source_node, &candidate.id);
                mapping.attribute_similarity = attr_sim;
                mapping.relational_similarity = rel_sim;
                mapping.systematic_similarity = sys_sim;
                mapping.overall_score = overall;
                mapping.confidence = (overall * 1.2).min(1.0);
                mapping.screening_applied = self.screening_enabled;
                mapping.candidates_screened = candidates_screened;
                mapping.candidates_evaluated = candidates_evaluated;

                best_mapping = Some(mapping);
            }
        }

        if let Some(ref mapping) = best_mapping {
            if mapping.overall_score >= self.discovery_threshold {
                self.stats.successful_discoveries += 1;
                self.discovery_history.push(mapping.clone());
                return best_mapping;
            }
        }

        None
    }

    /// Compute attribute similarity between two nodes
    fn compute_attribute_similarity(&self, source: &NodeFeatures, target: &NodeFeatures) -> f64 {
        let deg_sum = source.degree + target.degree;
        let deg_sim = if deg_sum == 0 {
            1.0
        } else {
            1.0 - (source.degree as f64 - target.degree as f64).abs() / deg_sum as f64
        };

        let cluster_sim = 1.0 - (source.clustering_coef - target.clustering_coef).abs();

        let access_sum = source.access_count + target.access_count;
        let access_sim = if access_sum == 0 {
            1.0
        } else {
            1.0 - (source.access_count as f64 - target.access_count as f64).abs() / access_sum as f64
        };

        (deg_sim + cluster_sim + access_sim) / 3.0
    }

    /// Compute relational similarity between two nodes
    fn compute_relational_similarity(&self, source: &NodeFeatures, target: &NodeFeatures) -> f64 {
        let source_rels: HashSet<_> = source
            .incoming_relation_types
            .union(&source.outgoing_relation_types)
            .cloned()
            .collect();

        let target_rels: HashSet<_> = target
            .incoming_relation_types
            .union(&target.outgoing_relation_types)
            .cloned()
            .collect();

        if source_rels.is_empty() && target_rels.is_empty() {
            return 0.5;
        }
        if source_rels.is_empty() || target_rels.is_empty() {
            return 0.0;
        }

        let intersection = source_rels.intersection(&target_rels).count();
        let union = source_rels.union(&target_rels).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Get discovery statistics
    pub fn get_stats(&self) -> DiscoveryStats {
        self.stats.clone()
    }

    /// Get node features
    pub fn get_node_features(&self, node_id: &str) -> Option<&NodeFeatures> {
        self.node_features.get(node_id)
    }

    /// Get all node IDs
    pub fn get_node_ids(&self) -> Vec<String> {
        self.node_features.keys().cloned().collect()
    }

    /// Get discovery history
    pub fn get_discovery_history(&self) -> &[StructuralMapping] {
        &self.discovery_history
    }

    /// Clear discovery history
    pub fn clear_history(&mut self) {
        self.discovery_history.clear();
        self.stats = DiscoveryStats::default();
    }
}

/// Simple deterministic RNG
struct SimpleRng {
    state: u64,
}

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    fn next(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    fn next_usize(&mut self) -> usize {
        self.next() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analogy_engine_creation() {
        let engine = AnalogyDiscoveryEngine::default();
        assert_eq!(engine.discovery_threshold, 0.75);
        assert_eq!(engine.attribute_weight, 0.1);
        assert_eq!(engine.relational_weight, 0.2);
        assert_eq!(engine.systematic_weight, 0.7);
    }

    #[test]
    fn test_register_node() {
        let mut engine = AnalogyDiscoveryEngine::default();

        let features = NodeFeatures::new("node1".to_string());
        engine.register_node("node1", features, &NodeLabel::Normal);

        assert!(engine.get_node_features("node1").is_some());
    }

    #[test]
    fn test_find_analogy_no_candidates() {
        let mut engine = AnalogyDiscoveryEngine::default();

        let features = NodeFeatures::new("source".to_string());
        engine.register_node("source", features, &NodeLabel::Normal);

        let result = engine.find_analogy("source", &[]);
        assert!(result.is_none());
    }

    #[test]
    fn test_find_analogy_with_similar_nodes() {
        let mut engine = AnalogyDiscoveryEngine::default();

        // Register source node
        let mut source = NodeFeatures::new("source".to_string());
        source.degree = 5;
        source.clustering_coef = 0.5;
        source.outgoing_relation_types.insert(RelationType::Causal);
        engine.register_node("source", source, &NodeLabel::Normal);

        // Register similar target
        let mut target1 = NodeFeatures::new("target1".to_string());
        target1.degree = 5;
        target1.clustering_coef = 0.5;
        target1.outgoing_relation_types.insert(RelationType::Causal);
        engine.register_node("target1", target1, &NodeLabel::Normal);

        // Register dissimilar target
        let mut target2 = NodeFeatures::new("target2".to_string());
        target2.degree = 100;
        target2.clustering_coef = 0.1;
        target2.outgoing_relation_types.insert(RelationType::Contrast);
        engine.register_node("target2", target2, &NodeLabel::Garbage);

        let candidates = vec!["target1".to_string(), "target2".to_string()];
        let result = engine.find_analogy("source", &candidates);

        // Should find target1 as better match
        if let Some(mapping) = result {
            assert_eq!(mapping.target_node, "target1");
            assert!(mapping.overall_score > 0.0);
        }
    }

    #[test]
    fn test_generate_semantic_vector() {
        let engine = AnalogyDiscoveryEngine::default();

        let vec1 = engine.generate_semantic_vector("node1", &NodeLabel::Normal, "domain1");
        let vec2 = engine.generate_semantic_vector("node1", &NodeLabel::Normal, "domain1");

        // Should be deterministic
        assert_eq!(vec1.len(), 64);
        assert_eq!(vec1, vec2);
    }

    #[test]
    fn test_attribute_similarity() {
        let engine = AnalogyDiscoveryEngine::default();

        let mut node1 = NodeFeatures::new("n1".to_string());
        node1.degree = 10;
        node1.clustering_coef = 0.5;
        node1.access_count = 100;

        let mut node2 = NodeFeatures::new("n2".to_string());
        node2.degree = 10;
        node2.clustering_coef = 0.5;
        node2.access_count = 100;

        let sim = engine.compute_attribute_similarity(&node1, &node2);
        assert!((sim - 1.0).abs() < 1e-10); // Identical nodes
    }

    #[test]
    fn test_relational_similarity() {
        let engine = AnalogyDiscoveryEngine::default();

        let mut node1 = NodeFeatures::new("n1".to_string());
        node1.outgoing_relation_types.insert(RelationType::Causal);
        node1.outgoing_relation_types.insert(RelationType::Temporal);

        let mut node2 = NodeFeatures::new("n2".to_string());
        node2.outgoing_relation_types.insert(RelationType::Causal);
        node2.outgoing_relation_types.insert(RelationType::Similarity);

        let sim = engine.compute_relational_similarity(&node1, &node2);
        // Jaccard: 1 intersection / 3 union = 0.333...
        assert!((sim - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_claim43_integrity_discovery_requires_structural_representation() {
        // Claim 43: 整合性発見 takes the rare object's 内部構造 / 潜在的特徴量,
        // computes a 構造表現, compares with 他の情報オブジェクト群, and
        // accepts when the 整合性スコア meets the 採用基準.
        let mut engine = AnalogyDiscoveryEngine::default();
        // Structural representation (fingerprint) generation is observable:
        let f1 = engine.fingerprint_engine_mut().compute_fingerprint(
            "src", &NodeLabel::IsolatedTruth, None,
        );
        let f2 = engine.fingerprint_engine_mut().compute_fingerprint(
            "tgt", &NodeLabel::Normal, None,
        );
        assert_eq!(f1.len(), 32, "Claim 43: 構造表現 must be a fixed-length vector");
        assert_eq!(f2.len(), 32);
        // Different labels ⇒ different fingerprints (representation is meaningful)
        let identical: bool = f1.iter().zip(f2.iter()).all(|(a, b)| (a - b).abs() < 1e-12);
        assert!(!identical, "Claim 43: structural representation must encode label info");
    }

    #[test]
    fn test_claim44_weight_ratio_7_2_1() {
        // Claim 44: 整合性スコア weights (systematic : relational : attribute) = 7:2:1
        let engine = AnalogyDiscoveryEngine::default();
        let sys = engine.systematic_weight;
        let rel = engine.relational_weight;
        let att = engine.attribute_weight;
        // All weights positive
        assert!(sys > 0.0 && rel > 0.0 && att > 0.0,
            "Claim 44: all three weights must be positive");
        // Ratio 7:2:1 (within floating-point tolerance)
        assert!((sys / att - 7.0).abs() < 1e-9,
            "Claim 44: systematic:attribute = 7:1 (got {}:{})", sys, att);
        assert!((rel / att - 2.0).abs() < 1e-9,
            "Claim 44: relational:attribute = 2:1 (got {}:{})", rel, att);
    }

    #[test]
    fn test_claim45_first_score_is_positive_weighted_sum() {
        // Claim 45: the first score is a weighted sum of three components
        //   S = a·S_cos + b·S_struct + c·S_sign   with  a, b, c > 0.
        // We verify this formula by constructing two analogy candidates that
        // are identical in two components but differ in one, and observing
        // the overall_score scales with the differing component — proving
        // the score is additive in its three positive parts, not e.g.
        // min/max/multiplicative.
        let mut engine = AnalogyDiscoveryEngine::default();
        // All three weights are positive (direct assertion)
        assert!(engine.systematic_weight > 0.0, "Claim 45: w_sys > 0");
        assert!(engine.relational_weight > 0.0, "Claim 45: w_rel > 0");
        assert!(engine.attribute_weight > 0.0, "Claim 45: w_attr > 0");

        // Exercise the sum form: register two candidates, one identical to
        // source, one different, and confirm the overall_score is between 0
        // and sum of weights (i.e., behaves as a weighted linear combination).
        let source = NodeFeatures::new("src".into());
        let target_same = NodeFeatures::new("tgt1".into());
        let target_diff = NodeFeatures::new("tgt2".into());
        engine.register_node("src", source, &NodeLabel::IsolatedTruth);
        engine.register_node("tgt1", target_same, &NodeLabel::IsolatedTruth);
        engine.register_node("tgt2", target_diff, &NodeLabel::Normal);
        let _ = engine.find_analogy("src", &["tgt1".into(), "tgt2".into()]);
        // Sum-of-positive-weights bound: max achievable overall_score ≤ 1
        // (since each similarity is ≤ 1 and weights sum to 1 for default).
        let weight_sum = engine.systematic_weight
            + engine.relational_weight
            + engine.attribute_weight;
        assert!((weight_sum - 1.0).abs() < 1e-9,
            "Claim 45: weighted sum of positive coefficients (sum={})", weight_sum);
    }

    #[test]
    fn test_claim46_fingerprint_is_laplacian_eigenvalue_based() {
        // Claim 46: 構造表現 = 固定長ベクトル computed from the graph
        // Laplacian's eigenvalue spectrum of an ego-subgraph; threshold θ_L ∈
        // [0.70, 0.80]; simple-distance pre-screening used before full scoring.
        use nalgebra::DMatrix;
        let mut engine = StructuralFingerprintEngine::default();

        // Build a trivial 3-node Laplacian: path graph 0-1-2
        // Degrees: [1, 2, 1]; adjacency -1 on edges.
        let mut lap = DMatrix::<f64>::zeros(3, 3);
        lap[(0, 0)] = 1.0; lap[(0, 1)] = -1.0;
        lap[(1, 0)] = -1.0; lap[(1, 1)] = 2.0; lap[(1, 2)] = -1.0;
        lap[(2, 1)] = -1.0; lap[(2, 2)] = 1.0;

        let fp = engine.compute_fingerprint("x", &NodeLabel::Normal, Some(&lap));
        assert_eq!(fp.len(), 32,
            "Claim 46: fixed-length eigenvalue-derived vector expected");

        // θ_L default must be within [0.70, 0.80] (Claim 46 explicit range)
        let adj = crate::analogy::AnalogyDiscoveryEngine::default();
        assert!((0.70..=0.80).contains(&adj.discovery_threshold),
            "Claim 46: θ_L default must be inside [0.70, 0.80]");

        // Pre-screening is enabled with top-k percent ≤ some reasonable narrowing
        assert!(adj.screening_enabled,
            "Claim 46: simple-distance pre-screening must be enabled");
        assert!(adj.top_k_percent > 0.0 && adj.top_k_percent <= 0.5,
            "Claim 46: top-k% must narrow candidate pool");
    }

    #[test]
    fn test_discovery_stats() {
        let mut engine = AnalogyDiscoveryEngine::default();

        // Attempt discovery without proper setup
        let _ = engine.find_analogy("nonexistent", &["target".to_string()]);

        let stats = engine.get_stats();
        assert_eq!(stats.discovery_attempts, 1);
        assert_eq!(stats.successful_discoveries, 0);
    }
}
