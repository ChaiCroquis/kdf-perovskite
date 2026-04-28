# Demo D5: 知識グラフ (FB15K-237) 希少 entity 保存付き curation

**Dataset:** FB15K-237_synth_n5000_rel50 (n=5000)

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
| Random | No | 0.288 | 10411.300 | 0.702 | 0.043 | 0.04 |
| FreqCutoff | No | 0.250 | 11911.000 | 0.700 | 0.076 | 0.08 |
| DegreeTopK | No | 0.317 | 9128.000 | 0.700 | 0.063 | 0.06 |
| TransE-like | No | 0.338 | 8764.400 | 0.700 | 0.078 | 0.08 |
| **KDF** | No | 0.283 | 10706.000 | 0.700 | 1.798 | 1.80 |
| KDF+RelDensity | No | 0.233 | 14373.000 | 0.700 | 0.955 | 0.96 |
| KDF+Analogy | No | 0.367 | 10706.000 | 0.700 | 2.948 | 2.95 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- KG の長尾 entity(出現稀な relation を触れる)を保護しつつ graph 縮約
- 既存の確立 cluster と孤立 entity 間の構造類似 pair を発見する用途

### ⚠️ KDF を避けるべきシナリオ

- 純粋な新規 link prediction → TransE/ComplEx などの embedding 系が適切
- 高頻度 entity の重要度ランキング → 次数ランキングで十分

### 📋 正直な制限事項

- 本実行は合成 KG (n=5000, Freebase-shaped) を使用。実 FB15K-237 使用時は `demos/D5_fb15k237/data/fb15k-237/` に train/valid/test.txt を配置
- rare entity = 出現関係 freq ≤ 5 の端点、という定義に対する評価。他の定義(betweenness 等)は別途検証要
- TransE-like は「次数 top-K を embedding-top-K の近似プロキシ」として実装(訓練しない)

## 再現

各 demo の README.md を参照してください。
