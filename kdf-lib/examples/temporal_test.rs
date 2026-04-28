//! Temporal KDF demonstration - time-aware data processing
use kdf::{cosine_similarity, KdfParams, TemporalKdf, TemporalParams};

fn main() {
    println!("=== Temporal KDF Test ===\n");

    // Simulate log entries with timestamps
    // Newer entries have higher timestamps
    let items = vec![
        vec![1.0, 0.0, 0.0],  // Old entry A
        vec![1.0, 0.1, 0.0],  // Old entry A' (similar)
        vec![0.0, 1.0, 0.0],  // Old entry B
        vec![1.0, 0.0, 0.0],  // New entry A (same as old)
        vec![0.0, 0.0, 1.0],  // New entry C (rare)
        vec![-1.0, 0.0, 0.0], // Very old rare entry
    ];

    // Timestamps: 0=oldest, 100=newest
    let timestamps = vec![
        10.0, // Old
        15.0, // Old
        20.0, // Old
        90.0, // New
        95.0, // New
        5.0,  // Very old
    ];

    // ========================================================================
    // 1. Standard KDF (time-unaware)
    // ========================================================================
    println!("## 1. Standard KDF (時間考慮なし)\n");

    let kdf = kdf::Kdf::with_defaults();
    let result = kdf.process(&items, 0.9, |a, b| cosine_similarity(a, b));

    println!("   Selected: {:?}", result.selected);
    for &i in &result.selected {
        println!(
            "      Item {}: timestamp={}, layer={:?}",
            i, timestamps[i], result.layers[i]
        );
    }

    // ========================================================================
    // 2. Temporal KDF (time-aware)
    // ========================================================================
    println!("\n## 2. Temporal KDF (時間減衰あり)\n");

    let temporal_params = TemporalParams {
        decay_rate: 0.05,      // 5% decay per time unit
        reference_time: 100.0, // Current time
        min_weight: 0.1,
    };

    let temporal_kdf = TemporalKdf::new(KdfParams::default(), temporal_params.clone());
    let result = temporal_kdf.process(&items, &timestamps, 0.9, |a, b| cosine_similarity(a, b));

    println!("   Selected: {:?}", result.selected);
    for &i in &result.selected {
        let temporal_weight = temporal_params.temporal_weight(timestamps[i]);
        println!(
            "      Item {}: timestamp={}, temporal_weight={:.3}, final_weight={:.3}, layer={:?}",
            i, timestamps[i], temporal_weight, result.selection_scores[i], result.layers[i]
        );
    }

    // ========================================================================
    // 3. Temporal weight demonstration
    // ========================================================================
    println!("\n## 3. 時間減衰の効果\n");

    println!("   | Age | Temporal Weight |");
    println!("   |-----|-----------------|");
    for age in [0, 10, 20, 50, 100].iter() {
        let ts = temporal_params.reference_time - *age as f64;
        let weight = temporal_params.temporal_weight(ts);
        println!("   | {:>3} | {:.3}           |", age, weight);
    }

    // ========================================================================
    // 4. Fresh vs Stale items
    // ========================================================================
    println!("\n## 4. Fresh / Stale アイテム分類\n");

    let fresh = temporal_kdf.fresh_items(
        &items,
        &timestamps,
        0.9,
        |a, b| cosine_similarity(a, b),
        0.5,
    );
    let stale_rare = temporal_kdf.stale_rare_items(
        &items,
        &timestamps,
        0.9,
        |a, b| cosine_similarity(a, b),
        0.3,
    );

    println!("   Fresh items (temporal_weight >= 0.5): {:?}", fresh);
    println!(
        "   Stale rare items (rare & weight < 0.3): {:?}",
        stale_rare
    );

    // ========================================================================
    // 5. Use case: Log deduplication with time priority
    // ========================================================================
    println!("\n## 5. ユースケース: ログ重複排除\n");

    let log_entries = vec![
        "ERROR: Connection refused",
        "ERROR: Connection refused",
        "ERROR: Connection refused",
        "ERROR: Connection refused",
        "WARNING: Disk space low",
        "ERROR: Out of memory", // Rare
    ];

    let log_timestamps = vec![
        10.0, // Old error
        20.0, // Old error (duplicate)
        30.0, // Old error (duplicate)
        90.0, // Recent error (same message but fresh)
        85.0, // Recent warning
        15.0, // Old but rare
    ];

    let temporal_kdf = TemporalKdf::new(
        KdfParams::default(),
        TemporalParams {
            decay_rate: 0.03,
            reference_time: 100.0,
            min_weight: 0.1,
        },
    );

    let result = temporal_kdf.process(&log_entries, &log_timestamps, 0.9, |a, b| {
        kdf::levenshtein_similarity(a, b)
    });

    println!("   Input logs: {} entries", log_entries.len());
    println!("   Selected: {} entries", result.selected.len());
    println!(
        "   冗長削減: {:.1}%\n",
        (1.0 - result.selected.len() as f64 / log_entries.len() as f64) * 100.0
    );

    println!("   Selected log entries:");
    for &i in &result.selected {
        let tw = TemporalParams {
            decay_rate: 0.03,
            reference_time: 100.0,
            min_weight: 0.1,
        }
        .temporal_weight(log_timestamps[i]);
        println!(
            "      [t={:>2}] (tw={:.2}) {:?} - {:?}",
            log_timestamps[i] as i32, tw, result.layers[i], log_entries[i]
        );
    }

    println!("\n✅ Temporal KDF 正常動作");
}
