//! Comprehensive feature test for all KDF functionality
use kdf::{Kdf, cosine_similarity, SelectionReason, Layer};

fn main() {
    println!("=== 新機能テスト ===\n");

    let kdf = Kdf::with_defaults();
    let items = vec![
        vec![1.0, 0.0, 0.0],  // 0: クラスタA
        vec![1.0, 0.1, 0.0],  // 1: クラスタA
        vec![1.0, 0.0, 0.1],  // 2: クラスタA
        vec![0.0, 1.0, 0.0],  // 3: クラスタB
        vec![0.0, 1.0, 0.1],  // 4: クラスタB
        vec![-1.0, -1.0, -1.0], // 5: レア
    ];

    let result = kdf.process(&items, 0.90, |a, b| cosine_similarity(a, b));

    println!("## 1. is_selected() テスト");
    for i in 0..items.len() {
        println!("   Item {}: is_selected={}", i, result.is_selected(i));
    }

    println!("\n## 2. reason() テスト");
    for i in 0..items.len() {
        let reason = result.reason(i);
        let desc = match reason {
            SelectionReason::Rare => "Rare (孤立)".to_string(),
            SelectionReason::Representative { group_size } =>
                format!("Representative ({}件の代表)", group_size),
            SelectionReason::NotSelected { representative } =>
                format!("NotSelected (代表: {})", representative),
        };
        println!("   Item {}: {}", i, desc);
    }

    println!("\n## 3. representative_of() テスト");
    for i in 0..items.len() {
        println!("   Item {} の代表: {}", i, result.representative_of(i));
    }

    println!("\n## 4. cluster_members() テスト");
    for &s in result.selected_indices() {
        let members = result.cluster_members(s);
        println!("   代表 {} のメンバー: {:?}", s, members);
    }

    println!("\n## 5. cluster_groups() テスト");
    let groups = result.cluster_groups();
    for (i, group) in groups.iter().enumerate() {
        println!("   グループ {}: {:?}", i, group);
    }

    println!("\n## 6. Layers テスト");
    for (i, &layer) in result.layers.iter().enumerate() {
        let layer_name = match layer {
            Layer::Core => "Core",
            Layer::Edge => "Edge",
            Layer::Rare => "Rare",
        };
        println!("   Item {}: {}", i, layer_name);
    }

    println!("\n## 結果サマリ");
    println!("   入力: {} 件", items.len());
    println!("   選択: {} 件 ({:?})", result.selected.len(), result.selected);

    // Assertions
    assert!(result.is_selected(5), "Rare item must be selected");
    assert_eq!(result.layers[5], Layer::Rare, "Item 5 must be Rare layer");

    // Verify reason returns correct type
    match result.reason(5) {
        SelectionReason::Rare => println!("   ✓ Item 5 correctly identified as Rare"),
        _ => panic!("Item 5 should have Rare reason"),
    }

    // Verify cluster_members works
    let members = result.cluster_members(5);
    assert!(members.contains(&5), "Cluster members should contain self");

    println!("\n✅ 全機能正常動作");
}
