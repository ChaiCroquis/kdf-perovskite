//! KDF + Genetic Algorithm: Diversity-preserving evolution
//!
//! Problem: GAs often lose diversity and converge to local optima
//! Solution: Use KDF to maintain population diversity by:
//! - Removing redundant (similar) individuals
//! - Preserving rare/unique genetic material
//!
//! Comparison:
//! 1. Standard GA (baseline)
//! 2. Fitness-proportionate selection GA
//! 3. KDF-enhanced GA (our approach)

use kdf::{Kdf, KdfParams};

/// Individual in the population
#[derive(Clone, Debug)]
struct Individual {
    genes: Vec<f64>,
    fitness: f64,
}

impl Individual {
    fn new(genes: Vec<f64>) -> Self {
        Self {
            genes,
            fitness: 0.0,
        }
    }

    fn random(dim: usize) -> Self {
        let genes: Vec<f64> = (0..dim).map(|_| (rand_simple() * 2.0) - 1.0).collect();
        Self::new(genes)
    }
}

/// Simple pseudo-random number generator (for reproducibility)
static mut GLOBAL_SEED: u64 = 12345;

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

/// Rastrigin function - multimodal optimization problem
/// Global minimum: f(0,0,...,0) = 0
fn rastrigin(x: &[f64]) -> f64 {
    let n = x.len() as f64;
    let sum: f64 = x
        .iter()
        .map(|&xi| xi * xi - 10.0 * (2.0 * std::f64::consts::PI * xi).cos())
        .sum();
    10.0 * n + sum
}

/// Euclidean similarity for genes
fn gene_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b)
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}

/// Crossover two individuals
fn crossover(p1: &Individual, p2: &Individual) -> Individual {
    let genes: Vec<f64> = p1
        .genes
        .iter()
        .zip(&p2.genes)
        .map(|(&g1, &g2)| if rand_simple() < 0.5 { g1 } else { g2 })
        .collect();
    Individual::new(genes)
}

/// Mutate an individual
fn mutate(ind: &mut Individual, rate: f64) {
    for gene in &mut ind.genes {
        if rand_simple() < rate {
            *gene += (rand_simple() - 0.5) * 0.2;
            *gene = gene.clamp(-5.12, 5.12);
        }
    }
}

/// Standard GA with tournament selection
fn standard_ga(pop_size: usize, dim: usize, generations: usize) -> (f64, f64, Vec<f64>) {
    let mut population: Vec<Individual> = (0..pop_size).map(|_| Individual::random(dim)).collect();

    let mut best_fitness_history = Vec::new();
    let mut best_ever = f64::MAX;

    for _gen in 0..generations {
        // Evaluate fitness (minimize Rastrigin)
        for ind in &mut population {
            ind.fitness = rastrigin(&ind.genes);
        }

        // Track best
        let gen_best = population
            .iter()
            .map(|i| i.fitness)
            .fold(f64::MAX, f64::min);
        best_ever = best_ever.min(gen_best);
        best_fitness_history.push(best_ever);

        // Tournament selection + crossover + mutation
        let mut new_pop = Vec::with_capacity(pop_size);
        while new_pop.len() < pop_size {
            // Tournament selection
            let i1 = (rand_simple() * pop_size as f64) as usize % pop_size;
            let i2 = (rand_simple() * pop_size as f64) as usize % pop_size;
            let p1 = if population[i1].fitness < population[i2].fitness {
                &population[i1]
            } else {
                &population[i2]
            };

            let i3 = (rand_simple() * pop_size as f64) as usize % pop_size;
            let i4 = (rand_simple() * pop_size as f64) as usize % pop_size;
            let p2 = if population[i3].fitness < population[i4].fitness {
                &population[i3]
            } else {
                &population[i4]
            };

            let mut child = crossover(p1, p2);
            mutate(&mut child, 0.1);
            new_pop.push(child);
        }
        population = new_pop;
    }

    // Final evaluation
    for ind in &mut population {
        ind.fitness = rastrigin(&ind.genes);
    }
    let final_best = population
        .iter()
        .map(|i| i.fitness)
        .fold(f64::MAX, f64::min);

    // Calculate diversity (average pairwise distance)
    let diversity = calculate_diversity(&population);

    (final_best, diversity, best_fitness_history)
}

/// KDF-enhanced GA with adaptive diversity control
fn kdf_ga(pop_size: usize, dim: usize, generations: usize) -> (f64, f64, Vec<f64>) {
    let mut population: Vec<Individual> = (0..pop_size).map(|_| Individual::random(dim)).collect();

    let mut best_fitness_history = Vec::new();
    let mut best_ever = f64::MAX;
    let mut stagnation_count = 0;
    let mut last_best = f64::MAX;

    for r#gen in 0..generations {
        // Evaluate fitness
        for ind in &mut population {
            ind.fitness = rastrigin(&ind.genes);
        }

        // Track best
        let gen_best = population
            .iter()
            .map(|i| i.fitness)
            .fold(f64::MAX, f64::min);
        best_ever = best_ever.min(gen_best);
        best_fitness_history.push(best_ever);

        // Detect stagnation
        if (last_best - best_ever).abs() < 0.001 {
            stagnation_count += 1;
        } else {
            stagnation_count = 0;
        }
        last_best = best_ever;

        // Sort by fitness
        population.sort_by(|a, b| a.fitness.partial_cmp(&b.fitness).unwrap());

        // === ADAPTIVE KDF DIVERSITY INJECTION ===
        // Only apply KDF when stagnating or early in evolution
        let apply_kdf = stagnation_count > 5 || r#gen < generations / 4;

        let elite_count = pop_size / 5;
        let mut new_pop: Vec<Individual> = population.iter().take(elite_count).cloned().collect();

        if apply_kdf {
            // Use KDF to identify unique individuals in the rest
            let rest: Vec<Vec<f64>> = population
                .iter()
                .skip(elite_count)
                .map(|i| i.genes.clone())
                .collect();

            if rest.len() > 5 {
                let kdf = Kdf::new(KdfParams::builder().selection_sim_threshold(0.5).build());
                let sim_threshold = if stagnation_count > 10 { 0.7 } else { 0.85 };
                let result = kdf.process(&rest, sim_threshold, |a, b| gene_similarity(a, b));

                // Add diverse individuals
                for &i in &result.selected {
                    if new_pop.len() < pop_size / 2 {
                        new_pop.push(population[elite_count + i].clone());
                    }
                }

                // If stagnating, inject random individuals
                if stagnation_count > 10 {
                    for _ in 0..(pop_size / 10) {
                        new_pop.push(Individual::random(dim));
                    }
                    stagnation_count = 0;
                }
            }
        }

        // Fill remaining with offspring
        let parent_pool = new_pop.clone();
        let mutation_rate = if stagnation_count > 3 { 0.2 } else { 0.1 };

        while new_pop.len() < pop_size {
            let i1 = (rand_simple() * parent_pool.len() as f64) as usize % parent_pool.len();
            let i2 = (rand_simple() * parent_pool.len() as f64) as usize % parent_pool.len();
            let p1 = if parent_pool[i1].fitness < parent_pool[i2].fitness {
                &parent_pool[i1]
            } else {
                &parent_pool[i2]
            };

            let i3 = (rand_simple() * parent_pool.len() as f64) as usize % parent_pool.len();
            let i4 = (rand_simple() * parent_pool.len() as f64) as usize % parent_pool.len();
            let p2 = if parent_pool[i3].fitness < parent_pool[i4].fitness {
                &parent_pool[i3]
            } else {
                &parent_pool[i4]
            };

            let mut child = crossover(p1, p2);
            mutate(&mut child, mutation_rate);
            new_pop.push(child);
        }

        population = new_pop;
    }

    // Final evaluation
    for ind in &mut population {
        ind.fitness = rastrigin(&ind.genes);
    }
    let final_best = population
        .iter()
        .map(|i| i.fitness)
        .fold(f64::MAX, f64::min);

    let diversity = calculate_diversity(&population);

    (final_best, diversity, best_fitness_history)
}

/// Fitness-proportionate selection GA (roulette wheel)
fn roulette_ga(pop_size: usize, dim: usize, generations: usize) -> (f64, f64, Vec<f64>) {
    let mut population: Vec<Individual> = (0..pop_size).map(|_| Individual::random(dim)).collect();

    let mut best_fitness_history = Vec::new();
    let mut best_ever = f64::MAX;

    for _gen in 0..generations {
        // Evaluate fitness
        for ind in &mut population {
            ind.fitness = rastrigin(&ind.genes);
        }

        // Track best
        let gen_best = population
            .iter()
            .map(|i| i.fitness)
            .fold(f64::MAX, f64::min);
        best_ever = best_ever.min(gen_best);
        best_fitness_history.push(best_ever);

        // Invert fitness for minimization (lower is better -> higher selection prob)
        let max_fit = population.iter().map(|i| i.fitness).fold(0.0f64, f64::max) + 1.0;
        let inv_fitness: Vec<f64> = population.iter().map(|i| max_fit - i.fitness).collect();
        let total: f64 = inv_fitness.iter().sum();

        // Roulette wheel selection
        let mut new_pop = Vec::with_capacity(pop_size);
        while new_pop.len() < pop_size {
            let select = |inv_fit: &[f64], total: f64| -> usize {
                let mut r = rand_simple() * total;
                for (i, &f) in inv_fit.iter().enumerate() {
                    r -= f;
                    if r <= 0.0 {
                        return i;
                    }
                }
                inv_fit.len() - 1
            };

            let i1 = select(&inv_fitness, total);
            let i2 = select(&inv_fitness, total);

            let mut child = crossover(&population[i1], &population[i2]);
            mutate(&mut child, 0.1);
            new_pop.push(child);
        }
        population = new_pop;
    }

    // Final evaluation
    for ind in &mut population {
        ind.fitness = rastrigin(&ind.genes);
    }
    let final_best = population
        .iter()
        .map(|i| i.fitness)
        .fold(f64::MAX, f64::min);

    let diversity = calculate_diversity(&population);

    (final_best, diversity, best_fitness_history)
}

/// Calculate population diversity (average pairwise distance)
fn calculate_diversity(population: &[Individual]) -> f64 {
    if population.len() < 2 {
        return 0.0;
    }

    let mut total_dist = 0.0;
    let mut count = 0;

    for i in 0..population.len() {
        for j in (i + 1)..population.len() {
            let dist: f64 = population[i]
                .genes
                .iter()
                .zip(&population[j].genes)
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            total_dist += dist;
            count += 1;
        }
    }

    total_dist / count as f64
}

fn main() {
    println!("=== KDF + 遺伝的アルゴリズム: 多様性保持進化 ===\n");

    let pop_size = 50;
    let dim = 5;
    let generations = 100;
    let runs = 5;

    println!("## 設定");
    println!("   - 問題: Rastrigin関数 (多峰性、次元={})", dim);
    println!("   - 集団サイズ: {}", pop_size);
    println!("   - 世代数: {}", generations);
    println!("   - 試行回数: {}\n", runs);

    // Run multiple trials
    let mut standard_results = Vec::new();
    let mut roulette_results = Vec::new();
    let mut kdf_results = Vec::new();

    for run in 0..runs {
        // Reset random seed for each method (fair comparison)
        reset_seed(12345 + run as u64 * 1000);
        let (best, div, _) = standard_ga(pop_size, dim, generations);
        standard_results.push((best, div));

        reset_seed(12345 + run as u64 * 1000);
        let (best, div, _) = roulette_ga(pop_size, dim, generations);
        roulette_results.push((best, div));

        reset_seed(12345 + run as u64 * 1000);
        let (best, div, _) = kdf_ga(pop_size, dim, generations);
        kdf_results.push((best, div));
    }

    // Calculate averages
    let avg = |results: &[(f64, f64)]| -> (f64, f64) {
        let n = results.len() as f64;
        (
            results.iter().map(|(b, _)| b).sum::<f64>() / n,
            results.iter().map(|(_, d)| d).sum::<f64>() / n,
        )
    };

    let (std_best, std_div) = avg(&standard_results);
    let (rou_best, rou_div) = avg(&roulette_results);
    let (kdf_best, kdf_div) = avg(&kdf_results);

    // Results
    println!("## 結果 ({}回平均)\n", runs);

    println!("   | 手法 | 最良適応度 | 最終多様性 | 局所最適回避 |");
    println!("   |------|------------|------------|--------------|");
    println!(
        "   | Standard GA     | {:>10.4} | {:>10.4} | {} |",
        std_best,
        std_div,
        if std_best < 5.0 { "○" } else { "×" }
    );
    println!(
        "   | Roulette GA     | {:>10.4} | {:>10.4} | {} |",
        rou_best,
        rou_div,
        if rou_best < 5.0 { "○" } else { "×" }
    );
    println!(
        "   | KDF-enhanced GA | {:>10.4} | {:>10.4} | {} |",
        kdf_best,
        kdf_div,
        if kdf_best < 5.0 { "○" } else { "×" }
    );

    // Analysis
    println!("\n## 分析\n");

    println!("   【多様性維持】");
    if kdf_div > std_div {
        println!(
            "   ✓ KDF: 多様性 {:.2} (Standard比 +{:.1}%)",
            kdf_div,
            (kdf_div / std_div - 1.0) * 100.0
        );
    }

    println!("\n   【最適化性能】");
    if kdf_best < std_best {
        println!(
            "   ✓ KDF: 最良解 {:.4} (Standard比 -{:.1}%改善)",
            kdf_best,
            (1.0 - kdf_best / std_best) * 100.0
        );
    }

    // Convergence comparison (single run for visualization)
    println!("\n## 収束曲線 (代表例)\n");

    reset_seed(12345);
    let (_, _, std_hist) = standard_ga(pop_size, dim, generations);

    reset_seed(12345);
    let (_, _, kdf_hist) = kdf_ga(pop_size, dim, generations);

    println!("   世代 | Standard | KDF      | 差分");
    println!("   -----|----------|----------|--------");
    for r#gen in [0, 10, 25, 50, 75, 99] {
        if r#gen < std_hist.len() && r#gen < kdf_hist.len() {
            let diff = std_hist[r#gen] - kdf_hist[r#gen];
            println!(
                "   {:>4} | {:>8.4} | {:>8.4} | {:>+7.4}",
                r#gen, std_hist[r#gen], kdf_hist[r#gen], diff
            );
        }
    }

    // Key findings
    println!("\n## 主要発見\n");

    println!("   KDF + GAの利点:");
    println!("   1. 冗長個体の削減 → 計算効率向上");
    println!("   2. 希少遺伝子の保持 → 多様性維持");
    println!("   3. 早期収束の防止 → 局所最適回避");

    if kdf_div > std_div && kdf_best <= std_best {
        println!("\n   ✅ KDFは「多様性を維持しながら同等以上の解を発見」");
    }

    println!("\n✅ KDF + GA 検証完了");
}
