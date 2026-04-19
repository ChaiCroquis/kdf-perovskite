//! KDF Verification for ML Training Data
//!
//! Verifies that KDF correctly:
//! 1. Decays redundant training samples (similar feature vectors)
//! 2. Preserves rare edge cases (isolated samples)

use std::collections::HashSet;

/// A training sample with feature vector
#[derive(Clone, Debug)]
struct Sample {
    id: String,
    features: Vec<f64>,
    label: String,
}

impl Sample {
    fn new(id: &str, features: Vec<f64>, label: &str) -> Self {
        Self {
            id: id.to_string(),
            features,
            label: label.to_string(),
        }
    }

    /// Cosine similarity between two samples
    fn similarity(&self, other: &Sample) -> f64 {
        let dot: f64 = self.features.iter()
            .zip(other.features.iter())
            .map(|(a, b)| a * b)
            .sum();

        let mag1: f64 = self.features.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2: f64 = other.features.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            return 0.0;
        }

        dot / (mag1 * mag2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layer {
    Core,    // Established knowledge
    Edge,    // New information
    Rare,    // Can't judge - preserve
    Garbage, // Timeout - delete
}

/// KDF implementation for ML data
struct KdfMlVerifier {
    samples: Vec<Sample>,
    similarity_threshold: f64,
    adjacency: Vec<Vec<bool>>,
    degrees: Vec<usize>,
    layers: Vec<Layer>,
}

impl KdfMlVerifier {
    fn new(threshold: f64) -> Self {
        Self {
            samples: Vec::new(),
            similarity_threshold: threshold,
            adjacency: Vec::new(),
            degrees: Vec::new(),
            layers: Vec::new(),
        }
    }

    fn add_sample(&mut self, id: &str, features: Vec<f64>, label: &str) {
        self.samples.push(Sample::new(id, features, label));
    }

    fn build_similarity_graph(&mut self) {
        let n = self.samples.len();
        self.adjacency = vec![vec![false; n]; n];
        self.degrees = vec![0; n];

        let mut edge_count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = self.samples[i].similarity(&self.samples[j]);
                if sim >= self.similarity_threshold {
                    self.adjacency[i][j] = true;
                    self.adjacency[j][i] = true;
                    self.degrees[i] += 1;
                    self.degrees[j] += 1;
                    edge_count += 1;
                }
            }
        }

        println!("Built similarity graph: {} samples, {} edges", n, edge_count);
    }

    fn classify_nodes(&mut self) {
        let n = self.samples.len();
        self.layers = vec![Layer::Edge; n];

        if n == 0 {
            return;
        }

        let avg_degree: f64 = self.degrees.iter().sum::<usize>() as f64 / n as f64;

        for i in 0..n {
            let deg = self.degrees[i];

            if deg == 0 {
                // Isolated: RARE layer (judgment deferral)
                self.layers[i] = Layer::Rare;
            } else if (deg as f64) > avg_degree * 1.5 {
                // High connectivity: CORE
                self.layers[i] = Layer::Core;
            } else if (deg as f64) < avg_degree * 0.5 {
                // Low connectivity: RARE
                self.layers[i] = Layer::Rare;
            } else {
                // Medium: EDGE
                self.layers[i] = Layer::Edge;
            }
        }
    }

    fn apply_decay(&self, iterations: usize) -> Vec<(usize, f64)> {
        // λ(C) = β(1 + γ·C^α)
        // C=0 → minimal decay, C>0 → faster decay
        let alpha = 1.5;
        let beta = 0.001;
        let gamma = 0.5;

        let mut weights: Vec<f64> = vec![1.0; self.samples.len()];

        for _ in 0..iterations {
            for i in 0..self.samples.len() {
                let c = self.degrees[i] as f64;
                let decay_rate = beta * (1.0 + gamma * c.powf(alpha));
                weights[i] *= (1.0 - decay_rate).max(0.0);
            }
        }

        let mut result: Vec<(usize, f64)> = weights
            .iter()
            .enumerate()
            .map(|(i, &w)| (i, w))
            .collect();
        result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        result
    }

    fn verify(&mut self) -> VerificationResult {
        self.build_similarity_graph();
        self.classify_nodes();
        let weights = self.apply_decay(100);

        let mut result = VerificationResult::new();

        // Record all nodes
        for (idx, weight) in &weights {
            result.add_node(
                self.samples[*idx].id.clone(),
                self.samples[*idx].label.clone(),
                self.layers[*idx],
                self.degrees[*idx],
                *weight,
            );
        }

        // Check KDF guarantees
        result.check_redundancy_reduction();
        result.check_rare_preservation();

        result
    }
}

struct NodeInfo {
    id: String,
    label: String,
    layer: Layer,
    degree: usize,
    weight: f64,
}

struct VerificationResult {
    nodes: Vec<NodeInfo>,
    redundancy_reduced: bool,
    rare_preserved: bool,
    messages: Vec<String>,
}

impl VerificationResult {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            redundancy_reduced: false,
            rare_preserved: false,
            messages: Vec::new(),
        }
    }

    fn add_node(&mut self, id: String, label: String, layer: Layer, degree: usize, weight: f64) {
        self.nodes.push(NodeInfo { id, label, layer, degree, weight });
    }

    fn check_redundancy_reduction(&mut self) {
        let connected: Vec<_> = self.nodes.iter().filter(|n| n.degree > 0).collect();
        let isolated: Vec<_> = self.nodes.iter().filter(|n| n.degree == 0).collect();

        if connected.is_empty() || isolated.is_empty() {
            self.messages.push("✗ 冗長性削減: 接続ノードまたは孤立ノードが不足".to_string());
            return;
        }

        let connected_avg: f64 = connected.iter().map(|n| n.weight).sum::<f64>() / connected.len() as f64;
        let isolated_avg: f64 = isolated.iter().map(|n| n.weight).sum::<f64>() / isolated.len() as f64;

        self.redundancy_reduced = connected_avg < isolated_avg;

        if self.redundancy_reduced {
            self.messages.push(format!(
                "✓ 冗長性削減: 類似サンプル平均weight {:.4} < 孤立サンプル平均weight {:.4}",
                connected_avg, isolated_avg
            ));
        } else {
            self.messages.push(format!(
                "✗ 冗長性削減: 期待通りに減衰していない"
            ));
        }
    }

    fn check_rare_preservation(&mut self) {
        let rare_nodes: Vec<_> = self.nodes.iter().filter(|n| n.layer == Layer::Rare).collect();

        if rare_nodes.is_empty() {
            self.messages.push("✗ レアケース保持: RAREノードなし".to_string());
            return;
        }

        let preserved = rare_nodes.iter().filter(|n| n.weight > 0.8).count();
        self.rare_preserved = preserved == rare_nodes.len();

        if self.rare_preserved {
            self.messages.push(format!(
                "✓ レアケース保持: エッジケース{}件が全て保持（weight > 0.8）",
                rare_nodes.len()
            ));
        } else {
            self.messages.push(format!(
                "✗ レアケース保持: RAREノード{}件中{}件のみ保持",
                rare_nodes.len(),
                preserved
            ));
        }
    }

    fn print_report(&self) {
        println!("\n========== KDF ML検証レポート ==========\n");

        println!("【サンプル分類結果】");
        println!("{:<12} {:<10} {:<8} {:<6} {:<8}", "ID", "Label", "Layer", "Degree", "Weight");
        println!("{}", "-".repeat(50));

        for node in &self.nodes {
            println!(
                "{:<12} {:<10} {:<8?} {:<6} {:.4}",
                node.id, node.label, node.layer, node.degree, node.weight
            );
        }

        println!("\n【検証結果】");
        for msg in &self.messages {
            println!("{}", msg);
        }

        println!("\n【総合判定】");
        if self.redundancy_reduced && self.rare_preserved {
            println!("✓ PASS: KDFはML学習データで期待通りに動作");
            println!("  - 重複サンプルは減衰（効率的な学習）");
            println!("  - エッジケースは保持（ロバスト性確保）");
        } else {
            println!("✗ FAIL: 一部の検証が失敗");
        }

        println!("\n=========================================");
    }
}

fn main() {
    println!("KDF ML Training Data Verification");
    println!("==================================\n");

    let mut verifier = KdfMlVerifier::new(0.9); // High threshold for cosine similarity

    // Redundant samples: Similar features (common patterns)
    // These represent duplicate/near-duplicate training data
    verifier.add_sample("common_1", vec![1.0, 0.9, 0.1, 0.0], "cat");
    verifier.add_sample("common_2", vec![1.0, 0.85, 0.15, 0.0], "cat");
    verifier.add_sample("common_3", vec![0.95, 0.9, 0.1, 0.05], "cat");
    verifier.add_sample("common_4", vec![0.98, 0.88, 0.12, 0.02], "cat");
    verifier.add_sample("common_5", vec![1.0, 0.92, 0.08, 0.0], "cat");

    // More redundant samples for another class
    verifier.add_sample("common_6", vec![0.0, 0.1, 0.9, 1.0], "dog");
    verifier.add_sample("common_7", vec![0.05, 0.15, 0.85, 0.95], "dog");
    verifier.add_sample("common_8", vec![0.0, 0.12, 0.88, 1.0], "dog");

    // Edge cases: Unique/rare samples that should be preserved
    // These are important for model robustness
    verifier.add_sample("edge_1", vec![0.5, 0.5, 0.5, 0.5], "unknown");  // Ambiguous
    verifier.add_sample("edge_2", vec![0.0, 0.0, 0.0, 1.0], "rare");     // Rare pattern
    verifier.add_sample("edge_3", vec![1.0, 1.0, 1.0, 1.0], "outlier");  // Outlier

    let result = verifier.verify();
    result.print_report();
}
