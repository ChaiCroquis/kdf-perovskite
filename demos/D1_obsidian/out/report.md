# Demo D1: Obsidian-style 知識ネットワーク自動キュレーション

**Dataset:** ObsidianVault_n2182_read2182 (n=2182)

**Patent section:** 明細書 §0002 (ナレッジベース) / Claim 1, 42, 46

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `rare_recall` ↑: 高い方が良い
- `analogy_pair_count` ↑: 高い方が良い
- `compression` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `precision_at_rare` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | rare_recall | analogy_pair_count | compression | precision_at_rare | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|---:|
| Random | No | 0.296 | 59.100 | 0.696 | 0.098 | 0.019 | 0.02 |
| OrphanOnly | No | 0.000 | 0.000 | 0.162 | 0.000 | 0.045 | 0.04 |
| TextSim | No | 0.128 | 4.000 | 0.700 | 0.043 | 0.140 | 0.14 |
| **KDF** | No | 0.863 | 4.000 | 0.868 | 0.659 | 0.361 | 0.36 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- ラベルのない個人知識ベース(Obsidian 等)の自動整理
- 構造類似(タグや単語が違うが関係同型)のノートペア発見
- 長期運用で古い孤立ノートを完全消去するのではなく、保護しつつ再接続候補を提示する用途

### ⚠️ KDF を避けるべきシナリオ

- LLM による意味的要約が目的の場合(KDF は summarization はしない)
- テキストの細かい意味解釈が必要なケース

### 📋 正直な制限事項

- KDF 自体はノート内容を理解しない。構造のみを見る
- indegree ≤ 2 を rare 真値とする運用に最適化した評価
- wall_ms は大規模 vault(10^5 超)では再検証が必要

## 再現

各 demo の README.md を参照してください。
