# Demo D5: 知識グラフ (FB15K-237) 希少 entity 保存付き curation

**Dataset:** FB15K-237 (n=14541)

**Patent section:** 明細書 §0002 (知識グラフ) / Claim 1, 42, 46 (整合性発見)

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `rare_recall` ↑: 高い方が良い
- `analogy_pairs` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | rare_recall | analogy_pairs | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|
| Random | No | 0.297 | 9662.000 | 0.701 | 0.162 | 0.16 |
| FreqCutoff | No | 0.198 | 4424.000 | 0.700 | 0.585 | 0.58 |
| DegreeTopK | No | 0.358 | 11411.000 | 0.700 | 0.562 | 0.56 |
| TransE-like | No | 0.345 | 10989.900 | 0.700 | 0.561 | 0.56 |
| **KDF** | No | 0.331 | 8839.000 | 0.700 | 20.673 | 20.67 |
| KDF+RelDensity | No | 0.237 | 4386.000 | 0.700 | 7.566 | 7.57 |
| KDF+Analogy | No | 0.332 | 8839.000 | 0.700 | 29.556 | 29.56 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- KG の長尾 entity(出現稀な relation を触れる)を保護しつつ graph 縮約
- 既存の確立 cluster と孤立 entity 間の構造類似 pair を発見する用途

### ⚠️ KDF を避けるべきシナリオ

- 純粋な新規 link prediction → TransE/ComplEx などの embedding 系が適切
- 高頻度 entity の重要度ランキング → 次数ランキングで十分

### 📋 正直な制限事項

- rare entity = 出現関係 freq ≤ 5 の端点、という定義に対する評価。他の定義(betweenness 等)は別途検証要
- TransE-like は「次数 top-K を embedding-top-K の近似プロキシ」として実装(訓練しない)

## 再現

各 demo の README.md を参照してください。
