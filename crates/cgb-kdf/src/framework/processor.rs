//! KDF Processor - Main entry point for KDF-based processing

use super::{ClassificationStats, DecayManager, Layer, NodeClassifier};
use crate::fingerprint::Fingerprint;

/// KDF Processor - Main entry point for KDF-based processing
#[derive(Default)]
pub struct KdfProcessor {
    /// Node classifier
    classifier: NodeClassifier,
    /// Decay manager
    decay_manager: DecayManager,
}


impl KdfProcessor {
    /// Create a new KDF processor
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize processor with graph data
    ///
    /// This performs node classification and sets up decay management.
    pub fn initialize(&mut self, node_count: usize, edges: &[(u32, u32, f64)]) {
        let classification = self.classifier.classify(node_count, edges);
        self.decay_manager.initialize(classification);
    }

    /// Check if a node should be skipped
    pub fn should_skip(&self, node: u32) -> bool {
        self.decay_manager.should_skip(node)
    }

    /// Check if a node is protected
    pub fn is_protected(&self, node: u32) -> bool {
        self.decay_manager.is_protected(node)
    }

    /// Get nodes to process in optimal order
    pub fn processing_order(&self) -> Vec<u32> {
        if let Some(ref class) = self.decay_manager.classification {
            self.classifier.processing_order(class)
        } else {
            Vec::new()
        }
    }

    /// Get classification statistics
    pub fn stats(&self) -> Option<&ClassificationStats> {
        self.decay_manager.stats()
    }

    /// Get the layer of a node
    pub fn get_layer(&self, node: u32) -> Option<Layer> {
        self.decay_manager.classification.as_ref()?
            .layers.get(&node).copied()
    }

    /// Record node access
    pub fn record_access(&mut self, node: u32) {
        self.decay_manager.record_access(node);
    }

    /// Get RARE fingerprint for preservation
    pub fn get_rare_fingerprint(&self, node: u32) -> Option<&Fingerprint> {
        self.decay_manager.get_rare_fingerprint(node)
    }
}
