//! Transfer Entropy Types

/// Transfer Entropy computation strategy
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TeStrategy {
    /// Edge Layer: Gaussian O(N)
    Screening,
    /// Rare Layer: Symbolic O(N log N)
    DeepProbe,
    /// Sleep Mode: KSG O(S*N²)
    Validation,
}

/// Result of transfer entropy computation
#[derive(Clone, Debug)]
pub struct TeResult {
    /// Transfer entropy value
    pub te: f64,
    /// Direction: source to target
    pub source_to_target: bool,
    /// P-value from significance testing
    pub p_value: Option<f64>,
    /// Whether TE is statistically significant
    pub is_significant: bool,
    /// Confidence level
    pub confidence: f64,
}

impl TeResult {
    /// Create a new TE result
    pub fn new(te: f64, source_to_target: bool) -> Self {
        Self {
            te,
            source_to_target,
            p_value: None,
            is_significant: te > 0.0,
            confidence: te.abs().min(1.0),
        }
    }

    /// Create a TE result with significance testing
    pub fn with_significance(
        te: f64,
        source_to_target: bool,
        p_value: f64,
        alpha: f64,
    ) -> Self {
        let is_sig = p_value < alpha && te > 0.0;
        Self {
            te,
            source_to_target,
            p_value: Some(p_value),
            is_significant: is_sig,
            confidence: if is_sig { (1.0 - p_value).min(1.0) } else { 0.0 },
        }
    }

    /// Check if this represents a causal relationship
    pub fn is_causal(&self, threshold: f64) -> bool {
        self.te > threshold && self.is_significant
    }
}

/// Discovered causal link between two nodes
#[derive(Clone, Debug)]
pub struct CausalLink {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Transfer entropy value
    pub te: f64,
    /// Strategy used for computation
    pub strategy: TeStrategy,
    /// P-value from significance testing
    pub p_value: Option<f64>,
    /// Whether the link is significant
    pub is_significant: bool,
    /// Confidence level
    pub confidence: f64,
}

impl CausalLink {
    /// Create a new causal link
    pub fn new(
        source: String,
        target: String,
        te: f64,
        strategy: TeStrategy,
    ) -> Self {
        Self {
            source,
            target,
            te,
            strategy,
            p_value: None,
            is_significant: true,
            confidence: te.abs().min(1.0),
        }
    }

    /// Check if this is a strong causal link
    pub fn is_strong(&self, threshold: f64) -> bool {
        self.te > threshold && self.is_significant
    }
}
