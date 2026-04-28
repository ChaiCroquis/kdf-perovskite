//! Multimodal KDF - Processing Multiple Data Types Together
//!
//! This example demonstrates how to apply KDF to multimodal data:
//! - Text + Numeric features
//! - Image features + Metadata
//! - Any combination of modalities
//!
//! Key approach: Combine similarities from different modalities
//!
//! Run: cargo run --example kdf_multimodal

use kdf::{cosine_similarity, levenshtein_similarity, Kdf};

/// A multimodal data point with text and numeric features
#[derive(Clone)]
#[allow(dead_code)]
struct TextNumericItem {
    id: usize,
    text: String,
    numeric_features: Vec<f64>,
}

/// A product item with multiple data types (e-commerce example)
#[derive(Clone)]
#[allow(dead_code)]
struct ProductItem {
    id: usize,
    name: String,
    description: String,
    category_embedding: Vec<f64>,
    price: f64,
    rating: f64,
    sales_count: u32,
}

impl ProductItem {
    /// Get numeric features as a vector
    fn numeric_features(&self) -> Vec<f64> {
        vec![
            self.price / 1000.0,                   // Normalize price
            self.rating / 5.0,                     // Normalize rating
            (self.sales_count as f64).ln() / 10.0, // Log-normalize sales
        ]
    }
}

/// Compute weighted multimodal similarity
fn multimodal_text_numeric_similarity(
    a: &TextNumericItem,
    b: &TextNumericItem,
    text_weight: f64,
) -> f64 {
    // Text similarity (Levenshtein)
    let text_sim = levenshtein_similarity(&a.text, &b.text);

    // Numeric similarity (Cosine)
    let numeric_sim = cosine_similarity(&a.numeric_features, &b.numeric_features);

    // Weighted combination
    text_weight * text_sim + (1.0 - text_weight) * numeric_sim
}

/// Compute product similarity across multiple modalities
fn product_multimodal_similarity(a: &ProductItem, b: &ProductItem) -> f64 {
    // Text similarity (name + description)
    let name_sim = levenshtein_similarity(&a.name, &b.name);
    let desc_sim = levenshtein_similarity(&a.description, &b.description);
    let text_sim = 0.6 * name_sim + 0.4 * desc_sim;

    // Category embedding similarity
    let category_sim = cosine_similarity(&a.category_embedding, &b.category_embedding);

    // Numeric features similarity
    let numeric_a = a.numeric_features();
    let numeric_b = b.numeric_features();
    let numeric_sim = cosine_similarity(&numeric_a, &numeric_b);

    // Weighted combination
    // Category is most important for product similarity
    0.4 * category_sim + 0.3 * text_sim + 0.3 * numeric_sim
}

/// Flexible multimodal similarity with configurable weights
struct MultimodalSimilarity<'a, T> {
    modality_extractors: Vec<Box<dyn Fn(&T) -> Vec<f64> + 'a>>,
    weights: Vec<f64>,
}

impl<'a, T> MultimodalSimilarity<'a, T> {
    fn new() -> Self {
        MultimodalSimilarity {
            modality_extractors: Vec::new(),
            weights: Vec::new(),
        }
    }

    fn add_modality<F: Fn(&T) -> Vec<f64> + 'a>(&mut self, extractor: F, weight: f64) {
        self.modality_extractors.push(Box::new(extractor));
        self.weights.push(weight);
    }

    fn compute(&self, a: &T, b: &T) -> f64 {
        if self.modality_extractors.is_empty() {
            return 0.0;
        }

        let total_weight: f64 = self.weights.iter().sum();
        if total_weight == 0.0 {
            return 0.0;
        }

        let mut total_sim = 0.0;
        for (i, extractor) in self.modality_extractors.iter().enumerate() {
            let vec_a = extractor(a);
            let vec_b = extractor(b);
            let sim = cosine_similarity(&vec_a, &vec_b);
            total_sim += self.weights[i] * sim;
        }

        total_sim / total_weight
    }
}

fn main() {
    println!("=== Multimodal KDF Demo ===\n");

    let kdf = Kdf::with_defaults();

    // =========================================================================
    // 1. Text + Numeric (Simple Multimodal)
    // =========================================================================
    println!("--- Text + Numeric Multimodal ---\n");

    let text_numeric_items = vec![
        // Cluster 1: Similar text, similar numbers
        TextNumericItem {
            id: 0,
            text: "machine learning".to_string(),
            numeric_features: vec![1.0, 0.5, 0.8],
        },
        TextNumericItem {
            id: 1,
            text: "machine learning models".to_string(),
            numeric_features: vec![1.0, 0.6, 0.7],
        },
        TextNumericItem {
            id: 2,
            text: "deep learning".to_string(),
            numeric_features: vec![0.9, 0.5, 0.9],
        },
        // Cluster 2: Different text, similar to each other
        TextNumericItem {
            id: 3,
            text: "web development".to_string(),
            numeric_features: vec![0.3, 0.8, 0.2],
        },
        TextNumericItem {
            id: 4,
            text: "web development basics".to_string(),
            numeric_features: vec![0.3, 0.9, 0.3],
        },
        // Rare: Unique text with outlier numeric features
        TextNumericItem {
            id: 5,
            text: "quantum computing".to_string(),
            numeric_features: vec![0.1, 0.1, 0.1],
        },
    ];

    // Process with different text weights
    for text_weight in [0.3, 0.5, 0.7] {
        let result = kdf.process(&text_numeric_items, 0.7, |a, b| {
            multimodal_text_numeric_similarity(a, b, text_weight)
        });

        println!("Text weight = {:.1}:", text_weight);
        println!("  Selected: {:?}", result.selected);
        println!("  Rare items: {:?}", result.rare_items());
        println!("  Layers: {:?}\n", result.layers);
    }

    // =========================================================================
    // 2. E-commerce Product Multimodal
    // =========================================================================
    println!("--- E-commerce Product Multimodal ---\n");

    let products = vec![
        // Electronics cluster
        ProductItem {
            id: 0,
            name: "Wireless Bluetooth Headphones".to_string(),
            description: "High quality audio, noise cancelling".to_string(),
            category_embedding: vec![1.0, 0.0, 0.0, 0.0],
            price: 89.99,
            rating: 4.5,
            sales_count: 1500,
        },
        ProductItem {
            id: 1,
            name: "Bluetooth Earbuds Pro".to_string(),
            description: "Wireless earbuds with great sound".to_string(),
            category_embedding: vec![1.0, 0.1, 0.0, 0.0],
            price: 79.99,
            rating: 4.3,
            sales_count: 2000,
        },
        ProductItem {
            id: 2,
            name: "Premium Wireless Headset".to_string(),
            description: "Professional audio quality".to_string(),
            category_embedding: vec![1.0, 0.0, 0.1, 0.0],
            price: 149.99,
            rating: 4.7,
            sales_count: 800,
        },
        // Clothing cluster
        ProductItem {
            id: 3,
            name: "Cotton T-Shirt".to_string(),
            description: "Comfortable everyday wear".to_string(),
            category_embedding: vec![0.0, 1.0, 0.0, 0.0],
            price: 19.99,
            rating: 4.2,
            sales_count: 5000,
        },
        ProductItem {
            id: 4,
            name: "Basic Cotton Tee".to_string(),
            description: "Soft cotton shirt".to_string(),
            category_embedding: vec![0.0, 1.0, 0.0, 0.0],
            price: 14.99,
            rating: 4.0,
            sales_count: 8000,
        },
        // Rare: Unique product
        ProductItem {
            id: 5,
            name: "Vintage Record Player".to_string(),
            description: "Classic vinyl turntable with modern features".to_string(),
            category_embedding: vec![0.5, 0.0, 0.5, 0.0],
            price: 299.99,
            rating: 4.8,
            sales_count: 150,
        },
        // Rare: Cross-category
        ProductItem {
            id: 6,
            name: "Smart Fitness Watch".to_string(),
            description: "Health tracking with style".to_string(),
            category_embedding: vec![0.5, 0.3, 0.0, 0.2],
            price: 199.99,
            rating: 4.6,
            sales_count: 1200,
        },
    ];

    let result_products = kdf.process(&products, 0.7, product_multimodal_similarity);

    println!("Product KDF Results:");
    println!("  Total products: {}", products.len());
    println!("  Selected: {:?}", result_products.selected);
    println!(
        "  Compression: {:.0}%",
        (1.0 - result_products.selected.len() as f64 / products.len() as f64) * 100.0
    );
    println!();

    println!("Products by layer:");
    for (i, product) in products.iter().enumerate() {
        let layer = &result_products.layers[i];
        let selected = if result_products.selected.contains(&i) {
            "*"
        } else {
            " "
        };
        println!(
            "  {} {} [{}] - \"{}\" (${:.2})",
            selected,
            format!("{:?}", layer).chars().next().unwrap(),
            i,
            product.name,
            product.price
        );
    }
    println!();

    println!("Rare products (unique, worth investigating):");
    for &idx in result_products.rare_items().iter() {
        println!("  - {}: \"{}\"", idx, products[idx].name);
    }
    println!();

    // =========================================================================
    // 3. Flexible Multimodal with Custom Weights
    // =========================================================================
    println!("--- Flexible Multimodal System ---\n");

    #[derive(Clone)]
    struct Document {
        title: String,
        content: String,
        topic_vector: Vec<f64>,
        metadata: Vec<f64>, // e.g., [length, citations, year]
    }

    let documents = vec![
        Document {
            title: "Introduction to AI".to_string(),
            content: "Artificial intelligence overview".to_string(),
            topic_vector: vec![1.0, 0.0, 0.0],
            metadata: vec![100.0, 50.0, 2023.0],
        },
        Document {
            title: "Machine Learning Basics".to_string(),
            content: "ML fundamentals explained".to_string(),
            topic_vector: vec![0.9, 0.1, 0.0],
            metadata: vec![150.0, 80.0, 2023.0],
        },
        Document {
            title: "Deep Learning Advances".to_string(),
            content: "Neural network breakthroughs".to_string(),
            topic_vector: vec![0.8, 0.2, 0.0],
            metadata: vec![200.0, 120.0, 2024.0],
        },
        Document {
            title: "Web Development".to_string(),
            content: "Building modern web apps".to_string(),
            topic_vector: vec![0.0, 0.0, 1.0],
            metadata: vec![80.0, 30.0, 2022.0],
        },
        Document {
            title: "Quantum Computing Ethics".to_string(),
            content: "Ethical considerations in quantum".to_string(),
            topic_vector: vec![0.3, 0.3, 0.4],
            metadata: vec![50.0, 5.0, 2024.0],
        },
    ];

    // Create flexible multimodal similarity
    let mut mm_sim: MultimodalSimilarity<Document> = MultimodalSimilarity::new();

    // Add topic modality (weight 0.5)
    mm_sim.add_modality(|d| d.topic_vector.clone(), 0.5);

    // Add metadata modality (weight 0.3)
    mm_sim.add_modality(
        |d| {
            // Normalize metadata
            vec![
                d.metadata[0] / 500.0,
                d.metadata[1] / 200.0,
                (d.metadata[2] - 2020.0) / 5.0,
            ]
        },
        0.3,
    );

    // Note: Text modality would need embedding in real use
    // Here we use a simple length-based approximation
    mm_sim.add_modality(
        |d| vec![d.title.len() as f64 / 50.0, d.content.len() as f64 / 100.0],
        0.2,
    );

    let result_docs = kdf.process(&documents, 0.7, |a, b| mm_sim.compute(a, b));

    println!("Document KDF with flexible multimodal:");
    println!("  Modalities: topic (0.5) + metadata (0.3) + text_length (0.2)");
    println!("  Selected: {:?}", result_docs.selected);
    println!("  Layers: {:?}", result_docs.layers);
    println!();

    for (i, doc) in documents.iter().enumerate() {
        let layer = &result_docs.layers[i];
        println!("  [{:?}] \"{}\"", layer, doc.title);
    }
    println!();

    // =========================================================================
    // Summary
    // =========================================================================
    println!("=== Summary ===");
    println!("Multimodal KDF enables:");
    println!("1. Processing data with multiple types (text, images, numbers)");
    println!("2. Configurable weights for each modality");
    println!("3. Finding rare items that are unique across ALL modalities");
    println!("4. Flexible extension to any number of data types");
    println!();
    println!("Key insight: An item is truly Rare only if it's unique");
    println!("across all relevant modalities, not just one.");
}
