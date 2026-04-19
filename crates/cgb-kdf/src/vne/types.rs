//! Core VNE types and result structures

/// VNE analysis result
#[derive(Clone, Debug)]
pub struct VNEResult {
    /// Von Neumann Entropy value
    pub entropy: f64,
    /// Eigenvalues of the density matrix
    pub eigenvalues: Vec<f64>,
    /// Spectral gap (λ₂ - λ₁)
    pub spectral_gap: f64,
    /// Number of connected components (zero eigenvalues)
    pub num_components: usize,
}

impl VNEResult {
    /// Create an empty VNE result
    pub fn empty() -> Self {
        Self {
            entropy: 0.0,
            eigenvalues: Vec::new(),
            spectral_gap: 0.0,
            num_components: 0,
        }
    }
}

/// Anomaly detection result
#[derive(Clone, Debug)]
pub struct AnomalyResult {
    /// Current VNE value
    pub vne: f64,
    /// Mean of historical VNE values
    pub mean: f64,
    /// Standard deviation of historical VNE values
    pub std_dev: f64,
    /// Z-score of current VNE
    pub z_score: f64,
    /// Whether this is an anomaly
    pub is_anomaly: bool,
    /// Size of history used for calculation
    pub history_size: usize,
}

/// Change detection result
#[derive(Clone, Debug)]
pub struct ChangeDetection {
    /// VNE before change
    pub vne_before: f64,
    /// VNE after change
    pub vne_after: f64,
    /// Absolute change
    pub absolute_change: f64,
    /// Relative change
    pub relative_change: f64,
    /// Whether the change is significant
    pub is_significant: bool,
    /// Spectral gap before
    pub spectral_gap_before: f64,
    /// Spectral gap after
    pub spectral_gap_after: f64,
}
