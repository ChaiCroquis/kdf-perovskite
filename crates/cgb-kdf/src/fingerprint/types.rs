//! Core types for fingerprinting

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Fingerprint vector type
pub type Fingerprint = Vec<f64>;

/// Cache key for fingerprints
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct FingerprintKey {
    pub node_id: String,
    pub label: String,
}

/// Node label for fingerprint generation
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum NodeLabel {
    IsolatedTruth,
    Normal,
    Garbage,
    Unknown,
}

impl std::str::FromStr for NodeLabel {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "isolated_truth" => NodeLabel::IsolatedTruth,
            "normal" => NodeLabel::Normal,
            "garbage" => NodeLabel::Garbage,
            _ => NodeLabel::Unknown,
        })
    }
}

impl NodeLabel {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeLabel::IsolatedTruth => "isolated_truth",
            NodeLabel::Normal => "normal",
            NodeLabel::Garbage => "garbage",
            NodeLabel::Unknown => "unknown",
        }
    }
}

/// Statistics for cache performance
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    pub total_computations: u64,
    pub cache_hits: u64,
    pub cache_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_computations + self.cache_hits;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}

/// Hash string to seed
pub(crate) fn hash_to_seed(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}
