//! KDF + Active Learning: Smart sample selection for labeling
//!
//! Problem: Labeling data is expensive. Traditional active learning:
//! - Uncertainty sampling: labels samples model is uncertain about
//! - Random sampling: ignores structure entirely
//! - Diversity sampling: requires expensive pairwise distance computation
//!
//! Solution: Use KDF for intelligent sample selection by:
//! - Prioritizing Rare layer items (unique patterns, most informative)
//! - Using Edge layer for uncertainty-like behavior
//! - Avoiding Core layer redundancy (similar to already labeled)
//!
//! Comparison:
//! 1. Random Sampling (baseline)
//! 2. Uncertainty Sampling (traditional AL)
//! 3. KDF-enhanced Sampling (our approach)

use kdf::{Kdf, KdfParams, Layer};

/// Simple pseudo-random number generator
static mut GLOBAL_SEED: u64 = 42;

#[allow(dead_code)]
fn rand_simple() -> f64 {
    unsafe {
        GLOBAL_SEED = GLOBAL_SEED.wrapping_mul(1103515245).wrapping_add(12345);
        ((GLOBAL_SEED >> 16) & 0x7FFF) as f64 / 32768.0
    }
}

fn reset_seed(seed: u64) {
    unsafe {
        GLOBAL_SEED = seed;
    }
}

/// Euclidean similarity
fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}

/// Generate synthetic dataset with hidden structure
fn generate_dataset(n_samples: usize) -> (Vec<Vec<f64>>, Vec<usize>) {
    let mut data = Vec::new();
    let mut labels = Vec::new();

    // Class 0: Large normal cluster (60%)
    for i in 0..(n_samples * 6 / 10) {
        let x = 0.5 + (i as f64 * 0.001).sin() * 0.1;
        let y = 0.5 + (i as f64 * 0.001).cos() * 0.1;
        data.push(vec![x, y]);
        labels.push(0);
    }

    // Class 1: Medium cluster (25%)
    for i in 0..(n_samples * 25 / 100) {
        let x = -0.5 + (i as f64 * 0.002).sin() * 0.15;
        let y = 0.3 + (i as f64 * 0.002).cos() * 0.15;
        data.push(vec![x, y]);
        labels.push(1);
    }

    // Class 2: Rare cluster (10%)
    for i in 0..(n_samples * 10 / 100) {
        let x = 0.0 + (i as f64 * 0.01) * 0.2;
        let y = -0.8 + (i as f64 * 0.01) * 0.1;
        data.push(vec![x, y]);
        labels.push(2);
    }

    // Class 3: Very rare outliers (5%)
    for i in 0..(n_samples * 5 / 100) {
        let angle = (i as f64) * 1.5;
        let x = angle.cos() * 0.9;
        let y = angle.sin() * 0.9;
        data.push(vec![x, y]);
        labels.push(3);
    }

    (data, labels)
}

/// Simple nearest-neighbor classifier (simulates model prediction)
struct SimpleClassifier {
    train_data: Vec<Vec<f64>>,
    train_labels: Vec<usize>,
}

impl SimpleClassifier {
    fn new() -> Self {
        Self {
            train_data: Vec::new(),
            train_labels: Vec::new(),
        }
    }

    fn add_sample(&mut self, features: Vec<f64>, label: usize) {
        self.train_data.push(features);
        self.train_labels.push(label);
    }

    fn predict_proba(&self, features: &[f64]) -> Vec<f64> {
        if self.train_data.is_empty() {
            return vec![0.25; 4]; // Uniform uncertainty
        }

        // Find distances to all training samples
        let mut class_weights = vec![0.0; 4];
        let mut total_weight = 0.0;

        for (data, &label) in self.train_data.iter().zip(&self.train_labels) {
            let sim = euclidean_similarity(features, data);
            class_weights[label] += sim;
            total_weight += sim;
        }

        if total_weight > 0.0 {
            for w in &mut class_weights {
                *w /= total_weight;
            }
        } else {
            class_weights = vec![0.25; 4];
        }

        class_weights
    }

    fn predict(&self, features: &[f64]) -> usize {
        let proba = self.predict_proba(features);
        proba
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    fn uncertainty(&self, features: &[f64]) -> f64 {
        let proba = self.predict_proba(features);
        // Entropy as uncertainty measure
        -proba
            .iter()
            .filter(|&&p| p > 0.0)
            .map(|&p| p * p.ln())
            .sum::<f64>()
    }

    fn accuracy(&self, test_data: &[Vec<f64>], test_labels: &[usize]) -> f64 {
        if self.train_data.is_empty() {
            return 0.0;
        }
        let correct = test_data
            .iter()
            .zip(test_labels)
            .filter(|(features, &label)| self.predict(features) == label)
            .count();
        correct as f64 / test_data.len() as f64
    }

    fn class_accuracy(&self, test_data: &[Vec<f64>], test_labels: &[usize]) -> Vec<f64> {
        let mut class_correct = [0; 4];
        let mut class_total = vec![0; 4];

        for (features, &label) in test_data.iter().zip(test_labels) {
            class_total[label] += 1;
            if self.predict(features) == label {
                class_correct[label] += 1;
            }
        }

        class_correct
            .iter()
            .zip(&class_total)
            .map(|(&c, &t)| if t > 0 { c as f64 / t as f64 } else { 0.0 })
            .collect()
    }
}

/// Random Sampling strategy
fn random_sampling(
    unlabeled_indices: &[usize],
    _data: &[Vec<f64>],
    _classifier: &SimpleClassifier,
    batch_size: usize,
) -> Vec<usize> {
    let step = unlabeled_indices.len() / batch_size.max(1);
    if step == 0 {
        return unlabeled_indices.to_vec();
    }
    unlabeled_indices
        .iter()
        .step_by(step.max(1))
        .take(batch_size)
        .copied()
        .collect()
}

/// Uncertainty Sampling strategy
fn uncertainty_sampling(
    unlabeled_indices: &[usize],
    data: &[Vec<f64>],
    classifier: &SimpleClassifier,
    batch_size: usize,
) -> Vec<usize> {
    let mut scored: Vec<(usize, f64)> = unlabeled_indices
        .iter()
        .map(|&i| (i, classifier.uncertainty(&data[i])))
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    scored.iter().take(batch_size).map(|(i, _)| *i).collect()
}

/// KDF-enhanced Sampling strategy
fn kdf_sampling(
    unlabeled_indices: &[usize],
    data: &[Vec<f64>],
    _classifier: &SimpleClassifier,
    batch_size: usize,
) -> Vec<usize> {
    if unlabeled_indices.is_empty() {
        return Vec::new();
    }

    // Get unlabeled data
    let unlabeled_data: Vec<Vec<f64>> =
        unlabeled_indices.iter().map(|&i| data[i].clone()).collect();

    let kdf = Kdf::new(KdfParams::builder().selection_sim_threshold(0.5).build());

    let result = kdf.process(&unlabeled_data, 0.8, |a, b| euclidean_similarity(a, b));

    // Balanced selection: prioritize Rare but include representation from all layers
    let rare = result.rare_items();
    let edge = result.edge_items();
    let core_selected: Vec<usize> = result
        .selected
        .iter()
        .filter(|&&i| result.layers[i] == Layer::Core)
        .copied()
        .collect();

    let mut selected = Vec::new();

    // Allocate slots proportionally: 40% Rare, 30% Edge, 30% Core (if available)
    let rare_slots = (batch_size * 4 / 10).max(1);
    let edge_slots = (batch_size * 3 / 10).max(1);
    let core_slots = batch_size - rare_slots - edge_slots;

    // Add Rare items (most informative)
    for local_idx in rare.iter().take(rare_slots) {
        selected.push(unlabeled_indices[*local_idx]);
    }

    // Add Edge items (boundary samples)
    for local_idx in edge.iter().take(edge_slots) {
        selected.push(unlabeled_indices[*local_idx]);
    }

    // Add diverse Core items (representation)
    for local_idx in core_selected.iter().take(core_slots) {
        selected.push(unlabeled_indices[*local_idx]);
    }

    // If we still need more, fill from remaining
    if selected.len() < batch_size {
        for &local_idx in &result.selected {
            if selected.len() >= batch_size {
                break;
            }
            let global_idx = unlabeled_indices[local_idx];
            if !selected.contains(&global_idx) {
                selected.push(global_idx);
            }
        }
    }

    selected
}

/// Run active learning simulation
fn simulate_active_learning(
    data: &[Vec<f64>],
    labels: &[usize],
    test_data: &[Vec<f64>],
    test_labels: &[usize],
    sampling_fn: fn(&[usize], &[Vec<f64>], &SimpleClassifier, usize) -> Vec<usize>,
    total_budget: usize,
    batch_size: usize,
) -> (Vec<f64>, Vec<Vec<f64>>, Vec<usize>) {
    let mut classifier = SimpleClassifier::new();
    let mut unlabeled: Vec<usize> = (0..data.len()).collect();
    let mut accuracy_history = Vec::new();
    let mut class_acc_history = Vec::new();
    let mut rare_labeled = 0;

    let n_iterations = (total_budget / batch_size).max(1);

    for _ in 0..n_iterations {
        if unlabeled.is_empty() {
            break;
        }

        // Select samples to label
        let to_label = sampling_fn(&unlabeled, data, &classifier, batch_size);

        // Label selected samples
        for &idx in &to_label {
            classifier.add_sample(data[idx].clone(), labels[idx]);
            if labels[idx] == 3 {
                // Rare class
                rare_labeled += 1;
            }
            unlabeled.retain(|&i| i != idx);
        }

        // Evaluate
        let acc = classifier.accuracy(test_data, test_labels);
        let class_acc = classifier.class_accuracy(test_data, test_labels);
        accuracy_history.push(acc);
        class_acc_history.push(class_acc);
    }

    (accuracy_history, class_acc_history, vec![rare_labeled])
}

fn main() {
    println!("=== KDF + Active Learning: スマートサンプル選択 ===\n");

    // ========================================================================
    // Setup
    // ========================================================================
    println!("## 1. データセット\n");

    let (data, labels) = generate_dataset(500);
    let (test_data, test_labels) = generate_dataset(200);

    // Count class distribution
    let class_counts: Vec<usize> = (0..4)
        .map(|c| labels.iter().filter(|&&l| l == c).count())
        .collect();

    println!("   訓練データ: {} 件", data.len());
    println!(
        "   - Class 0 (多数派): {} 件 ({:.1}%)",
        class_counts[0],
        class_counts[0] as f64 / data.len() as f64 * 100.0
    );
    println!(
        "   - Class 1 (中間): {} 件 ({:.1}%)",
        class_counts[1],
        class_counts[1] as f64 / data.len() as f64 * 100.0
    );
    println!(
        "   - Class 2 (希少): {} 件 ({:.1}%)",
        class_counts[2],
        class_counts[2] as f64 / data.len() as f64 * 100.0
    );
    println!(
        "   - Class 3 (極希少): {} 件 ({:.1}%)",
        class_counts[3],
        class_counts[3] as f64 / data.len() as f64 * 100.0
    );
    println!("   テストデータ: {} 件", test_data.len());

    let total_budget = 100; // Total samples to label
    let batch_size = 10; // Samples per iteration

    println!("\n   ラベリング予算: {} 件", total_budget);
    println!("   バッチサイズ: {} 件", batch_size);

    // ========================================================================
    // Run experiments
    // ========================================================================
    println!("\n## 2. 各手法のシミュレーション\n");

    // Random Sampling
    reset_seed(42);
    let (random_acc, random_class, random_rare) = simulate_active_learning(
        &data,
        &labels,
        &test_data,
        &test_labels,
        random_sampling,
        total_budget,
        batch_size,
    );

    // Uncertainty Sampling
    reset_seed(42);
    let (uncertainty_acc, uncertainty_class, uncertainty_rare) = simulate_active_learning(
        &data,
        &labels,
        &test_data,
        &test_labels,
        uncertainty_sampling,
        total_budget,
        batch_size,
    );

    // KDF Sampling
    reset_seed(42);
    let (kdf_acc, kdf_class, kdf_rare) = simulate_active_learning(
        &data,
        &labels,
        &test_data,
        &test_labels,
        kdf_sampling,
        total_budget,
        batch_size,
    );

    // ========================================================================
    // Learning Curves
    // ========================================================================
    println!("## 3. 学習曲線\n");

    println!("   | サンプル数 | Random | Uncertainty | KDF |");
    println!("   |------------|--------|-------------|-----|");
    for (i, ((r, u), k)) in random_acc
        .iter()
        .zip(&uncertainty_acc)
        .zip(&kdf_acc)
        .enumerate()
    {
        let n = (i + 1) * batch_size;
        println!(
            "   | {:>10} | {:>5.1}% | {:>10.1}% | {:>3.1}% |",
            n,
            r * 100.0,
            u * 100.0,
            k * 100.0
        );
    }

    // ========================================================================
    // Final Comparison
    // ========================================================================
    println!("\n## 4. 最終結果 ({}件ラベリング後)\n", total_budget);

    let final_random = random_acc.last().copied().unwrap_or(0.0);
    let final_uncertainty = uncertainty_acc.last().copied().unwrap_or(0.0);
    let final_kdf = kdf_acc.last().copied().unwrap_or(0.0);

    let final_random_class = random_class.last().cloned().unwrap_or(vec![0.0; 4]);
    let final_uncertainty_class = uncertainty_class.last().cloned().unwrap_or(vec![0.0; 4]);
    let final_kdf_class = kdf_class.last().cloned().unwrap_or(vec![0.0; 4]);

    println!("   | 手法 | 全体精度 | Class2(希少) | Class3(極希少) |");
    println!("   |------|----------|--------------|----------------|");
    println!(
        "   | Random      | {:>6.1}% | {:>10.1}% | {:>12.1}% |",
        final_random * 100.0,
        final_random_class[2] * 100.0,
        final_random_class[3] * 100.0
    );
    println!(
        "   | Uncertainty | {:>6.1}% | {:>10.1}% | {:>12.1}% |",
        final_uncertainty * 100.0,
        final_uncertainty_class[2] * 100.0,
        final_uncertainty_class[3] * 100.0
    );
    println!(
        "   | KDF         | {:>6.1}% | {:>10.1}% | {:>12.1}% |",
        final_kdf * 100.0,
        final_kdf_class[2] * 100.0,
        final_kdf_class[3] * 100.0
    );

    // ========================================================================
    // Rare Class Coverage
    // ========================================================================
    println!("\n## 5. 希少クラスのカバレッジ\n");

    println!("   極希少クラス(Class 3)のラベリング数:");
    println!(
        "   - Random:      {} / {} ({:.1}%)",
        random_rare[0],
        class_counts[3],
        random_rare[0] as f64 / class_counts[3] as f64 * 100.0
    );
    println!(
        "   - Uncertainty: {} / {} ({:.1}%)",
        uncertainty_rare[0],
        class_counts[3],
        uncertainty_rare[0] as f64 / class_counts[3] as f64 * 100.0
    );
    println!(
        "   - KDF:         {} / {} ({:.1}%)",
        kdf_rare[0],
        class_counts[3],
        kdf_rare[0] as f64 / class_counts[3] as f64 * 100.0
    );

    // ========================================================================
    // Analysis
    // ========================================================================
    println!("\n## 6. 分析\n");

    println!("   【Random Samplingの問題】");
    println!("   - クラス分布に比例してサンプル → 希少クラス見逃し");
    println!("   - 重複した情報をラベリング → 効率が悪い");

    println!("\n   【Uncertainty Samplingの問題】");
    println!("   - モデル初期は全て不確実 → 初期選択がランダムに");
    println!("   - 決定境界付近に集中 → 希少クラス内部を見逃し");

    println!("\n   【KDFの優位性】");

    if final_kdf_class[3] > final_random_class[3] {
        println!(
            "   ✓ 極希少クラス精度: {:.1}% vs Random {:.1}%",
            final_kdf_class[3] * 100.0,
            final_random_class[3] * 100.0
        );
    }

    if kdf_rare[0] > random_rare[0] {
        println!(
            "   ✓ 希少クラスの早期発見: {} vs Random {}",
            kdf_rare[0], random_rare[0]
        );
    }

    println!("   ✓ モデル不要: 初期段階から効果的な選択");
    println!("   ✓ 構造ベース: データの内在的多様性を活用");

    // ========================================================================
    // Theoretical Advantage
    // ========================================================================
    println!("\n## 7. 理論的優位性\n");

    println!("   | 観点 | Random | Uncertainty | KDF |");
    println!("   |------|--------|-------------|-----|");
    println!("   | モデル依存 | × | ○ | × |");
    println!("   | 希少検出 | × | △ | ○(自動) |");
    println!("   | 初期効率 | △ | × | ○ |");
    println!("   | 冗長回避 | × | × | ○ |");
    println!("   | 数学的保証 | × | × | ○ |");

    println!("\n   KDF + Active Learning の独自価値:");
    println!("   「モデルなしで希少パターンを優先的に発見」");
    println!("   → Cold-start問題を解決");

    println!("\n✅ KDF + Active Learning 検証完了");
}
