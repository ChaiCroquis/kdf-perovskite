//! Causal-Enhanced NREM Optimizer

use std::collections::{HashMap, HashSet};
use super::types::CausalLink;
use super::partition::CausalPartitionBuilder;

/// Causal-Enhanced NREM Optimizer
///
/// Uses TE causal links to build initial partition,
/// accelerating NREM optimization convergence.
pub struct CausalEnhancedNREMOptimizer {
    /// Partition builder
    partition_builder: CausalPartitionBuilder,
    /// Maximum NREM iterations
    pub nrem_max_iterations: u64,
    /// Use causal initialization
    pub use_causal_init: bool,
    /// Statistics
    stats: CausalNREMStats,
}

/// Statistics for Causal-Enhanced NREM
#[derive(Clone, Debug, Default)]
pub struct CausalNREMStats {
    /// Number of optimizations
    pub optimizations: u64,
    /// Number with causal init
    pub causal_inits: u64,
    /// Average iterations with causal init
    pub avg_iterations_with_causal: f64,
    /// Average iterations without causal init
    pub avg_iterations_without_causal: f64,
}

impl CausalEnhancedNREMOptimizer {
    /// Create a new optimizer
    pub fn new(te_threshold: f64, nrem_max_iterations: u64, use_causal_init: bool) -> Self {
        Self {
            partition_builder: CausalPartitionBuilder::new(te_threshold, 2, 50),
            nrem_max_iterations,
            use_causal_init,
            stats: CausalNREMStats::default(),
        }
    }
}

impl Default for CausalEnhancedNREMOptimizer {
    fn default() -> Self {
        Self::new(0.01, 1000, true)
    }
}

impl CausalEnhancedNREMOptimizer {
    /// Optimize with causal links
    pub fn optimize_with_causal_links(
        &mut self,
        edges: &[(String, String, f64)],
        causal_links: &[CausalLink],
    ) -> CausalNREMResult {
        use super::super::sleep_mode::SleepModeOptimizer;

        self.stats.optimizations += 1;

        // Collect all nodes
        let mut nodes: HashSet<String> = HashSet::new();
        for (u, v, _) in edges {
            nodes.insert(u.clone());
            nodes.insert(v.clone());
        }
        let node_list: Vec<String> = nodes.into_iter().collect();

        // Build initial partition
        let (initial_partition, init_type) = if self.use_causal_init && !causal_links.is_empty() {
            let partition = self.partition_builder.build_partition_from_links(
                causal_links,
                Some(&node_list),
            );
            self.stats.causal_inits += 1;
            (partition, "causal")
        } else {
            let partition: HashMap<String, u32> = node_list
                .iter()
                .enumerate()
                .map(|(i, n)| (n.clone(), i as u32))
                .collect();
            (partition, "singleton")
        };

        let initial_modules = initial_partition.values().collect::<HashSet<_>>().len();

        // Run NREM optimization
        let mut optimizer = SleepModeOptimizer::new(
            1.0,
            0.001,
            self.nrem_max_iterations,
            1000,
        );

        let nrem_result = optimizer.run_nrem_phase(edges, Some(initial_partition.clone()));

        // Update statistics
        let iterations = nrem_result.iterations;
        if init_type == "causal" {
            let n = self.stats.causal_inits as f64;
            let avg = self.stats.avg_iterations_with_causal;
            self.stats.avg_iterations_with_causal = (avg * (n - 1.0) + iterations as f64) / n;
        } else {
            let n_without = (self.stats.optimizations - self.stats.causal_inits) as f64;
            if n_without > 0.0 {
                let avg = self.stats.avg_iterations_without_causal;
                self.stats.avg_iterations_without_causal = (avg * (n_without - 1.0) + iterations as f64) / n_without;
            }
        }

        CausalNREMResult {
            partition: nrem_result.partition,
            initial_entropy: nrem_result.initial_entropy,
            final_entropy: nrem_result.final_entropy,
            entropy_reduction: nrem_result.entropy_reduction,
            compression_ratio: nrem_result.compression_ratio,
            iterations: nrem_result.iterations,
            init_type: init_type.to_string(),
            initial_modules,
            initial_partition,
        }
    }

    /// Get statistics
    pub fn get_statistics(&self) -> CausalNREMStats {

        // Future: speedup estimation when both values are non-zero
        self.stats.clone()
    }
}

/// Result of causal-enhanced NREM optimization
#[derive(Clone, Debug)]
pub struct CausalNREMResult {
    /// Optimized partition
    pub partition: HashMap<String, u32>,
    /// Initial structural entropy
    pub initial_entropy: f64,
    /// Final structural entropy
    pub final_entropy: f64,
    /// Entropy reduction
    pub entropy_reduction: f64,
    /// Compression ratio
    pub compression_ratio: f64,
    /// Iterations performed
    pub iterations: u64,
    /// Initialization type
    pub init_type: String,
    /// Number of initial modules
    pub initial_modules: usize,
    /// Initial partition used
    pub initial_partition: HashMap<String, u32>,
}
