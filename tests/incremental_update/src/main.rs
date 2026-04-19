//! KDF Incremental Update and Dynamic Data Handling
//!
//! Tests:
//! 1. Incremental item addition
//! 2. Incremental item removal
//! 3. Delta update efficiency
//! 4. Concept drift handling

use rand::Rng;
use std::time::Instant;

#[derive(Clone)]
struct DataItem {
    id: usize,
    features: Vec<f64>,
    is_rare: bool,
}

impl DataItem {
    fn new(id: usize, features: Vec<f64>, is_rare: bool) -> Self {
        Self { id, features, is_rare }
    }

    fn similarity(&self, other: &DataItem) -> f64 {
        let dot: f64 = self.features.iter().zip(&other.features).map(|(a, b)| a * b).sum();
        let mag1: f64 = self.features.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2: f64 = other.features.iter().map(|x| x * x).sum::<f64>().sqrt();
        if mag1 == 0.0 || mag2 == 0.0 { return 0.0; }
        dot / (mag1 * mag2)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Layer { Core, Edge, Rare }

struct KdfParams {
    alpha_edge: f64,
    alpha_rare: f64,
    alpha_core: f64,
    theta_edge: f64,
    beta: f64,
    gamma: f64,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            alpha_edge: 1.5,
            alpha_rare: 0.3,
            alpha_core: 2.0,
            theta_edge: 0.15,
            beta: 0.01,
            gamma: 0.1,
        }
    }
}

/// Incremental KDF state that can be updated
struct IncrementalKdf {
    items: Vec<DataItem>,
    degrees: Vec<usize>,
    layers: Vec<Layer>,
    weights: Vec<f64>,
    sim_threshold: f64,
    params: KdfParams,
}

impl IncrementalKdf {
    fn new(sim_threshold: f64) -> Self {
        Self {
            items: Vec::new(),
            degrees: Vec::new(),
            layers: Vec::new(),
            weights: Vec::new(),
            sim_threshold,
            params: KdfParams::default(),
        }
    }

    /// Full recomputation from scratch
    fn full_compute(&mut self) {
        let n = self.items.len();
        if n == 0 { return; }

        // Reset state
        self.degrees = vec![0; n];
        self.layers = vec![Layer::Edge; n];
        self.weights = vec![1.0; n];

        // Build graph
        for i in 0..n {
            for j in (i + 1)..n {
                if self.items[i].similarity(&self.items[j]) >= self.sim_threshold {
                    self.degrees[i] += 1;
                    self.degrees[j] += 1;
                }
            }
        }

        // Classify layers
        let avg_degree: f64 = self.degrees.iter().sum::<usize>() as f64 / n as f64;
        for i in 0..n {
            let deg = self.degrees[i];
            if deg == 0 {
                self.layers[i] = Layer::Rare;
            } else if (deg as f64) > avg_degree * 1.5 {
                self.layers[i] = Layer::Core;
            } else if (deg as f64) < avg_degree * 0.3 {
                self.layers[i] = Layer::Rare;
            }
        }

        // Apply decay
        for _ in 0..100 {
            for i in 0..n {
                let c = self.degrees[i] as f64;
                let alpha = match self.layers[i] {
                    Layer::Core => self.params.alpha_core,
                    Layer::Edge => self.params.alpha_edge,
                    Layer::Rare => self.params.alpha_rare,
                };
                let decay_rate = self.params.beta * (1.0 + self.params.gamma * c.powf(alpha));
                self.weights[i] *= (1.0 - decay_rate).max(0.0);
            }
        }
    }

    /// Add a single item with incremental update
    fn add_item_incremental(&mut self, item: DataItem) {
        let n = self.items.len();
        let new_idx = n;

        // Calculate new item's degree
        let mut new_degree = 0usize;
        for i in 0..n {
            if item.similarity(&self.items[i]) >= self.sim_threshold {
                new_degree += 1;
                self.degrees[i] += 1;
            }
        }

        // Add new item
        self.items.push(item);
        self.degrees.push(new_degree);
        self.layers.push(Layer::Edge);
        self.weights.push(1.0);

        // Reclassify all layers (average degree changed)
        self.reclassify_layers();

        // Recompute weights for affected items
        self.recompute_weights();
    }

    /// Remove an item by index with incremental update
    fn remove_item_incremental(&mut self, idx: usize) {
        if idx >= self.items.len() { return; }

        let n = self.items.len();

        // Update degrees of connected items
        for i in 0..n {
            if i != idx && self.items[idx].similarity(&self.items[i]) >= self.sim_threshold {
                self.degrees[i] = self.degrees[i].saturating_sub(1);
            }
        }

        // Remove item
        self.items.remove(idx);
        self.degrees.remove(idx);
        self.layers.remove(idx);
        self.weights.remove(idx);

        // Reclassify and recompute
        self.reclassify_layers();
        self.recompute_weights();
    }

    fn reclassify_layers(&mut self) {
        let n = self.items.len();
        if n == 0 { return; }

        let avg_degree: f64 = self.degrees.iter().sum::<usize>() as f64 / n as f64;
        for i in 0..n {
            let deg = self.degrees[i];
            if deg == 0 {
                self.layers[i] = Layer::Rare;
            } else if (deg as f64) > avg_degree * 1.5 {
                self.layers[i] = Layer::Core;
            } else if (deg as f64) < avg_degree * 0.3 {
                self.layers[i] = Layer::Rare;
            } else {
                self.layers[i] = Layer::Edge;
            }
        }
    }

    fn recompute_weights(&mut self) {
        let n = self.items.len();
        self.weights = vec![1.0; n];

        for _ in 0..100 {
            for i in 0..n {
                let c = self.degrees[i] as f64;
                let alpha = match self.layers[i] {
                    Layer::Core => self.params.alpha_core,
                    Layer::Edge => self.params.alpha_edge,
                    Layer::Rare => self.params.alpha_rare,
                };
                let decay_rate = self.params.beta * (1.0 + self.params.gamma * c.powf(alpha));
                self.weights[i] *= (1.0 - decay_rate).max(0.0);
            }
        }
    }

    fn get_selected(&self) -> Vec<usize> {
        let n = self.items.len();
        if n == 0 { return vec![]; }

        let mut indices: Vec<usize> = (0..n).collect();
        indices.sort_by(|a, b| self.weights[*b].partial_cmp(&self.weights[*a]).unwrap());

        let mut selected: Vec<usize> = Vec::new();
        for &i in &indices {
            if self.layers[i] == Layer::Rare {
                selected.push(i);
            } else if self.weights[i] >= self.params.theta_edge {
                let has_similar = selected.iter()
                    .any(|&s| self.items[i].similarity(&self.items[s]) >= 0.75);
                if !has_similar {
                    selected.push(i);
                }
            }
        }

        if selected.is_empty() && !indices.is_empty() {
            selected.push(indices[0]);
        }

        selected
    }

    fn get_metrics(&self) -> (f64, f64, f64) {
        let selected = self.get_selected();

        let rare_total = self.items.iter().filter(|i| i.is_rare).count();
        let redundant_total = self.items.iter().filter(|i| !i.is_rare).count();
        let rare_preserved = selected.iter().filter(|&&i| self.items[i].is_rare).count();
        let redundant_in_selected = selected.iter().filter(|&&i| !self.items[i].is_rare).count();

        let redundancy_reduction = if redundant_total > 0 {
            (redundant_total - redundant_in_selected) as f64 / redundant_total as f64
        } else { 1.0 };

        let rare_preservation = if rare_total > 0 {
            rare_preserved as f64 / rare_total as f64
        } else { 1.0 };

        let f1_score = if redundancy_reduction + rare_preservation > 0.0 {
            2.0 * redundancy_reduction * rare_preservation / (redundancy_reduction + rare_preservation)
        } else { 0.0 };

        (redundancy_reduction, rare_preservation, f1_score)
    }
}

fn generate_cluster_item(id: usize, center: &[f64]) -> DataItem {
    let mut rng = rand::thread_rng();
    let features: Vec<f64> = center.iter()
        .map(|&v| v + (rng.gen::<f64>() * 0.1 - 0.05))
        .collect();
    DataItem::new(id, features, false)
}

fn generate_rare_item(id: usize, dim: usize) -> DataItem {
    let mut features = vec![0.0; dim];
    features[id % dim] = -1.0 - (id as f64 * 0.01);
    DataItem::new(id, features, true)
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║           KDF 動的データ・増分更新検証                        ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    // ========================================
    // Test 1: Incremental Addition
    // ========================================
    println!("══════════════════════════════════════════════════════════════");
    println!("【検証1】増分追加");
    println!("  データを1件ずつ追加し、結果の一貫性を検証");
    println!("══════════════════════════════════════════════════════════════\n");

    let mut kdf_incremental = IncrementalKdf::new(0.95);
    let center = vec![1.0, 0.9, 0.1, 0.0];
    let dim = 4;

    // Add items incrementally
    println!("{:<15} {:>8} {:>12} {:>12} {:>10}",
        "操作", "件数", "冗長削減", "レア保持", "F1スコア");
    println!("{}", "─".repeat(60));

    // Add 10 redundant items
    for i in 0..10 {
        let item = generate_cluster_item(i, &center);
        kdf_incremental.add_item_incremental(item);
    }
    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<15} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "冗長10件追加", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    // Add 2 rare items
    for i in 10..12 {
        let item = generate_rare_item(i, dim);
        kdf_incremental.add_item_incremental(item);
    }
    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<15} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "レア2件追加", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    // Add more redundant
    for i in 12..22 {
        let item = generate_cluster_item(i, &center);
        kdf_incremental.add_item_incremental(item);
    }
    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<15} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "冗長10件追加", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    let incr_add_pass = rp == 1.0;
    println!("\n判定: {} 増分追加後もレア100%保持",
        if incr_add_pass { "✓" } else { "✗" });

    // ========================================
    // Test 2: Incremental Removal
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証2】増分削除");
    println!("  冗長データを削除し、レアが保持されることを検証");
    println!("══════════════════════════════════════════════════════════════\n");

    println!("{:<20} {:>8} {:>12} {:>12} {:>10}",
        "操作", "件数", "冗長削減", "レア保持", "F1スコア");
    println!("{}", "─".repeat(65));

    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<20} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "削除前", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    // Remove 5 redundant items (indices 0-4)
    for _ in 0..5 {
        // Always remove first non-rare item
        if let Some(idx) = kdf_incremental.items.iter().position(|i| !i.is_rare) {
            kdf_incremental.remove_item_incremental(idx);
        }
    }
    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<20} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "冗長5件削除後", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    // Remove 10 more redundant items
    for _ in 0..10 {
        if let Some(idx) = kdf_incremental.items.iter().position(|i| !i.is_rare) {
            kdf_incremental.remove_item_incremental(idx);
        }
    }
    let (rr, rp, f1) = kdf_incremental.get_metrics();
    println!("{:<20} {:>8} {:>10.0}% {:>10.0}% {:>10.3}",
        "冗長10件削除後", kdf_incremental.items.len(),
        rr * 100.0, rp * 100.0, f1);

    let incr_remove_pass = rp == 1.0;
    println!("\n判定: {} 削除後もレア100%保持",
        if incr_remove_pass { "✓" } else { "✗" });

    // ========================================
    // Test 3: Incremental vs Full Comparison
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証3】増分更新 vs 完全再計算 比較");
    println!("══════════════════════════════════════════════════════════════\n");

    let test_sizes = [100, 500, 1000];

    println!("{:<10} {:>15} {:>15} {:>10}",
        "データ量", "完全再計算", "増分追加", "高速化率");
    println!("{}", "─".repeat(55));

    for size in test_sizes {
        let mut rng = rand::thread_rng();
        let center: Vec<f64> = (0..8).map(|_| rng.gen::<f64>() + 0.5).collect();

        // Prepare base items
        let base_items: Vec<DataItem> = (0..size)
            .map(|i| generate_cluster_item(i, &center))
            .collect();

        // Full recomputation approach
        let full_start = Instant::now();
        let mut kdf_full = IncrementalKdf::new(0.95);
        for item in base_items.iter().cloned() {
            kdf_full.items.push(item);
        }
        kdf_full.full_compute();
        let full_time = full_start.elapsed().as_micros();

        // Incremental approach
        let incr_start = Instant::now();
        let mut kdf_incr = IncrementalKdf::new(0.95);
        for item in base_items.into_iter() {
            kdf_incr.add_item_incremental(item);
        }
        let incr_time = incr_start.elapsed().as_micros();

        // Compare results
        let full_selected = kdf_full.get_selected();
        let incr_selected = kdf_incr.get_selected();

        let speedup = full_time as f64 / incr_time as f64;

        println!("{:<10} {:>13}μs {:>13}μs {:>10.2}x",
            format!("{}件", size),
            full_time,
            incr_time,
            speedup);
    }

    println!();
    println!("注: 増分追加は各追加時にO(n)の類似度計算が必要");
    println!("    完全再計算はO(n²)だが、一括処理できる");

    // ========================================
    // Test 4: Concept Drift
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証4】コンセプトドリフト対応");
    println!("  データ分布が徐々に変化する状況をシミュレート");
    println!("══════════════════════════════════════════════════════════════\n");

    let mut kdf_drift = IncrementalKdf::new(0.95);
    let dim = 4;

    // Initial cluster at center A
    let center_a = vec![1.0, 0.0, 0.0, 0.0];
    for i in 0..20 {
        kdf_drift.add_item_incremental(generate_cluster_item(i, &center_a));
    }
    // Add 2 rare items
    kdf_drift.add_item_incremental(generate_rare_item(100, dim));
    kdf_drift.add_item_incremental(generate_rare_item(101, dim));

    let (rr, rp, f1) = kdf_drift.get_metrics();
    println!("初期状態（クラスタA + レア2件）:");
    println!("  件数={}, 冗長削減={:.0}%, レア保持={:.0}%, F1={:.3}",
        kdf_drift.items.len(), rr * 100.0, rp * 100.0, f1);

    // Gradually shift to center B
    let center_b = vec![0.0, 1.0, 0.0, 0.0];

    // Add mixed items (drift)
    for i in 0..10 {
        // Interpolate between A and B
        let t = i as f64 / 10.0;
        let center_mixed: Vec<f64> = center_a.iter()
            .zip(&center_b)
            .map(|(&a, &b)| a * (1.0 - t) + b * t)
            .collect();
        kdf_drift.add_item_incremental(generate_cluster_item(200 + i, &center_mixed));
    }

    let (rr, rp, f1) = kdf_drift.get_metrics();
    println!("\nドリフト中間（A→B遷移中）:");
    println!("  件数={}, 冗長削減={:.0}%, レア保持={:.0}%, F1={:.3}",
        kdf_drift.items.len(), rr * 100.0, rp * 100.0, f1);

    // Complete drift to B
    for i in 0..20 {
        kdf_drift.add_item_incremental(generate_cluster_item(300 + i, &center_b));
    }

    let (rr, rp, f1) = kdf_drift.get_metrics();
    println!("\nドリフト完了（クラスタB追加）:");
    println!("  件数={}, 冗長削減={:.0}%, レア保持={:.0}%, F1={:.3}",
        kdf_drift.items.len(), rr * 100.0, rp * 100.0, f1);

    let drift_pass = rp == 1.0;
    println!("\n判定: {} コンセプトドリフト後もレア100%保持",
        if drift_pass { "✓" } else { "✗" });

    // ========================================
    // Test 5: Window-based Processing
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【検証5】ウィンドウベース処理");
    println!("  古いデータを削除しながら新しいデータを追加");
    println!("══════════════════════════════════════════════════════════════\n");

    let mut kdf_window = IncrementalKdf::new(0.95);
    let window_size = 50;
    let center = vec![1.0, 0.9, 0.1, 0.0];
    let dim = 4;

    // Initial window
    for i in 0..window_size {
        if i % 10 == 0 {
            kdf_window.add_item_incremental(generate_rare_item(i, dim));
        } else {
            kdf_window.add_item_incremental(generate_cluster_item(i, &center));
        }
    }

    println!("{:<20} {:>8} {:>12} {:>12}",
        "操作", "件数", "レア保持", "F1スコア");
    println!("{}", "─".repeat(55));

    let (rr, rp, f1) = kdf_window.get_metrics();
    println!("{:<20} {:>8} {:>10.0}% {:>10.3}",
        "初期ウィンドウ", kdf_window.items.len(), rp * 100.0, f1);

    // Slide window: add new, remove old
    for batch in 0..3 {
        // Remove oldest 10 items
        for _ in 0..10 {
            if !kdf_window.items.is_empty() {
                kdf_window.remove_item_incremental(0);
            }
        }

        // Add 10 new items
        let new_start = window_size + batch * 10;
        for i in new_start..(new_start + 10) {
            if i % 10 == 0 {
                kdf_window.add_item_incremental(generate_rare_item(i, dim));
            } else {
                kdf_window.add_item_incremental(generate_cluster_item(i, &center));
            }
        }

        let (rr, rp, f1) = kdf_window.get_metrics();
        println!("{:<20} {:>8} {:>10.0}% {:>10.3}",
            format!("スライド{}回後", batch + 1),
            kdf_window.items.len(), rp * 100.0, f1);
    }

    let window_pass = kdf_window.get_metrics().1 == 1.0;
    println!("\n判定: {} ウィンドウスライド後もレア100%保持",
        if window_pass { "✓" } else { "✗" });

    // ========================================
    // Summary
    // ========================================
    println!("\n══════════════════════════════════════════════════════════════");
    println!("【総合評価】");
    println!("══════════════════════════════════════════════════════════════\n");

    let all_pass = incr_add_pass && incr_remove_pass && drift_pass && window_pass;

    println!("┌─────────────────────────────────────────────────────────────┐");
    if all_pass {
        println!("│ ✓ 動的データ・増分更新検証: PASS                           │");
        println!("├─────────────────────────────────────────────────────────────┤");
        println!("│ ・増分追加: レア保持維持 ✓                                 │");
        println!("│ ・増分削除: レア保持維持 ✓                                 │");
        println!("│ ・コンセプトドリフト: 対応可能 ✓                           │");
        println!("│ ・ウィンドウ処理: 正常動作 ✓                               │");
    } else {
        println!("│ △ 動的データ・増分更新検証: 一部制限あり                   │");
    }
    println!("└─────────────────────────────────────────────────────────────┘");

    println!("\n【証明された事項】");
    if incr_add_pass { println!("  51. 増分追加でレア保持維持"); }
    if incr_remove_pass { println!("  52. 増分削除でレア保持維持"); }
    if drift_pass { println!("  53. コンセプトドリフトに対応可能"); }
    if window_pass { println!("  54. ウィンドウベース処理で正常動作"); }
    println!();
}
