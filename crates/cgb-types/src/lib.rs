//! Basic type definitions for CGB
//!
//! This crate provides fundamental types used across the CGB ecosystem.

/// Node identifier
pub type NodeId = u32;

/// Module (cluster) identifier
pub type ModuleId = u32;

/// Edge with weight
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Edge {
    /// Source node
    pub source: NodeId,
    /// Target node
    pub target: NodeId,
    /// Edge weight (always f64)
    pub weight: f64,
}

impl Edge {
    /// Create a new edge
    pub fn new(source: NodeId, target: NodeId, weight: f64) -> Self {
        Self { source, target, weight }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edge_new() {
        let edge = Edge::new(1, 2, 3.5);
        assert_eq!(edge.source, 1);
        assert_eq!(edge.target, 2);
        assert!((edge.weight - 3.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_edge_copy() {
        let edge1 = Edge::new(0, 1, 1.0);
        let edge2 = edge1; // Copy
        assert_eq!(edge1.source, edge2.source);
    }

    #[test]
    fn test_edge_equality() {
        let edge1 = Edge::new(0, 1, 1.0);
        let edge2 = Edge::new(0, 1, 1.0);
        let edge3 = Edge::new(0, 1, 2.0);
        assert_eq!(edge1, edge2);
        assert_ne!(edge1, edge3);
    }
}
