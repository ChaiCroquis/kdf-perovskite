# Phase A — Wilcoxon signed-rank 統計検定(全 demos 横断)

各 demo で、KDF variants vs 各 baseline の対応サンプル(trial seed 揃え)を用いた
Wilcoxon signed-rank 検定。実装は Rust の `real_data_bench::wilcoxon` と同一
(A&S 7.1.26 erf、正規近似、連続性補正)をクロスチェック目的で Python 側に再実装。

## D1: Obsidian-style 知識ネットワーク自動キュレーション

Dataset: ObsidianVault_n2182_read2182 (n=2182)

### Metric: `analogy_pair_count`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | OrphanOnly | 10 | +4.000 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | -51.000 | -2.75 | 0.006 | **YES** |
| KDF | TextSim | — | — | — | — | — |

### Metric: `compression`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | OrphanOnly | 10 | +0.707 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.170 | +2.75 | 0.006 | **YES** |
| KDF | TextSim | 10 | +0.169 | +2.75 | 0.006 | **YES** |

### Metric: `precision_at_rare`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | OrphanOnly | 10 | +0.659 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.562 | +2.75 | 0.006 | **YES** |
| KDF | TextSim | 10 | +0.616 | +2.75 | 0.006 | **YES** |

### Metric: `rare_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | OrphanOnly | 10 | +0.863 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.575 | +2.75 | 0.006 | **YES** |
| KDF | TextSim | 10 | +0.735 | +2.75 | 0.006 | **YES** |

## D2: HTTP アクセスログ圧縮 — 稀なエラー応答の自動保持

Dataset: NASA-HTTP (synthetic, Zipf) (n=20000)

### Metric: `compression`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | Head | — | — | — | — | — |
| KDF | Random | — | — | — | — | — |
| KDF | Reservoir | — | — | — | — | — |
| KDF | StratifiedLabeled | — | — | — | — | — |
| KDF | TailBasedLabeled | — | — | — | — | — |
| KDF+RelDensity | Head | — | — | — | — | — |
| KDF+RelDensity | Random | — | — | — | — | — |
| KDF+RelDensity | Reservoir | — | — | — | — | — |
| KDF+RelDensity | StratifiedLabeled | — | — | — | — | — |
| KDF+RelDensity | TailBasedLabeled | — | — | — | — | — |
| _(no paired data)_ | | | | | | |

### Metric: `rare_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | Head | 10 | -0.036 | -2.75 | 0.006 | **YES** |
| KDF | Random | 9 | -0.021 | -2.43 | 0.015 | no |
| KDF | Reservoir | 9 | -0.016 | -2.07 | 0.038 | no |
| KDF | StratifiedLabeled | 10 | -0.922 | -2.75 | 0.006 | **YES** |
| KDF | TailBasedLabeled | 10 | -0.922 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | Head | 10 | +0.193 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | +0.208 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Reservoir | 10 | +0.214 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | StratifiedLabeled | 10 | -0.693 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | TailBasedLabeled | 10 | -0.693 | -2.75 | 0.006 | **YES** |

## D3: ML 学習データ長尾クラス保持 curation

Dataset: synthetic_longtail_n2000_c10 (n=2000)

### Metric: `diversity`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | ClassBalance | 10 | -1.510 | -2.75 | 0.006 | **YES** |
| KDF | HerdingProxy | 10 | +16.094 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | -0.441 | -2.55 | 0.011 | no |
| KDF | Stratified | 10 | -0.135 | -1.12 | 0.262 | no |
| KDF+Analogy | ClassBalance | 10 | -1.510 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | HerdingProxy | 10 | +16.094 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | -0.441 | -2.55 | 0.011 | no |
| KDF+Analogy | Stratified | 10 | -0.135 | -1.12 | 0.262 | no |
| KDF+RelDensity | ClassBalance | 10 | -2.417 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | HerdingProxy | 10 | +15.187 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | -1.348 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | Stratified | 10 | -1.042 | -2.75 | 0.006 | **YES** |

### Metric: `minority_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | ClassBalance | 10 | -0.267 | -2.75 | 0.006 | **YES** |
| KDF | HerdingProxy | 10 | +0.294 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.001 | -1.22 | 0.221 | no |
| KDF | Stratified | 10 | -0.009 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | ClassBalance | 10 | -0.263 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | HerdingProxy | 10 | +0.298 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | +0.005 | -1.17 | 0.241 | no |
| KDF+Analogy | Stratified | 10 | -0.005 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | ClassBalance | 10 | -0.284 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | HerdingProxy | 10 | +0.276 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | -0.016 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | Stratified | 10 | -0.027 | -2.75 | 0.006 | **YES** |

## D4: 推薦システム long-tail アイテム保持 curation

Dataset: synthetic_movielens_n500x300 (n=300)

### Metric: `tail_ndcg`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | MF-proxy | 10 | -0.003 | -2.75 | 0.006 | **YES** |
| KDF | PopularityTop | 10 | +0.001 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | -0.084 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | MF-proxy | 10 | +0.017 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | PopularityTop | 10 | +0.021 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | -0.064 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | MF-proxy | 10 | +0.159 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | PopularityTop | 10 | +0.163 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | +0.078 | +2.75 | 0.006 | **YES** |

### Metric: `tail_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | MF-proxy | — | — | — | — | — |
| KDF | PopularityTop | — | — | — | — | — |
| KDF | Random | 10 | -0.139 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | MF-proxy | — | — | — | — | — |
| KDF+Analogy | PopularityTop | — | — | — | — | — |
| KDF+Analogy | Random | 10 | -0.139 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | MF-proxy | 10 | +0.195 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | PopularityTop | 10 | +0.195 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | +0.056 | +2.75 | 0.006 | **YES** |

## D5: 知識グラフ (FB15K-237) 希少 entity 保存付き curation

Dataset: FB15K-237_synth_n5000_rel50 (n=5000)

### Metric: `analogy_pairs`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | DegreeTopK | 10 | +1578.000 | +2.75 | 0.006 | **YES** |
| KDF | FreqCutoff | 10 | -1205.000 | -2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +242.000 | +1.73 | 0.083 | no |
| KDF | TransE-like | 10 | +2123.000 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | DegreeTopK | 10 | +1578.000 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | FreqCutoff | 10 | -1205.000 | -2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | +242.000 | +1.73 | 0.083 | no |
| KDF+Analogy | TransE-like | 10 | +2123.000 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | DegreeTopK | 10 | +5245.000 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | FreqCutoff | 10 | +2462.000 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | +3909.000 | +2.75 | 0.006 | **YES** |
| KDF+RelDensity | TransE-like | 10 | +5790.000 | +2.75 | 0.006 | **YES** |

### Metric: `rare_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | DegreeTopK | 10 | -0.033 | -2.75 | 0.006 | **YES** |
| KDF | FreqCutoff | 10 | +0.033 | +2.75 | 0.006 | **YES** |
| KDF | Random | 7 | +0.000 | -0.08 | 0.933 | no |
| KDF | TransE-like | 8 | -0.050 | -2.45 | 0.014 | no |
| KDF+Analogy | DegreeTopK | 10 | +0.050 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | FreqCutoff | 10 | +0.117 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 9 | +0.083 | +2.37 | 0.018 | no |
| KDF+Analogy | TransE-like | 9 | +0.033 | +1.84 | 0.066 | no |
| KDF+RelDensity | DegreeTopK | 10 | -0.083 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | FreqCutoff | 10 | -0.017 | -2.75 | 0.006 | **YES** |
| KDF+RelDensity | Random | 10 | -0.050 | -1.94 | 0.053 | no |
| KDF+RelDensity | TransE-like | 10 | -0.100 | -2.75 | 0.006 | **YES** |

## D6: Forum / SNS テキスト dedup + 少数意見保持

Dataset: synthetic_forum_n136 (n=136)

### Metric: `dup_reduction`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | ExactDup | 10 | -0.854 | -2.75 | 0.006 | **YES** |
| KDF | MinHash | 10 | -0.854 | -2.75 | 0.006 | **YES** |
| KDF | Random | 10 | -0.268 | -2.75 | 0.006 | **YES** |
| KDF | SimHash | 10 | -0.049 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | ExactDup | 10 | -0.867 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | MinHash | 10 | -0.867 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | Random | 10 | -0.281 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | SimHash | 10 | -0.062 | -2.75 | 0.006 | **YES** |

### Metric: `minority_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | ExactDup | 10 | -1.000 | -2.75 | 0.006 | **YES** |
| KDF | MinHash | 10 | -0.100 | -2.75 | 0.006 | **YES** |
| KDF | Random | 10 | -0.400 | -2.75 | 0.006 | **YES** |
| KDF | SimHash | — | — | — | — | — |
| KDF+TextSim | ExactDup | 10 | -0.900 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | MinHash | — | — | — | — | — |
| KDF+TextSim | Random | 10 | -0.300 | -2.75 | 0.006 | **YES** |
| KDF+TextSim | SimHash | 10 | +0.100 | +2.75 | 0.006 | **YES** |

## D7: GitHub Issue アーカイブ + reopen 候補の構造類似発見

Dataset: synthetic_issues_n500 (n=500)

### Metric: `precision`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | LabelMatch | 10 | +0.060 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.040 | +2.60 | 0.009 | **YES** |
| KDF | StaleBot | 10 | +0.080 | +2.75 | 0.006 | **YES** |
| KDF | TextSim | 10 | +0.080 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | LabelMatch | 10 | +0.013 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | -0.007 | -1.27 | 0.203 | no |
| KDF+Analogy | StaleBot | 10 | +0.033 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | TextSim | 10 | +0.033 | +2.75 | 0.006 | **YES** |

### Metric: `reopen_recall`

| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |
|---|---|---:|---:|---:|---:|:---:|
| KDF | LabelMatch | 10 | +0.243 | +2.75 | 0.006 | **YES** |
| KDF | Random | 10 | +0.162 | +2.60 | 0.009 | **YES** |
| KDF | StaleBot | 10 | +0.324 | +2.75 | 0.006 | **YES** |
| KDF | TextSim | 10 | +0.324 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | LabelMatch | 10 | +0.054 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | Random | 10 | -0.027 | -1.27 | 0.203 | no |
| KDF+Analogy | StaleBot | 10 | +0.135 | +2.75 | 0.006 | **YES** |
| KDF+Analogy | TextSim | 10 | +0.135 | +2.75 | 0.006 | **YES** |

## 総合サマリ

統計的有意性 (α=0.01) の判定を各 (demo, metric, KDF vs baseline) タプルで実施。

`report.json` の raw_trials 配列は seed 一対一対応なので、paired Wilcoxon が
最適。α=0.01 に設定して多重検定を部分的に吸収(補正なし、複数確認用途)。
