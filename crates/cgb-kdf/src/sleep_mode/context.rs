//! Context structures for node move operations and module statistics

use std::collections::HashMap;

/// Context for a node move operation
///
/// Contains all neighbor statistics needed to compute entropy changes.
#[derive(Clone, Debug)]
pub struct NodeMoveContext {
    /// Node being moved
    pub node_id: u32,
    /// Source module
    pub from_module: u32,
    /// Destination module
    pub to_module: u32,
    /// Total degree of the node
    pub node_degree: f64,
    /// Sum of edge weights to neighbors in source module
    pub neighbors_in_from: f64,
    /// Sum of edge weights to neighbors in destination module
    pub neighbors_in_to: f64,
    /// Sum of edge weights to neighbors outside both modules
    pub neighbors_outside: f64,
}

/// Module statistics for incremental entropy computation
///
/// Uses u32 node IDs internally for performance.
#[derive(Clone, Debug)]
pub struct ModuleStats {
    /// Volume: sum of degrees of nodes in module
    pub volume: f64,
    /// Cut: sum of edge weights going outside module
    pub cut: f64,
    /// Internal entropy term
    pub internal_entropy: f64,
    /// Degree of each node in module (indexed by node ID)
    node_degrees: HashMap<u32, f64>,
}

impl ModuleStats {
    /// Create new empty stats
    pub fn new() -> Self {
        Self {
            volume: 0.0,
            cut: 0.0,
            internal_entropy: 0.0,
            node_degrees: HashMap::new(),
        }
    }

    /// Get node degree
    pub fn get_degree(&self, node_id: u32) -> f64 {
        self.node_degrees.get(&node_id).copied().unwrap_or(0.0)
    }

    /// Set node degree
    pub fn set_degree(&mut self, node_id: u32, degree: f64) {
        self.node_degrees.insert(node_id, degree);
    }

    /// Remove node
    pub fn remove_node(&mut self, node_id: u32) {
        self.node_degrees.remove(&node_id);
    }

    /// Iterate over node degrees
    pub fn degrees(&self) -> impl Iterator<Item = (&u32, &f64)> {
        self.node_degrees.iter()
    }

    /// Recompute internal entropy from current node degrees
    pub fn recompute_internal_entropy(&mut self) {
        self.internal_entropy = 0.0;
        if self.volume > 0.0 {
            for &d in self.node_degrees.values() {
                if d > 0.0 {
                    let p = d / self.volume;
                    self.internal_entropy -= p * p.log2();
                }
            }
        }
    }
}

impl Default for ModuleStats {
    fn default() -> Self {
        Self::new()
    }
}
