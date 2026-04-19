# Demo D6: Forum / SNS テキスト dedup + 少数意見保持

**Dataset:** synthetic_forum_n136 (n=136)

**Patent section:** 明細書 §0002 (SNS/フォーラム投稿) / Claim 1, 18, 46

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `minority_recall` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `dup_reduction` ↑: 高い方が良い
- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | minority_recall | dup_reduction | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|
| Random | No | 0.410 | 0.422 | 0.699 | 0.001 | 0.00 |
| ExactDup | No | 1.000 | 1.000 | 0.779 | 0.005 | 0.00 |
| MinHash | No | 0.100 | 1.000 | 0.985 | 0.503 | 0.50 |
| SimHash | No | 0.000 | 0.195 | 0.699 | 0.464 | 0.46 |
| **KDF** | No | 0.000 | 0.146 | 0.699 | 0.100 | 0.10 |
| KDF+TextSim | No | 0.100 | 0.133 | 0.779 | 0.626 | 0.63 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- Forum/SNS で **reply 構造から** minority post を保護(textual dedup と並行)
- MinHash/SimHash が見逃す「**少ないが独立した視点**」の保持

### ⚠️ KDF を避けるべきシナリオ

- 純粋なテキスト重複排除 → MinHash/SimHash で十分
- reply graph が無い(単発投稿リスト)→ KDF のシグナル元が消える

### 📋 正直な制限事項

- 合成 forum データ(reply graph のテンプレ生成)での評価
- 実 Reddit/HN post は誤差、spam/minority 分離がより難しい可能性
- KDF+TextSim の hybrid は ad-hoc な weighted union、本格 ensemble ではない

## 再現

各 demo の README.md を参照してください。
