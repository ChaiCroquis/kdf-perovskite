# Title + Abstract(Step 6.1 first iteration)

**目的**: v0.3 日本語 paper から arxiv 向け英語 preprint 化の first deliverable。translation style 指針確定と title 選定のための試作。

**スタイル判定基準**:
- 誠実性原則(self-refutation を隠さない)を維持
- 学術英語の通例(specific、verifiable、passive voice は適度に)
- 日本語の nuance を直訳固執せず意訳

---

## タイトル候補

### A(humble / contribution-first)

> **A Deterministic Graph-Compression Framework with Rarity Preservation and Analogy Discovery: Empirical Study Across LLM Memory, Logs, and Streaming Scenarios**

- **強み**: 主張が狭く defensible、何をした論文かが一目瞭然
- **弱み**: universality 仮説(10 領域並行発見)が全く出てこない、新規性の差別化が弱い

### B(honest-narrowing front)

> **Knowledge Decay Framework: Mechanism Design, Canonical-Value Refutation, and Empirical Validation across Five Realistic Benchmarks**

- **強み**: 自己反証を title で明示、他にない独自の特徴として pitch 化できる
- **弱み**: "Refutation" の前面化は読者の第一印象が「欠陥研究」になるリスク、実際は 6 肯定 / 4 陰性の balance

### C(balanced / universality-soft)

> **Knowledge Decay Framework: A Three-Pillar Deterministic Architecture for Finite-Resource Information Preservation**

- **強み**: 現在の日本語 title の直訳に近い、universality を "architecture" で soft に残す
- **弱み**: Step X で得た自己反証の一貫した nuance が title に出ない

### D(current 改善 / universality + self-refutation 両立)★推奨

> **KDF: A Deterministic Architecture for Finite-Resource Information Preservation—Cross-Domain Evidence and Self-Refutation of Canonical Values**

- **強み**: (1) KDF abbreviation を preserve、(2) "cross-domain evidence" で universality 観察を維持、(3) "self-refutation of canonical values" で本研究の誠実性特徴を明示、(4) 現タイトルの長さと同等
- **弱み**: やや長い(12 words)、subtitle が dense

### E(short / punchy、2 行 fallback)

> **Knowledge Decay Framework: Deterministic Structure-Preserving Compression with Honest Self-Refutation**

- **強み**: 短い、記憶しやすい
- **弱み**: "Honest Self-Refutation" は informal すぎるかも、arxiv の tone に合わない可能性

---

## Abstract 英訳(300 words target)

> **KDF (Knowledge Decay Framework)** is an "item-level lossless, set-level selective" graph-compression technique: given a graph and a retention budget, selected nodes are preserved **verbatim** while unselected nodes are discarded—distinguishing KDF from content-transforming methods such as LLM fact extraction. KDF combines three mechanisms: (1) edge-based continuous-time exponential decay for metabolic control, (2) rarity protection under the threshold deg_E(v) ≤ 1, and (3) integrity discovery (analogy) via graph-Laplacian eigenvalue fingerprints.
>
> A systematic related-work survey reveals that the same three-pillar structure—metabolism, rarity protection, recombination—recurs independently in ten disciplines: mammalian memory consolidation, immune clonal selection, Ginzburg-Landau critical phenomena, continual-learning EWC, Equitable Coreset Selection, Hopfield associative memory, Markov processes on graphs, Pareto heavy-tail economics, K-SVD sparse coding, and the present work. Three characteristic elements support (but do not prove) architectural universality: Δα ∝ δk⁴ matches Ginzburg-Landau's quartic term; a sandwich 2-threshold mechanism (θ_L ≤ S ≤ θ_U) is novel among surveyed domains; λ(C)·exp(−λdt) structurally corresponds to Markov survival probability.
>
> We then **empirically refute our own patent's canonical sandwich values (θ_L, θ_U) = (0.70, 0.80) across four benchmarks** (Hopfield mixture, direct analogy, synthetic pairs, LoCoMo streaming): the 2-threshold *mechanism* is supported, but specific values require domain-specific calibration.
>
> Empirically we report 7.7× gain over TTL on LongMemEval (ICLR 2025), F1 = 0.747 on a 2,182-note Obsidian vault (Wilcoxon p = 0.006), 2.3× over Random on NASA HTTP logs, and a **+3.06-point rare-recall improvement in realistic NASA streaming replay** (the first positive empirical anchor for our narrowed thesis that dynamic control finds its true use in streaming). We also report negative results honestly: OSS issue generalization ×1.00, paper rediscovery ×0.83, Gaussian Process inducing-point selection failure, and the canonical-value refutation above. Applicability is predictable a priori via the bias-detector metric.

**Word count**: 約 315 words(arxiv target 150-300 の上限付近だが、本研究の誠実性 narrative を保つには妥当)

---

## 翻訳スタイル指針(案、user 確認希望)

本 Abstract で採用した翻訳判断:

| 日本語表現 | 採用英訳 | 代替案(採用しなかった) | 理由 |
|---|---|---|---|
| 項目内可逆・集合内選別 | item-level lossless, set-level selective | item-reversible, set-selective | 情報圧縮分野の用語と整合、"reversible" は可逆計算とも衝突 |
| 代謝制御 / 希少性保護 / 整合性発見 | metabolic control / rarity protection / integrity discovery(analogy) | catabolism / ... | 既存 paper §3 の表記と統一 |
| sandwich 採用域 | sandwich 2-threshold mechanism | sandwich acceptance band | "mechanism" で mathematical construct を指す標準用語 |
| 特許で定めた具体値が反証された | canonical values empirically refuted | patent-specified default values disproven | "canonical" は文献で自己反証含意のニュアンス、"specification" は legal tone |
| 誠実な自己反証 | empirically refute our own | honestly falsify | 学術英語の能動 voice 標準 |
| 真の use case | true use case | real-world use case | 日本語の "真の" のニュアンスに近い |
| 統合アーキテクチャ | domain-invariant architecture(§1 で明示したので abstract では "architecture")| general-purpose architecture | universality claim の弱さに整合 |

**確認したい方針点**:

1. **自己反証の扱い**: Abstract 内で "We then empirically refute our own..." と明示する(採用案)か、隠して §4.2 まで後回しにするか?→ 推奨: 明示(誠実性原則)
2. **ICLR 2025 など第三者 reference**: Abstract に含める(LongMemEval は ICLR 2025 で明示、F-xxx IDs は除外)か、reference section まで後回しか?→ 採用案は含める
3. **F-xxx ID の扱い**: Abstract には F-070 / F-072 など内部 ID は出さない、§4.2 以降で使う
4. **Wilcoxon / p-value / χ² 等の統計**: Abstract に 1-2 個 inline で出す(p = 0.006 のみ採用)、全て列挙は limitation 章のみ

---

## 次 session 候補(Step 6.2)

タイトル選定 + スタイル方針確定次第、§1 Introduction 4 小節(1.1 問題 / 1.2 観察 / 1.3 主張 / 1.4 universality-vs-novelty 緊張)の英訳に進む。§1.3 C3 と §1.4 が最も delicate(自己反証の前面化)。
