//! Demo D3 — ML training dataset curation preserving long-tail classes.
//!
//! # Hypothesis
//! Per Stage 1 meta-analysis: rareness = minority class labels. These are
//! defined *independently of graph structure* (labels sit on nodes, not
//! edges). Stage 1 classified this as "D5-type: KDF marginal". We test
//! this prediction and find: KDF with feature-similarity graph + Analogy
//! can exceed degree-based baselines, but Stratified (with labels) remains
//! the unbeatable reference.
//!
//! # Data
//! Synthetic 10-class classification dataset with long-tail class frequencies
//! (class 0 = 40%, class 9 = 1%). Each sample is a 32-d feature vector.
//! We build a k-NN graph (k=5) in feature space and treat it as KDF's input.
//!
//! No actual ML training happens — we only evaluate **selection quality** by
//! the retained minority-class ratio and feature-space coverage after pruning.
//!
//! # Baselines
//! - Random sampling
//! - Stratified (requires class labels)
//! - Herding proxy: keep samples near per-class centroid (weak baseline)
//! - ClassBalance: keep N per class (requires labels, like stratified oracle)
//! - KDF baseline (classifier on k-NN graph)
//! - KDF+Analogy (fingerprint-isolated bonus)

use kdf_demos_common::{visualizer::emit_artifacts, Axis, Conclusion, DemoReport, Metric, MethodResult};
use rand::prelude::*;
use rand::rngs::SmallRng;
use real_data_bench::Dataset;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

const DEMO_ID: &str = "D3";
const DEMO_TITLE: &str = "ML 学習データ長尾クラス保持 curation";
const N_TRIALS: usize = 10;
const SELECTION_FRAC: f64 = 0.30;
const N_SAMPLES: usize = 2_000;
const N_CLASSES: usize = 10;
const FEATURE_DIM: usize = 32;
const K_NN: usize = 5;

fn main() {
    let seeds: Vec<u64> = (0..N_TRIALS as u64).map(|i| 7000 + i).collect();

    // Build once (seed 0 for determinism of dataset) — it's reused per trial
    // so that "variance" comes from the samplers' own randomness.
    let (samples, labels) = synthesize_longtail_dataset(N_SAMPLES, N_CLASSES, FEATURE_DIM, 42);
    let class_freq = class_frequencies(&labels);
    let minority_classes: HashSet<u32> = class_freq.iter()
        .filter(|(_, &c)| c < (N_SAMPLES as u32) / N_CLASSES as u32)
        .map(|(&c, _)| c)
        .collect();

    println!("Dataset: n={}, classes={:?}", samples.len(), class_freq);
    println!("Minority classes (below mean freq): {:?}", minority_classes);

    // Build feature-space k-NN graph once
    let ds = build_knn_graph(&samples, K_NN);
    println!("kNN graph: nodes={}, edges={}", ds.n_nodes, ds.edges.len());

    let keep = (N_SAMPLES as f64 * SELECTION_FRAC).ceil() as usize;

    let methods: Vec<(String, bool, Box<dyn Fn(&[Vec<f64>], &[u32], &Dataset, u64) -> HashSet<u32>>)> = vec![
        ("Random".into(), false, Box::new(move |_s, _l, _d, seed| sample_random(seed, N_SAMPLES, keep))),
        ("Stratified".into(), true, Box::new(move |_s, labels, _d, seed| sample_stratified(labels, keep, seed))),
        ("HerdingProxy".into(), false, Box::new(move |samples, _l, _d, _seed| sample_herding(samples, keep))),
        ("ClassBalance".into(), true, Box::new(move |_s, labels, _d, seed| sample_class_balance(labels, keep, seed))),
        ("KDF".into(), false, Box::new(move |_s, _l, ds, _seed| sample_kdf(ds, keep))),
        ("KDF+RelDensity".into(), false, Box::new(move |_s, _l, ds, _seed| sample_kdf_reldensity(ds, keep))),
        ("KDF+Analogy".into(), false, Box::new(move |_s, _l, ds, _seed| sample_kdf_analogy(ds, keep))),
    ];

    let mut method_results: Vec<MethodResult> = Vec::new();
    let mut raw_trials: BTreeMap<String, Vec<f64>> = BTreeMap::new();

    for (name, needs_label, sampler) in &methods {
        let mut minority_recalls = Vec::new();
        let mut diversities = Vec::new();
        let mut compressions = Vec::new();
        let mut walls = Vec::new();
        for &seed in &seeds {
            let t0 = Instant::now();
            let sel = sampler(&samples, &labels, &ds, seed);
            let ms = t0.elapsed().as_secs_f64() * 1000.0;

            let minority_recall = minority_class_recall(&sel, &labels, &minority_classes);
            let diversity = feature_diversity(&sel, &samples);
            let comp = 1.0 - sel.len() as f64 / N_SAMPLES as f64;

            minority_recalls.push(minority_recall);
            diversities.push(diversity);
            compressions.push(comp);
            walls.push(ms);
            raw_trials.entry(format!("{}/minority_recall", name)).or_default().push(minority_recall);
            raw_trials.entry(format!("{}/diversity", name)).or_default().push(diversity);
        }
        let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        let r = mean(&minority_recalls);
        let d = mean(&diversities);
        let c = mean(&compressions);
        let w = mean(&walls);
        println!("{:16} minority_recall={:.3}  diversity={:.3}  comp={:.3}  ms={:.2}",
            name, r, d, c, w);
        let mut metrics = BTreeMap::new();
        metrics.insert("minority_recall".into(), r);
        metrics.insert("diversity".into(), d);
        metrics.insert("compression".into(), c);
        metrics.insert("label_free".into(), if *needs_label { 0.0 } else { 1.0 });
        metrics.insert("wall_ms".into(), w);
        method_results.push(MethodResult {
            method: name.clone(), requires_labels: *needs_label,
            metrics, wall_ms: w, notes: String::new(),
        });
    }

    let metric_definitions = vec![
        Metric { name: "minority_recall".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::KdfStrength },
        Metric { name: "label_free".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::KdfStrength },
        Metric { name: "diversity".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::Tie },
        Metric { name: "compression".into(), higher_is_better: true, mean: 0.0, stderr: 0.0, axis: Axis::Tie },
        Metric { name: "wall_ms".into(), higher_is_better: false, mean: 0.0, stderr: 0.0, axis: Axis::KdfWeakness },
    ];

    let report = DemoReport {
        demo_id: DEMO_ID.to_string(),
        title: DEMO_TITLE.to_string(),
        dataset_name: format!("synthetic_longtail_n{}_c{}", N_SAMPLES, N_CLASSES),
        n_items: N_SAMPLES,
        patent_section: "明細書 §0002 (学習データ) / Claim 1, 18, 46".into(),
        metric_definitions,
        method_results,
        raw_trials,
        conclusion: Conclusion {
            kdf_recommended_for: vec![
                "**ラベル未取得**の段階でデータキュレーションしたい場合(事前フィルタリング)".into(),
                "Feature 空間の構造(kNN 等)を活用できる pipeline".into(),
                "Herding 等の unsupervised baseline よりは minority を残せる運用".into(),
            ],
            kdf_not_recommended_for: vec![
                "**ラベルが確実に得られる** 環境 → Stratified / ClassBalance が絶対強".into(),
                "Feature vector が不在 or 意味のない特徴量だけの dataset".into(),
                "モデル訓練中の動的選択(active learning) → これは別の仕組みが必要".into(),
            ],
            honest_limits: vec![
                "**合成 dataset**(正規分布 cluster + noise)での評価。実 MNIST/CIFAR では結果が変わる可能性大".into(),
                "訓練を実行していない(downstream accuracy は未測定)".into(),
                "Stage 1 meta-analysis の予測「D5 型(label 独立 → KDF marginal)」を概ね確認する結果".into(),
            ],
        },
    };

    let out_dir = std::path::Path::new("demos/D3_ml_longtail/out");
    emit_artifacts(&report, out_dir).expect("emit");
    println!("\n✅ D3 artifacts written to {}", out_dir.display());
}

// ============================================================================
// Dataset synthesis + metrics
// ============================================================================

fn synthesize_longtail_dataset(n: usize, n_classes: usize, dim: usize, seed: u64) -> (Vec<Vec<f64>>, Vec<u32>) {
    let mut rng = SmallRng::seed_from_u64(seed);
    // Zipf-ish class frequencies: class 0 has 40% of samples, class n_classes-1 has ~1%
    let class_weights: Vec<f64> = (1..=n_classes).map(|r| 1.0 / r as f64).collect();
    let cw_sum: f64 = class_weights.iter().sum();
    let class_probs: Vec<f64> = class_weights.iter().map(|w| w / cw_sum).collect();

    // Per-class centroids in feature space
    let centroids: Vec<Vec<f64>> = (0..n_classes).map(|c| {
        (0..dim).map(|d| ((c * 31 + d * 17) % 100) as f64 / 10.0).collect()
    }).collect();

    let mut samples = Vec::with_capacity(n);
    let mut labels = Vec::with_capacity(n);
    for _ in 0..n {
        let u = rng.gen::<f64>();
        let mut acc = 0.0;
        let mut c = n_classes - 1;
        for (i, p) in class_probs.iter().enumerate() {
            acc += p;
            if u < acc { c = i; break; }
        }
        // Sample = centroid + Gaussian noise
        let feat: Vec<f64> = centroids[c].iter()
            .map(|&m| m + (rng.gen::<f64>() - 0.5) * 2.0)
            .collect();
        samples.push(feat);
        labels.push(c as u32);
    }
    (samples, labels)
}

fn class_frequencies(labels: &[u32]) -> BTreeMap<u32, u32> {
    let mut m: BTreeMap<u32, u32> = BTreeMap::new();
    for &c in labels { *m.entry(c).or_insert(0) += 1; }
    m
}

fn build_knn_graph(samples: &[Vec<f64>], k: usize) -> Dataset {
    let n = samples.len();
    let mut edges = Vec::with_capacity(n * k);
    for i in 0..n {
        // Find k nearest by Euclidean distance (brute force — OK for n=2000)
        let mut dists: Vec<(usize, f64)> = (0..n).filter(|&j| j != i)
            .map(|j| (j, l2_distance(&samples[i], &samples[j])))
            .collect();
        dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(j, _) in dists.iter().take(k) {
            edges.push((i as u32, j as u32, 1.0));
        }
    }
    Dataset {
        name: "knn_longtail".into(),
        n_nodes: n,
        edges,
        rare_ground_truth: HashSet::new(), // not used — ground truth lives in labels
        description: format!("kNN graph (k={}) in feature space", k),
    }
}

fn l2_distance(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
}

fn minority_class_recall(sel: &HashSet<u32>, labels: &[u32], minority: &HashSet<u32>) -> f64 {
    let minority_total = labels.iter().filter(|&c| minority.contains(c)).count().max(1);
    let minority_sel = sel.iter().filter(|&&i| minority.contains(&labels[i as usize])).count();
    minority_sel as f64 / minority_total as f64
}

fn feature_diversity(sel: &HashSet<u32>, samples: &[Vec<f64>]) -> f64 {
    // Mean pairwise distance as a diversity proxy (sampled pairs, O(k^2) but capped)
    let mut sel_vec: Vec<u32> = sel.iter().copied().collect();
    sel_vec.sort(); // determinism
    if sel_vec.len() < 2 { return 0.0; }
    let cap = sel_vec.len().min(100);
    let mut total = 0.0;
    let mut count = 0;
    for i in 0..cap {
        for j in (i + 1)..cap {
            total += l2_distance(&samples[sel_vec[i] as usize], &samples[sel_vec[j] as usize]);
            count += 1;
        }
    }
    if count == 0 { 0.0 } else { total / count as f64 }
}

// ============================================================================
// Samplers
// ============================================================================

fn sample_random(seed: u64, n: usize, keep: usize) -> HashSet<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut indices: Vec<u32> = (0..n as u32).collect();
    indices.shuffle(&mut rng);
    indices.into_iter().take(keep).collect()
}

fn sample_stratified(labels: &[u32], keep: usize, seed: u64) -> HashSet<u32> {
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut by_class: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (i, &c) in labels.iter().enumerate() {
        by_class.entry(c).or_default().push(i as u32);
    }
    // Reserve proportional per-class quota, but with floor=1 to protect minority
    let n_classes = by_class.len();
    let floor = 1;
    let mut out: HashSet<u32> = HashSet::new();
    let total_quota = keep;
    let mut allocated = 0usize;
    // First: floor per class
    for (_, idx) in by_class.iter_mut() {
        idx.shuffle(&mut rng);
        for &i in idx.iter().take(floor) { out.insert(i); allocated += 1; }
    }
    // Then: proportional to class size for remaining budget
    let remain = total_quota.saturating_sub(allocated);
    let total_samples: f64 = labels.len() as f64;
    for (_, idx) in by_class.iter() {
        let quota = (idx.len() as f64 / total_samples * remain as f64).round() as usize;
        for &i in idx.iter().take(floor + quota) {
            if out.len() >= total_quota { break; }
            out.insert(i);
        }
    }
    out
}

fn sample_herding(samples: &[Vec<f64>], keep: usize) -> HashSet<u32> {
    // Keep points closest to global mean (weak diversity baseline)
    let n = samples.len();
    let dim = samples[0].len();
    let mut mean = vec![0.0; dim];
    for s in samples {
        for (m, v) in mean.iter_mut().zip(s) { *m += v; }
    }
    for m in mean.iter_mut() { *m /= n as f64; }
    let mut dists: Vec<(u32, f64)> = (0..n as u32)
        .map(|i| (i, l2_distance(&samples[i as usize], &mean)))
        .collect();
    dists.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dists.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn sample_class_balance(labels: &[u32], keep: usize, seed: u64) -> HashSet<u32> {
    // Uniform per-class (labels required). Gold-standard oracle.
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut by_class: BTreeMap<u32, Vec<u32>> = BTreeMap::new();
    for (i, &c) in labels.iter().enumerate() {
        by_class.entry(c).or_default().push(i as u32);
    }
    let per_class = keep / by_class.len().max(1);
    let mut out: HashSet<u32> = HashSet::new();
    for (_, idx) in by_class.iter_mut() {
        idx.shuffle(&mut rng);
        for &i in idx.iter().take(per_class) { out.insert(i); }
    }
    out
}

fn sample_kdf(ds: &Dataset, keep: usize) -> HashSet<u32> {
    use cgb_kdf::{Layer, NodeClassifier};
    let mut classifier = NodeClassifier::default();
    let class = classifier.classify(ds.n_nodes, &ds.edges);
    let score = |l: Layer| -> i32 {
        match l { Layer::Rare => 3, Layer::Core => 2, Layer::Edge => 1, Layer::Garbage => 0 }
    };
    let mut scored: Vec<(u32, i32)> = (0..ds.n_nodes as u32)
        .map(|id| (id, score(class.layers.get(&id).copied().unwrap_or(Layer::Edge))))
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.into_iter().take(keep).map(|(id, _)| id).collect()
}

/// KDF + Phase 7 S2 RelativeDensity — relative-to-neighbor rareness.
/// Applied to the kNN graph, minority-class samples whose neighborhood is
/// denser than their own degree should surface as "relatively rare".
fn sample_kdf_reldensity(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let n = ds.n_nodes;
    let mut deg = vec![0usize; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        if (u as usize) < n && (v as usize) < n {
            deg[u as usize] += 1;
            deg[v as usize] += 1;
            adj[u as usize].push(v);
            adj[v as usize].push(u);
        }
    }
    let mut scored: Vec<(u32, f64)> = (0..n as u32).map(|id| {
        let neighbors = &adj[id as usize];
        if neighbors.is_empty() { return (id, -1.0); }
        let local_avg: f64 = neighbors.iter().map(|&v| deg[v as usize] as f64).sum::<f64>()
            / neighbors.len() as f64;
        let ratio = deg[id as usize] as f64 / local_avg.max(1.0);
        (id, 1.0 - ratio)
    }).collect();
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.into_iter().take(keep).map(|(i, _)| i).collect()
}

fn sample_kdf_analogy(ds: &Dataset, keep: usize) -> HashSet<u32> {
    let mut out = sample_kdf(ds, (keep as f64 * 0.8) as usize);
    // Analogy bonus: pick nodes with unusual neighbor-degree histograms
    let n = ds.n_nodes;
    let mut deg = vec![0u32; n];
    let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n];
    for &(u, v, _) in &ds.edges {
        deg[u as usize] += 1;
        deg[v as usize] += 1;
        adj[u as usize].push(v);
        adj[v as usize].push(u);
    }
    let fps: Vec<[f64; 4]> = (0..n).map(|i| {
        let mut bins = [0.0f64; 4];
        for &v in &adj[i] {
            let d = deg[v as usize];
            let idx = if d < 3 { 0 } else if d < 7 { 1 } else if d < 15 { 2 } else { 3 };
            bins[idx] += 1.0;
        }
        let tot: f64 = bins.iter().sum();
        if tot > 0.0 { for b in bins.iter_mut() { *b /= tot; } }
        bins
    }).collect();
    let mut median = [0.0f64; 4];
    for dim in 0..4 {
        let mut v: Vec<f64> = fps.iter().map(|fp| fp[dim]).collect();
        v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        median[dim] = v[v.len() / 2];
    }
    let mut ranked: Vec<(u32, f64)> = (0..n as u32)
        .filter(|id| !out.contains(id))
        .map(|id| (id, fps[id as usize].iter().zip(median.iter()).map(|(a, b)| (a - b).abs()).sum::<f64>()))
        .collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let bonus = keep.saturating_sub(out.len());
    for (id, _) in ranked.into_iter().take(bonus) { out.insert(id); }
    out
}
