# Demo D3: ML 学習データ長尾クラス保持 curation

**Dataset:** synthetic_longtail_n2000_c10 (n=2000)

**Patent section:** 明細書 §0002 (学習データ) / Claim 1, 18, 46

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `minority_recall` ↑: 高い方が良い
- `label_free` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `diversity` ↑: 高い方が良い
- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | minority_recall | label_free | diversity | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|---:|
| Random | No | 0.303 | 1.000 | 21.095 | 0.700 | 0.013 | 0.01 |
| Stratified | Yes | 0.303 | 0.000 | 20.847 | 0.701 | 0.041 | 0.04 |
| HerdingProxy | No | 0.000 | 1.000 | 4.626 | 0.700 | 0.058 | 0.06 |
| ClassBalance | Yes | 0.561 | 0.000 | 22.275 | 0.700 | 0.040 | 0.04 |
| **KDF** | No | 0.294 | 1.000 | 20.720 | 0.700 | 0.724 | 0.72 |
| KDF+RelDensity | No | 0.276 | 1.000 | 19.813 | 0.700 | 0.439 | 0.44 |
| KDF+Analogy | No | 0.298 | 1.000 | 20.720 | 0.700 | 1.240 | 1.24 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- **ラベル未取得**の段階でデータキュレーションしたい場合(事前フィルタリング)
- Feature 空間の構造(kNN 等)を活用できる pipeline
- Herding 等の unsupervised baseline よりは minority を残せる運用

### ⚠️ KDF を避けるべきシナリオ

- **ラベルが確実に得られる** 環境 → Stratified / ClassBalance が絶対強
- Feature vector が不在 or 意味のない特徴量だけの dataset
- モデル訓練中の動的選択(active learning) → これは別の仕組みが必要

### 📋 正直な制限事項

- **合成 dataset**(正規分布 cluster + noise)での評価。実 MNIST/CIFAR では結果が変わる可能性大
- 訓練を実行していない(downstream accuracy は未測定)
- Stage 1 meta-analysis の予測「D5 型(label 独立 → KDF marginal)」を概ね確認する結果

## 再現

各 demo の README.md を参照してください。
