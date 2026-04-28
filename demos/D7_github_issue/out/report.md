# Demo D7: GitHub Issue アーカイブ + reopen 候補の構造類似発見

**Dataset:** synthetic_issues_n500 (n=500)

**Patent section:** 明細書 §0002 (アーカイブ管理) / Claim 1, 42, 46 (整合性発見)

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `reopen_recall` ↑: 高い方が良い
- `precision` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | reopen_recall | precision | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|
| Random | No | 0.346 | 0.085 | 0.700 | 0.004 | 0.00 |
| StaleBot | No | 0.162 | 0.040 | 0.700 | 0.007 | 0.01 |
| LabelMatch | No | 0.243 | 0.060 | 0.700 | 0.006 | 0.01 |
| TextSim | No | 0.162 | 0.040 | 0.700 | 14.782 | 14.78 |
| **KDF** | No | 0.486 | 0.120 | 0.700 | 0.187 | 0.19 |
| KDF+Analogy | No | 0.297 | 0.073 | 0.700 | 0.086 | 0.09 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- issue tracker の自動アーカイブで、**過去 closed issue が現在の open issue と構造類似**する場合に reopen 候補として surface
- ラベル / author / reference 構造が豊富な issue tracker

### ⚠️ KDF を避けるべきシナリオ

- 単純な age-based stale 運用 → StaleBot で十分、KDF overhead に見合わない
- 完全なテキスト意味解釈が必要 → LLM triage の方が強い

### 📋 正直な制限事項

- 合成 issue archive (n=500) での評価、実 rust-lang/rust 等での数値は異なる
- reopen_truth は合成生成した label+reference パターン一致であり、実運用の reopen とは異なる
- 実際の issue は title/body text を LLM で解釈すべきで、本 demo は構造のみ

## 再現

各 demo の README.md を参照してください。
