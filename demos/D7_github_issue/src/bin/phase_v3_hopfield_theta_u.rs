//! Phase V3 — θ_U Hopfield 仮説の最小実験的検証 (C3 conjecture)
//!
//! paper §4.2 Conjecture: KDF の上限閾値 θ_U は Hopfield 連想記憶の
//! spurious attractor 問題に対する棄却機構として機能しうる(未検証)。
//!
//! 本実験は 100-neuron Hopfield ネットワークを自前実装し、
//! 学習パターン数 P を capacity 近傍 (~0.138*N = ~14) で変化させ、
//! 以下の 2 条件を比較する:
//!
//! A) θ_U フィルタなし(従来の Hopfield 想起)
//! B) θ_U フィルタあり(想起後状態が既存パターンと cosine > θ_U なら
//!    別のパターンとの重複も検査し、重複が threshold を超えれば spurious
//!    として棄却)
//!
//! 測定指標:
//! - recall rate: 正しい学習パターン返還率
//! - spurious rate: 非学習パターン(mixture state)に収束した率
//! - theta_U がどれほど spurious を削減できるか
//!
//! **失敗基準**: θ_U フィルタが spurious を 0〜20% 未満しか削減できなければ
//! C3 conjecture は (現状の定式化では) 効果薄として **paper §4.2 を修正**する。

use rand::{rngs::SmallRng, Rng, SeedableRng};

const N: usize = 100; // neurons

/// Generate a random bipolar pattern in {-1, +1}^N
fn random_pattern(rng: &mut SmallRng) -> Vec<i8> {
    (0..N)
        .map(|_| if rng.gen::<bool>() { 1 } else { -1 })
        .collect()
}

/// Hebbian weight learning: W_ij = (1/N) * sum_p ξ_i^(p) ξ_j^(p)
fn hebb_weights(patterns: &[Vec<i8>]) -> Vec<Vec<f64>> {
    let mut w = vec![vec![0.0; N]; N];
    for pat in patterns {
        for i in 0..N {
            for j in 0..N {
                if i != j {
                    w[i][j] += (pat[i] as f64) * (pat[j] as f64);
                }
            }
        }
    }
    let scale = 1.0 / (N as f64);
    for row in w.iter_mut() {
        for v in row.iter_mut() {
            *v *= scale;
        }
    }
    w
}

/// Synchronous update until convergence or step cap.
fn recall(state: Vec<i8>, w: &[Vec<f64>], max_steps: usize) -> Vec<i8> {
    let mut s = state;
    for _ in 0..max_steps {
        let mut changed = false;
        let mut new_s = s.clone();
        for i in 0..N {
            let mut h = 0.0;
            for j in 0..N {
                h += w[i][j] * (s[j] as f64);
            }
            let new_val = if h >= 0.0 { 1i8 } else { -1i8 };
            if new_val != new_s[i] {
                new_s[i] = new_val;
                changed = true;
            }
        }
        s = new_s;
        if !changed {
            break;
        }
    }
    s
}

/// Cosine similarity for bipolar vectors (= normalized dot product / N).
fn cos_sim(a: &[i8], b: &[i8]) -> f64 {
    let dot: i64 = a
        .iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x as i64) * (y as i64))
        .sum();
    dot as f64 / N as f64
}

/// Flip `n_flips` bits of pattern at random positions to create cue.
fn flip_noise(pattern: &[i8], n_flips: usize, rng: &mut SmallRng) -> Vec<i8> {
    let mut cue = pattern.to_vec();
    for _ in 0..n_flips {
        let idx = rng.gen_range(0..N);
        cue[idx] = -cue[idx];
    }
    cue
}

/// Classify recalled state:
///   - Recall: max cos sim with any stored pattern >= 0.95 AND argmax unique
///   - Spurious: max cos sim < 0.95 OR state matches multiple patterns >= 0.7
///
/// With theta_U filter: additionally reject if recalled state has cos >= theta_U
///   with TWO OR MORE stored patterns simultaneously (mixture state indicator).
fn classify(
    recalled: &[i8],
    patterns: &[Vec<i8>],
    target_idx: usize,
    theta_u: Option<f64>,
) -> &'static str {
    let sims: Vec<f64> = patterns.iter().map(|p| cos_sim(recalled, p)).collect();
    let max_sim = sims.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let argmax = sims.iter().position(|&s| s == max_sim).unwrap();

    // θ_U filter: if 2+ patterns have cos >= θ_U, this is a mixture/spurious.
    if let Some(tu) = theta_u {
        let high_sim_count = sims.iter().filter(|&&s| s.abs() >= tu).count();
        if high_sim_count >= 2 {
            return "spurious_rejected_by_theta_U";
        }
    }

    if max_sim >= 0.95 {
        if argmax == target_idx {
            "recall_correct"
        } else {
            "recall_wrong_pattern"
        }
    } else if max_sim <= -0.95 {
        "recall_inverse"
    } else {
        "spurious_mixture"
    }
}

fn run_trial(
    p: usize,
    n_flips: usize,
    theta_u: Option<f64>,
    seed: u64,
) -> (usize, usize, usize, usize) {
    // returns (recall_correct, recall_wrong, spurious_mixture, spurious_rejected_by_theta_u)
    let mut rng = SmallRng::seed_from_u64(seed);
    let patterns: Vec<Vec<i8>> = (0..p).map(|_| random_pattern(&mut rng)).collect();
    let w = hebb_weights(&patterns);

    let mut c_correct = 0;
    let mut c_wrong = 0;
    let mut c_spurious = 0;
    let mut c_rejected = 0;

    for (idx, pat) in patterns.iter().enumerate() {
        let cue = flip_noise(pat, n_flips, &mut rng);
        let recalled = recall(cue, &w, 50);
        match classify(&recalled, &patterns, idx, theta_u) {
            "recall_correct" => c_correct += 1,
            "recall_wrong_pattern" | "recall_inverse" => c_wrong += 1,
            "spurious_mixture" => c_spurious += 1,
            "spurious_rejected_by_theta_U" => c_rejected += 1,
            _ => {}
        }
    }
    (c_correct, c_wrong, c_spurious, c_rejected)
}

fn main() {
    println!("# Phase V3 — θ_U Hopfield 実験");
    println!(
        "\nN = {} neurons, capacity ≈ 0.138*N = {}",
        N,
        (0.138 * N as f64) as usize
    );
    println!("各条件で 5 seeds の平均。cue は 10 bits flip のノイズ版。\n");

    // Sweep P and theta_u conditions
    let p_values = [5, 10, 14, 18, 22];
    let theta_u_cases: Vec<Option<f64>> =
        vec![None, Some(0.80), Some(0.70), Some(0.55), Some(0.40)];

    println!("| P (patterns) | θ_U | recall_correct | spurious_mixture | spurious_rejected | effective recall |");
    println!("|---:|---:|---:|---:|---:|---:|");

    let mut results: Vec<(usize, Option<f64>, f64, f64, f64, f64)> = Vec::new();
    for &p in &p_values {
        for theta_u in &theta_u_cases {
            let mut sums = [0usize; 4];
            let n_seeds = 5;
            for seed in 0..n_seeds {
                let (c, w, s, r) = run_trial(p, 10, *theta_u, seed as u64);
                sums[0] += c;
                sums[1] += w;
                sums[2] += s;
                sums[3] += r;
            }
            let total = (p * n_seeds) as f64;
            let recall_rate = sums[0] as f64 / total;
            let spurious_rate = sums[2] as f64 / total;
            let rejected_rate = sums[3] as f64 / total;
            let effective_recall = sums[0] as f64 / (total - sums[3] as f64).max(1.0);

            let tu_str = theta_u
                .map(|t| format!("{:.2}", t))
                .unwrap_or_else(|| "—".to_string());
            println!(
                "| {} | {} | {:.2} | {:.2} | {:.2} | {:.2} |",
                p, tu_str, recall_rate, spurious_rate, rejected_rate, effective_recall
            );
            results.push((
                p,
                *theta_u,
                recall_rate,
                spurious_rate,
                rejected_rate,
                effective_recall,
            ));
        }
    }

    println!("\n## 解釈");
    println!("- **recall_correct**: 目的パターンに正しく収束した率");
    println!("- **spurious_mixture**: どのパターンにも強く収束しなかった(mixture 状態)");
    println!(
        "- **spurious_rejected**: θ_U フィルタが「複数パターン類似度 >= θ_U」として棄却した率"
    );
    println!("- **effective_recall**: θ_U 棄却分を除外した純粋な recall 精度(棄却=「分からない」と答える戦略)\n");

    // Quantify: θ_U filter effect
    println!("## 仮説検証:");
    println!();
    println!("**C3 conjecture** は「θ_U が spurious 抑制に寄与する」。");
    println!("具体的には、θ_U=0.80 条件で spurious_mixture 率が baseline (None) より削減されれば支持。\n");

    // For each P, compare baseline vs theta_u=0.80
    for &p in &p_values {
        let baseline = results.iter().find(|r| r.0 == p && r.1.is_none()).unwrap();
        let tu80 = results
            .iter()
            .find(|r| r.0 == p && r.1 == Some(0.80))
            .unwrap();
        let baseline_spurious = baseline.3; // spurious_mixture rate
        let tu_spurious = tu80.3;
        let rejection_contribution = tu80.4; // rejected rate
        println!(
            "- P={}: baseline spurious={:.2}, θ_U=0.80 spurious={:.2}, θ_U rejected={:.2}",
            p, baseline_spurious, tu_spurious, rejection_contribution
        );

        if rejection_contribution > 0.20
            && (baseline_spurious - tu_spurious + rejection_contribution) > 0.10
        {
            println!("  → θ_U が spurious を 20%+ 検出(conjecture を支持)");
        } else if rejection_contribution < 0.05 {
            println!("  → θ_U の検出率 < 5%(conjecture 未支持)");
        }
    }
}
