//! Serialization/Deserialization test
use kdf::{IncrementalKdf, KdfParams, cosine_similarity};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestItem {
    id: usize,
    features: Vec<f64>,
}

fn main() {
    println!("=== Serialization Test ===\n");

    // Create IncrementalKdf with items
    let mut ikdf: IncrementalKdf<TestItem> = IncrementalKdf::new(KdfParams::default(), 0.85);

    for i in 0..5 {
        let item = TestItem {
            id: i,
            features: vec![i as f64 * 0.1, 1.0 - i as f64 * 0.1, 0.5],
        };
        ikdf.add(item, |a, b| cosine_similarity(&a.features, &b.features));
    }

    println!("## Before serialization:");
    println!("   Items: {}", ikdf.len());
    println!("   Layers: {:?}", ikdf.layers());
    let selected_before = ikdf.get_selected(|a, b| cosine_similarity(&a.features, &b.features));
    println!("   Selected: {:?}", selected_before);

    // Serialize
    let json = serde_json::to_string_pretty(&ikdf).unwrap();
    println!("\n## JSON (first 500 chars):");
    println!("{}", &json[..json.len().min(500)]);

    // Deserialize
    let ikdf2: IncrementalKdf<TestItem> = serde_json::from_str(&json).unwrap();

    println!("\n## After deserialization:");
    println!("   Items: {}", ikdf2.len());
    println!("   Layers: {:?}", ikdf2.layers());
    let selected_after = ikdf2.get_selected(|a, b| cosine_similarity(&a.features, &b.features));
    println!("   Selected: {:?}", selected_after);

    // Verify
    assert_eq!(ikdf.len(), ikdf2.len());
    assert_eq!(ikdf.layers(), ikdf2.layers());
    assert_eq!(ikdf.selection_scores(), ikdf2.selection_scores());
    assert_eq!(ikdf.degrees(), ikdf2.degrees());

    // Verify items
    for (a, b) in ikdf.items().iter().zip(ikdf2.items().iter()) {
        assert_eq!(a, b);
    }

    println!("\n✅ Serialization/Deserialization 正常動作");
}
