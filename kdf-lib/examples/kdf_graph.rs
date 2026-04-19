//! Graph KDF - Applying KDF to Graph Data
//!
//! This example demonstrates how to apply KDF to graph structures:
//! - Node-level KDF: Find representative and rare nodes
//! - Edge-level KDF: Find unique relationships
//! - Structural similarity based on graph topology
//!
//! Run: cargo run --example kdf_graph

use kdf::{Kdf, Layer};
use std::collections::{HashMap, HashSet};

/// Simple undirected graph representation
#[derive(Clone)]
struct Graph {
    /// Number of nodes
    n_nodes: usize,
    /// Adjacency list
    edges: HashMap<usize, HashSet<usize>>,
    /// Node features (optional)
    node_features: Vec<Vec<f64>>,
}

impl Graph {
    fn new(n_nodes: usize) -> Self {
        Graph {
            n_nodes,
            edges: HashMap::new(),
            node_features: vec![vec![]; n_nodes],
        }
    }

    fn add_edge(&mut self, u: usize, v: usize) {
        self.edges.entry(u).or_insert_with(HashSet::new).insert(v);
        self.edges.entry(v).or_insert_with(HashSet::new).insert(u);
    }

    fn set_node_feature(&mut self, node: usize, features: Vec<f64>) {
        self.node_features[node] = features;
    }

    fn neighbors(&self, node: usize) -> HashSet<usize> {
        self.edges.get(&node).cloned().unwrap_or_default()
    }

    fn degree(&self, node: usize) -> usize {
        self.neighbors(node).len()
    }

    fn get_all_edges(&self) -> Vec<(usize, usize)> {
        let mut edges = Vec::new();
        for (&u, neighbors) in &self.edges {
            for &v in neighbors {
                if u < v {  // Avoid duplicates for undirected graph
                    edges.push((u, v));
                }
            }
        }
        edges
    }
}

/// Compute Jaccard similarity between node neighborhoods
fn jaccard_neighborhood_similarity(graph: &Graph, u: usize, v: usize) -> f64 {
    let neighbors_u = graph.neighbors(u);
    let neighbors_v = graph.neighbors(v);

    let intersection = neighbors_u.intersection(&neighbors_v).count();
    let union = neighbors_u.union(&neighbors_v).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Compute cosine similarity on node features
fn cosine_node_similarity(features_a: &[f64], features_b: &[f64]) -> f64 {
    if features_a.is_empty() || features_b.is_empty() {
        return 0.0;
    }

    let dot: f64 = features_a.iter().zip(features_b.iter()).map(|(a, b)| a * b).sum();
    let norm_a: f64 = features_a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = features_b.iter().map(|x| x * x).sum::<f64>().sqrt();

    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Combined structural + feature similarity
fn combined_similarity(graph: &Graph, u: usize, v: usize, alpha: f64) -> f64 {
    let structural = jaccard_neighborhood_similarity(graph, u, v);
    let feature = cosine_node_similarity(&graph.node_features[u], &graph.node_features[v]);

    alpha * structural + (1.0 - alpha) * feature
}

/// Edge representation for KDF
#[derive(Clone)]
#[allow(dead_code)]
struct EdgeData {
    u: usize,
    v: usize,
    u_degree: usize,
    v_degree: usize,
    common_neighbors: usize,
}

impl EdgeData {
    fn new(graph: &Graph, u: usize, v: usize) -> Self {
        let u_deg = graph.degree(u);
        let v_deg = graph.degree(v);
        let neighbors_u = graph.neighbors(u);
        let neighbors_v = graph.neighbors(v);
        let common = neighbors_u.intersection(&neighbors_v).count();

        EdgeData {
            u,
            v,
            u_degree: u_deg,
            v_degree: v_deg,
            common_neighbors: common,
        }
    }

    /// Convert edge to feature vector
    fn to_vector(&self) -> Vec<f64> {
        vec![
            self.u_degree as f64,
            self.v_degree as f64,
            self.common_neighbors as f64,
            (self.u_degree + self.v_degree) as f64,
            (self.u_degree.min(self.v_degree)) as f64,
            (self.u_degree.max(self.v_degree)) as f64,
        ]
    }
}

fn edge_cosine_similarity(e1: &EdgeData, e2: &EdgeData) -> f64 {
    let v1 = e1.to_vector();
    let v2 = e2.to_vector();
    cosine_node_similarity(&v1, &v2)
}

fn main() {
    println!("=== Graph KDF Demo ===\n");

    let kdf = Kdf::with_defaults();

    // =========================================================================
    // 1. Create a sample graph (social network-like)
    // =========================================================================
    println!("--- Building Sample Graph ---\n");

    let mut graph = Graph::new(12);

    // Community 1: Dense cluster (nodes 0-4)
    for i in 0..5 {
        for j in i+1..5 {
            graph.add_edge(i, j);
        }
    }

    // Community 2: Another dense cluster (nodes 5-8)
    for i in 5..9 {
        for j in i+1..9 {
            graph.add_edge(i, j);
        }
    }

    // Bridge between communities
    graph.add_edge(4, 5);

    // Peripheral nodes
    graph.add_edge(0, 9);  // Node 9: connected to only one node in community 1
    graph.add_edge(10, 7); // Node 10: peripheral to community 2

    // Isolated node (11) - no edges

    // Set some node features
    for i in 0..5 {
        graph.set_node_feature(i, vec![1.0, 0.0, i as f64 * 0.1]); // Community 1 features
    }
    for i in 5..9 {
        graph.set_node_feature(i, vec![0.0, 1.0, (i - 5) as f64 * 0.1]); // Community 2 features
    }
    graph.set_node_feature(9, vec![0.5, 0.5, 0.0]); // Bridge-like
    graph.set_node_feature(10, vec![0.0, 1.0, 0.5]); // Similar to community 2
    graph.set_node_feature(11, vec![0.3, 0.3, 0.9]); // Unique features

    println!("Graph: {} nodes, {} edges", graph.n_nodes, graph.get_all_edges().len());
    for i in 0..graph.n_nodes {
        println!("  Node {}: degree={}, features={:?}", i, graph.degree(i), graph.node_features[i]);
    }
    println!();

    // =========================================================================
    // 2. Node-level KDF (Structural Similarity)
    // =========================================================================
    println!("--- Node-Level KDF (Structural) ---\n");

    // Create node indices for processing
    let node_indices: Vec<usize> = (0..graph.n_nodes).collect();

    let result_nodes_struct = kdf.process(&node_indices, 0.3, |&u, &v| {
        jaccard_neighborhood_similarity(&graph, u, v)
    });

    println!("Structural similarity results:");
    println!("  Selected nodes: {:?}", result_nodes_struct.selected);
    println!("  Layers: {:?}", result_nodes_struct.layers);
    println!("  Rare nodes: {:?}", result_nodes_struct.rare_items());

    // Explain rare nodes
    for &idx in result_nodes_struct.rare_items().iter() {
        println!("  Node {} is Rare: degree={}, structurally isolated", idx, graph.degree(idx));
    }
    println!();

    // =========================================================================
    // 3. Node-level KDF (Combined: Structure + Features)
    // =========================================================================
    println!("--- Node-Level KDF (Combined) ---\n");

    let result_nodes_combined = kdf.process(&node_indices, 0.5, |&u, &v| {
        combined_similarity(&graph, u, v, 0.5)
    });

    println!("Combined similarity results:");
    println!("  Selected nodes: {:?}", result_nodes_combined.selected);
    println!("  Layers: {:?}", result_nodes_combined.layers);
    println!("  Rare nodes: {:?}", result_nodes_combined.rare_items());
    println!();

    // =========================================================================
    // 4. Edge-level KDF
    // =========================================================================
    println!("--- Edge-Level KDF ---\n");

    let edges = graph.get_all_edges();
    let edge_data: Vec<EdgeData> = edges.iter()
        .map(|&(u, v)| EdgeData::new(&graph, u, v))
        .collect();

    println!("Total edges: {}", edges.len());
    for (i, (u, v)) in edges.iter().enumerate() {
        let ed = &edge_data[i];
        println!("  Edge {}: ({}-{}) deg=({},{}) common={}",
            i, u, v, ed.u_degree, ed.v_degree, ed.common_neighbors);
    }
    println!();

    let result_edges = kdf.process(&edge_data, 0.9, |e1, e2| {
        edge_cosine_similarity(e1, e2)
    });

    println!("Edge KDF results:");
    println!("  Selected edges: {:?}", result_edges.selected);
    println!("  Rare edges: {:?}", result_edges.rare_items());

    println!("\nRare edges (unique relationships):");
    for &idx in result_edges.rare_items().iter() {
        let (u, v) = edges[idx];
        println!("  ({}-{}): This edge is structurally unique", u, v);
    }
    println!();

    // =========================================================================
    // 5. Applications
    // =========================================================================
    println!("=== Applications ===\n");

    println!("1. Node Sampling for GNN Training:");
    println!("   - Core nodes: Common patterns, sample fewer");
    println!("   - Rare nodes: Unique patterns, keep all");
    let core_nodes: Vec<_> = (0..graph.n_nodes)
        .filter(|&i| result_nodes_struct.layers[i] == Layer::Core)
        .collect();
    let rare_nodes: Vec<_> = result_nodes_struct.rare_items();
    println!("   Core nodes to subsample: {:?}", core_nodes);
    println!("   Rare nodes to preserve: {:?}", rare_nodes);
    println!();

    println!("2. Graph Simplification:");
    println!("   Keep {} of {} edges ({:.0}% reduction)",
        result_edges.selected.len(),
        edges.len(),
        (1.0 - result_edges.selected.len() as f64 / edges.len() as f64) * 100.0
    );
    println!();

    println!("3. Anomaly Detection:");
    println!("   Rare nodes often represent:");
    println!("   - Isolated users in social networks");
    println!("   - Unusual transaction patterns");
    println!("   - Potential anomalies worth investigating");
    println!();

    println!("4. Community Detection Aid:");
    println!("   - Bridge nodes (connecting communities) often become Rare");
    println!("   - Node 4-5 bridge edge preserved as important");

    println!("\n=== Summary ===");
    println!("Graph KDF enables:");
    println!("- Efficient graph sampling while preserving rare patterns");
    println!("- Identification of structurally unique nodes/edges");
    println!("- Graph compression for large-scale processing");
}
