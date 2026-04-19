//! KDF (Knowledge Decay Framework) High-Level Components
//!
//! Rust implementation of KDF v3.0:
//!
//! # Core Concept
//!
//! KDF classifies nodes into layers and applies decay-based processing:
//! - **CORE**: High-connectivity nodes - full processing
//! - **EDGE**: Medium-connectivity nodes - standard processing
//! - **RARE**: Isolated but important - protect with fingerprint
//! - **GARBAGE**: Noise/artifacts - skip processing
//!
//! # Components
//!
//! - [`KdfProcessor`]: Unified entry point for KDF-based processing
//! - [`NodeClassifier`]: Automatic node classification into layers
//! - [`DecayManager`]: Decay tracking and skip/protect decisions
//! - [`StructuralFingerprintEngine`]: Graph Laplacian fingerprinting
//! - [`PreScreeningOptimizer`]: Top-K% candidate filtering
//! - [`SleepModeOptimizer`]: NREM/REM optimization phases
//!
//! # Usage
//!
//! ```ignore
//! use cgb::kdf::{KdfProcessor, Layer};
//!
//! let mut kdf = KdfProcessor::new();
//! kdf.initialize(node_count, &edges);
//!
//! // Get nodes to process (excludes GARBAGE)
//! for node in kdf.processing_order() {
//!     if kdf.is_protected(node) {
//!         // RARE node - preserve, don't modify
//!         continue;
//!     }
//!     // Process node...
//! }
//!
//! // Check skip rate
//! if let Some(stats) = kdf.stats() {
//!     println!("Skipped {:.1}% of nodes", stats.skip_rate() * 100.0);
//! }
//! ```

pub mod framework;
pub mod fingerprint;
pub mod prescreening;
pub mod analogy;
pub mod sleep_mode;
pub mod interning;
pub mod exact_solver;

// New modules (migrated from Python)
pub mod vne;
pub mod causal;
pub mod spectral_te;
pub mod text_processor;
pub mod engines;

// Framework (unified entry point)
pub use framework::{
    Layer, NodeClassification, ClassificationStats,
    NodeClassifier, FastNodeClassifier, DecayManager, MasterSpecParams, KdfProcessor,
    // Rev.12 (Analogy Discovery + multi-stage review)
    ReviewPhase, RareNodeState, Rev12Stats, Rev12Error, KdfProcessorRev12,
    T_WAIT_MIN, T_WAIT_MAX, T_WAIT_DEFAULT,
    DISCOVERY_THRESHOLD_DEFAULT, DISCOVERY_THRESHOLD_UPPER_DEFAULT,
    // Phase 1 additions (Claim 20-32)
    MetaController,
    HierarchicalRegionManager, RegionConfig, RegionKind,
    TransitionController, TransitionScore, ActivationScore, SemanticImportance,
};

// Existing components
pub use fingerprint::{StructuralFingerprintEngine, Fingerprint, PrecomputedFingerprint};
pub use prescreening::{PreScreeningOptimizer, ScreeningStats};
pub use analogy::{AnalogyDiscoveryEngine, NodeFeatures, StructuralMapping, RelationType};
pub use sleep_mode::{SleepModeOptimizer, NREMResult, IncrementalEntropyCache, NodeMoveContext};
pub use interning::NodeIdMap;
pub use exact_solver::{ExactSolver, HybridSolver, ExactResult, HybridResult, SolverStrategy};

// VNE (Von Neumann Entropy) Integration
pub use vne::{
    VNEResult, AnomalyResult, ChangeDetection,
    VNEMonitor, VNETriggeredSleepMode,
    von_neumann_entropy, von_neumann_entropy_detailed, detect_change,
};

// Causal Discovery (Transfer Entropy)
pub use causal::{
    TeStrategy, TeResult, CausalLink,
    GaussianEstimator, SymbolicEstimator, KsgEstimator,
    CausalEngine, CausalKdfV3, SleepCycleResult,
    CausalPartitionBuilder, CausalCluster,
    CausalEnhancedNREMOptimizer, CausalNREMResult,
};

// Spectral TE Prioritization
pub use spectral_te::{
    NodeSpectralInfo, PairPriority, SpectralTEPrioritizer,
    prioritize_te_computation,
};

// Text Processing
pub use text_processor::{
    Token, TextProcessor, DomainClassifier,
    simple_tokenize, extract_keywords,
};

// High-Level Engines
pub use engines::{
    TaskType, NodeType, KnowledgeNode, AnalysisResult,
    KDFMeaningEngine, TopicRelearnResult, Project,
    KDFThinkEngine, ThinkAnalysisResult, ClusterSummary, LayerHealth,
    KDFSleepEngine, HeavyTask, TaskResult, NREMOptimizationResult,
};
