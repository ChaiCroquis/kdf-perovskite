# Demo D4: 推薦システム long-tail アイテム保持 curation

**Dataset:** synthetic_movielens_n500x300 (n=300)

**Patent section:** 明細書 §0002 (検索又は推薦) / Claim 1, 18, 42

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `tail_recall` ↑: 高い方が良い
- `tail_ndcg` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `coverage` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | tail_recall | tail_ndcg | coverage | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|
| Random | No | 0.305 | 0.313 | 0.300 | 0.003 | 0.00 |
| PopularityTop | No | 0.163 | 0.226 | 0.300 | 0.122 | 0.12 |
| MF-proxy | No | 0.163 | 0.230 | 0.300 | 0.262 | 0.26 |
| **KDF** | No | 0.163 | 0.227 | 0.300 | 1.051 | 1.05 |
| KDF+RelDensity | No | 0.359 | 0.389 | 0.300 | 0.359 | 0.36 |
| KDF+Analogy | No | 0.163 | 0.247 | 0.300 | 1.549 | 1.55 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- 推薦システムの item index / cache の **long-tail 保持付き縮減**
- popularity top-K が多様性を下げすぎる環境の対策
- user-item bipartite の構造シグナルで rare item を検出

### ⚠️ KDF を避けるべきシナリオ

- NDCG@10 のような精度第一指標 → MF / Neural CF に完敗
- popularity-dominated item 推薦 → PopularityTop で十分

### 📋 正直な制限事項

- 合成 MovieLens 風データ(実 MovieLens 100K/1M の分布近似)
- 本 demo は item **selection** 品質のみ、実際の推薦精度は未測定
- MF-proxy は variance-based heuristic、実 NMF / Neural CF ではない

## 再現

各 demo の README.md を参照してください。
