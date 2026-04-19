//! Sleep mode optimizer and NREM phase implementation

use std::collections::HashMap;

use crate::interning::NodeIdMap;

use super::{
    context::NodeMoveContext,
    cooling::AdaptiveCoolingScheduler,
    entropy_cache::IncrementalEntropyCache,
    rng::SimpleRng,
};

/// NREM optimization result
#[derive(Clone, Debug)]
pub struct NREMResult {
    /// Optimized partition (String-based for API compatibility)
    pub partition: HashMap<String, u32>,
    /// Initial entropy
    pub initial_entropy: f64,
    /// Final entropy
    pub final_entropy: f64,
    /// Entropy reduction
    pub entropy_reduction: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Iterations performed
    pub iterations: u64,
    /// Acceptance rate
    pub acceptance_rate: f64,
    /// Final module count
    pub module_count: usize,
}

/// Sleep Mode Optimizer
///
/// High-level interface for NREM phase (structural compression).
/// Uses boundary conversion pattern: String at API, u32 internally.
pub struct SleepModeOptimizer {
    /// Cache for incremental updates
    cache: IncrementalEntropyCache,
    /// Cooling scheduler
    scheduler: AdaptiveCoolingScheduler,
    /// Maximum iterations
    pub max_iterations: u64,
    /// Moves per iteration
    pub moves_per_iteration: usize,
    /// Best partition found (u32-based internally)
    best_partition: Vec<u32>,
    /// Best entropy found
    best_entropy: f64,
    /// Node ID map for String ↔ u32 conversion
    id_map: NodeIdMap,
}

impl SleepModeOptimizer {
    /// Create a new optimizer
    pub fn new(
        initial_temperature: f64,
        final_temperature: f64,
        max_iterations: u64,
        resync_interval: u64,
    ) -> Self {
        Self {
            cache: IncrementalEntropyCache::new(resync_interval),
            scheduler: AdaptiveCoolingScheduler::new(
                initial_temperature,
                final_temperature,
                0.99,
                0.9999,
                0.1,
                100,
            ),
            max_iterations,
            moves_per_iteration: 10,
            best_partition: Vec::new(),
            best_entropy: f64::INFINITY,
            id_map: NodeIdMap::new(),
        }
    }

    /// Run NREM phase optimization
    ///
    /// # Arguments
    /// * `edges` - Graph edges as (u, v, weight) tuples (String-based API)
    /// * `initial_partition` - Initial partition (None = each node is own module)
    pub fn run_nrem_phase(
        &mut self,
        edges: &[(String, String, f64)],
        initial_partition: Option<HashMap<String, u32>>,
    ) -> NREMResult {
        // Handle empty graph case
        if edges.is_empty() {
            return NREMResult {
                partition: initial_partition.unwrap_or_default(),
                initial_entropy: 0.0,
                final_entropy: 0.0,
                entropy_reduction: 0.0,
                compression_ratio: 0.0,
                iterations: 0,
                acceptance_rate: 0.0,
                module_count: 0,
            };
        }

        // === BOUNDARY: Convert String → u32 ===
        self.id_map = NodeIdMap::new();

        // Intern all edges (this populates the id_map)
        let interned_edges = self.id_map.intern_edges(edges);
        let node_count = self.id_map.len();

        if node_count == 0 {
            return NREMResult {
                partition: initial_partition.unwrap_or_default(),
                initial_entropy: 0.0,
                final_entropy: 0.0,
                entropy_reduction: 0.0,
                compression_ratio: 0.0,
                iterations: 0,
                acceptance_rate: 0.0,
                module_count: 0,
            };
        }

        // Build initial partition (u32-based)
        let partition: Vec<u32> = match initial_partition {
            Some(ref p) => self.id_map.intern_partition(p),
            None => (0..node_count as u32).collect(), // Each node in own module
        };

        let modules: Vec<u32> = partition
            .iter()
            .cloned()
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Handle single module case
        if modules.len() <= 1 {
            let initial_entropy =
                self.cache
                    .initialize_from_interned(&interned_edges, &partition, node_count);
            return NREMResult {
                partition: self.id_map.extern_partition(&partition),
                initial_entropy,
                final_entropy: initial_entropy,
                entropy_reduction: 0.0,
                compression_ratio: 0.0,
                iterations: 0,
                acceptance_rate: 0.0,
                module_count: modules.len(),
            };
        }

        // === INTERNAL: All u32 operations ===

        // Build adjacency (u32-based)
        let mut adjacency: Vec<Vec<(u32, f64)>> = vec![Vec::new(); node_count];
        for &(u, v, w) in &interned_edges {
            adjacency[u as usize].push((v, w));
            adjacency[v as usize].push((u, w));
        }

        // Compute node degrees
        let mut node_degrees: Vec<f64> = vec![0.0; node_count];
        for &(u, v, w) in &interned_edges {
            node_degrees[u as usize] += w;
            node_degrees[v as usize] += w;
        }

        // Initialize cache
        let initial_entropy =
            self.cache
                .initialize_from_interned(&interned_edges, &partition, node_count);
        self.best_entropy = initial_entropy;
        self.best_partition = partition;

        self.scheduler.reset();

        let mut accepted_moves = 0u64;
        let mut rejected_moves = 0u64;
        let mut rng = SimpleRng::new(42);

        for _iteration in 0..self.max_iterations {
            if self.scheduler.is_converged() {
                break;
            }

            for _ in 0..self.moves_per_iteration {
                // Random node (u32)
                let node_id = (rng.next_usize() % node_count) as u32;

                let current_module = match self.cache.get_module(node_id) {
                    Some(m) => m,
                    None => continue,
                };

                // Random target module
                let target_idx = rng.next_usize() % modules.len();
                let target_module = modules[target_idx];

                if target_module == current_module {
                    continue;
                }

                // Compute neighbor weights (all u32 operations)
                let neighbors = &adjacency[node_id as usize];
                let mut neighbors_in_from = 0.0;
                let mut neighbors_in_to = 0.0;
                let mut neighbors_outside = 0.0;

                for &(neighbor_id, weight) in neighbors {
                    if let Some(neighbor_module) = self.cache.get_module(neighbor_id) {
                        if neighbor_module == current_module {
                            neighbors_in_from += weight;
                        } else if neighbor_module == target_module {
                            neighbors_in_to += weight;
                        } else {
                            neighbors_outside += weight;
                        }
                    }
                }

                let node_degree = node_degrees[node_id as usize];

                // Build move context
                let move_ctx = NodeMoveContext {
                    node_id,
                    from_module: current_module,
                    to_module: target_module,
                    node_degree,
                    neighbors_in_from,
                    neighbors_in_to,
                    neighbors_outside,
                };

                // Compute delta
                let delta = self.cache.compute_move_delta(&move_ctx);

                // Metropolis acceptance
                let accept = if delta < 0.0 {
                    true
                } else {
                    let t = self.scheduler.get_temperature();
                    if t > 0.0 {
                        let prob = (-delta / t).exp();
                        rng.next_f64() < prob
                    } else {
                        false
                    }
                };

                if accept {
                    let new_entropy = self.cache.apply_move(&move_ctx);

                    accepted_moves += 1;

                    if new_entropy < self.best_entropy {
                        self.best_entropy = new_entropy;
                        self.best_partition = self.cache.get_partition().to_vec();
                    }
                } else {
                    rejected_moves += 1;
                }
            }

            // Update temperature
            let current_entropy = self.cache.get_entropy();
            self.scheduler.update(current_entropy);
        }

        let total_moves = accepted_moves + rejected_moves;
        let acceptance_rate = if total_moves > 0 {
            accepted_moves as f64 / total_moves as f64
        } else {
            0.0
        };

        // === BOUNDARY: Convert u32 → String ===
        NREMResult {
            partition: self.id_map.extern_partition(&self.best_partition),
            initial_entropy,
            final_entropy: self.best_entropy,
            entropy_reduction: initial_entropy - self.best_entropy,
            compression_ratio: if initial_entropy > 0.0 {
                1.0 - self.best_entropy / initial_entropy
            } else {
                0.0
            },
            iterations: self
                .max_iterations
                .min((accepted_moves + rejected_moves) / self.moves_per_iteration as u64),
            acceptance_rate,
            module_count: self.cache.get_module_count(),
        }
    }

    /// Get best partition (String-based for API)
    pub fn get_best_partition(&self) -> HashMap<String, u32> {
        self.id_map.extern_partition(&self.best_partition)
    }

    /// Get best entropy
    pub fn get_best_entropy(&self) -> f64 {
        self.best_entropy
    }
}

impl Default for SleepModeOptimizer {
    fn default() -> Self {
        Self::new(1.0, 0.001, 10000, 1000)
    }
}
