//! Incremental entropy cache for O(1) updates

use std::collections::HashMap;

use crate::interning::NodeIdMap;

use super::context::{ModuleStats, NodeMoveContext};

/// Incremental Entropy Cache for O(1) updates
///
/// Uses u32 node IDs internally. String conversion happens at boundaries.
pub struct IncrementalEntropyCache {
    /// Resync interval to prevent floating-point drift
    pub resync_interval: u64,
    /// Current step count
    step_count: u64,
    /// Module statistics
    module_stats: HashMap<u32, ModuleStats>,
    /// Node to module mapping (indexed by node ID)
    node_to_module: Vec<u32>,
    /// Total graph volume
    total_volume: f64,
    /// Current total entropy
    total_entropy: f64,
    /// Accumulated delta for drift detection
    accumulated_delta: f64,
}

impl IncrementalEntropyCache {
    /// Create a new cache
    pub fn new(resync_interval: u64) -> Self {
        Self {
            resync_interval,
            step_count: 0,
            module_stats: HashMap::new(),
            node_to_module: Vec::new(),
            total_volume: 0.0,
            total_entropy: 0.0,
            accumulated_delta: 0.0,
        }
    }

    /// Initialize from interned graph data
    ///
    /// # Arguments
    /// * `edges` - List of (u_id, v_id, weight) tuples (already interned)
    /// * `partition` - Node ID to module ID mapping (indexed by node ID)
    /// * `node_count` - Total number of nodes
    pub fn initialize_from_interned(
        &mut self,
        edges: &[(u32, u32, f64)],
        partition: &[u32],
        node_count: usize,
    ) -> f64 {
        self.node_to_module = partition.to_vec();
        self.module_stats.clear();
        self.step_count = 0;
        self.accumulated_delta = 0.0;

        // Compute total volume
        self.total_volume = 0.0;
        for (_, _, weight) in edges {
            self.total_volume += 2.0 * weight; // Undirected graph
        }

        if self.total_volume == 0.0 {
            self.total_entropy = 0.0;
            return 0.0;
        }

        // Collect all modules
        let modules: std::collections::HashSet<u32> = partition.iter().cloned().collect();

        // Compute node degrees
        let mut node_degrees: Vec<f64> = vec![0.0; node_count];
        for (u, v, weight) in edges {
            node_degrees[*u as usize] += weight;
            node_degrees[*v as usize] += weight;
        }

        // Initialize module stats
        for module_id in modules {
            let mut stats = ModuleStats::new();

            // Find nodes in this module
            for (node_id, &node_module) in partition.iter().enumerate() {
                if node_module == module_id {
                    let degree = node_degrees[node_id];
                    stats.set_degree(node_id as u32, degree);
                    stats.volume += degree;
                }
            }

            // Compute cut
            for (u, v, weight) in edges {
                let u_module = partition.get(*u as usize).copied();
                let v_module = partition.get(*v as usize).copied();

                if u_module == Some(module_id) && v_module != Some(module_id) {
                    stats.cut += weight;
                }
                if v_module == Some(module_id) && u_module != Some(module_id) {
                    stats.cut += weight;
                }
            }

            // Compute internal entropy
            stats.recompute_internal_entropy();

            self.module_stats.insert(module_id, stats);
        }

        self.total_entropy = self.compute_total_entropy();
        self.total_entropy
    }

    /// Initialize from string-based graph data (legacy API)
    ///
    /// This creates a temporary NodeIdMap for conversion.
    /// For repeated calls, prefer using initialize_from_interned with a persistent NodeIdMap.
    pub fn initialize_from_edges(
        &mut self,
        edges: &[(String, String, f64)],
        partition: &HashMap<String, u32>,
    ) -> f64 {
        let mut id_map = NodeIdMap::new();

        // Intern all nodes from partition first (preserves order)
        for name in partition.keys() {
            id_map.get_or_insert(name);
        }

        // Intern edges
        let interned_edges = id_map.intern_edges(edges);

        // Convert partition
        let interned_partition = id_map.intern_partition(partition);

        self.initialize_from_interned(&interned_edges, &interned_partition, id_map.len())
    }

    /// Compute total entropy from module stats
    fn compute_total_entropy(&self) -> f64 {
        if self.total_volume == 0.0 {
            return 0.0;
        }

        let mut entropy = 0.0;

        for stats in self.module_stats.values() {
            if stats.volume == 0.0 {
                continue;
            }

            let v_ratio = stats.volume / self.total_volume;

            // Internal entropy contribution
            entropy += v_ratio * stats.internal_entropy;

            // Cut entropy contribution
            if v_ratio > 0.0 {
                entropy -= (stats.cut / self.total_volume) * v_ratio.log2();
            }
        }

        entropy
    }

    /// Compute entropy change from moving a node (using u32 IDs)
    pub fn compute_move_delta(&self, ctx: &NodeMoveContext) -> f64 {
        if ctx.from_module == ctx.to_module {
            return 0.0;
        }

        let from_stats = match self.module_stats.get(&ctx.from_module) {
            Some(s) => s,
            None => return 0.0,
        };

        let to_stats = match self.module_stats.get(&ctx.to_module) {
            Some(s) => s,
            None => return 0.0,
        };

        // Compute new volumes
        let new_from_volume = from_stats.volume - ctx.node_degree;
        let new_to_volume = to_stats.volume + ctx.node_degree;

        // Compute new cuts
        let new_from_cut = from_stats.cut + ctx.neighbors_in_from
            - (ctx.node_degree - ctx.neighbors_in_from - ctx.neighbors_in_to);
        let new_to_cut = to_stats.cut - ctx.neighbors_in_to
            + (ctx.node_degree - ctx.neighbors_in_from - ctx.neighbors_in_to);

        // Simplified entropy change estimation
        let old_contribution = self.module_entropy_contribution(from_stats)
            + self.module_entropy_contribution(to_stats);

        let mut new_from_stats = from_stats.clone();
        new_from_stats.volume = new_from_volume;
        new_from_stats.cut = new_from_cut.max(0.0);

        let mut new_to_stats = to_stats.clone();
        new_to_stats.volume = new_to_volume;
        new_to_stats.cut = new_to_cut.max(0.0);

        let new_contribution = self.module_entropy_contribution(&new_from_stats)
            + self.module_entropy_contribution(&new_to_stats);

        new_contribution - old_contribution
    }

    /// Compute entropy contribution of a single module
    fn module_entropy_contribution(&self, stats: &ModuleStats) -> f64 {
        if self.total_volume == 0.0 || stats.volume == 0.0 {
            return 0.0;
        }

        let v_ratio = stats.volume / self.total_volume;
        let mut contribution = v_ratio * stats.internal_entropy;

        if v_ratio > 0.0 {
            contribution -= (stats.cut / self.total_volume) * v_ratio.log2();
        }

        contribution
    }

    /// Apply a node move and update cache (using u32 ID)
    pub fn apply_move(&mut self, ctx: &NodeMoveContext) -> f64 {
        if ctx.from_module == ctx.to_module {
            return self.total_entropy;
        }

        let delta = self.compute_move_delta(ctx);

        // Update from_module stats
        if let Some(stats) = self.module_stats.get_mut(&ctx.from_module) {
            stats.volume -= ctx.node_degree;
            stats.remove_node(ctx.node_id);
            stats.cut += ctx.neighbors_in_from;
            stats.cut -= ctx.neighbors_outside;
            stats.cut = stats.cut.max(0.0);
            stats.recompute_internal_entropy();
        }

        // Update to_module stats
        if let Some(stats) = self.module_stats.get_mut(&ctx.to_module) {
            stats.volume += ctx.node_degree;
            stats.set_degree(ctx.node_id, ctx.node_degree);
            stats.cut -= ctx.neighbors_in_to;
            stats.cut += ctx.neighbors_outside;
            stats.cut = stats.cut.max(0.0);
            stats.recompute_internal_entropy();
        }

        // Update node mapping
        if (ctx.node_id as usize) < self.node_to_module.len() {
            self.node_to_module[ctx.node_id as usize] = ctx.to_module;
        }

        // Update entropy
        self.total_entropy += delta;
        self.accumulated_delta += delta.abs();
        self.step_count += 1;

        // Check for resync
        if self.step_count >= self.resync_interval {
            self.total_entropy = self.compute_total_entropy();
            self.step_count = 0;
            self.accumulated_delta = 0.0;
        }

        self.total_entropy
    }

    /// Get current entropy
    pub fn get_entropy(&self) -> f64 {
        self.total_entropy
    }

    /// Get partition (u32-based)
    pub fn get_partition(&self) -> &[u32] {
        &self.node_to_module
    }

    /// Get module for a node
    pub fn get_module(&self, node_id: u32) -> Option<u32> {
        self.node_to_module.get(node_id as usize).copied()
    }

    /// Get module count (non-empty modules)
    pub fn get_module_count(&self) -> usize {
        self.module_stats
            .values()
            .filter(|s| s.volume > 0.0)
            .count()
    }
}
