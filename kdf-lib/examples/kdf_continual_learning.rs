//! KDF Continual Learning
//!
//! KDFで情報を整理し、継続学習に応用する
//!
//! KDFの本質:
//! - Core層 = 知識飽和（冗長情報、新タスクでも再学習されやすい）
//! - Rare層 = 判断材料不足（捨てると判断できなくなる）
//!
//! 応用:
//! - Rare層を優先的にリプレイバッファに保持
//! - 結果として破滅的忘却の軽減が可能であった
//! - 注意: Rare = 重要 ではなく、Rare = 代替不可能

use kdf::Kdf;
use std::collections::HashMap;

fn main() {
    println!("# KDF 継続学習 (Continual Learning)\n");
    println!("KDFで情報を整理し、リプレイバッファ管理に応用する");
    println!("（結果として破滅的忘却の軽減も可能であった）\n");

    // シナリオ1: タスク間での知識保持
    demo_task_knowledge_retention();

    // シナリオ2: メモリ効率的なリプレイバッファ
    demo_memory_efficient_replay();

    // シナリオ3: クラスインクリメンタル学習
    demo_class_incremental();

    println!("\n✅ 継続学習デモ完了");
}

/// タスク間での知識保持
fn demo_task_knowledge_retention() {
    println!("## 1. タスク間知識保持\n");

    let kdf = Kdf::with_defaults();

    // タスク1のデータ
    println!("   【タスク1: 数字認識 (0-4)】");
    let task1_data = generate_task_data(0, 5, 50); // クラス0-4
    let task1_result = kdf.process(&task1_data.features, 0.85, |a, b| {
        euclidean_similarity(a, b)
    });

    println!("   データ: {} 件", task1_data.features.len());
    println!(
        "   Core: {} 件 (一般的パターン)",
        task1_result.core_items().len()
    );
    println!(
        "   Edge: {} 件 (境界パターン)",
        task1_result.edge_items().len()
    );
    println!(
        "   Rare: {} 件 (固有パターン)\n",
        task1_result.rare_items().len()
    );

    // タスク2を学習する前に、タスク1の重要データを選択
    println!("   【タスク2学習前: タスク1からの保持データ選択】");

    // 戦略1: ランダム
    let random_keep = select_random(&task1_data.features, 15);
    let random_rare_kept = count_rare_kept(&random_keep, &task1_result);

    // 戦略2: KDF優先 (Rare > Edge > Core)
    let kdf_keep = select_kdf_priority(&task1_result, 15);
    let kdf_rare_kept = count_rare_kept(&kdf_keep, &task1_result);

    println!("   保持データ: 15件\n");
    println!("   戦略              Rare保持数   期待される忘却防止効果");
    println!("   {}", "-".repeat(55));
    println!(
        "   ランダム          {:>5}        低 (固有パターンを失いやすい)",
        random_rare_kept
    );
    println!(
        "   KDF優先           {:>5}        高 (固有パターンを優先保持)",
        kdf_rare_kept
    );

    println!("\n   → KDF優先戦略はRare層を優先的に保持し、破滅的忘却を軽減\n");
}

/// メモリ効率的なリプレイバッファ
fn demo_memory_efficient_replay() {
    println!("## 2. メモリ効率的リプレイバッファ\n");

    let kdf = Kdf::with_defaults();

    // 複数タスクのデータを蓄積
    let mut all_data: Vec<TaskData> = Vec::new();
    let mut replay_buffer: Vec<(usize, Vec<f64>)> = Vec::new(); // (task_id, features)
    let buffer_limit = 30; // バッファの上限

    let task_names = ["画像分類", "物体検出", "セグメンテーション"];

    for (task_id, task_name) in task_names.iter().enumerate() {
        println!("   【タスク{}: {}】", task_id + 1, task_name);

        // 新タスクのデータ生成
        let task_data = generate_task_data(task_id * 3, 3, 40);
        all_data.push(task_data.clone());

        // 現在のタスクにKDFを適用
        let result = kdf.process(&task_data.features, 0.85, |a, b| euclidean_similarity(a, b));

        // Rare層を優先的にバッファに追加
        let mut new_entries: Vec<(usize, Vec<f64>)> = Vec::new();

        // Rareを全て追加
        for i in result.rare_items().iter() {
            new_entries.push((task_id, task_data.features[*i].clone()));
        }
        // Edgeを追加
        for i in result.edge_items().iter() {
            new_entries.push((task_id, task_data.features[*i].clone()));
        }
        // Coreは余裕があれば
        for i in result.core_items().iter().take(5) {
            new_entries.push((task_id, task_data.features[*i].clone()));
        }

        // バッファに追加
        replay_buffer.extend(new_entries);

        // バッファが上限を超えた場合、古いタスクのCore層から削除
        if replay_buffer.len() > buffer_limit {
            // 古いタスクのデータを優先的に削減 (ただしRareは保持)
            replay_buffer = compress_buffer(&replay_buffer, buffer_limit, &kdf);
        }

        // タスクごとの保持状況を表示
        let task_counts: HashMap<usize, usize> =
            replay_buffer
                .iter()
                .fold(HashMap::new(), |mut acc, (tid, _)| {
                    *acc.entry(*tid).or_insert(0) += 1;
                    acc
                });

        print!("   バッファ状況: ");
        for t in 0..=task_id {
            print!("タスク{}={} ", t + 1, task_counts.get(&t).unwrap_or(&0));
        }
        println!("(合計: {}件)\n", replay_buffer.len());
    }

    println!("   → 各タスクのRare層が優先的に保持され、メモリ効率的に知識を維持\n");
}

/// クラスインクリメンタル学習
fn demo_class_incremental() {
    println!("## 3. クラスインクリメンタル学習\n");

    let kdf = Kdf::with_defaults();

    println!("   シナリオ: 10クラスを2クラスずつ学習\n");

    let mut exemplar_set: Vec<(usize, Vec<f64>)> = Vec::new(); // (class_id, features)
    let exemplar_per_class = 5;

    for phase in 0..5 {
        let class_start = phase * 2;
        let class_end = class_start + 2;

        println!(
            "   【Phase {}: クラス {}-{} を学習】",
            phase + 1,
            class_start,
            class_end - 1
        );

        // 新クラスのデータ
        for class_id in class_start..class_end {
            let class_data = generate_class_data(class_id, 30);
            let result = kdf.process(&class_data, 0.85, |a, b| euclidean_similarity(a, b));

            // 各クラスから代表サンプルを選択 (KDF優先)
            let mut selected = Vec::new();

            // Rareを優先
            for &i in result.rare_items().iter().take(exemplar_per_class) {
                selected.push((class_id, class_data[i].clone()));
            }

            // 残りをEdge/Coreから
            let remaining = exemplar_per_class.saturating_sub(selected.len());
            for &i in result.edge_items().iter().take(remaining) {
                selected.push((class_id, class_data[i].clone()));
            }

            let remaining = exemplar_per_class.saturating_sub(selected.len());
            for &i in result.core_items().iter().take(remaining) {
                selected.push((class_id, class_data[i].clone()));
            }

            exemplar_set.extend(selected);
        }

        // 現在の exemplar set の状況
        let class_counts: HashMap<usize, usize> =
            exemplar_set
                .iter()
                .fold(HashMap::new(), |mut acc, (cid, _)| {
                    *acc.entry(*cid).or_insert(0) += 1;
                    acc
                });

        println!("   Exemplar Set: {:?}", class_counts);
    }

    // 最終的なカバレッジ分析
    println!("\n   【最終分析】");
    println!("   総Exemplar数: {} 件", exemplar_set.len());
    println!("   クラスあたり: {} 件", exemplar_per_class);
    println!("\n   KDF選択の利点:");
    println!("   - 各クラスの固有パターン (Rare) を優先保持");
    println!("   - 境界ケース (Edge) も含めてクラス特性を表現");
    println!("   - 冗長なサンプル (Core) は最小限に抑制");
}

// ============================================================================
// ヘルパー構造体・関数
// ============================================================================

#[derive(Clone)]
#[allow(dead_code)]
struct TaskData {
    features: Vec<Vec<f64>>,
    labels: Vec<usize>,
}

/// タスクデータの生成
fn generate_task_data(
    class_start: usize,
    num_classes: usize,
    samples_per_class: usize,
) -> TaskData {
    let mut features = Vec::new();
    let mut labels = Vec::new();

    for class_id in class_start..(class_start + num_classes) {
        let class_data = generate_class_data(class_id, samples_per_class);
        for f in class_data {
            features.push(f);
            labels.push(class_id);
        }
    }

    TaskData { features, labels }
}

/// クラスデータの生成
fn generate_class_data(class_id: usize, n: usize) -> Vec<Vec<f64>> {
    let center_x = (class_id % 5) as f64 * 2.0;
    let center_y = (class_id / 5) as f64 * 2.0;

    let mut data = Vec::new();

    // 一般的なサンプル (Core向け)
    for _i in 0..(n * 7 / 10) {
        let x = center_x + (rand_f64() - 0.5) * 0.5;
        let y = center_y + (rand_f64() - 0.5) * 0.5;
        data.push(vec![x, y]);
    }

    // 境界サンプル (Edge向け)
    for i in 0..(n * 2 / 10) {
        let angle = i as f64 * 0.5;
        let x = center_x + angle.cos() * 0.8;
        let y = center_y + angle.sin() * 0.8;
        data.push(vec![x, y]);
    }

    // 固有サンプル (Rare向け)
    for i in 0..(n / 10 + 1) {
        let x = center_x + (i as f64 - 1.0) * 2.0;
        let y = center_y + 3.0;
        data.push(vec![x, y]);
    }

    data
}

/// ランダム選択
fn select_random(data: &[Vec<f64>], n: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..data.len()).collect();
    let mut seed = 42u64;

    // Fisher-Yates shuffle
    for i in (1..indices.len()).rev() {
        seed = seed.wrapping_mul(1103515245).wrapping_add(12345);
        let j = (seed as usize) % (i + 1);
        indices.swap(i, j);
    }

    indices.into_iter().take(n).collect()
}

/// KDF優先選択 (Rare > Edge > Core)
fn select_kdf_priority(result: &kdf::KdfResult, n: usize) -> Vec<usize> {
    let mut selected = Vec::new();

    // Rare優先
    selected.extend(result.rare_items().iter().take(n));

    // Edge追加
    let remaining = n.saturating_sub(selected.len());
    selected.extend(result.edge_items().iter().take(remaining));

    // Core追加
    let remaining = n.saturating_sub(selected.len());
    selected.extend(result.core_items().iter().take(remaining));

    selected.into_iter().take(n).collect()
}

/// Rare保持数をカウント
fn count_rare_kept(kept: &[usize], result: &kdf::KdfResult) -> usize {
    let rare_items = result.rare_items();
    let rare_set: std::collections::HashSet<_> = rare_items.iter().collect();
    kept.iter().filter(|i| rare_set.contains(i)).count()
}

/// バッファ圧縮 (Rare優先保持)
fn compress_buffer(
    buffer: &[(usize, Vec<f64>)],
    limit: usize,
    kdf: &Kdf,
) -> Vec<(usize, Vec<f64>)> {
    if buffer.len() <= limit {
        return buffer.to_vec();
    }

    // タスクごとにグループ化
    let mut by_task: HashMap<usize, Vec<Vec<f64>>> = HashMap::new();
    for (tid, feat) in buffer {
        by_task.entry(*tid).or_default().push(feat.clone());
    }

    let mut new_buffer = Vec::new();
    let per_task = limit / by_task.len().max(1);

    for (tid, features) in by_task {
        if features.len() <= per_task {
            for f in features {
                new_buffer.push((tid, f));
            }
        } else {
            // KDFで選択
            let result = kdf.process(&features, 0.85, |a, b| euclidean_similarity(a, b));
            let selected = select_kdf_priority(&result, per_task);
            for i in selected {
                new_buffer.push((tid, features[i].clone()));
            }
        }
    }

    new_buffer
}

/// 簡易乱数生成
fn rand_f64() -> f64 {
    use std::time::SystemTime;
    static mut SEED: u64 = 0;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        if SEED == 0 {
            SEED = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64;
        }
        (SEED as f64) / (u64::MAX as f64)
    }
}

/// ユークリッド類似度
fn euclidean_similarity(a: &[f64], b: &[f64]) -> f64 {
    let dist: f64 = a
        .iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    1.0 / (1.0 + dist)
}
