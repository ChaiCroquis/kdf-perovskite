# Title + Abstract v2(Step 6.1 確定、2026-04-19)

user 合意事項を反映した final draft。Step 6.2 以降は本 v2 を参照して §1 以降に進む。

## 合意事項

| 項目 | 決定 |
|---|---|
| **Title** | **D**(Cross-Domain Evidence and Self-Refutation of Canonical Values)|
| **Self-refutation 露出** | Abstract に明示(i)|
| **Abstract 長さ** | 250 words target(事実骨格のみ、哲学的背景は §1 に譲る)|
| **翻訳スタイル** | [feedback_translation_style.md](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md)参照 |

## Title(FINAL)

> **KDF: A Deterministic Architecture for Finite-Resource Information Preservation—Cross-Domain Evidence and Self-Refutation of Canonical Values**

## Abstract(FINAL、245 words)

> **KDF (Knowledge Decay Framework)** is a deterministic graph-compression technique: given a graph and a retention budget, selected nodes are preserved **verbatim** while unselected nodes are discarded, distinguishing KDF from content-transforming methods such as LLM-based fact extraction. KDF combines three mechanisms—edge-based continuous-time exponential decay (metabolic control), rarity protection under the absolute threshold deg_E(v) ≤ 1, and integrity discovery (analogy) via graph-Laplacian eigenvalue fingerprints—a three-pillar structure that our related-work survey finds independently recurring across ten disciplines, including mammalian memory consolidation, immune clonal selection, Ginzburg-Landau critical phenomena, continual-learning EWC, Hopfield associative memory, and K-SVD sparse coding.
>
> Empirically, KDF delivers 7.7× gain over industry-standard TTL on LongMemEval (ICLR 2025), F1 = 0.747 on a 2,182-note Obsidian vault (Wilcoxon p = 0.006), 2.3× over Random for rare-event preservation on NASA HTTP logs, and a **+3.06-point rare-recall improvement in a realistic streaming replay of the NASA log**—providing the first positive empirical anchor for our narrowed thesis that dynamic control components find their true use in streaming scenarios rather than static queries.
>
> However, we also transparently report refutations of our own prior claims. The sandwich 2-threshold *mechanism* is supported, but **our patent's canonical values (θ_L, θ_U) = (0.70, 0.80) are empirically refuted across four benchmarks** (Hopfield mixture, direct analogy, synthetic pairs, LoCoMo streaming): specific values require domain-specific calibration. Additional honest negatives include OSS issue generalization ×1.00 across three repositories, paper rediscovery ×0.83, and Gaussian-Process inducing-point selection failures. Applicability is predictable a priori via a zero-dependency bias-detector metric.

**word count 実測**: 245 words(P1=68、P2=73、P3=104)

---

## Abstract 構成の justification

- **P1 — What built(68 words)**: architecture definition + 3 mechanisms + "three-pillar structure across ten disciplines" 1 sentence で universality observation を最小 footprint で incorporate(title D の "Cross-Domain Evidence" 伏線)
- **P2 — What validated(73 words)**: empirical facts 4 件(LongMemEval / Obsidian / NASA static / NASA streaming +3.06pt)、streaming validation を narrowed thesis の "first positive empirical anchor" として position
- **P3 — What refuted(104 words)**: 4-benchmark canonical value 反証 + 3 件の generalization 失敗 + bias-detector による事前判別可能性 → title D の "Self-Refutation of Canonical Values" を実証

## 哲学的背景を §1 に譲った箇所(事実骨格から除外)

| 概念 | Abstract から除外 | §1 Introduction の該当節 |
|---|---|---|
| 3 特徴要素(δk⁴ / sandwich / Markov)の architectural correspondence | 今回除外 | §1.3 C2 "構造的類似性" + §4 全節 |
| Structural Holes(Burt 1992)対応 | 今回除外 | §Theoretical Foundation |
| L3 層特化 / 6-layer 情報理論 | 今回除外 | kdf_meta_philosophy.md(paper 本体外、appendix か blog 用)|
| 決定論による資産性 / 誠実性の character 論 | 今回除外 | design_philosophy.md + §6.3 特許と license |

これらは §1 以降で Abstract の fact skeleton に theoretical / philosophical な meaning を付与する構造で扱う。

---

## Next — Step 6.2 scope

v2 title + Abstract 確定を受け、次 session では **§1 Introduction 英訳**:

- 1.1 問題設定(long-running networks、storage tradeoff、label scarcity)
- 1.2 観察(10 領域並行発見の survey、Tartaglia 2025 独立並行例を含む)
- 1.3 主張 C1 / C2 / C3(C3 は canonical-value 反証を明示)
- 1.4 Universality ↔ novelty 緊張(two readings: favorable と critical)
- 1.5 本論文構成

§1.3 C3 と §1.4 が delicate — 翻訳スタイル memory の「誠実性 nuance 保存」「断定を避ける hedge」を適用。

## 参照

- 確定 title / abstract: 本ファイル
- Sub-step 計画: [PLAN.md](PLAN.md)
- 翻訳スタイル: [memory/feedback_translation_style.md](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md)
- 日本語 source: [paper_draft.md v0.3](../paper_draft.md)
