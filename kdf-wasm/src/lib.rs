//! KDF WebAssembly Bindings
//!
//! Enables running KDF directly in the browser

use wasm_bindgen::prelude::*;
use kdf::{Kdf, levenshtein_similarity, cosine_similarity, euclidean_similarity};
use serde::{Serialize, Deserialize};

/// Layer classification
#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum Layer {
    Core = 0,
    Edge = 1,
    Rare = 2,
}

impl From<kdf::Layer> for Layer {
    fn from(layer: kdf::Layer) -> Self {
        match layer {
            kdf::Layer::Core => Layer::Core,
            kdf::Layer::Edge => Layer::Edge,
            kdf::Layer::Rare => Layer::Rare,
        }
    }
}

/// KDF processing result
#[derive(Serialize, Deserialize)]
pub struct KdfResult {
    pub selected: Vec<usize>,
    pub layers: Vec<String>,
    /// Selection scores (renamed from weights - 'weights' is not a KDF concept)
    pub selection_scores: Vec<f64>,
    /// Backward compatibility alias
    pub weights: Vec<f64>,
    pub total: usize,
    pub core_count: usize,
    pub edge_count: usize,
    pub rare_count: usize,
    pub compression_rate: f64,
}

/// KDF processor for WebAssembly
#[wasm_bindgen]
pub struct KdfWasm {
    inner: Kdf,
}

#[wasm_bindgen]
impl KdfWasm {
    /// Create a new KDF processor
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        KdfWasm {
            inner: Kdf::with_defaults(),
        }
    }

    /// Process text data using Levenshtein similarity
    ///
    /// Returns a JSON object with results
    #[wasm_bindgen]
    pub fn process_text(&self, data: JsValue, threshold: f64) -> Result<JsValue, JsValue> {
        let data: Vec<String> = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse data: {}", e)))?;

        if data.is_empty() {
            return Err(JsValue::from_str("Data cannot be empty"));
        }

        let total = data.len();
        let result = self.inner.process(&data, threshold, |a, b| {
            levenshtein_similarity(a, b)
        });

        let kdf_result = self.to_result(result, total);
        serde_wasm_bindgen::to_value(&kdf_result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Process numeric vectors using cosine similarity
    #[wasm_bindgen]
    pub fn process_vectors(&self, data: JsValue, threshold: f64) -> Result<JsValue, JsValue> {
        let data: Vec<Vec<f64>> = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse data: {}", e)))?;

        if data.is_empty() {
            return Err(JsValue::from_str("Data cannot be empty"));
        }

        let total = data.len();
        let result = self.inner.process(&data, threshold, |a, b| {
            cosine_similarity(a, b)
        });

        let kdf_result = self.to_result(result, total);
        serde_wasm_bindgen::to_value(&kdf_result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    /// Process numeric vectors using Euclidean similarity
    #[wasm_bindgen]
    pub fn process_euclidean(&self, data: JsValue, threshold: f64) -> Result<JsValue, JsValue> {
        let data: Vec<Vec<f64>> = serde_wasm_bindgen::from_value(data)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse data: {}", e)))?;

        if data.is_empty() {
            return Err(JsValue::from_str("Data cannot be empty"));
        }

        let total = data.len();
        let result = self.inner.process(&data, threshold, |a, b| {
            euclidean_similarity(a, b)
        });

        let kdf_result = self.to_result(result, total);
        serde_wasm_bindgen::to_value(&kdf_result)
            .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
    }

    fn to_result(&self, result: kdf::KdfResult, total: usize) -> KdfResult {
        let layers: Vec<String> = result.layers.iter().map(|l| {
            match l {
                kdf::Layer::Core => "Core".to_string(),
                kdf::Layer::Edge => "Edge".to_string(),
                kdf::Layer::Rare => "Rare".to_string(),
            }
        }).collect();

        let core_count = result.layers.iter().filter(|l| matches!(l, kdf::Layer::Core)).count();
        let edge_count = result.layers.iter().filter(|l| matches!(l, kdf::Layer::Edge)).count();
        let rare_count = result.layers.iter().filter(|l| matches!(l, kdf::Layer::Rare)).count();
        let selected_len = result.selected.len();

        KdfResult {
            selected: result.selected,
            layers,
            selection_scores: result.selection_scores.clone(),
            weights: result.selection_scores,  // Backward compatibility
            total,
            core_count,
            edge_count,
            rare_count,
            compression_rate: 1.0 - selected_len as f64 / total as f64,
        }
    }
}

/// Compute Levenshtein similarity between two strings
#[wasm_bindgen]
pub fn wasm_levenshtein_similarity(a: &str, b: &str) -> f64 {
    levenshtein_similarity(a, b)
}

/// Compute cosine similarity between two vectors
#[wasm_bindgen]
pub fn wasm_cosine_similarity(a: JsValue, b: JsValue) -> Result<f64, JsValue> {
    let a: Vec<f64> = serde_wasm_bindgen::from_value(a)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse a: {}", e)))?;
    let b: Vec<f64> = serde_wasm_bindgen::from_value(b)
        .map_err(|e| JsValue::from_str(&format!("Failed to parse b: {}", e)))?;
    Ok(cosine_similarity(&a, &b))
}
