//! High-Level KDF Engines Module
//!
//! Provides high-level abstractions for KDF operations:
//! - MeaningEngine: Document management and topic learning
//! - ThinkEngine: Analysis and insight generation
//! - SleepEngine: Background optimization and crystallization
//!
//! # Reference
//!
//! Python implementations:
//! - python/kdf/meaning_engine.py
//! - python/kdf/think_engine.py
//! - python/kdf/sleep_engine.py

use std::collections::HashMap;

// Module declarations
mod graph;
mod meaning;
mod think;
mod sleep;

#[cfg(test)]
mod tests;

// Re-exports
pub use meaning::{
    KDFMeaningEngine,
    Project,
    TopicRelearnResult,
    QuickAnalysisResult,
    ClusterInsight,
};

pub use think::{
    KDFThinkEngine,
    ThinkEngineStats,
    ThinkAnalysisResult,
    ClusterSummary,
    LayerHealth,
    RelatedConcept,
};

pub use sleep::{
    KDFSleepEngine,
    HeavyTask,
    TaskResult,
    NREMOptimizationResult,
    SleepEngineStats,
};

// ============================================================================
// Common Types
// ============================================================================

/// Task type for background processing
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskType {
    /// KeyGraph analysis
    KeygraphAnalysis,
    /// KDF relearning
    KdfRelearn,
    /// Crystallization
    Crystallization,
    /// Consolidation
    Consolidation,
    /// Unknown task
    Unknown,
}

/// Node type for knowledge nodes
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    /// Raw unprocessed node
    Raw,
    /// Processed node
    Processed,
    /// Crystallized stable node
    Crystal,
    /// Initial knowledge node
    Genesis,
}

/// Knowledge node
#[derive(Clone, Debug)]
pub struct KnowledgeNode {
    /// Content
    pub content: String,
    /// Node type
    pub node_type: NodeType,
    /// Weight
    pub weight: f64,
    /// Decay resistance
    pub decay_resistance: f64,
    /// Is genesis node
    pub is_genesis: bool,
    /// Task completed
    pub task_completed: bool,
    /// Embedding vector
    pub embedding: Option<Vec<f64>>,
    /// Creation timestamp
    pub created_at: u64,
    /// Metadata
    pub metadata: HashMap<String, String>,
}

impl KnowledgeNode {
    /// Create a new knowledge node
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            node_type: NodeType::Raw,
            weight: 1.0,
            decay_resistance: 0.5,
            is_genesis: false,
            task_completed: false,
            embedding: None,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            metadata: HashMap::new(),
        }
    }
}

/// Analysis result
#[derive(Clone, Debug)]
pub struct AnalysisResult {
    /// Insights generated
    pub insights: Vec<String>,
    /// Clusters found
    pub clusters: Vec<Vec<String>>,
    /// Analogies discovered
    pub analogies: Vec<HashMap<String, String>>,
    /// Statistics
    pub statistics: HashMap<String, f64>,
    /// Timestamp
    pub analyzed_at: u64,
}
