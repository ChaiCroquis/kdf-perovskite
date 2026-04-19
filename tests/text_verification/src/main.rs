//! KDF Text/Document Verification
//!
//! This test verifies that KDF correctly:
//! 1. Decays redundant documents (high similarity)
//! 2. Preserves isolated documents (low similarity / unique)

use std::collections::HashMap;

/// Simple document representation
#[derive(Clone, Debug)]
struct Document {
    id: String,
    content: String,
    tokens: Vec<String>,
}

impl Document {
    fn new(id: &str, content: &str) -> Self {
        let tokens = Self::tokenize(content);
        Self {
            id: id.to_string(),
            content: content.to_string(),
            tokens,
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        // Use character n-grams (3-gram) for Japanese text
        // This handles the lack of word boundaries in Japanese
        let chars: Vec<char> = text.chars()
            .filter(|c| !c.is_whitespace() && *c != '。' && *c != '、' && *c != '.')
            .collect();

        if chars.len() < 3 {
            return chars.iter().map(|c| c.to_string()).collect();
        }

        chars.windows(3)
            .map(|w| w.iter().collect::<String>())
            .collect()
    }

    /// Jaccard similarity between two documents
    fn similarity(&self, other: &Document) -> f64 {
        let set1: std::collections::HashSet<_> = self.tokens.iter().collect();
        let set2: std::collections::HashSet<_> = other.tokens.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// KDF Layer classification
#[derive(Debug, Clone, Copy, PartialEq)]
enum Layer {
    Core,
    Edge,
    Rare,
    Garbage,
}

/// Simple KDF implementation for verification
struct KdfVerifier {
    documents: Vec<Document>,
    similarity_threshold: f64,
    edges: Vec<(usize, usize, f64)>,
    degrees: Vec<usize>,
    layers: Vec<Layer>,
    weights: HashMap<(usize, usize), f64>,
}

impl KdfVerifier {
    fn new(similarity_threshold: f64) -> Self {
        Self {
            documents: Vec::new(),
            similarity_threshold,
            edges: Vec::new(),
            degrees: Vec::new(),
            layers: Vec::new(),
            weights: HashMap::new(),
        }
    }

    fn add_document(&mut self, id: &str, content: &str) {
        self.documents.push(Document::new(id, content));
    }

    fn build_similarity_graph(&mut self) {
        let n = self.documents.len();
        self.degrees = vec![0; n];
        self.edges.clear();
        self.weights.clear();

        // Build edges based on similarity
        for i in 0..n {
            for j in (i + 1)..n {
                let sim = self.documents[i].similarity(&self.documents[j]);
                if sim >= self.similarity_threshold {
                    self.edges.push((i, j, sim));
                    self.degrees[i] += 1;
                    self.degrees[j] += 1;
                    self.weights.insert((i, j), sim);
                    self.weights.insert((j, i), sim);
                }
            }
        }

        println!("Built similarity graph: {} nodes, {} edges", n, self.edges.len());
    }

    fn classify_nodes(&mut self) {
        let n = self.documents.len();
        self.layers = vec![Layer::Edge; n];

        // Classification based on degree
        let avg_degree: f64 = if n > 0 {
            self.degrees.iter().sum::<usize>() as f64 / n as f64
        } else {
            0.0
        };

        for i in 0..n {
            let deg = self.degrees[i];
            if deg == 0 {
                // Isolated: could be RARE or GARBAGE
                // For now, mark as RARE (judgment deferral)
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

    fn apply_decay(&mut self, iterations: usize) -> Vec<(usize, f64)> {
        // Apply decay based on connectivity (C^α formula)
        // Higher connectivity = faster decay
        // Key insight: C=0 (isolated) → minimal decay, C>0 (connected) → fast decay
        let alpha = 1.5;
        let beta = 0.001;  // Base decay rate (very slow for isolated nodes)
        let gamma = 0.5;   // Sensitivity to connectivity

        let mut node_weights: Vec<f64> = vec![1.0; self.documents.len()];

        for _ in 0..iterations {
            for i in 0..self.documents.len() {
                let c = self.degrees[i] as f64;
                // λ(C) = β(1 + γ·C^α)
                // When C=0: decay_rate = β (very slow)
                // When C>0: decay_rate increases with C^α
                let decay_rate = beta * (1.0 + gamma * c.powf(alpha));
                node_weights[i] *= (1.0 - decay_rate).max(0.0);
            }
        }

        // Return sorted by weight
        let mut result: Vec<(usize, f64)> = node_weights
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
        let decayed_weights = self.apply_decay(100);

        let mut result = VerificationResult::new();

        // Count by layer
        for (i, layer) in self.layers.iter().enumerate() {
            let doc = &self.documents[i];
            let deg = self.degrees[i];
            let weight = decayed_weights.iter().find(|(idx, _)| *idx == i).map(|(_, w)| *w).unwrap_or(0.0);

            result.add_node(doc.id.clone(), *layer, deg, weight);
        }

        // Verify expected behaviors
        result.check_redundancy_reduction(&decayed_weights);
        result.check_isolation_preservation(&self.layers, &self.documents);

        result
    }
}

struct VerificationResult {
    nodes: Vec<NodeInfo>,
    redundancy_reduced: bool,
    isolated_preserved: bool,
    messages: Vec<String>,
}

struct NodeInfo {
    id: String,
    layer: Layer,
    degree: usize,
    weight: f64,
}

impl VerificationResult {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            redundancy_reduced: false,
            isolated_preserved: false,
            messages: Vec::new(),
        }
    }

    fn add_node(&mut self, id: String, layer: Layer, degree: usize, weight: f64) {
        self.nodes.push(NodeInfo { id, layer, degree, weight });
    }

    fn check_redundancy_reduction(&mut self, _weights: &[(usize, f64)]) {
        // Check if higher connectivity leads to lower weights (KDF core principle)
        // This is the mathematical guarantee: C大 → 速く減衰

        // Get nodes with connections (degree > 0)
        let connected_nodes: Vec<_> = self.nodes.iter()
            .filter(|n| n.degree > 0)
            .collect();

        // Get isolated nodes (degree = 0)
        let isolated_nodes: Vec<_> = self.nodes.iter()
            .filter(|n| n.degree == 0)
            .collect();

        if connected_nodes.is_empty() || isolated_nodes.is_empty() {
            self.messages.push("✗ 冗長性削減: 接続ノードまたは孤立ノードが不足".to_string());
            return;
        }

        // Calculate average weights
        let connected_avg: f64 = connected_nodes.iter().map(|n| n.weight).sum::<f64>()
            / connected_nodes.len() as f64;
        let isolated_avg: f64 = isolated_nodes.iter().map(|n| n.weight).sum::<f64>()
            / isolated_nodes.len() as f64;

        // KDF guarantee: connected nodes should have lower average weight than isolated nodes
        self.redundancy_reduced = connected_avg < isolated_avg;

        if self.redundancy_reduced {
            self.messages.push(format!(
                "✓ 冗長性削減: 接続ノード平均weight {:.4} < 孤立ノード平均weight {:.4}",
                connected_avg, isolated_avg
            ));
            self.messages.push(format!(
                "  → 冗長データは減衰、孤立データは保持（KDF数式通り）"
            ));
        } else {
            self.messages.push(format!(
                "✗ 冗長性削減: 期待通りに減衰していない (connected={:.4}, isolated={:.4})",
                connected_avg, isolated_avg
            ));
        }
    }

    fn check_isolation_preservation(&mut self, _layers: &[Layer], _documents: &[Document]) {
        // Check if RARE layer nodes are preserved (high weight)
        let rare_nodes: Vec<_> = self.nodes.iter()
            .filter(|n| n.layer == Layer::Rare)
            .collect();

        let preserved_count = rare_nodes.iter()
            .filter(|n| n.weight > 0.8)
            .count();

        self.isolated_preserved = rare_nodes.is_empty() || preserved_count == rare_nodes.len();

        if self.isolated_preserved {
            self.messages.push(format!(
                "✓ 孤立保持: RAREノード{}件が全て保持（weight > 0.8）",
                rare_nodes.len()
            ));
        } else {
            self.messages.push(format!(
                "✗ 孤立保持: RAREノード{}件中{}件のみ保持",
                rare_nodes.len(),
                preserved_count
            ));
        }
    }

    fn print_report(&self) {
        println!("\n========== KDF検証レポート ==========\n");

        println!("【ノード分類結果】");
        println!("{:<15} {:<10} {:<8} {:<10}", "ID", "Layer", "Degree", "Weight");
        println!("{}", "-".repeat(45));

        for node in &self.nodes {
            println!(
                "{:<15} {:<10} {:<8} {:<10.4}",
                node.id,
                format!("{:?}", node.layer),
                node.degree,
                node.weight
            );
        }

        println!("\n【検証結果】");
        for msg in &self.messages {
            println!("{}", msg);
        }

        println!("\n【総合判定】");
        if self.redundancy_reduced && self.isolated_preserved {
            println!("✓ PASS: KDFは期待通りに動作");
            println!("  - 冗長なデータは減衰された");
            println!("  - 孤立したデータは保持された");
        } else {
            println!("✗ FAIL: 一部の検証が失敗");
        }

        println!("\n=====================================");
    }
}

fn main() {
    println!("KDF Text/Document Verification");
    println!("================================\n");

    let mut verifier = KdfVerifier::new(0.15); // Low threshold to catch Japanese text similarity

    // Add redundant tech articles (should decay)
    verifier.add_document("tech_001", "Rustは高速で安全なシステムプログラミング言語です。メモリ安全性を保証します。");
    verifier.add_document("tech_002", "Rust言語は高速かつ安全なシステム開発向け言語。メモリの安全性が特徴です。");
    verifier.add_document("tech_003", "Rustはシステムプログラミングのための高速で安全な言語。メモリ安全を実現。");
    verifier.add_document("tech_004", "高速で安全なRust言語はシステム開発に最適。メモリ安全性を提供します。");
    verifier.add_document("tech_005", "Rustプログラミング言語は高速性と安全性を両立。メモリ管理が安全。");

    // Add redundant news (should decay)
    verifier.add_document("news_001", "本日、東京で大規模なITカンファレンスが開催された。多くの参加者が集まった。");
    verifier.add_document("news_002", "東京にて本日、大規模ITカンファレンス開催。多数の参加者。");
    verifier.add_document("news_003", "大規模ITカンファレンスが東京で本日開催された。参加者多数。");

    // Add isolated unique documents (should be preserved)
    verifier.add_document("unique_001", "量子コンピューティングの新しいアルゴリズムが発表された。従来の100倍の速度。");
    verifier.add_document("unique_002", "深海で新種の発光生物が発見された。水深8000メートルの環境に適応。");
    verifier.add_document("unique_003", "古代エジプトの未発見の墓が砂漠で見つかった。3000年前の遺物を含む。");

    // Run verification
    let result = verifier.verify();
    result.print_report();
}
