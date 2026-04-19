//! Graph traits and implementations
//!
//! This crate provides the core graph abstraction used across CGB.

use std::cell::OnceCell;

// Re-export types for convenience
pub use cgb_types::{NodeId, Edge, ModuleId};

/// Graph view trait for read-only access
///
/// This trait uses `Box<dyn Iterator>` for FFI compatibility.
pub trait GraphView {
    /// Number of nodes
    fn node_count(&self) -> usize;

    /// Number of edges
    fn edge_count(&self) -> usize;

    /// Iterator over all nodes
    fn nodes(&self) -> Box<dyn Iterator<Item = NodeId> + '_>;

    /// Iterator over all edges
    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_>;

    /// Iterator over neighbors of a node (with weights)
    fn neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = (NodeId, f64)> + '_>;

    /// Weighted degree of a node
    fn weighted_degree(&self, node: NodeId) -> f64 {
        self.neighbors(node).map(|(_, w)| w).sum()
    }

    /// Check if node exists
    fn contains_node(&self, node: NodeId) -> bool;

    /// Get edge weight between two nodes
    fn edge_weight(&self, source: NodeId, target: NodeId) -> Option<f64>;
}

/// Edge list based graph (most generic)
pub struct EdgeListGraph {
    node_count: usize,
    edges: Vec<Edge>,
    /// Adjacency list (lazy built)
    adjacency: OnceCell<Vec<Vec<(NodeId, f64)>>>,
}

impl EdgeListGraph {
    /// Create a new graph from edges
    pub fn new(node_count: usize, edges: Vec<Edge>) -> Self {
        Self {
            node_count,
            edges,
            adjacency: OnceCell::new(),
        }
    }

    /// Create from tuple format (source, target, weight)
    pub fn from_tuples(node_count: usize, edges: &[(u32, u32, f64)]) -> Self {
        let edges = edges
            .iter()
            .map(|&(s, t, w)| Edge::new(s, t, w))
            .collect();
        Self::new(node_count, edges)
    }

    fn build_adjacency(&self) -> Vec<Vec<(NodeId, f64)>> {
        let mut adj = vec![Vec::new(); self.node_count];
        for e in &self.edges {
            adj[e.source as usize].push((e.target, e.weight));
            adj[e.target as usize].push((e.source, e.weight)); // undirected
        }
        adj
    }

    fn adjacency(&self) -> &Vec<Vec<(NodeId, f64)>> {
        self.adjacency.get_or_init(|| self.build_adjacency())
    }
}

impl GraphView for EdgeListGraph {
    fn node_count(&self) -> usize {
        self.node_count
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    fn nodes(&self) -> Box<dyn Iterator<Item = NodeId> + '_> {
        Box::new(0..self.node_count as NodeId)
    }

    fn edges(&self) -> Box<dyn Iterator<Item = Edge> + '_> {
        Box::new(self.edges.iter().copied())
    }

    fn neighbors(&self, node: NodeId) -> Box<dyn Iterator<Item = (NodeId, f64)> + '_> {
        let adj = self.adjacency();
        if let Some(neighbors) = adj.get(node as usize) {
            Box::new(neighbors.iter().copied())
        } else {
            Box::new(std::iter::empty())
        }
    }

    fn contains_node(&self, node: NodeId) -> bool {
        (node as usize) < self.node_count
    }

    fn edge_weight(&self, source: NodeId, target: NodeId) -> Option<f64> {
        self.neighbors(source)
            .find(|(n, _)| *n == target)
            .map(|(_, w)| w)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_list_graph() {
        let graph = EdgeListGraph::from_tuples(4, &[
            (0, 1, 1.0),
            (1, 2, 2.0),
            (2, 3, 3.0),
        ]);

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 3);
        assert!(graph.contains_node(0));
        assert!(graph.contains_node(3));
        assert!(!graph.contains_node(4));
    }

    #[test]
    fn test_neighbors() {
        let graph = EdgeListGraph::from_tuples(3, &[
            (0, 1, 1.0),
            (0, 2, 2.0),
        ]);

        let neighbors: Vec<_> = graph.neighbors(0).collect();
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_weighted_degree() {
        let graph = EdgeListGraph::from_tuples(3, &[
            (0, 1, 1.0),
            (0, 2, 2.0),
        ]);

        assert_eq!(graph.weighted_degree(0), 3.0);
        assert_eq!(graph.weighted_degree(1), 1.0);
        assert_eq!(graph.weighted_degree(2), 2.0);
    }

    #[test]
    fn test_edge_weight() {
        let graph = EdgeListGraph::from_tuples(3, &[
            (0, 1, 1.5),
            (1, 2, 2.5),
        ]);

        // Existing edges
        assert_eq!(graph.edge_weight(0, 1), Some(1.5));
        assert_eq!(graph.edge_weight(1, 0), Some(1.5)); // Undirected
        assert_eq!(graph.edge_weight(1, 2), Some(2.5));

        // Non-existing edges
        assert_eq!(graph.edge_weight(0, 2), None);
        assert_eq!(graph.edge_weight(0, 0), None); // Self-loop
    }

    #[test]
    fn test_empty_graph() {
        let graph = EdgeListGraph::from_tuples(0, &[]);
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
        assert!(!graph.contains_node(0));
    }

    #[test]
    fn test_isolated_nodes() {
        let graph = EdgeListGraph::from_tuples(3, &[]);
        assert_eq!(graph.node_count(), 3);
        assert_eq!(graph.edge_count(), 0);
        assert!(graph.contains_node(0));
        assert_eq!(graph.weighted_degree(0), 0.0);
        assert_eq!(graph.neighbors(0).count(), 0);
    }

    #[test]
    fn test_edges_iterator() {
        let graph = EdgeListGraph::from_tuples(3, &[
            (0, 1, 1.0),
            (1, 2, 2.0),
        ]);

        let edges: Vec<_> = graph.edges().collect();
        assert_eq!(edges.len(), 2);
    }

    #[test]
    fn test_nodes_iterator() {
        let graph = EdgeListGraph::from_tuples(4, &[(0, 1, 1.0)]);
        let nodes: Vec<_> = graph.nodes().collect();
        assert_eq!(nodes, vec![0, 1, 2, 3]);
    }
}
