//! Precomputed fingerprint for fast similarity calculations

use super::types::Fingerprint;

/// Precomputed fingerprint with cached norm and sorted values for fast similarity
///
/// This struct precomputes expensive operations (norm, sort, gradient) once
/// so that pairwise similarity calculations can skip redundant work.
#[derive(Clone, Debug)]
pub struct PrecomputedFingerprint {
    /// Original fingerprint vector
    pub raw: Fingerprint,
    /// Precomputed L2 norm (√Σx²)
    pub norm: f64,
    /// Precomputed sorted fingerprint (for structure similarity)
    pub sorted: Fingerprint,
    /// Precomputed gradient signs: +1, -1, or 0
    pub gradient_signs: Vec<i8>,
}

impl PrecomputedFingerprint {
    /// Create a precomputed fingerprint from a raw fingerprint
    pub fn from_fingerprint(fp: &Fingerprint) -> Self {
        // Compute norm
        let norm = fp.iter().map(|x| x.powi(2)).sum::<f64>().sqrt();

        // Compute sorted version
        let mut sorted = fp.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // Compute gradient signs
        let gradient_signs: Vec<i8> = fp
            .windows(2)
            .map(|w| {
                let diff = w[1] - w[0];
                if diff > 0.0 {
                    1
                } else if diff < 0.0 {
                    -1
                } else {
                    0
                }
            })
            .collect();

        Self {
            raw: fp.clone(),
            norm,
            sorted,
            gradient_signs,
        }
    }

    /// Check if norm is zero (for early termination)
    #[inline]
    pub fn is_zero_norm(&self) -> bool {
        self.norm == 0.0
    }
}
