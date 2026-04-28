//! KDF Python Bindings
//!
//! Simple Python interface for Knowledge Decay Framework
//!
//! Usage:
//!   from kdf_rs import Kdf, Layer
//!
//!   kdf = Kdf()
//!   result = kdf.process_text(["hello", "hello", "world", "unique"], threshold=0.7)
//!   print(result.rare_items())  # ["unique"]

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// Import from kdf crate with explicit path
use ::kdf::{
    cosine_similarity as rust_cosine, euclidean_similarity as rust_euclidean,
    levenshtein_similarity as rust_levenshtein,
};
use ::kdf::{Kdf as RustKdf, KdfParams, KdfResult as RustKdfResult, Layer as RustLayer};

/// Layer classification
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Core = 0,
    Edge = 1,
    Rare = 2,
}

impl From<RustLayer> for Layer {
    fn from(layer: RustLayer) -> Self {
        match layer {
            RustLayer::Core => Layer::Core,
            RustLayer::Edge => Layer::Edge,
            RustLayer::Rare => Layer::Rare,
        }
    }
}

#[pymethods]
impl Layer {
    fn __repr__(&self) -> String {
        match self {
            Layer::Core => "Layer.Core".to_string(),
            Layer::Edge => "Layer.Edge".to_string(),
            Layer::Rare => "Layer.Rare".to_string(),
        }
    }

    fn __str__(&self) -> String {
        match self {
            Layer::Core => "Core".to_string(),
            Layer::Edge => "Edge".to_string(),
            Layer::Rare => "Rare".to_string(),
        }
    }
}

/// KDF processing result
#[pyclass]
#[derive(Clone)]
pub struct KdfResult {
    #[pyo3(get)]
    pub selected: Vec<usize>,
    #[pyo3(get)]
    pub layers: Vec<Layer>,
    /// Selection scores (renamed from weights - 'weights' is not a KDF concept)
    #[pyo3(get)]
    pub selection_scores: Vec<f64>,
    total: usize,
}

#[pymethods]
impl KdfResult {
    /// Backward compatibility: weights property (deprecated, use selection_scores)
    #[getter]
    fn weights(&self) -> Vec<f64> {
        self.selection_scores.clone()
    }

    /// Get indices of Core layer items
    fn core_items(&self) -> Vec<usize> {
        self.selected
            .iter()
            .filter(|&&i| matches!(self.layers.get(i), Some(Layer::Core)))
            .cloned()
            .collect()
    }

    /// Get indices of Edge layer items
    fn edge_items(&self) -> Vec<usize> {
        self.selected
            .iter()
            .filter(|&&i| matches!(self.layers.get(i), Some(Layer::Edge)))
            .cloned()
            .collect()
    }

    /// Get indices of Rare layer items
    fn rare_items(&self) -> Vec<usize> {
        self.selected
            .iter()
            .filter(|&&i| matches!(self.layers.get(i), Some(Layer::Rare)))
            .cloned()
            .collect()
    }

    /// Compression rate (0.0 to 1.0)
    fn compression_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            1.0 - self.selected.len() as f64 / self.total as f64
        }
    }

    /// Summary statistics
    fn summary(&self) -> PyObject {
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new_bound(py);
            dict.set_item("total", self.total).unwrap();
            dict.set_item("selected", self.selected.len()).unwrap();
            dict.set_item("compression_rate", self.compression_rate())
                .unwrap();
            dict.set_item("core_count", self.core_items().len())
                .unwrap();
            dict.set_item("edge_count", self.edge_items().len())
                .unwrap();
            dict.set_item("rare_count", self.rare_items().len())
                .unwrap();
            dict.unbind().into_any()
        })
    }

    fn __repr__(&self) -> String {
        format!(
            "KdfResult(selected={}, core={}, edge={}, rare={}, compression={:.1}%)",
            self.selected.len(),
            self.core_items().len(),
            self.edge_items().len(),
            self.rare_items().len(),
            self.compression_rate() * 100.0
        )
    }
}

impl From<(RustKdfResult, usize)> for KdfResult {
    fn from((result, total): (RustKdfResult, usize)) -> Self {
        let layers: Vec<Layer> = result.layers.iter().map(|&l| l.into()).collect();
        KdfResult {
            selected: result.selected,
            layers,
            selection_scores: result.selection_scores,
            total,
        }
    }
}

/// Result of auto-threshold analysis
#[pyclass]
#[derive(Clone)]
pub struct AutoThresholdResult {
    #[pyo3(get)]
    pub threshold: f64,
    #[pyo3(get)]
    pub result: KdfResult,
    #[pyo3(get)]
    pub thresholds_evaluated: Vec<f64>,
    #[pyo3(get)]
    pub scores: Vec<f64>,
}

#[pymethods]
impl AutoThresholdResult {
    fn __repr__(&self) -> String {
        format!(
            "AutoThresholdResult(threshold={:.2}, selected={}, evaluated={})",
            self.threshold,
            self.result.selected.len(),
            self.thresholds_evaluated.len()
        )
    }

    /// Get the best threshold found
    fn best_threshold(&self) -> f64 {
        self.threshold
    }

    /// Get score for each evaluated threshold as dict
    fn threshold_scores(&self) -> PyObject {
        Python::with_gil(|py| {
            let dict = pyo3::types::PyDict::new_bound(py);
            for (t, s) in self.thresholds_evaluated.iter().zip(self.scores.iter()) {
                dict.set_item(format!("{:.2}", t), *s).unwrap();
            }
            dict.unbind().into_any()
        })
    }
}

/// KDF processor
#[pyclass]
pub struct Kdf {
    inner: RustKdf,
}

#[pymethods]
impl Kdf {
    /// Create a new KDF processor with default parameters
    #[new]
    #[pyo3(signature = (alpha_core=2.0, alpha_edge=1.5, alpha_rare=0.3, iterations=100))]
    fn new(alpha_core: f64, alpha_edge: f64, alpha_rare: f64, iterations: usize) -> Self {
        let params = KdfParams::builder()
            .alpha_core(alpha_core)
            .alpha_edge(alpha_edge)
            .alpha_rare(alpha_rare)
            .iterations(iterations)
            .build();
        Kdf {
            inner: RustKdf::new(params),
        }
    }

    /// Process text data using Levenshtein similarity
    ///
    /// Args:
    ///     data: List of strings
    ///     threshold: Similarity threshold (0.0-1.0, default 0.7)
    ///
    /// Returns:
    ///     KdfResult with selected items and layer classification
    #[pyo3(signature = (data, threshold=0.7))]
    fn process_text(&self, data: Vec<String>, threshold: f64) -> PyResult<KdfResult> {
        if data.is_empty() {
            return Err(PyValueError::new_err("Data cannot be empty"));
        }

        let result = self
            .inner
            .process(&data, threshold, |a, b| rust_levenshtein(a, b));

        Ok((result, data.len()).into())
    }

    /// Process numeric vectors using cosine similarity
    ///
    /// Args:
    ///     data: List of vectors (List[List[float]])
    ///     threshold: Similarity threshold (0.0-1.0, default 0.7)
    ///
    /// Returns:
    ///     KdfResult with selected items and layer classification
    #[pyo3(signature = (data, threshold=0.7))]
    fn process_vectors(&self, data: Vec<Vec<f64>>, threshold: f64) -> PyResult<KdfResult> {
        if data.is_empty() {
            return Err(PyValueError::new_err("Data cannot be empty"));
        }

        let result = self
            .inner
            .process(&data, threshold, |a, b| rust_cosine(a, b));

        Ok((result, data.len()).into())
    }

    /// Process numeric vectors using Euclidean similarity
    ///
    /// Args:
    ///     data: List of vectors (List[List[float]])
    ///     threshold: Similarity threshold (0.0-1.0, default 0.7)
    ///
    /// Returns:
    ///     KdfResult with selected items and layer classification
    #[pyo3(signature = (data, threshold=0.7))]
    fn process_euclidean(&self, data: Vec<Vec<f64>>, threshold: f64) -> PyResult<KdfResult> {
        if data.is_empty() {
            return Err(PyValueError::new_err("Data cannot be empty"));
        }

        let result = self
            .inner
            .process(&data, threshold, |a, b| rust_euclidean(a, b));

        Ok((result, data.len()).into())
    }

    /// Auto-threshold for vector data (cosine similarity)
    ///
    /// Automatically finds the optimal similarity threshold.
    ///
    /// Args:
    ///     data: List of vectors (List[List[float]])
    ///
    /// Returns:
    ///     AutoThresholdResult with optimal threshold and results
    fn process_vectors_auto(&self, data: Vec<Vec<f64>>) -> PyResult<AutoThresholdResult> {
        if data.is_empty() {
            return Err(PyValueError::new_err("Data cannot be empty"));
        }

        let n = data.len();
        let rust_result = self.inner.process_auto(&data, |a, b| rust_cosine(a, b));

        let layers: Vec<Layer> = rust_result
            .result
            .layers
            .iter()
            .map(|&l| l.into())
            .collect();
        let kdf_result = KdfResult {
            selected: rust_result.result.selected,
            layers,
            selection_scores: rust_result.result.selection_scores,
            total: n,
        };

        Ok(AutoThresholdResult {
            threshold: rust_result.threshold,
            result: kdf_result,
            thresholds_evaluated: rust_result.thresholds_evaluated,
            scores: rust_result.scores,
        })
    }

    /// Auto-threshold for text data (Levenshtein similarity)
    ///
    /// Automatically finds the optimal similarity threshold.
    ///
    /// Args:
    ///     data: List of strings
    ///
    /// Returns:
    ///     AutoThresholdResult with optimal threshold and results
    fn process_text_auto(&self, data: Vec<String>) -> PyResult<AutoThresholdResult> {
        if data.is_empty() {
            return Err(PyValueError::new_err("Data cannot be empty"));
        }

        let n = data.len();
        let rust_result = self
            .inner
            .process_auto(&data, |a, b| rust_levenshtein(a, b));

        let layers: Vec<Layer> = rust_result
            .result
            .layers
            .iter()
            .map(|&l| l.into())
            .collect();
        let kdf_result = KdfResult {
            selected: rust_result.result.selected,
            layers,
            selection_scores: rust_result.result.selection_scores,
            total: n,
        };

        Ok(AutoThresholdResult {
            threshold: rust_result.threshold,
            result: kdf_result,
            thresholds_evaluated: rust_result.thresholds_evaluated,
            scores: rust_result.scores,
        })
    }

    fn __repr__(&self) -> String {
        "Kdf()".to_string()
    }
}

/// Compute Levenshtein similarity between two strings
#[pyfunction]
fn levenshtein_similarity(a: &str, b: &str) -> f64 {
    rust_levenshtein(a, b)
}

/// Compute cosine similarity between two vectors
#[pyfunction]
fn cosine_similarity(a: Vec<f64>, b: Vec<f64>) -> f64 {
    rust_cosine(&a, &b)
}

/// Compute Euclidean similarity between two vectors
#[pyfunction]
fn euclidean_similarity(a: Vec<f64>, b: Vec<f64>) -> f64 {
    rust_euclidean(&a, &b)
}

/// Python module
#[pymodule]
fn kdf_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Layer>()?;
    m.add_class::<KdfResult>()?;
    m.add_class::<AutoThresholdResult>()?;
    m.add_class::<Kdf>()?;
    m.add_function(wrap_pyfunction!(levenshtein_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(cosine_similarity, m)?)?;
    m.add_function(wrap_pyfunction!(euclidean_similarity, m)?)?;
    Ok(())
}
