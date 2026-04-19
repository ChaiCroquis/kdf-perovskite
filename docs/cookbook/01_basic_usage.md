# Cookbook 01 — 基本的な使い方

`cgb-kdf` で Claim 1 の 3 手段(代謝制御・希少性保護・整合性発見)を一度に使う最小例。

## 前提

```toml
# Cargo.toml
[dependencies]
cgb-kdf = { path = "crates/cgb-kdf" }
```

## 最小コード

```rust
use cgb_kdf::{KdfProcessorRev12, NodeClassifier, Layer};

fn main() {
    // データ: 5 つのハブと 2 つの希少ノード(各 1 ハブに接続)
    let edges = vec![
        // Dense hubs
        (0, 1, 1.0), (0, 2, 1.0), (0, 3, 1.0), (0, 4, 1.0),
        (1, 2, 1.0), (1, 3, 1.0), (2, 3, 1.0),
        // Rare nodes (degree=1 → Claim 42 希少範囲)
        (5, 0, 1.0),
        (6, 1, 1.0),
    ];
    let node_count = 7;

    // 1. 分類 (Claim 18 保護属性付与)
    let mut classifier = NodeClassifier::default();
    let classification = classifier.classify(node_count, &edges);
    for (&id, &layer) in &classification.layers {
        println!("node {} → {:?}", id, layer);
    }

    // 2. Rev.12 プロセッサ(Claim 36 多段審査)
    let mut processor = KdfProcessorRev12::default();
    processor.initialize(node_count, &edges);

    // 3. 1サイクル実行
    let actions = processor.process_review_cycle();
    for (node, action) in actions {
        println!("{} → {}", node, action);
    }

    // 4. スポーク up フラグ確認 (Claim 40)
    for (node, target, score) in processor.get_spoke_up_nodes() {
        println!("希少 {} が {} と整合性 {:.3} で接続", node, target, score);
    }
}
```

## 期待出力例

```
node 0 → Core      # ハブ
node 1 → Edge
node 2 → Edge
node 3 → Edge
node 4 → Garbage   # 接続少
node 5 → Rare      # 希少保護対象
node 6 → Rare
```

## 参考

- Claim 1 (独立): 3 手段を同時具備
- Claim 18 (従属): Rare は代謝制御から保護
- Claim 42 (従属): 希少範囲外(deg>1)は候補から除外
