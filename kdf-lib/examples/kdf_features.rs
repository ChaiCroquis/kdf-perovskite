//! KDF-specific features demonstration: anomaly scoring, diversity sampling, statistics
use kdf::{Kdf, cosine_similarity};

fn main() {
    println!("=== KDF-Specific Features Test ===\n");

    // Create test dataset: 3 clusters + 2 isolated points
    let items = vec![
        // Cluster 1: similar items
        vec![1.0, 0.0, 0.0],
        vec![0.98, 0.1, 0.0],
        vec![0.95, 0.15, 0.0],
        // Cluster 2: similar items
        vec![0.0, 1.0, 0.0],
        vec![0.1, 0.98, 0.0],
        vec![0.15, 0.95, 0.0],
        // Cluster 3: similar items
        vec![0.0, 0.0, 1.0],
        vec![0.0, 0.1, 0.98],
        // Isolated points (anomalies)
        vec![-1.0, -1.0, 0.0],  // Very different
        vec![0.5, 0.5, 0.5],    // Between all clusters
    ];

    let kdf = Kdf::with_defaults();
    let result = kdf.process(&items, 0.85, |a, b| cosine_similarity(a, b));

    // ========================================================================
    // 1. Anomaly Scoring
    // ========================================================================
    println!("## 1. Anomaly Scoring");
    println!("   (0.0 = normal, 1.0 = highly anomalous)\n");

    for i in 0..items.len() {
        let score = result.anomaly_score(i);
        let layer = result.layers[i];
        println!("   Item {:2}: score={:.3} layer={:?}", i, score, layer);
    }

    // Top anomalies
    println!("\n   Top 3 anomalies:");
    for (idx, score) in result.top_anomalies(3) {
        println!("      Item {}: score={:.3}", idx, score);
    }

    // Anomalies above threshold
    println!("\n   Anomalies above 0.5:");
    for (idx, score) in result.anomalies_above(0.5) {
        println!("      Item {}: score={:.3}", idx, score);
    }

    // ========================================================================
    // 2. Diversity Sampling
    // ========================================================================
    println!("\n## 2. Diversity Sampling");

    let diverse_5 = kdf.diverse_sample(&items, 5, |a, b| cosine_similarity(a, b));
    println!("   Select 5 diverse items: {:?}", diverse_5);

    // Verify diversity: these should be spread across different clusters
    println!("   Selected item details:");
    for &idx in &diverse_5 {
        let vec = &items[idx];
        let layer = result.layers[idx];
        println!("      Item {:2}: {:?} ({:?})", idx, vec, layer);
    }

    let diverse_3 = kdf.diverse_sample(&items, 3, |a, b| cosine_similarity(a, b));
    println!("\n   Select 3 diverse items: {:?}", diverse_3);

    // ========================================================================
    // 3. Statistics
    // ========================================================================
    println!("\n## 3. Statistics");

    let stats = result.stats();
    println!("   Total items:      {}", stats.total_items);
    println!("   Selected count:   {}", stats.selected_count);
    println!("   Layer counts:");
    for (layer, count) in &stats.layer_counts {
        println!("      {:?}: {}", layer, count);
    }
    println!("   Avg degree:       {:.2}", stats.avg_degree);
    println!("   Max degree:       {}", stats.max_degree);
    println!("   Cluster count:    {}", stats.cluster_count);
    println!("   Avg cluster size: {:.2}", stats.avg_cluster_size);
    println!("   Max cluster size: {}", stats.max_cluster_size);
    println!("   Isolation ratio:  {:.2}%", stats.isolation_ratio * 100.0);
    println!("   Redundancy ratio: {:.2}%", stats.redundancy_ratio * 100.0);

    // ========================================================================
    // Verification
    // ========================================================================
    println!("\n## Verification");

    // Anomaly detection: isolated items should have high scores
    let isolated_scores: Vec<f64> = vec![8, 9].iter()
        .map(|&i| result.anomaly_score(i))
        .collect();
    let cluster_scores: Vec<f64> = vec![0, 3, 6].iter()
        .map(|&i| result.anomaly_score(i))
        .collect();

    let avg_isolated = isolated_scores.iter().sum::<f64>() / isolated_scores.len() as f64;
    let avg_cluster = cluster_scores.iter().sum::<f64>() / cluster_scores.len() as f64;

    println!("   Avg anomaly score (isolated items): {:.3}", avg_isolated);
    println!("   Avg anomaly score (cluster items):  {:.3}", avg_cluster);

    if avg_isolated > avg_cluster {
        println!("   ✓ Isolated items correctly identified as more anomalous");
    } else {
        println!("   ✗ Anomaly scoring needs adjustment");
    }

    // Diversity sampling: should include items from different clusters
    let mut cluster_coverage = 0;
    for &idx in &diverse_5 {
        if idx <= 2 { cluster_coverage |= 1; }       // Cluster 1
        else if idx <= 5 { cluster_coverage |= 2; }  // Cluster 2
        else if idx <= 7 { cluster_coverage |= 4; }  // Cluster 3
        else { cluster_coverage |= 8; }              // Isolated
    }

    let clusters_hit = (cluster_coverage & 1) + ((cluster_coverage >> 1) & 1)
                     + ((cluster_coverage >> 2) & 1) + ((cluster_coverage >> 3) & 1);
    println!("   Diversity sample covers {} groups", clusters_hit);

    if clusters_hit >= 3 {
        println!("   ✓ Diversity sampling covers multiple clusters");
    } else {
        println!("   ✗ Diversity sampling not covering enough clusters");
    }

    // Stats verification
    if stats.total_items == 10 {
        println!("   ✓ Stats correctly count total items");
    }

    println!("\n✅ KDF-Specific Features 正常動作");
}
