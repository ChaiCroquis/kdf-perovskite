//! KDF Meaning Engine
//!
//! Handles document management and topic learning.

use super::super::framework::Layer;
use super::super::text_processor::{DomainClassifier, TextProcessor};
use super::graph::SimpleGraph;
use std::collections::{HashMap, HashSet};

/// KDF-based Meaning Engine
///
/// Handles document management and topic learning.
/// Replaces MeaningEngineStub.
pub struct KDFMeaningEngine {
    /// Internal graph
    graph: SimpleGraph,
    /// Text processor
    processor: TextProcessor,
    /// Domain classifier (reserved for future use)
    #[allow(dead_code)]
    domain_classifier: DomainClassifier,
    /// Stored documents
    documents: HashMap<String, String>,
    /// Screening enabled
    pub screening_enabled: bool,
    /// Top-K percent for screening
    pub top_k_percent: f64,
}

impl KDFMeaningEngine {
    /// Create a new meaning engine
    pub fn new(screening_enabled: bool, top_k_percent: f64) -> Self {
        Self {
            graph: SimpleGraph::new(),
            processor: TextProcessor::new(),
            domain_classifier: DomainClassifier::new(),
            documents: HashMap::new(),
            screening_enabled,
            top_k_percent,
        }
    }
}

impl Default for KDFMeaningEngine {
    fn default() -> Self {
        Self::new(true, 0.05)
    }
}

impl KDFMeaningEngine {
    /// Add a document
    ///
    /// # Arguments
    /// * `doc_id` - Document ID
    /// * `content` - Text content
    /// * `label` - Node label ("normal", "isolated_truth", "garbage")
    /// * `domain` - Domain (None for auto-detection)
    pub fn add_document(
        &mut self,
        doc_id: &str,
        content: &str,
        label: Option<&str>,
        _domain: Option<&str>,
    ) {
        // Store document
        self.documents
            .insert(doc_id.to_string(), content.to_string());

        // Extract nouns
        let nouns = self.processor.extract_nouns(content, true);

        // Compute links with existing documents
        let mut links: Vec<(String, f64)> = Vec::new();
        for (existing_id, existing_content) in &self.documents {
            if existing_id != doc_id {
                let existing_nouns: HashSet<String> = self
                    .processor
                    .extract_nouns(existing_content, true)
                    .into_iter()
                    .collect();
                let current_nouns: HashSet<String> = nouns.iter().cloned().collect();

                // Jaccard similarity
                let intersection = existing_nouns.intersection(&current_nouns).count();
                let union = existing_nouns.union(&current_nouns).count();
                let similarity = if union > 0 {
                    intersection as f64 / union as f64
                } else {
                    0.0
                };

                if similarity > 0.1 {
                    links.push((existing_id.clone(), similarity));
                }
            }
        }

        // Determine layer based on label and connectivity
        let layer = match label.unwrap_or("normal") {
            "isolated_truth" => Layer::Rare,
            "garbage" => Layer::Garbage,
            _ if links.len() > 3 => Layer::Core,
            _ => Layer::Edge,
        };

        // Add to graph
        self.graph.add_node(doc_id, layer);

        // Add edges
        for (target_id, weight) in links {
            self.graph.add_edge(doc_id, &target_id, weight);
        }
    }

    /// Relearn topics
    ///
    /// # Arguments
    /// * `steps` - Number of simulation steps
    pub fn relearn_topics(&mut self, steps: usize) -> TopicRelearnResult {
        // Run simulation (increment step count)
        for _ in 0..steps {
            self.graph.step_count += 1;
        }

        // Build project hierarchy
        let projects = self.build_project_hierarchy();

        TopicRelearnResult {
            topics_updated: projects.established_projects.len(),
            rare_insights: projects.rare_insights.len(),
            projects: projects.established_projects,
            rare_insights_list: projects.rare_insights,
            discovery_rate: 0.0,
        }
    }

    /// Build project hierarchy
    fn build_project_hierarchy(&self) -> ProjectHierarchy {
        // Get Core layer clusters
        let core_clusters = self.graph.extract_clusters(Layer::Core);

        // Build projects
        let mut projects = Vec::new();
        for (i, cluster) in core_clusters.iter().enumerate() {
            let label = self.generate_cluster_label(cluster);
            projects.push(Project {
                id: format!("project_{}", i),
                name: label,
                documents: cluster.clone(),
                size: cluster.len(),
            });
        }

        // Get Rare layer nodes
        let rare_nodes = self.graph.get_nodes_by_layer(Layer::Rare);

        ProjectHierarchy {
            established_projects: projects,
            rare_insights: rare_nodes,
        }
    }

    /// Generate cluster label
    fn generate_cluster_label(&self, cluster: &[String]) -> String {
        // Combine all document text
        let mut all_text = String::new();
        for node_id in cluster {
            if let Some(content) = self.documents.get(node_id) {
                all_text.push_str(content);
                all_text.push(' ');
            }
        }

        // Extract frequent nouns
        let nouns = self.processor.extract_nouns(&all_text, true);
        let mut freq: HashMap<String, usize> = HashMap::new();
        for noun in nouns {
            *freq.entry(noun).or_insert(0) += 1;
        }

        // Top 3 words
        let mut items: Vec<(String, usize)> = freq.into_iter().collect();
        items.sort_by(|a, b| b.1.cmp(&a.1));

        let top_words: Vec<String> = items.into_iter().take(3).map(|(w, _)| w).collect();

        if top_words.is_empty() {
            "Untitled_Cluster".to_string()
        } else {
            top_words.join("_")
        }
    }

    /// Get statistics
    pub fn get_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        stats.insert("total_documents".to_string(), self.documents.len() as f64);
        stats.insert("total_nodes".to_string(), self.graph.nodes.len() as f64);
        stats.insert("total_edges".to_string(), self.graph.edges.len() as f64);
        stats.insert(
            "core_nodes".to_string(),
            self.graph.get_nodes_by_layer(Layer::Core).len() as f64,
        );
        stats.insert(
            "edge_nodes".to_string(),
            self.graph.get_nodes_by_layer(Layer::Edge).len() as f64,
        );
        stats.insert(
            "rare_nodes".to_string(),
            self.graph.get_nodes_by_layer(Layer::Rare).len() as f64,
        );

        stats
    }

    /// Quick analysis
    pub fn quick_analysis(&self) -> QuickAnalysisResult {
        let clusters = self.graph.extract_clusters(Layer::Core);

        let mut insights = Vec::new();
        for cluster in &clusters {
            if !cluster.is_empty() {
                insights.push(ClusterInsight {
                    cluster_size: cluster.len(),
                    documents: cluster.clone(),
                });
            }
        }

        QuickAnalysisResult {
            insights,
            total_clusters: clusters.len(),
        }
    }
}

/// Project in hierarchy
#[derive(Clone, Debug)]
pub struct Project {
    /// Project ID
    pub id: String,
    /// Project name
    pub name: String,
    /// Documents in project
    pub documents: Vec<String>,
    /// Project size
    pub size: usize,
}

/// Project hierarchy
#[derive(Clone, Debug)]
struct ProjectHierarchy {
    established_projects: Vec<Project>,
    rare_insights: Vec<String>,
}

/// Topic relearn result
#[derive(Clone, Debug)]
pub struct TopicRelearnResult {
    /// Topics updated
    pub topics_updated: usize,
    /// Rare insights count
    pub rare_insights: usize,
    /// Projects
    pub projects: Vec<Project>,
    /// Rare insights list
    pub rare_insights_list: Vec<String>,
    /// Discovery rate
    pub discovery_rate: f64,
}

/// Quick analysis result
#[derive(Clone, Debug)]
pub struct QuickAnalysisResult {
    /// Cluster insights
    pub insights: Vec<ClusterInsight>,
    /// Total clusters
    pub total_clusters: usize,
}

/// Cluster insight
#[derive(Clone, Debug)]
pub struct ClusterInsight {
    /// Cluster size
    pub cluster_size: usize,
    /// Documents
    pub documents: Vec<String>,
}
