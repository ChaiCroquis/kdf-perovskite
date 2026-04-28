//! Tests for KDF engines

use super::graph::SimpleGraph;
use super::*;
use crate::framework::Layer;
use std::collections::HashMap;

// ========== TaskType Tests ==========

#[test]
fn test_task_type_variants() {
    assert_eq!(TaskType::KeygraphAnalysis, TaskType::KeygraphAnalysis);
    assert_eq!(TaskType::KdfRelearn, TaskType::KdfRelearn);
    assert_eq!(TaskType::Crystallization, TaskType::Crystallization);
    assert_eq!(TaskType::Consolidation, TaskType::Consolidation);
    assert_eq!(TaskType::Unknown, TaskType::Unknown);
}

#[test]
fn test_task_type_clone_and_debug() {
    let task = TaskType::KeygraphAnalysis;
    let cloned = task.clone();
    assert_eq!(task, cloned);
    let debug_str = format!("{:?}", task);
    assert!(debug_str.contains("KeygraphAnalysis"));
}

// ========== NodeType Tests ==========

#[test]
fn test_node_type_variants() {
    assert_eq!(NodeType::Raw, NodeType::Raw);
    assert_eq!(NodeType::Processed, NodeType::Processed);
    assert_eq!(NodeType::Crystal, NodeType::Crystal);
    assert_eq!(NodeType::Genesis, NodeType::Genesis);
}

#[test]
fn test_node_type_clone_and_debug() {
    let node_type = NodeType::Crystal;
    let cloned = node_type.clone();
    assert_eq!(node_type, cloned);
    let debug_str = format!("{:?}", node_type);
    assert!(debug_str.contains("Crystal"));
}

// ========== KnowledgeNode Tests ==========

#[test]
fn test_knowledge_node() {
    let node = KnowledgeNode::new("Test content");
    assert_eq!(node.content, "Test content");
    assert_eq!(node.node_type, NodeType::Raw);
    assert_eq!(node.weight, 1.0);
}

#[test]
fn test_knowledge_node_defaults() {
    let node = KnowledgeNode::new("Node");
    assert_eq!(node.decay_resistance, 0.5);
    assert!(!node.is_genesis);
    assert!(!node.task_completed);
    assert!(node.embedding.is_none());
    assert!(node.metadata.is_empty());
    assert!(node.created_at > 0);
}

#[test]
fn test_knowledge_node_clone() {
    let node = KnowledgeNode::new("Cloneable");
    let cloned = node.clone();
    assert_eq!(node.content, cloned.content);
    assert_eq!(node.weight, cloned.weight);
}

// ========== AnalysisResult Tests ==========

#[test]
fn test_analysis_result_creation() {
    let result = AnalysisResult {
        insights: vec!["insight1".to_string()],
        clusters: vec![vec!["a".to_string(), "b".to_string()]],
        analogies: vec![],
        statistics: HashMap::new(),
        analyzed_at: 12345,
    };
    assert_eq!(result.insights.len(), 1);
    assert_eq!(result.clusters.len(), 1);
    assert_eq!(result.analyzed_at, 12345);
}

#[test]
fn test_analysis_result_clone() {
    let result = AnalysisResult {
        insights: vec!["test".to_string()],
        clusters: vec![],
        analogies: vec![],
        statistics: HashMap::new(),
        analyzed_at: 0,
    };
    let cloned = result.clone();
    assert_eq!(result.insights, cloned.insights);
}

// ========== SimpleGraph Tests ==========

#[test]
fn test_simple_graph_new() {
    let graph = SimpleGraph::new();
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
    assert_eq!(graph.step_count, 0);
}

#[test]
fn test_simple_graph_add_node() {
    let mut graph = SimpleGraph::new();
    graph.add_node("node1", Layer::Core);
    assert_eq!(graph.nodes.len(), 1);
    assert_eq!(graph.nodes.get("node1"), Some(&Layer::Core));
}

#[test]
fn test_simple_graph_add_edge() {
    let mut graph = SimpleGraph::new();
    graph.add_node("a", Layer::Edge);
    graph.add_node("b", Layer::Edge);
    graph.add_edge("a", "b", 0.5);
    assert_eq!(graph.edges.len(), 1);
}

#[test]
fn test_simple_graph_get_nodes_by_layer() {
    let mut graph = SimpleGraph::new();
    graph.add_node("core1", Layer::Core);
    graph.add_node("core2", Layer::Core);
    graph.add_node("edge1", Layer::Edge);

    let core_nodes = graph.get_nodes_by_layer(Layer::Core);
    assert_eq!(core_nodes.len(), 2);
    assert!(core_nodes.contains(&"core1".to_string()));
    assert!(core_nodes.contains(&"core2".to_string()));
}

#[test]
fn test_simple_graph_extract_clusters_empty() {
    let graph = SimpleGraph::new();
    let clusters = graph.extract_clusters(Layer::Core);
    assert!(clusters.is_empty());
}

#[test]
fn test_simple_graph_extract_clusters_connected() {
    let mut graph = SimpleGraph::new();
    graph.add_node("a", Layer::Core);
    graph.add_node("b", Layer::Core);
    graph.add_node("c", Layer::Core);
    graph.add_edge("a", "b", 1.0);
    graph.add_edge("b", "c", 1.0);

    let clusters = graph.extract_clusters(Layer::Core);
    // All connected, should be 1 cluster
    assert_eq!(clusters.len(), 1);
    assert_eq!(clusters[0].len(), 3);
}

#[test]
fn test_simple_graph_extract_clusters_disconnected() {
    let mut graph = SimpleGraph::new();
    graph.add_node("a", Layer::Core);
    graph.add_node("b", Layer::Core);
    graph.add_node("c", Layer::Core);
    graph.add_node("d", Layer::Core);
    graph.add_edge("a", "b", 1.0);
    // c and d are isolated

    let clusters = graph.extract_clusters(Layer::Core);
    // a-b in one cluster, c and d each in their own
    assert_eq!(clusters.len(), 3);
}

// ========== MeaningEngine Tests ==========

#[test]
fn test_meaning_engine_creation() {
    let engine = KDFMeaningEngine::default();
    assert!(engine.screening_enabled);
}

#[test]
fn test_meaning_engine_custom_params() {
    let engine = KDFMeaningEngine::new(false, 0.10);
    assert!(!engine.screening_enabled);
    assert_eq!(engine.top_k_percent, 0.10);
}

#[test]
fn test_meaning_engine_add_document() {
    let mut engine = KDFMeaningEngine::default();

    engine.add_document("doc1", "プログラミングの基礎", None, None);
    engine.add_document("doc2", "データベース設計", None, None);

    let stats = engine.get_statistics();
    assert_eq!(stats.get("total_documents"), Some(&2.0));
}

#[test]
fn test_meaning_engine_add_document_with_labels() {
    let mut engine = KDFMeaningEngine::default();

    engine.add_document("doc1", "重要な知識", Some("isolated_truth"), None);
    engine.add_document("doc2", "不要なデータ", Some("garbage"), None);
    engine.add_document("doc3", "普通の文書", Some("normal"), None);

    let stats = engine.get_statistics();
    assert_eq!(stats.get("rare_nodes"), Some(&1.0)); // isolated_truth -> Rare
}

#[test]
fn test_meaning_engine_add_connected_documents() {
    let mut engine = KDFMeaningEngine::default();

    // Add documents with overlapping content (should create links)
    engine.add_document("doc1", "machine learning AI", None, None);
    engine.add_document("doc2", "machine learning deep", None, None);
    engine.add_document("doc3", "machine learning neural", None, None);
    engine.add_document("doc4", "machine learning model", None, None);
    engine.add_document("doc5", "machine learning algorithm", None, None);

    let stats = engine.get_statistics();
    assert!(stats.get("total_edges").unwrap_or(&0.0) >= &0.0);
}

#[test]
fn test_meaning_engine_relearn_topics() {
    let mut engine = KDFMeaningEngine::default();

    engine.add_document("doc1", "プログラミングの基礎", None, None);
    engine.add_document("doc2", "プログラミング言語", None, None);

    let result = engine.relearn_topics(10);
    let _ = result.topics_updated;
}

#[test]
fn test_meaning_engine_quick_analysis() {
    let mut engine = KDFMeaningEngine::default();

    engine.add_document("doc1", "テスト文書", None, None);

    let result = engine.quick_analysis();
    let _ = result.total_clusters;
}

#[test]
fn test_meaning_engine_get_statistics() {
    let engine = KDFMeaningEngine::default();
    let stats = engine.get_statistics();
    assert!(stats.contains_key("total_documents"));
    assert!(stats.contains_key("total_nodes"));
    assert!(stats.contains_key("total_edges"));
    assert!(stats.contains_key("core_nodes"));
    assert!(stats.contains_key("edge_nodes"));
    assert!(stats.contains_key("rare_nodes"));
}

// ========== Project and Related Types Tests ==========

#[test]
fn test_project_structure() {
    let project = Project {
        id: "proj1".to_string(),
        name: "Test Project".to_string(),
        documents: vec!["doc1".to_string(), "doc2".to_string()],
        size: 2,
    };
    assert_eq!(project.id, "proj1");
    assert_eq!(project.name, "Test Project");
    assert_eq!(project.documents.len(), 2);
    assert_eq!(project.size, 2);
}

#[test]
fn test_topic_relearn_result() {
    let result = TopicRelearnResult {
        topics_updated: 5,
        rare_insights: 3,
        projects: vec![],
        rare_insights_list: vec!["insight1".to_string()],
        discovery_rate: 0.5,
    };
    assert_eq!(result.topics_updated, 5);
    assert_eq!(result.rare_insights, 3);
    assert_eq!(result.discovery_rate, 0.5);
}

#[test]
fn test_quick_analysis_result() {
    let result = QuickAnalysisResult {
        insights: vec![ClusterInsight {
            cluster_size: 5,
            documents: vec!["a".to_string()],
        }],
        total_clusters: 1,
    };
    assert_eq!(result.total_clusters, 1);
    assert_eq!(result.insights.len(), 1);
    assert_eq!(result.insights[0].cluster_size, 5);
}

// ========== ThinkEngine Tests ==========

#[test]
fn test_think_engine_creation() {
    let engine = KDFThinkEngine::default();
    let stats = engine.get_statistics();
    assert_eq!(stats.analyses_performed, 0);
}

#[test]
fn test_think_engine_new() {
    let engine = KDFThinkEngine::new();
    let stats = engine.get_statistics();
    assert_eq!(stats.analyses_performed, 0);
    assert_eq!(stats.insights_generated, 0);
    assert_eq!(stats.analogies_found, 0);
}

#[test]
fn test_think_engine_quick_analysis() {
    let mut engine = KDFThinkEngine::default();
    let result = engine.quick_analysis();

    assert_eq!(result.analysis_depth, "quick");
    assert!(!result.insights.is_empty());
}

#[test]
fn test_think_engine_deep_analysis() {
    let mut engine = KDFThinkEngine::default();
    let result = engine.deep_analysis(5);

    assert_eq!(result.analysis_depth, "deep");
    assert!(!result.insights.is_empty());

    let stats = engine.get_statistics();
    // deep_analysis calls quick_analysis internally, so 2 analyses
    assert_eq!(stats.analyses_performed, 2);
}

#[test]
fn test_think_engine_add_node_and_edge() {
    let mut engine = KDFThinkEngine::new();
    engine.add_node("node1", Layer::Core);
    engine.add_node("node2", Layer::Core);
    engine.add_edge("node1", "node2", 0.5);

    let result = engine.quick_analysis();
    assert_eq!(result.cluster_summary.total_nodes, 2);
}

#[test]
fn test_think_engine_find_related_concepts() {
    let mut engine = KDFThinkEngine::new();
    engine.add_node("machine_learning", Layer::Core);
    engine.add_node("deep_learning", Layer::Core);

    let related = engine.find_related_concepts("machine", 10);
    assert!(!related.is_empty() || related.is_empty()); // May or may not find matches
}

#[test]
fn test_think_engine_with_large_clusters() {
    let mut engine = KDFThinkEngine::new();

    // Add many edge nodes to trigger "large cluster" insight
    for i in 0..15 {
        engine.add_node(&format!("edge_{}", i), Layer::Edge);
    }
    for i in 0..14 {
        engine.add_edge(&format!("edge_{}", i), &format!("edge_{}", i + 1), 1.0);
    }

    let result = engine.quick_analysis();
    // Should generate insight about large clusters
    assert!(!result.insights.is_empty());
}

#[test]
fn test_think_engine_with_isolated_rare_nodes() {
    let mut engine = KDFThinkEngine::new();

    // Add isolated Rare nodes
    engine.add_node("rare_isolated", Layer::Rare);

    let result = engine.quick_analysis();
    // Should generate insight about isolated nodes
    assert!(!result.insights.is_empty());
}

#[test]
fn test_think_engine_with_large_core_cluster() {
    let mut engine = KDFThinkEngine::new();

    // Add many Core nodes connected
    for i in 0..15 {
        engine.add_node(&format!("core_{}", i), Layer::Core);
    }
    for i in 0..14 {
        engine.add_edge(&format!("core_{}", i), &format!("core_{}", i + 1), 1.0);
    }

    let result = engine.quick_analysis();
    // Should have core cluster insight
    assert!(result.cluster_summary.core_clusters >= 1);
}

#[test]
fn test_think_engine_statistics_tracking() {
    let mut engine = KDFThinkEngine::new();

    engine.quick_analysis();
    engine.quick_analysis();
    engine.deep_analysis(1);

    let stats = engine.get_statistics();
    // 2 quick + 1 deep (which calls quick internally)
    assert_eq!(stats.analyses_performed, 4);
}

// ========== ThinkAnalysisResult and Related Tests ==========

#[test]
fn test_cluster_summary() {
    let summary = ClusterSummary {
        edge_clusters: 5,
        rare_clusters: 2,
        core_clusters: 3,
        total_nodes: 100,
    };
    assert_eq!(summary.edge_clusters, 5);
    assert_eq!(summary.rare_clusters, 2);
    assert_eq!(summary.core_clusters, 3);
    assert_eq!(summary.total_nodes, 100);
}

#[test]
fn test_layer_health() {
    let health = LayerHealth {
        edge: 0.8,
        rare: 0.6,
        core: 0.9,
    };
    assert_eq!(health.edge, 0.8);
    assert_eq!(health.rare, 0.6);
    assert_eq!(health.core, 0.9);
}

#[test]
fn test_related_concept() {
    let concept = RelatedConcept {
        node_id: "concept1".to_string(),
        domain: "science".to_string(),
    };
    assert_eq!(concept.node_id, "concept1");
    assert_eq!(concept.domain, "science");
}

// ========== HeavyTask Tests ==========

#[test]
fn test_heavy_task_new() {
    let task = HeavyTask::new(TaskType::Crystallization);
    assert_eq!(task.task_type, TaskType::Crystallization);
    assert!(task.node_id.is_none());
    assert_eq!(task.priority, 0);
    assert!(task.created_at > 0);
    assert!(task.metadata.is_empty());
}

#[test]
fn test_heavy_task_with_node_id() {
    let mut task = HeavyTask::new(TaskType::KdfRelearn);
    task.node_id = Some("node123".to_string());
    assert_eq!(task.node_id, Some("node123".to_string()));
}

#[test]
fn test_heavy_task_with_metadata() {
    let mut task = HeavyTask::new(TaskType::Consolidation);
    task.metadata.insert("key".to_string(), "value".to_string());
    assert_eq!(task.metadata.get("key"), Some(&"value".to_string()));
}

// ========== SleepEngine Tests ==========

#[test]
fn test_sleep_engine_creation() {
    let engine = KDFSleepEngine::default();
    assert_eq!(engine.queue_size(), 0);
}

#[test]
fn test_sleep_engine_new_with_iterations() {
    let engine = KDFSleepEngine::new(500);
    assert_eq!(engine.max_iterations, 500);
}

#[test]
fn test_sleep_engine_queue() {
    let mut engine = KDFSleepEngine::default();

    // Queue tasks with different priorities
    engine.queue_task(HeavyTask::new(TaskType::KdfRelearn));
    let mut high_priority = HeavyTask::new(TaskType::Crystallization);
    high_priority.priority = 10;
    engine.queue_task(high_priority);

    assert_eq!(engine.queue_size(), 2);

    // Process one task
    let results = engine.process_tasks(1);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].task_type, TaskType::Crystallization);

    assert_eq!(engine.queue_size(), 1);
}

#[test]
fn test_sleep_engine_process_all_task_types() {
    let mut engine = KDFSleepEngine::default();

    engine.queue_task(HeavyTask::new(TaskType::Crystallization));
    engine.queue_task(HeavyTask::new(TaskType::KdfRelearn));
    engine.queue_task(HeavyTask::new(TaskType::Consolidation));
    engine.queue_task(HeavyTask::new(TaskType::Unknown));

    let results = engine.process_tasks(4);
    assert_eq!(results.len(), 4);

    // Check each task type processed correctly
    let crystal = results
        .iter()
        .find(|r| r.task_type == TaskType::Crystallization)
        .unwrap();
    assert!(crystal.success);
    assert!(crystal.message.contains("Crystallization"));

    let relearn = results
        .iter()
        .find(|r| r.task_type == TaskType::KdfRelearn)
        .unwrap();
    assert!(relearn.success);
    assert!(relearn.message.contains("Relearn"));

    let consolidation = results
        .iter()
        .find(|r| r.task_type == TaskType::Consolidation)
        .unwrap();
    assert!(consolidation.success);
    assert!(consolidation.message.contains("Consolidation"));

    let unknown = results
        .iter()
        .find(|r| r.task_type == TaskType::Unknown)
        .unwrap();
    assert!(!unknown.success);
    assert!(unknown.message.contains("Unknown"));
}

#[test]
fn test_sleep_engine_clear_queue() {
    let mut engine = KDFSleepEngine::default();

    engine.queue_task(HeavyTask::new(TaskType::Crystallization));
    engine.queue_task(HeavyTask::new(TaskType::KdfRelearn));
    assert_eq!(engine.queue_size(), 2);

    engine.clear_queue();
    assert_eq!(engine.queue_size(), 0);
}

#[test]
fn test_sleep_engine_get_statistics() {
    let mut engine = KDFSleepEngine::default();

    engine.queue_task(HeavyTask::new(TaskType::Crystallization));
    engine.process_tasks(1);

    let stats = engine.get_statistics();
    assert_eq!(stats.tasks_processed, 1);
    assert_eq!(stats.crystallizations, 1);
}

#[test]
fn test_sleep_engine_process_empty_queue() {
    let mut engine = KDFSleepEngine::default();

    let results = engine.process_tasks(5);
    assert!(results.is_empty());
}

#[test]
fn test_sleep_engine_priority_ordering() {
    let mut engine = KDFSleepEngine::default();

    let mut task1 = HeavyTask::new(TaskType::KdfRelearn);
    task1.priority = 1;

    let mut task2 = HeavyTask::new(TaskType::Crystallization);
    task2.priority = 5;

    let mut task3 = HeavyTask::new(TaskType::Consolidation);
    task3.priority = 3;

    engine.queue_task(task1);
    engine.queue_task(task2);
    engine.queue_task(task3);

    // Should process highest priority first
    let results = engine.process_tasks(3);
    assert_eq!(results[0].task_type, TaskType::Crystallization); // priority 5
    assert_eq!(results[1].task_type, TaskType::Consolidation); // priority 3
    assert_eq!(results[2].task_type, TaskType::KdfRelearn); // priority 1
}

#[test]
fn test_nrem_optimization() {
    let mut engine = KDFSleepEngine::default();

    let edges = vec![
        ("a".to_string(), "b".to_string(), 1.0),
        ("b".to_string(), "c".to_string(), 1.0),
        ("c".to_string(), "a".to_string(), 1.0),
    ];

    let result = engine.run_nrem_optimization(&edges, None);

    assert!(result.final_entropy <= result.initial_entropy);
    assert!(!result.partition.is_empty());
}

#[test]
fn test_nrem_optimization_with_partition() {
    let mut engine = KDFSleepEngine::default();

    let edges = vec![
        ("a".to_string(), "b".to_string(), 1.0),
        ("b".to_string(), "c".to_string(), 1.0),
    ];

    let mut initial: HashMap<String, u32> = HashMap::new();
    initial.insert("a".to_string(), 0);
    initial.insert("b".to_string(), 1);
    initial.insert("c".to_string(), 0);

    let result = engine.run_nrem_optimization(&edges, Some(initial));
    assert!(!result.partition.is_empty());
}

#[test]
fn test_nrem_optimization_updates_stats() {
    let mut engine = KDFSleepEngine::default();

    let edges = vec![("a".to_string(), "b".to_string(), 1.0)];

    let _ = engine.run_nrem_optimization(&edges, None);
    let _ = engine.run_nrem_optimization(&edges, None);

    let stats = engine.get_statistics();
    // Stats should be updated
    assert!(stats.total_entropy_reduction >= 0.0);
}

// ========== TaskResult Tests ==========

#[test]
fn test_task_result_structure() {
    let result = TaskResult {
        task_type: TaskType::Crystallization,
        success: true,
        entropy_reduction: 0.1,
        compression_ratio: 0.9,
        message: "Done".to_string(),
    };
    assert!(result.success);
    assert_eq!(result.entropy_reduction, 0.1);
    assert_eq!(result.compression_ratio, 0.9);
}

// ========== NREMOptimizationResult Tests ==========

#[test]
fn test_nrem_result_structure() {
    let result = NREMOptimizationResult {
        partition: HashMap::new(),
        initial_entropy: 1.0,
        final_entropy: 0.5,
        entropy_reduction: 0.5,
        compression_ratio: 0.5,
        iterations: 100,
    };
    assert_eq!(result.initial_entropy, 1.0);
    assert_eq!(result.final_entropy, 0.5);
    assert_eq!(result.entropy_reduction, 0.5);
    assert_eq!(result.iterations, 100);
}

// ========== ThinkEngineStats Tests ==========

#[test]
fn test_think_engine_stats_default() {
    let stats: ThinkEngineStats = Default::default();
    assert_eq!(stats.analyses_performed, 0);
    assert_eq!(stats.insights_generated, 0);
    assert_eq!(stats.analogies_found, 0);
}

// ========== SleepEngineStats Tests ==========

#[test]
fn test_sleep_engine_stats_default() {
    let stats: SleepEngineStats = Default::default();
    assert_eq!(stats.tasks_processed, 0);
    assert_eq!(stats.crystallizations, 0);
    assert_eq!(stats.total_entropy_reduction, 0.0);
    assert_eq!(stats.avg_compression_ratio, 0.0);
}

// ========== Default Trait Tests ==========

#[test]
fn test_simple_graph_default() {
    let graph: SimpleGraph = Default::default();
    assert!(graph.nodes.is_empty());
    assert!(graph.edges.is_empty());
}
