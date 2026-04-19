//! KDF Verification for Logs/Events
//!
//! Verifies that KDF correctly:
//! 1. Decays repetitive log entries (same errors, routine messages)
//! 2. Preserves rare events (unusual errors, security incidents)

/// A log entry
#[derive(Clone, Debug)]
struct LogEntry {
    id: String,
    level: String,
    message: String,
    tokens: Vec<String>,
}

impl LogEntry {
    fn new(id: &str, level: &str, message: &str) -> Self {
        let tokens = Self::tokenize(message);
        Self {
            id: id.to_string(),
            level: level.to_string(),
            message: message.to_string(),
            tokens,
        }
    }

    fn tokenize(msg: &str) -> Vec<String> {
        // Extract meaningful tokens from log message
        // Remove common noise like timestamps, IDs
        msg.split(|c: char| c.is_whitespace() || c == ':' || c == '=' || c == '[' || c == ']')
            .filter(|s| !s.is_empty() && s.len() > 2)
            .filter(|s| !s.chars().all(|c| c.is_numeric())) // Remove pure numbers
            .map(|s| s.to_lowercase())
            .collect()
    }

    /// Jaccard similarity
    fn similarity(&self, other: &LogEntry) -> f64 {
        use std::collections::HashSet;
        let set1: HashSet<_> = self.tokens.iter().collect();
        let set2: HashSet<_> = other.tokens.iter().collect();

        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();

        if union == 0 {
            return 0.0;
        }
        intersection as f64 / union as f64
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layer {
    Core,
    Edge,
    Rare,
    #[allow(dead_code)]
    Garbage,
}

struct KdfLogsVerifier {
    entries: Vec<LogEntry>,
    similarity_threshold: f64,
    degrees: Vec<usize>,
    layers: Vec<Layer>,
}

impl KdfLogsVerifier {
    fn new(threshold: f64) -> Self {
        Self {
            entries: Vec::new(),
            similarity_threshold: threshold,
            degrees: Vec::new(),
            layers: Vec::new(),
        }
    }

    fn add_entry(&mut self, id: &str, level: &str, message: &str) {
        self.entries.push(LogEntry::new(id, level, message));
    }

    fn build_similarity_graph(&mut self) {
        let n = self.entries.len();
        self.degrees = vec![0; n];

        let mut edge_count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let sim = self.entries[i].similarity(&self.entries[j]);
                if sim >= self.similarity_threshold {
                    self.degrees[i] += 1;
                    self.degrees[j] += 1;
                    edge_count += 1;
                }
            }
        }

        println!("Built similarity graph: {} entries, {} edges", n, edge_count);
    }

    fn classify_nodes(&mut self) {
        let n = self.entries.len();
        self.layers = vec![Layer::Edge; n];

        if n == 0 {
            return;
        }

        let avg_degree: f64 = self.degrees.iter().sum::<usize>() as f64 / n as f64;

        for i in 0..n {
            let deg = self.degrees[i];
            if deg == 0 {
                self.layers[i] = Layer::Rare;
            } else if (deg as f64) > avg_degree * 1.5 {
                self.layers[i] = Layer::Core;
            } else if (deg as f64) < avg_degree * 0.5 {
                self.layers[i] = Layer::Rare;
            } else {
                self.layers[i] = Layer::Edge;
            }
        }
    }

    fn apply_decay(&self, iterations: usize) -> Vec<(usize, f64)> {
        let alpha = 1.5;
        let beta = 0.001;
        let gamma = 0.5;

        let mut weights: Vec<f64> = vec![1.0; self.entries.len()];

        for _ in 0..iterations {
            for i in 0..self.entries.len() {
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

        for (idx, weight) in &weights {
            result.add_node(
                self.entries[*idx].id.clone(),
                self.entries[*idx].level.clone(),
                self.layers[*idx],
                self.degrees[*idx],
                *weight,
            );
        }

        result.check_redundancy_reduction();
        result.check_rare_preservation();

        result
    }
}

struct NodeInfo {
    id: String,
    level: String,
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

    fn add_node(&mut self, id: String, level: String, layer: Layer, degree: usize, weight: f64) {
        self.nodes.push(NodeInfo { id, level, layer, degree, weight });
    }

    fn check_redundancy_reduction(&mut self) {
        let connected: Vec<_> = self.nodes.iter().filter(|n| n.degree > 0).collect();
        let isolated: Vec<_> = self.nodes.iter().filter(|n| n.degree == 0).collect();

        if connected.is_empty() || isolated.is_empty() {
            self.messages.push("✗ 冗長性削減: 接続または孤立ノードが不足".to_string());
            return;
        }

        let connected_avg: f64 = connected.iter().map(|n| n.weight).sum::<f64>() / connected.len() as f64;
        let isolated_avg: f64 = isolated.iter().map(|n| n.weight).sum::<f64>() / isolated.len() as f64;

        self.redundancy_reduced = connected_avg < isolated_avg;

        if self.redundancy_reduced {
            self.messages.push(format!(
                "✓ 冗長性削減: 繰り返しログ平均weight {:.4} < 孤立ログ平均weight {:.4}",
                connected_avg, isolated_avg
            ));
        } else {
            self.messages.push("✗ 冗長性削減: 期待通りに減衰していない".to_string());
        }
    }

    fn check_rare_preservation(&mut self) {
        let rare_nodes: Vec<_> = self.nodes.iter().filter(|n| n.layer == Layer::Rare).collect();

        if rare_nodes.is_empty() {
            self.messages.push("✗ レアイベント保持: RAREノードなし".to_string());
            return;
        }

        let preserved = rare_nodes.iter().filter(|n| n.weight > 0.8).count();
        self.rare_preserved = preserved == rare_nodes.len();

        if self.rare_preserved {
            self.messages.push(format!(
                "✓ レアイベント保持: 異常ログ{}件が全て保持（weight > 0.8）",
                rare_nodes.len()
            ));
        } else {
            self.messages.push(format!(
                "✗ レアイベント保持: RAREノード{}件中{}件のみ保持",
                rare_nodes.len(),
                preserved
            ));
        }
    }

    fn print_report(&self) {
        println!("\n========== KDF ログ検証レポート ==========\n");

        println!("【ログ分類結果】");
        println!("{:<12} {:<8} {:<8} {:<6} {:<8}", "ID", "Level", "Layer", "Degree", "Weight");
        println!("{}", "-".repeat(50));

        for node in &self.nodes {
            println!(
                "{:<12} {:<8} {:<8?} {:<6} {:.4}",
                node.id, node.level, node.layer, node.degree, node.weight
            );
        }

        println!("\n【検証結果】");
        for msg in &self.messages {
            println!("{}", msg);
        }

        println!("\n【総合判定】");
        if self.redundancy_reduced && self.rare_preserved {
            println!("✓ PASS: KDFはログ/イベントで期待通りに動作");
            println!("  - 繰り返しログは減衰（ストレージ効率化）");
            println!("  - 異常イベントは保持（障害分析可能）");
        } else {
            println!("✗ FAIL: 一部の検証が失敗");
        }

        println!("\n==========================================");
    }
}

fn main() {
    println!("KDF Log/Event Verification");
    println!("==========================\n");

    let mut verifier = KdfLogsVerifier::new(0.4);

    // Repetitive logs: Connection attempts (should decay)
    verifier.add_entry("conn_1", "INFO", "Connection established to server 192.168.1.100");
    verifier.add_entry("conn_2", "INFO", "Connection established to server 192.168.1.101");
    verifier.add_entry("conn_3", "INFO", "Connection established to server 192.168.1.102");
    verifier.add_entry("conn_4", "INFO", "Connection established to server 192.168.1.103");

    // Repetitive logs: Health checks (should decay)
    verifier.add_entry("health_1", "DEBUG", "Health check passed for service-a");
    verifier.add_entry("health_2", "DEBUG", "Health check passed for service-b");
    verifier.add_entry("health_3", "DEBUG", "Health check passed for service-c");

    // Repetitive errors: Common timeout (should decay)
    verifier.add_entry("timeout_1", "ERROR", "Request timeout after 30000ms to database");
    verifier.add_entry("timeout_2", "ERROR", "Request timeout after 30000ms to database");
    verifier.add_entry("timeout_3", "ERROR", "Request timeout after 30000ms to database");

    // RARE events: Security incident (should preserve!)
    verifier.add_entry("security_1", "CRITICAL", "Unauthorized access attempt detected from IP 10.0.0.99 with invalid credentials");

    // RARE events: System crash (should preserve!)
    verifier.add_entry("crash_1", "CRITICAL", "Out of memory exception in payment processing module causing service restart");

    // RARE events: Data corruption (should preserve!)
    verifier.add_entry("corrupt_1", "ERROR", "Data integrity check failed for customer records table checksum mismatch");

    let result = verifier.verify();
    result.print_report();
}
