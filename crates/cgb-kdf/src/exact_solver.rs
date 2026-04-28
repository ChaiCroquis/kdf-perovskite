//! Exact Solver for Structural Entropy Optimization
//!
//! Provides exact (optimal) solutions for small graph partitioning problems
//! using Branch & Bound algorithm.
//!
//! # Design Philosophy
//!
//! From the report: "AIが直感で当たりをつけ、古典的アルゴリズムが厳密に検証する"
//!
//! - Simulated Annealing (SA): Fast approximate solution for large graphs

#![allow(missing_docs)]
//! - Branch & Bound (B&B): Exact optimal solution for small graphs
//! - Hybrid: SA for initial guess, B&B for local verification
//!
//! # Complexity
//!
//! - Worst case: O(k^N) where k = number of modules, N = number of nodes
//! - With pruning: Much faster in practice due to bound-based elimination
//! - Recommended: N ≤ 20 for exact solution, N ≤ 50 with good initial bound

use std::collections::HashMap;

use super::interning::NodeIdMap;

/// Result of exact optimization
#[derive(Clone, Debug)]
pub struct ExactResult {
    /// Optimal partition (String-based for API)
    pub partition: HashMap<String, u32>,
    /// Optimal (minimum) structural entropy
    pub optimal_entropy: f64,
    /// Number of nodes explored in B&B tree
    pub nodes_explored: u64,
    /// Number of nodes pruned
    pub nodes_pruned: u64,
    /// Whether solution is proven optimal
    pub is_optimal: bool,
    /// Time taken in milliseconds
    pub time_ms: f64,
}

/// Branch & Bound state for a partial assignment
#[derive(Clone)]
struct BBNode {
    /// Partial assignment: node_id -> module_id (None = unassigned)
    assignment: Vec<Option<u32>>,
    /// Current level (number of assigned nodes)
    level: usize,
    /// Lower bound on entropy for this partial assignment.
    /// Reserved: future branch-and-bound pruning will compare against
    /// upper-bound to skip subtrees. Currently solver uses upper-bound only.
    #[allow(dead_code)]
    lower_bound: f64,
}

/// Exact Solver using Branch & Bound
pub struct ExactSolver {
    /// Maximum nodes to explore before giving up
    pub max_nodes: u64,
    /// Maximum modules to consider
    pub max_modules: u32,
    /// Use initial bound from heuristic
    pub use_initial_bound: bool,
}

impl ExactSolver {
    /// Create new solver with default settings
    pub fn new() -> Self {
        Self {
            max_nodes: 1_000_000,
            max_modules: 10,
            use_initial_bound: true,
        }
    }

    /// Create solver with custom settings
    pub fn with_settings(max_nodes: u64, max_modules: u32) -> Self {
        Self {
            max_nodes,
            max_modules,
            use_initial_bound: true,
        }
    }

    /// Solve exactly using Branch & Bound
    ///
    /// # Arguments
    /// * `edges` - Graph edges (String-based API)
    /// * `initial_partition` - Optional initial solution for upper bound
    ///
    /// # Returns
    /// ExactResult with optimal partition or best found if limit reached
    pub fn solve(
        &self,
        edges: &[(String, String, f64)],
        initial_partition: Option<HashMap<String, u32>>,
    ) -> ExactResult {
        let start = std::time::Instant::now();

        // Convert to internal representation
        let mut id_map = NodeIdMap::new();
        let interned_edges = id_map.intern_edges(edges);
        let n = id_map.len();

        if n == 0 {
            return ExactResult {
                partition: HashMap::new(),
                optimal_entropy: 0.0,
                nodes_explored: 0,
                nodes_pruned: 0,
                is_optimal: true,
                time_ms: 0.0,
            };
        }

        // Build adjacency matrix for fast access
        let adj_matrix = self.build_adjacency_matrix(&interned_edges, n);
        let node_degrees = self.compute_degrees(&interned_edges, n);
        let total_volume: f64 = node_degrees.iter().sum();

        // Determine number of modules to try
        let k = self.max_modules.min(n as u32);

        // Initialize upper bound
        let mut best_entropy = f64::INFINITY;
        let mut best_assignment: Vec<u32> = (0..n as u32).collect(); // Singleton

        // Use initial partition if provided
        if let Some(ref init) = initial_partition {
            let init_interned = id_map.intern_partition(init);
            let init_entropy =
                self.compute_entropy(&init_interned, &adj_matrix, &node_degrees, total_volume);
            if init_entropy < best_entropy {
                best_entropy = init_entropy;
                best_assignment = init_interned;
            }
        }

        // Also try all-in-one module as initial bound
        let all_one: Vec<u32> = vec![0; n];
        let all_one_entropy =
            self.compute_entropy(&all_one, &adj_matrix, &node_degrees, total_volume);
        if all_one_entropy < best_entropy {
            best_entropy = all_one_entropy;
            best_assignment = all_one;
        }

        let mut nodes_explored = 0u64;
        let mut nodes_pruned = 0u64;
        let mut is_optimal = true;

        // Branch & Bound with DFS
        let mut stack: Vec<BBNode> = Vec::new();

        // Initial node: all unassigned
        stack.push(BBNode {
            assignment: vec![None; n],
            level: 0,
            lower_bound: 0.0, // Optimistic bound
        });

        while let Some(node) = stack.pop() {
            nodes_explored += 1;

            // Check limit
            if nodes_explored > self.max_nodes {
                is_optimal = false;
                break;
            }

            // If all assigned, evaluate
            if node.level == n {
                let assignment: Vec<u32> = node.assignment.iter().map(|x| x.unwrap()).collect();
                let entropy =
                    self.compute_entropy(&assignment, &adj_matrix, &node_degrees, total_volume);
                if entropy < best_entropy {
                    best_entropy = entropy;
                    best_assignment = assignment;
                }
                continue;
            }

            // Compute lower bound for current partial assignment
            let lower_bound = self.compute_lower_bound(
                &node.assignment,
                &adj_matrix,
                &node_degrees,
                total_volume,
            );

            // Prune if lower bound exceeds best
            if lower_bound >= best_entropy {
                nodes_pruned += 1;
                continue;
            }

            // Branch: try each module for next node
            let next_node_id = node.level;

            // First, try existing modules
            let used_modules: std::collections::HashSet<u32> =
                node.assignment.iter().filter_map(|x| *x).collect();

            for &m in &used_modules {
                let mut new_assignment = node.assignment.clone();
                new_assignment[next_node_id] = Some(m);
                stack.push(BBNode {
                    assignment: new_assignment,
                    level: node.level + 1,
                    lower_bound,
                });
            }

            // Then, try new module (if under limit)
            let next_module = used_modules.iter().max().map(|x| x + 1).unwrap_or(0);
            if next_module < k {
                let mut new_assignment = node.assignment.clone();
                new_assignment[next_node_id] = Some(next_module);
                stack.push(BBNode {
                    assignment: new_assignment,
                    level: node.level + 1,
                    lower_bound,
                });
            }
        }

        let elapsed = start.elapsed();

        ExactResult {
            partition: id_map.extern_partition(&best_assignment),
            optimal_entropy: best_entropy,
            nodes_explored,
            nodes_pruned,
            is_optimal,
            time_ms: elapsed.as_secs_f64() * 1000.0,
        }
    }

    /// Build adjacency matrix from edges
    fn build_adjacency_matrix(&self, edges: &[(u32, u32, f64)], n: usize) -> Vec<Vec<f64>> {
        let mut adj = vec![vec![0.0; n]; n];
        for &(u, v, w) in edges {
            adj[u as usize][v as usize] = w;
            adj[v as usize][u as usize] = w;
        }
        adj
    }

    /// Compute node degrees
    fn compute_degrees(&self, edges: &[(u32, u32, f64)], n: usize) -> Vec<f64> {
        let mut degrees = vec![0.0; n];
        for &(u, v, w) in edges {
            degrees[u as usize] += w;
            degrees[v as usize] += w;
        }
        degrees
    }

    /// Compute structural entropy for a complete assignment
    fn compute_entropy(
        &self,
        assignment: &[u32],
        adj_matrix: &[Vec<f64>],
        node_degrees: &[f64],
        total_volume: f64,
    ) -> f64 {
        if total_volume == 0.0 {
            return 0.0;
        }

        let n = assignment.len();
        let modules: std::collections::HashSet<u32> = assignment.iter().cloned().collect();

        let mut entropy = 0.0;

        for &m in &modules {
            let nodes_in_m: Vec<usize> = assignment
                .iter()
                .enumerate()
                .filter(|&(_, &mod_id)| mod_id == m)
                .map(|(i, _)| i)
                .collect();

            if nodes_in_m.is_empty() {
                continue;
            }

            // Volume of module
            let volume: f64 = nodes_in_m.iter().map(|&i| node_degrees[i]).sum();
            if volume == 0.0 {
                continue;
            }

            // Cut of module
            let mut cut = 0.0;
            for &i in &nodes_in_m {
                for j in 0..n {
                    if assignment[j] != m {
                        cut += adj_matrix[i][j];
                    }
                }
            }

            // Internal entropy
            let mut internal_entropy = 0.0;
            for &i in &nodes_in_m {
                let d = node_degrees[i];
                if d > 0.0 {
                    let p = d / volume;
                    internal_entropy -= p * p.log2();
                }
            }

            // Contribution to total entropy
            let v_ratio = volume / total_volume;
            entropy += v_ratio * internal_entropy;
            if v_ratio > 0.0 {
                entropy -= (cut / total_volume) * v_ratio.log2();
            }
        }

        entropy
    }

    /// Compute lower bound for partial assignment
    ///
    /// Uses the entropy of assigned nodes as lower bound
    /// (optimistic: assumes unassigned nodes contribute 0)
    fn compute_lower_bound(
        &self,
        assignment: &[Option<u32>],
        adj_matrix: &[Vec<f64>],
        node_degrees: &[f64],
        total_volume: f64,
    ) -> f64 {
        if total_volume == 0.0 {
            return 0.0;
        }

        // Create partial assignment (only assigned nodes)
        let assigned: Vec<(usize, u32)> = assignment
            .iter()
            .enumerate()
            .filter_map(|(i, &m)| m.map(|mod_id| (i, mod_id)))
            .collect();

        if assigned.is_empty() {
            return 0.0;
        }

        let modules: std::collections::HashSet<u32> = assigned.iter().map(|(_, m)| *m).collect();
        let n = assignment.len();

        let mut entropy = 0.0;

        for &m in &modules {
            let nodes_in_m: Vec<usize> = assigned
                .iter()
                .filter(|(_, mod_id)| *mod_id == m)
                .map(|(i, _)| *i)
                .collect();

            if nodes_in_m.is_empty() {
                continue;
            }

            // Volume (only assigned nodes)
            let volume: f64 = nodes_in_m.iter().map(|&i| node_degrees[i]).sum();
            if volume == 0.0 {
                continue;
            }

            // Cut (only between assigned nodes)
            let mut cut = 0.0;
            for &i in &nodes_in_m {
                for j in 0..n {
                    if let Some(other_m) = assignment[j]
                        && other_m != m
                    {
                        cut += adj_matrix[i][j];
                    }
                    // Edges to unassigned nodes are optimistically ignored
                }
            }

            // Internal entropy (lower bound)
            let mut internal_entropy = 0.0;
            for &i in &nodes_in_m {
                let d = node_degrees[i];
                if d > 0.0 {
                    let p = d / volume;
                    internal_entropy -= p * p.log2();
                }
            }

            let v_ratio = volume / total_volume;
            entropy += v_ratio * internal_entropy;
            if v_ratio > 0.0 && cut > 0.0 {
                entropy -= (cut / total_volume) * v_ratio.log2();
            }
        }

        entropy
    }
}

impl Default for ExactSolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Hybrid Solver: SA for large, B&B for small/verification
pub struct HybridSolver {
    /// Threshold for exact solution
    pub exact_threshold: usize,
    /// SA optimizer for large graphs
    sa_max_iterations: u64,
    /// Exact solver for small graphs
    exact_solver: ExactSolver,
}

impl HybridSolver {
    /// Create new hybrid solver
    pub fn new(exact_threshold: usize) -> Self {
        Self {
            exact_threshold,
            sa_max_iterations: 1000,
            exact_solver: ExactSolver::new(),
        }
    }

    /// Solve with automatic strategy selection
    pub fn solve(
        &mut self,
        edges: &[(String, String, f64)],
        initial_partition: Option<HashMap<String, u32>>,
    ) -> HybridResult {
        // Count unique nodes
        let mut nodes: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for (u, v, _) in edges {
            nodes.insert(u);
            nodes.insert(v);
        }
        let n = nodes.len();

        if n <= self.exact_threshold {
            // Use exact solver
            let result = self.exact_solver.solve(edges, initial_partition);
            HybridResult {
                partition: result.partition,
                entropy: result.optimal_entropy,
                is_exact: result.is_optimal,
                strategy_used: SolverStrategy::Exact,
                nodes_explored: result.nodes_explored,
                time_ms: result.time_ms,
            }
        } else {
            // Use SA then optionally verify subproblems
            let mut sa =
                super::sleep_mode::SleepModeOptimizer::new(1.0, 0.001, self.sa_max_iterations, 100);
            let start = std::time::Instant::now();
            let sa_result = sa.run_nrem_phase(edges, initial_partition);
            let elapsed = start.elapsed();

            HybridResult {
                partition: sa_result.partition,
                entropy: sa_result.final_entropy,
                is_exact: false,
                strategy_used: SolverStrategy::SimulatedAnnealing,
                nodes_explored: sa_result.iterations * 10,
                time_ms: elapsed.as_secs_f64() * 1000.0,
            }
        }
    }
}

/// Strategy used by hybrid solver
#[derive(Clone, Debug, PartialEq)]
pub enum SolverStrategy {
    Exact,
    SimulatedAnnealing,
    Hybrid,
}

/// Result from hybrid solver
#[derive(Clone, Debug)]
pub struct HybridResult {
    pub partition: HashMap<String, u32>,
    pub entropy: f64,
    pub is_exact: bool,
    pub strategy_used: SolverStrategy,
    pub nodes_explored: u64,
    pub time_ms: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_small_graph() -> Vec<(String, String, f64)> {
        vec![
            ("a".to_string(), "b".to_string(), 1.0),
            ("b".to_string(), "c".to_string(), 1.0),
            ("c".to_string(), "a".to_string(), 1.0),
            ("d".to_string(), "e".to_string(), 1.0),
            ("a".to_string(), "d".to_string(), 0.1), // Weak link between clusters
        ]
    }

    #[test]
    fn test_exact_solver_small() {
        let edges = create_small_graph();
        let solver = ExactSolver::new();
        let result = solver.solve(&edges, None);

        assert!(result.is_optimal);
        assert!(result.optimal_entropy >= 0.0);
        assert_eq!(result.partition.len(), 5);
        println!(
            "Exact solution: entropy={:.4}, explored={}",
            result.optimal_entropy, result.nodes_explored
        );
    }

    #[test]
    fn test_exact_solver_with_initial() {
        let edges = create_small_graph();
        let initial: HashMap<String, u32> = [
            ("a".to_string(), 0),
            ("b".to_string(), 0),
            ("c".to_string(), 0),
            ("d".to_string(), 1),
            ("e".to_string(), 1),
        ]
        .into_iter()
        .collect();

        let solver = ExactSolver::new();
        let result = solver.solve(&edges, Some(initial));

        assert!(result.is_optimal);
        println!(
            "With initial: entropy={:.4}, pruned={}",
            result.optimal_entropy, result.nodes_pruned
        );
    }

    #[test]
    fn test_hybrid_solver_auto() {
        let edges = create_small_graph();
        let mut solver = HybridSolver::new(10); // Exact for <= 10 nodes

        let result = solver.solve(&edges, None);

        assert_eq!(result.strategy_used, SolverStrategy::Exact);
        assert!(result.is_exact);
    }

    #[test]
    fn test_hybrid_solver_large() {
        // Generate larger graph
        let mut edges = Vec::new();
        for i in 0..30 {
            for j in (i + 1)..30 {
                if (i + j) % 5 == 0 {
                    edges.push((format!("n{}", i), format!("n{}", j), 1.0));
                }
            }
        }

        let mut solver = HybridSolver::new(10); // 30 > 10, so SA
        let result = solver.solve(&edges, None);

        assert_eq!(result.strategy_used, SolverStrategy::SimulatedAnnealing);
        assert!(!result.is_exact);
    }

    #[test]
    fn test_empty_graph() {
        let edges: Vec<(String, String, f64)> = vec![];
        let solver = ExactSolver::new();
        let result = solver.solve(&edges, None);

        assert!(result.is_optimal);
        assert_eq!(result.optimal_entropy, 0.0);
    }

    #[test]
    fn test_optimal_is_better_than_random() {
        let edges = create_small_graph();
        let solver = ExactSolver::new();

        // Random partition
        let random_partition: HashMap<String, u32> = [
            ("a".to_string(), 0),
            ("b".to_string(), 1),
            ("c".to_string(), 2),
            ("d".to_string(), 3),
            ("e".to_string(), 4),
        ]
        .into_iter()
        .collect();

        let random_entropy = {
            let mut id_map = NodeIdMap::new();
            let interned_edges = id_map.intern_edges(&edges);
            let n = id_map.len();
            let adj = solver.build_adjacency_matrix(&interned_edges, n);
            let deg = solver.compute_degrees(&interned_edges, n);
            let total: f64 = deg.iter().sum();
            let assignment = id_map.intern_partition(&random_partition);
            solver.compute_entropy(&assignment, &adj, &deg, total)
        };

        let result = solver.solve(&edges, None);

        assert!(result.optimal_entropy <= random_entropy);
        println!(
            "Random: {:.4}, Optimal: {:.4}",
            random_entropy, result.optimal_entropy
        );
    }

    #[test]
    #[ignore] // Run with: cargo test --release benchmark_exact -- --ignored --nocapture
    fn benchmark_exact_vs_sa() {
        use std::time::Instant;

        fn generate_clustered_graph(n: usize, clusters: usize) -> Vec<(String, String, f64)> {
            let mut edges = Vec::new();
            let nodes_per_cluster = n / clusters;

            for c in 0..clusters {
                let start = c * nodes_per_cluster;
                let end = start + nodes_per_cluster;

                // Dense intra-cluster edges
                for i in start..end {
                    for j in (i + 1)..end {
                        edges.push((format!("n{}", i), format!("n{}", j), 1.0));
                    }
                }

                // Sparse inter-cluster edges
                if c + 1 < clusters {
                    edges.push((format!("n{}", start), format!("n{}", end), 0.1));
                }
            }
            edges
        }

        println!("\nExact vs SA Benchmark");
        println!("=====================");
        println!(
            "{:<8} {:<12} {:<12} {:<12} {:<10}",
            "Nodes", "Exact(ms)", "SA(ms)", "Entropy", "Optimal?"
        );
        println!("{}", "-".repeat(60));

        for &n in &[6, 8, 10, 12, 15] {
            let edges = generate_clustered_graph(n, 2);

            // Exact solver
            let solver = ExactSolver::new();
            let start = Instant::now();
            let exact_result = solver.solve(&edges, None);
            let exact_time = start.elapsed().as_secs_f64() * 1000.0;

            // SA solver
            let mut sa = crate::sleep_mode::SleepModeOptimizer::new(1.0, 0.01, 500, 50);
            let start = Instant::now();
            let sa_result = sa.run_nrem_phase(&edges, None);
            let sa_time = start.elapsed().as_secs_f64() * 1000.0;

            let is_optimal = (sa_result.final_entropy - exact_result.optimal_entropy).abs() < 0.001;

            println!(
                "{:<8} {:<12.2} {:<12.2} {:<12.4} {:<10}",
                n,
                exact_time,
                sa_time,
                exact_result.optimal_entropy,
                if is_optimal {
                    "✓ SA=Opt"
                } else {
                    "SA differs"
                }
            );
        }

        println!("\nConclusion: B&B is exact but exponential, SA is fast but approximate.");
        println!("HybridSolver automatically selects the best strategy based on graph size.");
    }
}
