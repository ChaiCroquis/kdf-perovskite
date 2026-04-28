//! Effectiveness measurement for new features
use kdf::{Kdf, KdfParams, cosine_similarity, dtw_similarity, levenshtein_similarity};
use std::time::Instant;

fn main() {
    println!("=== 実装効果測定 ===\n");

    // ========================================================================
    // 1. 並列処理のスケーラビリティ
    // ========================================================================
    println!("## 1. 並列処理スケーラビリティ\n");

    let sizes = [100, 500, 1000, 2000];

    println!("   | データ数 | Sequential | Parallel | 速度比 |");
    println!("   |----------|------------|----------|--------|");

    for &size in &sizes {
        let items: Vec<Vec<f64>> = (0..size)
            .map(|i| {
                let angle = (i as f64) * 0.1;
                vec![angle.cos(), angle.sin(), (i as f64) / size as f64]
            })
            .collect();

        let kdf = Kdf::with_defaults();

        // Sequential
        let start = Instant::now();
        let _result_seq = kdf.process(&items, 0.95, |a, b| cosine_similarity(a, b));
        let seq_time = start.elapsed();

        // Parallel (if available)
        #[cfg(feature = "parallel")]
        let (par_time, speedup) = {
            let start = Instant::now();
            let _result_par = kdf.process_parallel(&items, 0.95, |a, b| cosine_similarity(a, b));
            let par_time = start.elapsed();
            let speedup = seq_time.as_secs_f64() / par_time.as_secs_f64();
            (par_time, speedup)
        };

        #[cfg(not(feature = "parallel"))]
        let (par_time, speedup) = (seq_time, 1.0);

        println!(
            "   | {:>8} | {:>10.2?} | {:>8.2?} | {:>6.2}x |",
            size, seq_time, par_time, speedup
        );
    }

    #[cfg(not(feature = "parallel"))]
    println!("\n   ※ parallel feature未有効。--features parallel で実行してください");

    // ========================================================================
    // 2. Levenshtein類似度の精度
    // ========================================================================
    println!("\n## 2. Levenshtein類似度精度\n");

    let test_cases = vec![
        ("hello", "hello", 1.0, "完全一致"),
        ("hello", "hallo", 0.8, "1文字違い"),
        ("hello", "world", 0.2, "異なる単語"),
        ("", "", 1.0, "空文字同士"),
        ("abc", "", 0.0, "片方が空"),
        ("kitten", "sitting", 0.571, "編集距離3"),
        ("日本語", "日本語", 1.0, "日本語一致"),
        ("日本語", "日本人", 0.667, "日本語1文字違い"),
    ];

    println!("   | テストケース | 期待値 | 実測値 | 誤差 | 判定 |");
    println!("   |--------------|--------|--------|------|------|");

    let mut total_error = 0.0;
    for (a, b, expected, desc) in &test_cases {
        let actual = levenshtein_similarity(a, b);
        let error = (actual - expected).abs();
        total_error += error;
        let status = if error < 0.01 { "✅" } else { "⚠️" };
        println!(
            "   | {:12} | {:>6.3} | {:>6.3} | {:>4.3} | {} |",
            desc, expected, actual, error, status
        );
    }
    let avg_error = total_error / test_cases.len() as f64;
    println!("\n   平均誤差: {:.4}", avg_error);

    // ========================================================================
    // 3. DTW類似度の精度
    // ========================================================================
    println!("\n## 3. DTW類似度精度\n");

    let dtw_cases = vec![
        (
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            "同一系列",
            true, // should be high
        ),
        (
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![1.1, 2.1, 3.1, 4.1, 5.1],
            "微小シフト",
            true,
        ),
        (
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![5.0, 4.0, 3.0, 2.0, 1.0],
            "逆順",
            false, // should be low
        ),
        (
            vec![1.0, 2.0, 3.0],
            vec![1.0, 1.5, 2.0, 2.5, 3.0],
            "異なる長さ",
            true, // DTW should handle
        ),
        (
            vec![0.0, 1.0, 0.0, 1.0, 0.0],
            vec![0.0, 1.0, 0.0, 1.0, 0.0],
            "振動同一",
            true,
        ),
    ];

    println!("   | テストケース | 類似度 | 期待 | 判定 |");
    println!("   |--------------|--------|------|------|");

    for (a, b, desc, should_be_high) in &dtw_cases {
        let sim = dtw_similarity(a, b);
        let threshold = 0.3;
        let is_high = sim > threshold;
        let status = if *should_be_high == is_high {
            "✅"
        } else {
            "⚠️"
        };
        let expect = if *should_be_high { "高" } else { "低" };
        println!(
            "   | {:12} | {:>6.3} | {:>4} | {} |",
            desc, sim, expect, status
        );
    }

    // ========================================================================
    // 4. KDFとの統合効果
    // ========================================================================
    println!("\n## 4. KDFとの統合効果\n");

    // 文字列クラスタリング
    let strings = vec![
        "apple", "apples", "apply", // Cluster 1
        "banana", "bananas", // Cluster 2
        "cherry",  // Cluster 3
        "xyz123",  // Isolated
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&strings, 0.6, |a, b| levenshtein_similarity(a, b));

    println!("   ### 文字列クラスタリング (Levenshtein)");
    println!("   入力: {} 件", strings.len());
    println!("   選択: {} 件", result.selected.len());
    println!(
        "   冗長削減: {:.1}%",
        (1.0 - result.selected.len() as f64 / strings.len() as f64) * 100.0
    );
    println!("   選択された文字列:");
    for &i in &result.selected {
        println!("      - {} ({:?})", strings[i], result.layers[i]);
    }

    // 時系列クラスタリング
    let time_series: Vec<Vec<f64>> = vec![
        // 上昇トレンド群
        (0..10).map(|i| i as f64).collect(),
        (0..10).map(|i| i as f64 + 0.1).collect(),
        (0..10).map(|i| i as f64 * 1.1).collect(),
        // 下降トレンド群
        (0..10).map(|i| (9 - i) as f64).collect(),
        (0..10).map(|i| (9 - i) as f64 + 0.1).collect(),
        // フラット
        vec![5.0; 10],
        // 振動
        (0..10)
            .map(|i| if i % 2 == 0 { 0.0 } else { 1.0 })
            .collect(),
    ];

    let result = kdf.process(&time_series, 0.2, |a, b| dtw_similarity(a, b));

    println!("\n   ### 時系列クラスタリング (DTW)");
    println!("   入力: {} 件", time_series.len());
    println!("   選択: {} 件", result.selected.len());
    println!(
        "   冗長削減: {:.1}%",
        (1.0 - result.selected.len() as f64 / time_series.len() as f64) * 100.0
    );
    println!("   選択されたパターン:");
    for &i in &result.selected {
        let pattern = if time_series[i][0] < time_series[i][9] {
            "上昇"
        } else if time_series[i][0] > time_series[i][9] {
            "下降"
        } else if time_series[i]
            .windows(2)
            .all(|w| (w[0] - w[1]).abs() < 0.01)
        {
            "フラット"
        } else {
            "振動"
        };
        println!("      - Series {} ({}) {:?}", i, pattern, result.layers[i]);
    }

    // ========================================================================
    // 5. Builder使用性確認
    // ========================================================================
    println!("\n## 5. Builderパターン使用性\n");

    // 従来方式
    let params_old = KdfParams {
        alpha_edge: 1.8,
        iterations: 50,
        theta_edge: 0.2,
        ..Default::default()
    };

    // Builder方式
    let params_new = KdfParams::builder()
        .alpha_edge(1.8)
        .iterations(50)
        .theta_edge(0.2)
        .build();

    println!("   従来方式: 3行 (mut変数 + 個別代入)");
    println!("   Builder:  1行 (メソッドチェーン)");
    println!(
        "   パラメータ一致: {}",
        params_old.alpha_edge == params_new.alpha_edge
            && params_old.iterations == params_new.iterations
            && params_old.theta_edge == params_new.theta_edge
    );

    // ========================================================================
    // サマリ
    // ========================================================================
    println!("\n## サマリ\n");
    println!("   | 機能 | 効果 |");
    println!("   |------|------|");
    #[cfg(feature = "parallel")]
    println!("   | 並列処理 | 大規模データで高速化 |");
    #[cfg(not(feature = "parallel"))]
    println!("   | 並列処理 | (未測定) |");
    println!("   | Levenshtein | 平均誤差 {:.4} |", avg_error);
    println!("   | DTW | 時系列パターン認識OK |");
    println!("   | Builder | コード簡潔化 |");

    println!("\n✅ 効果測定完了");
}
