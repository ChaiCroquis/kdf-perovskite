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

pub mod analogy;
pub mod exact_solver;
pub mod fingerprint;
pub mod framework;
pub mod interning;
pub mod prescreening;
pub mod sleep_mode;

// New modules (migrated from Python)
pub mod causal;
pub mod engines;
pub mod spectral_te;
pub mod text_processor;
pub mod vne;

// Framework (unified entry point)
pub use framework::{
    ActivationScore,
    ClassificationStats,
    DecayManager,
    FastNodeClassifier,
    HierarchicalRegionManager,
    KdfProcessor,
    KdfProcessorRev12,
    Layer,
    MasterSpecParams,
    // Phase 1 additions (Claim 20-32)
    MetaController,
    NodeClassification,
    NodeClassifier,
    RareNodeState,
    RegionConfig,
    RegionKind,
    Rev12Error,
    Rev12Stats,
    // Rev.12 (Analogy Discovery + multi-stage review)
    ReviewPhase,
    SemanticImportance,
    TransitionController,
    TransitionScore,
    DISCOVERY_THRESHOLD_DEFAULT,
    DISCOVERY_THRESHOLD_UPPER_DEFAULT,
    T_WAIT_DEFAULT,
    T_WAIT_MAX,
    T_WAIT_MIN,
};

// Existing components
pub use analogy::{AnalogyDiscoveryEngine, NodeFeatures, RelationType, StructuralMapping};
pub use exact_solver::{ExactResult, ExactSolver, HybridResult, HybridSolver, SolverStrategy};
pub use fingerprint::{Fingerprint, PrecomputedFingerprint, StructuralFingerprintEngine};
pub use interning::NodeIdMap;
pub use prescreening::{PreScreeningOptimizer, ScreeningStats};
pub use sleep_mode::{IncrementalEntropyCache, NREMResult, NodeMoveContext, SleepModeOptimizer};

// VNE (Von Neumann Entropy) Integration
pub use vne::{
    detect_change, von_neumann_entropy, von_neumann_entropy_detailed, AnomalyResult,
    ChangeDetection, VNEMonitor, VNEResult, VNETriggeredSleepMode,
};

// Causal Discovery (Transfer Entropy)
pub use causal::{
    CausalCluster, CausalEngine, CausalEnhancedNREMOptimizer, CausalKdfV3, CausalLink,
    CausalNREMResult, CausalPartitionBuilder, GaussianEstimator, KsgEstimator, SleepCycleResult,
    SymbolicEstimator, TeResult, TeStrategy,
};

// Spectral TE Prioritization
pub use spectral_te::{
    prioritize_te_computation, NodeSpectralInfo, PairPriority, SpectralTEPrioritizer,
};

// Text Processing
pub use text_processor::{
    extract_keywords, simple_tokenize, DomainClassifier, TextProcessor, Token,
};

// High-Level Engines
pub use engines::{
    AnalysisResult, ClusterSummary, HeavyTask, KDFMeaningEngine, KDFSleepEngine, KDFThinkEngine,
    KnowledgeNode, LayerHealth, NREMOptimizationResult, NodeType, Project, TaskResult, TaskType,
    ThinkAnalysisResult, TopicRelearnResult,
};
