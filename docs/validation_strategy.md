# Validation Strategy — 戦略的 validation 優先度マトリクス

**最終更新**: 2026-04-19
**目的**: domain_validation.md (B1-B8) と classical_algorithm_revival.md (C1-C10) の未検証候補を、**後続影響 × 商用価値 × meaningfulness** で優先度付けし、"できるけど意味ない" タスクを明示的に除外した実行 roadmap を提供する。

---

## 🎯 評価 axes

各 candidate を以下 4 axes で評価:

1. **Downstream impact**: 他の candidate/市場への波及(1-5)
2. **Commercial value**: 実際に誰かがお金を払う市場が存在するか(1-5 + 市場規模 estimate)
3. **Meaningfulness**: "できるけど意味ない" と対立する概念 — 「やった結果が誰かの役に立つか」
4. **Execution cost**: 時間 + API 費用

---

## 📊 Tier 分類

### 🥇 **Tier 1 — 今すぐ実施**(high impact × meaningful × $0)

| # | Validation | Downstream | Commercial | Meaning | Cost |
|:-:|---|---|---|---|---:|
| **C2** | **Betweenness centrality via KDF pruning** | 5/5(C1/C3/C5/C6 全て同一手法)| **4/5**(graph analytics tool 市場 $10億+: Gephi, Neo4j, TigerGraph)| **Meaningful** — 「古典 algorithm 復権」thesis の core 検証 | $0, 2-3 日 |
| **C1** | **Floyd-Warshall via KDF pruning** | 4/5(C2 と同 infra で実装可)| 3/5(routing / logistics niche、数十億円市場) | Meaningful | $0, 1-2 日 |
| **B1** | **Git commit pruning** | 4/5(B2 / Ext-8 unlock)| **5/5**(GitHub / GitLab / Atlassian 巨大市場、$10B+ GitHub alone)| **Meaningful** | $0, 2-3 日 |

→ **この 3 件で "KDF 汎用性" の最重要 claim を 1 週間以内に validate 可能**

---

### 🥈 Tier 2 — Tier 1 成功後実施(medium impact × meaningful × $0)

| # | Validation | Downstream | Commercial | Meaning | Cost |
|:-:|---|---|---|---|---:|
| ~~B2 naive~~ | ~~Naive call graph curation~~ | **検証 → F-064 で negative**、naive 版は Tier 4 に | — | 意味なし(構造 signal が API と相関せず) | 完了 |
| B2 proper | Proper static-analysis call graph | 3/5(Ext-8 unlock) | 4/5(Sourcegraph 等、$2.6B 評価) | Meaningful but engineering-heavy | $0, 2-3 週間 |
| ~~C5~~ | ~~GP inducing points via KDF~~ | **検証 → F-063 で negative**、Tier 4 に降格 | — | 意味なし(density coverage ≠ KDF 適性) | 完了 |

---

### 🥉 Tier 3 — 機会があれば実施(high commercial × access constraints)

| # | Validation | Downstream | Commercial | Meaning | Cost |
|:-:|---|---|---|---|---:|
| ~~B4(naive k-NN feature graph)~~ | ~~金融 fraud archival~~ | **検証 → F-066 で negative**、IsolationForest に完敗(92% vs 28%) | — | 意味なし(feature-space density ≠ structural rareness) | 完了 |
| B4 proper | Transaction graph (account/temporal edges) fraud archival | 2/5 | **5/5**(regulated finance)| **Meaningful**(未検証、future work)| $0, 1 週間 |
| **B6** | **医療 event timeline (MIMIC-III)** | 2/5 | **5/5**(HIPAA 準拠の長期 EHR 市場) | Meaningful | $0, 1 ヶ月 (MIMIC access) |
| ~~C4~~ | ~~Kernel SVM via KDF selection~~ | **検証 → F-067 で marginal**、Tier 4 に降格 | — | SVM は subset 選択に robust で KDF 独自 value なし | 完了 |

---

### 🚫 **Tier 4 — できるけど意味ない、やらない**

| # | Candidate | "意味ない" 理由 | 除外判断 |
|:-:|---|---|:-:|
| **C4** | Kernel SVM subset selection | **F-067 で検証、marginal**(Random と tie、KMeans に僅敗)。SVM は subset 選択に robust で KDF の特別 value なし | ❌ 検証済 Skip |
| **B4 naive** | Credit card fraud via feature k-NN graph | **F-066 で検証、negative**(28% vs IsolationForest 92%)。Feature-space density ≠ structural rareness | ❌ 検証済 Skip |
| **B2 naive** | Naive call graph curation(name-match)| **F-064 で検証、negative**(Random を下回る)。Public API は in-degree 高で KDF の Rare protection と逆方向 | ❌ 検証済 Skip |
| **C5** | GP inducing points | **F-063 で検証、negative**(Random 以下、KMeans / TopDegree に完敗)。GP の inducing point は density coverage が要件で、KDF の structural rareness とは逆の方向 | ❌ 検証済 Skip |
| **B5** | Citation network pruning | **F-039 で既に negative evidence**、OpenAlex 論文再発見が ×0.83(Random 以下)。同じ data で別 task にしても根本問題(structure が意味を符号化していない)は残る | ❌ Skip |
| **B7** | 法務 discovery(Enron)| Enron email は 20 年前、**NLP benchmark として saturated**。新規性低い、投稿先 venue も狭い | ❌ Skip |
| **B8** | ゲノムシーケンス saliency | **Domain expert なしで結果の妥当性判断できない**、phylogenetic 系は bio-informatician が必要、発明者は物理出身 | ❌ Skip |
| **C3** | Girvan-Newman community detection | **Louvain / Leiden が既に実用十分**、Girvan-Newman 復権の実用 value は低い、niche academic interest のみ | ❌ Skip |
| **C7** | Bootstrap / Permutation test | **CLT + asymptotic test で現代統計は十分**、bootstrap 復権の需要が薄い、統計 community の dominant paradigm と対立 | ❌ Skip |
| **C8** | Graph Laplacian eigen-decomp | **pruning が eigenvalue を大きく動かすリスク**、spectral clustering の精度保証が難しい、"あり得ない" negative 結果の可能性大 | ❌ Skip |
| **C9** | Multiple Sequence Alignment | **B8 と同じ理由**、bio expert が必須 | ❌ Skip |
| **C10** | Exact Graph Coloring | **NP-hard approximation は既に研究がある**、KDF preprocessing の贈り物が小さい、商用 buyer が想像できない | ❌ Skip |

---

## 🎯 推奨実施 sequence

### 今日〜明日(Tier 1、$0)

1. **KDF-generic-select binary を作る**(30 分) — 任意の graph JSON を入力に、KDF 選別を返す Rust binary
2. **C2 Betweenness centrality 実験**(2-3 時間) — 中規模 graph で KDF-prune × Brandes を full graph に対する rank correlation / top-K recall で評価
3. **C1 Floyd-Warshall 実験**(1-2 時間) — 同一 graph で APSP 近似精度評価
4. **B1 Git commit pruning 実験**(3-4 時間) — kdf-perovskite 自身 or 公開 repo の git log で merge commit recall

### 明後日(Tier 2 もし Tier 1 成功)

5. **B2 Call graph curation**(1 週間) — tree-sitter or rust-analyzer で call graph 抽出
6. **C5 GP inducing points**(1 週間) — 時系列回帰 benchmark で SGPR 比較

### Tier 3 は market access 次第

- B4/B6/C4 は data access タイミングで実施
- 学術 paper 投稿時の "future work" 枠に置いてもよい

### Tier 4 は**明示的に skip**

論文や pitch で "試していない領域" として言及する価値もない(meaningful でないから)。

---

## 💰 商用価値内訳(Tier 1-3)

### B1 Git commit pruning → **GitHub / GitLab 市場**

- Primary buyer: GitHub ($10B+ 売上、Microsoft 傘下)
- Product form: Action / plugin for "important commit preservation"
- Use case: 100万 commit repo の long-term archival
- Monetization: per-repo monthly fee ($5-50/month/repo)
- Competitive moat: deterministic + auditable + LLM-free

### B2 Call graph curation → **Code analysis 市場**

- Primary buyer: Sourcegraph, Datadog, Snyk
- Product form: Language Server integration
- Use case: 巨大 codebase の dependency analysis
- Monetization: enterprise license ($10K-100K/year)

### C1 Floyd-Warshall revival → **Logistics / Network 市場**

- Primary buyer: 物流(Amazon, FedEx)、ネットワーク運用(Cisco, Arista)
- Product form: KDF preprocessor for all-pairs analysis
- Use case: 全対災害対応 routing、network latency analysis
- Monetization: consulting engagement(重い integration)

### C2 Betweenness centrality → **Graph analytics tool 市場**

- Primary buyer: Gephi (open), Neo4j ($3B 評価)、TigerGraph
- Product form: Plugin / library function
- Use case: social network / citation network influencer detection
- Monetization: graph analytics platform subscriptions

### C5 GP inducing points → **ML platform 市場 (niche)**

- Primary buyer: Domino Data Lab, Weights & Biases
- Product form: scikit-learn-compatible preprocessor
- Use case: interpretable regression for regulated ML
- Monetization: MLOps platform integration

### B4/B6 Regulated archival → **Compliance 市場**

- Primary buyer: financial compliance software vendors、EHR vendors (Epic, Cerner)
- Product form: SDK / middleware
- Use case: long-term audit trail preservation
- Monetization: per-seat enterprise license ($500-5000/seat/year)

---

## 🪤 ドツボ警告(Tier 1 実施時)

- **C2 で benchmark の選び方を過度に蕩尽しない**(Stanford SNAP の 1-2 dataset で十分)
- **Top-K 基準を後出ししない**(K=50 で始める、結果見てから動かさない)
- **"少し改善" を chase しない**(有意な gain が出なければ candidate を捨てる覚悟で)
- **Rust infrastructure を universal にしすぎない**(特定 benchmark の for-loop で十分)

---

## 🔗 関連 document

- [domain_validation.md](domain_validation.md) — 応用領域別 validation 状況(B1-B8)
- [classical_algorithm_revival.md](classical_algorithm_revival.md) — 古典 algorithm 復権候補(C1-C10)
- [design_philosophy.md](design_philosophy.md) — 設計方針
- [extension_ideas.md](extension_ideas.md) — 拡張機能案
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 検証済 findings
