//! KDF Recommendation - Diverse Recommendations
//!
//! This example shows how to use KDF for diverse recommendations:
//! - Balance popular (Core) and unique (Rare) items
//! - Avoid redundant recommendations
//! - Personalization with diversity
//!
//! Run: cargo run --example kdf_recommend

use kdf::{cosine_similarity, Kdf, Layer};
use std::collections::HashSet;

/// A recommendable item (e.g., movie, product, article)
#[derive(Clone)]
#[allow(dead_code)]
struct Item {
    id: usize,
    name: String,
    features: Vec<f64>, // Embedding or feature vector
    popularity: f64,    // 0-1, higher = more popular
}

/// Recommendation result with explanation
struct Recommendation {
    item_idx: usize,
    reason: String,
    score: f64,
}

/// KDF-based diverse recommender
struct KdfRecommender {
    kdf: Kdf,
    threshold: f64,
    diversity_weight: f64, // 0-1, higher = more diverse
}

impl KdfRecommender {
    fn new(threshold: f64, diversity_weight: f64) -> Self {
        KdfRecommender {
            kdf: Kdf::with_defaults(),
            threshold,
            diversity_weight,
        }
    }

    /// Generate diverse recommendations
    fn recommend(
        &self,
        items: &[Item],
        user_history: &[usize],
        n_recommendations: usize,
    ) -> Vec<Recommendation> {
        // Filter out items user has already seen
        let seen: HashSet<_> = user_history.iter().cloned().collect();
        let candidate_indices: Vec<usize> =
            (0..items.len()).filter(|i| !seen.contains(i)).collect();

        if candidate_indices.is_empty() {
            return vec![];
        }

        // Get candidates
        let candidates: Vec<&Item> = candidate_indices.iter().map(|&i| &items[i]).collect();

        // Apply KDF to find layer structure
        let result = self.kdf.process(&candidates, self.threshold, |a, b| {
            cosine_similarity(&a.features, &b.features)
        });

        // Score items considering layer and popularity
        let mut scored_items: Vec<(usize, f64, &str)> = Vec::new();

        for (local_idx, &global_idx) in candidate_indices.iter().enumerate() {
            let item = &items[global_idx];
            let layer = result.layers[local_idx];
            let is_selected = result.selected.contains(&local_idx);

            // Base score from popularity
            let pop_score = item.popularity;

            // Layer bonus
            let layer_bonus = match layer {
                Layer::Rare => self.diversity_weight * 0.5, // Boost rare items
                Layer::Edge => self.diversity_weight * 0.25, // Medium boost
                Layer::Core => 0.0,                         // No bonus for common
            };

            // Selection penalty (avoid redundant items)
            let selection_bonus = if is_selected { 0.1 } else { 0.0 };

            // Combined score
            let final_score =
                (1.0 - self.diversity_weight) * pop_score + layer_bonus + selection_bonus;

            let reason = match layer {
                Layer::Rare => "Unique find",
                Layer::Edge => "Interesting choice",
                Layer::Core if is_selected => "Popular pick",
                Layer::Core => "Trending",
            };

            scored_items.push((global_idx, final_score, reason));
        }

        // Sort by score descending
        scored_items.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top N, but ensure layer diversity
        let mut recommendations = Vec::new();
        let mut layers_included: HashSet<&str> = HashSet::new();

        // First pass: ensure at least one from each layer
        for layer_type in ["Unique find", "Interesting choice", "Popular pick"] {
            if recommendations.len() >= n_recommendations {
                break;
            }
            for &(idx, score, reason) in &scored_items {
                if reason == layer_type && !layers_included.contains(reason) {
                    recommendations.push(Recommendation {
                        item_idx: idx,
                        reason: reason.to_string(),
                        score,
                    });
                    layers_included.insert(reason);
                    break;
                }
            }
        }

        // Fill remaining slots
        for (idx, score, reason) in scored_items {
            if recommendations.len() >= n_recommendations {
                break;
            }
            if !recommendations.iter().any(|r| r.item_idx == idx) {
                recommendations.push(Recommendation {
                    item_idx: idx,
                    reason: reason.to_string(),
                    score,
                });
            }
        }

        recommendations
    }

    /// Analyze item pool diversity
    fn analyze_diversity(&self, items: &[Item]) -> (usize, usize, usize) {
        let result = self.kdf.process(items, self.threshold, |a, b| {
            cosine_similarity(&a.features, &b.features)
        });

        let core = result.layers.iter().filter(|&&l| l == Layer::Core).count();
        let edge = result.layers.iter().filter(|&&l| l == Layer::Edge).count();
        let rare = result.layers.iter().filter(|&&l| l == Layer::Rare).count();

        (core, edge, rare)
    }
}

fn main() {
    println!("=== KDF Recommendation Demo ===\n");

    // Create item catalog
    let items = vec![
        // Popular action movies (Core cluster)
        Item {
            id: 0,
            name: "Action Hero 1".to_string(),
            features: vec![1.0, 0.0, 0.5, 0.8],
            popularity: 0.9,
        },
        Item {
            id: 1,
            name: "Action Hero 2".to_string(),
            features: vec![1.0, 0.1, 0.5, 0.7],
            popularity: 0.85,
        },
        Item {
            id: 2,
            name: "Action Hero 3".to_string(),
            features: vec![1.0, 0.0, 0.6, 0.8],
            popularity: 0.8,
        },
        // Popular comedies (Core cluster)
        Item {
            id: 3,
            name: "Comedy Night 1".to_string(),
            features: vec![0.0, 1.0, 0.3, 0.2],
            popularity: 0.88,
        },
        Item {
            id: 4,
            name: "Comedy Night 2".to_string(),
            features: vec![0.1, 1.0, 0.3, 0.3],
            popularity: 0.75,
        },
        // Edge items (moderate popularity, somewhat unique)
        Item {
            id: 5,
            name: "Action Comedy".to_string(),
            features: vec![0.5, 0.5, 0.5, 0.5],
            popularity: 0.6,
        },
        Item {
            id: 6,
            name: "Dramedy".to_string(),
            features: vec![0.3, 0.7, 0.1, 0.4],
            popularity: 0.55,
        },
        // Rare items (unique, niche)
        Item {
            id: 7,
            name: "Art House Film".to_string(),
            features: vec![0.2, 0.1, 0.9, 0.1],
            popularity: 0.3,
        },
        Item {
            id: 8,
            name: "Documentary Special".to_string(),
            features: vec![0.0, 0.0, 0.1, 0.9],
            popularity: 0.25,
        },
        Item {
            id: 9,
            name: "Experimental Cinema".to_string(),
            features: vec![0.3, 0.3, 0.3, 0.1],
            popularity: 0.15,
        },
    ];

    // =========================================================================
    // 1. Standard vs Diverse Recommendations
    // =========================================================================
    println!("--- Standard vs Diverse Recommendations ---\n");

    let user_history = vec![0, 3]; // User has watched Action Hero 1 and Comedy Night 1

    // Low diversity (standard popularity-based)
    let standard_rec = KdfRecommender::new(0.8, 0.2);
    let std_recs = standard_rec.recommend(&items, &user_history, 5);

    println!("Standard Recommendations (diversity=0.2):");
    for rec in &std_recs {
        println!(
            "  {} - \"{}\" (score: {:.2}, {})",
            rec.item_idx, items[rec.item_idx].name, rec.score, rec.reason
        );
    }
    println!();

    // High diversity
    let diverse_rec = KdfRecommender::new(0.8, 0.8);
    let div_recs = diverse_rec.recommend(&items, &user_history, 5);

    println!("Diverse Recommendations (diversity=0.8):");
    for rec in &div_recs {
        println!(
            "  {} - \"{}\" (score: {:.2}, {})",
            rec.item_idx, items[rec.item_idx].name, rec.score, rec.reason
        );
    }
    println!();

    // =========================================================================
    // 2. Pool Diversity Analysis
    // =========================================================================
    println!("--- Item Pool Diversity ---\n");

    let (core, edge, rare) = diverse_rec.analyze_diversity(&items);
    println!("Item distribution:");
    println!(
        "  Core (common): {} items ({:.0}%)",
        core,
        100.0 * core as f64 / items.len() as f64
    );
    println!(
        "  Edge (moderate): {} items ({:.0}%)",
        edge,
        100.0 * edge as f64 / items.len() as f64
    );
    println!(
        "  Rare (unique): {} items ({:.0}%)",
        rare,
        100.0 * rare as f64 / items.len() as f64
    );
    println!();

    // =========================================================================
    // 3. Personalized Diversity
    // =========================================================================
    println!("--- Personalized Recommendations ---\n");

    // User who likes action (recommend more unique content)
    let action_fan_history = vec![0, 1, 2]; // Watched all action movies
    let recs = diverse_rec.recommend(&items, &action_fan_history, 4);

    println!("For Action Fan (diverse recommendations):");
    for rec in &recs {
        println!(
            "  {} - \"{}\" ({})",
            rec.item_idx, items[rec.item_idx].name, rec.reason
        );
    }
    println!();

    // New user (show popular + diverse)
    let new_user_history: Vec<usize> = vec![];
    let recs = diverse_rec.recommend(&items, &new_user_history, 5);

    println!("For New User:");
    for rec in &recs {
        println!(
            "  {} - \"{}\" ({})",
            rec.item_idx, items[rec.item_idx].name, rec.reason
        );
    }
    println!();

    // =========================================================================
    // 4. Comparison with Pure Popularity
    // =========================================================================
    println!("--- Pure Popularity vs KDF Recommendation ---\n");

    let mut pop_sorted: Vec<_> = items
        .iter()
        .enumerate()
        .filter(|(i, _)| !user_history.contains(i))
        .collect();
    pop_sorted.sort_by(|a, b| b.1.popularity.partial_cmp(&a.1.popularity).unwrap());

    println!("Pure Popularity (top 5):");
    for (i, item) in pop_sorted.iter().take(5) {
        println!("  {} - \"{}\" (pop: {:.2})", i, item.name, item.popularity);
    }
    println!();

    let kdf_recs = diverse_rec.recommend(&items, &user_history, 5);
    println!("KDF Diverse (top 5):");
    for rec in &kdf_recs {
        println!(
            "  {} - \"{}\" (pop: {:.2}, {})",
            rec.item_idx, items[rec.item_idx].name, items[rec.item_idx].popularity, rec.reason
        );
    }
    println!();

    println!("=== Summary ===");
    println!("KDF Recommendation benefits:");
    println!("1. Avoids redundant recommendations (Core items de-duplicated)");
    println!("2. Surfaces unique content (Rare items get visibility)");
    println!("3. Balances popularity with discovery");
    println!("4. Explainable recommendations with layer-based reasons");
}
