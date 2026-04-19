//! KDF Think Engine
//!
//! Provides analysis and insight generation.

use super::graph::SimpleGraph;
use super::super::framework::Layer;
use super::super::text_processor::TextProcessor;

/// KDF-based Think Engine
///
/// Provides analysis and insight generation.
pub struct KDFThinkEngine {
    /// Internal graph
    graph: SimpleGraph,
    /// Text processor
    text_processor: TextProcessor,
    /// Statistics
    stats: ThinkEngineStats,
}

/// Think engine statistics
#[derive(Clone, Debug, Default)]
pub struct ThinkEngineStats {
    /// Analyses performed
    pub analyses_performed: u64,
    /// Insights generated
    pub insights_generated: u64,
    /// Analogies found
    pub analogies_found: u64,
}

impl KDFThinkEngine {
    /// Create a new think engine
    pub fn new() -> Self {
        Self {
            graph: SimpleGraph::new(),
            text_processor: TextProcessor::new(),
            stats: ThinkEngineStats::default(),
        }
    }

    /// Create with existing graph (internal use)
    #[allow(dead_code)]
    pub(crate) fn with_graph(graph: SimpleGraph) -> Self {
        Self {
            graph,
            text_processor: TextProcessor::new(),
            stats: ThinkEngineStats::default(),
        }
    }

    /// Quick analysis
    pub fn quick_analysis(&mut self) -> ThinkAnalysisResult {
        self.stats.analyses_performed += 1;

        // Extract clusters from each layer
        let edge_clusters = self.graph.extract_clusters(Layer::Edge);
        let rare_clusters = self.graph.extract_clusters(Layer::Rare);
        let core_clusters = self.graph.extract_clusters(Layer::Core);

        // Generate insights
        let insights = self.generate_insights(&edge_clusters, &rare_clusters, &core_clusters);
        self.stats.insights_generated += insights.len() as u64;

        let total_nodes = self.graph.nodes.len();

        ThinkAnalysisResult {
            insights,
            cluster_summary: ClusterSummary {
                edge_clusters: edge_clusters.len(),
                rare_clusters: rare_clusters.len(),
                core_clusters: core_clusters.len(),
                total_nodes,
            },
            layer_health: LayerHealth {
                edge: 0.5,
                rare: 0.5,
                core: 0.5,
            },
            analysis_depth: "quick".to_string(),
        }
    }

    /// Deep analysis
    pub fn deep_analysis(&mut self, simulation_steps: usize) -> ThinkAnalysisResult {
        self.stats.analyses_performed += 1;

        // Run simulation steps
        for _ in 0..simulation_steps {
            self.graph.step_count += 1;
        }

        // Get quick analysis
        let mut result = self.quick_analysis();
        result.analysis_depth = "deep".to_string();

        result
    }

    /// Generate insights from clusters
    fn generate_insights(
        &self,
        edge_clusters: &[Vec<String>],
        rare_clusters: &[Vec<String>],
        core_clusters: &[Vec<String>],
    ) -> Vec<String> {
        let mut insights = Vec::new();

        // Edge layer analysis
        if !edge_clusters.is_empty() {
            let avg_size: f64 = edge_clusters.iter().map(|c| c.len() as f64).sum::<f64>()
                / edge_clusters.len() as f64;
            if avg_size > 5.0 {
                insights.push(format!(
                    "Edge層に大きなクラスタ（平均{:.1}ノード）が形成されています。知識の統合が進んでいます。",
                    avg_size
                ));
            } else if edge_clusters.len() > 10 {
                insights.push(format!(
                    "Edge層に{}個の小さなクラスタがあります。新しい知識が多様なトピックに分散しています。",
                    edge_clusters.len()
                ));
            }
        }

        // Rare layer analysis
        if !rare_clusters.is_empty() {
            let isolated_count = rare_clusters.iter().filter(|c| c.len() == 1).count();
            if isolated_count > 0 {
                insights.push(format!(
                    "Rare層に{}個の孤立ノードがあります。これらは潜在的に重要な「孤立した真実」かもしれません。",
                    isolated_count
                ));
            }
        }

        // Core layer analysis
        if !core_clusters.is_empty() {
            let largest_core = core_clusters.iter().map(|c| c.len()).max().unwrap_or(0);
            if largest_core > 10 {
                insights.push(format!(
                    "Core層に大きなクラスタ（{}ノード）があります。確立された知識体系が形成されています。",
                    largest_core
                ));
            }
        }

        // Default insight
        if insights.is_empty() {
            insights.push("システムは正常に動作しています。特筆すべきパターンは検出されませんでした。".to_string());
        }

        insights
    }

    /// Find related concepts
    pub fn find_related_concepts(&self, concept: &str, top_k: usize) -> Vec<RelatedConcept> {
        let nouns = self.text_processor.extract_nouns(concept, true);

        // Search Core layer for matching nodes
        let core_nodes = self.graph.get_nodes_by_layer(Layer::Core);

        let mut related = Vec::new();
        for node_id in core_nodes {
            if nouns.iter().any(|noun| node_id.contains(noun)) {
                related.push(RelatedConcept {
                    node_id: node_id.clone(),
                    domain: "unknown".to_string(),
                });
            }
        }

        related.truncate(top_k);
        related
    }

    /// Get statistics
    pub fn get_statistics(&self) -> ThinkEngineStats {
        self.stats.clone()
    }

    /// Get graph reference (internal use)
    #[allow(dead_code)]
    pub(crate) fn get_graph(&self) -> &SimpleGraph {
        &self.graph
    }

    /// Add node to graph
    pub fn add_node(&mut self, id: &str, layer: Layer) {
        self.graph.add_node(id, layer);
    }

    /// Add edge to graph
    pub fn add_edge(&mut self, from: &str, to: &str, weight: f64) {
        self.graph.add_edge(from, to, weight);
    }
}

impl Default for KDFThinkEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Think analysis result
#[derive(Clone, Debug)]
pub struct ThinkAnalysisResult {
    /// Insights
    pub insights: Vec<String>,
    /// Cluster summary
    pub cluster_summary: ClusterSummary,
    /// Layer health
    pub layer_health: LayerHealth,
    /// Analysis depth
    pub analysis_depth: String,
}

/// Cluster summary
#[derive(Clone, Debug)]
pub struct ClusterSummary {
    /// Edge clusters
    pub edge_clusters: usize,
    /// Rare clusters
    pub rare_clusters: usize,
    /// Core clusters
    pub core_clusters: usize,
    /// Total nodes
    pub total_nodes: usize,
}

/// Layer health
#[derive(Clone, Debug)]
pub struct LayerHealth {
    /// Edge health
    pub edge: f64,
    /// Rare health
    pub rare: f64,
    /// Core health
    pub core: f64,
}

/// Related concept
#[derive(Clone, Debug)]
pub struct RelatedConcept {
    /// Node ID
    pub node_id: String,
    /// Domain
    pub domain: String,
}
