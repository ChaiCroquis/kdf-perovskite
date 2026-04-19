# §5 Empirical Evaluation — English translation (Step 6.5, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 292–320.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

This section presents six positive and four negative empirical cases. The **structure of §5.2** — grouping "generalization failures found in testing" and "refutations of our own canonical values" under the same heading — is load-bearing: it frames the canonical-value refutation as an internal honesty action, not an external criticism. Translator's notes at the end call out the P9 dual finding (static-negative / streaming-positive) and the §5.2 opening sentence as the passages most susceptible to softening.

---

## 5. Empirical Evaluation

### 5.1 Positive Results (Six Cases)

| Problem | Data | KDF result | Comparison |
|---|---|---|---|
| P1 LLM-agent memory | LongMemEval 500 Q (ICLR 2025) | Recall $= 0.821$ | Industry-default TTL $= 0.107$ (**$\times 7.7$**); Random $= 0.294$ ($\times 2.8$) |
| P2 Personal knowledge base | Obsidian vault of 2,182 notes (PII-masked) | $F_1 = 0.747$ | Wilcoxon $p = 0.006$ vs. Random / OrphanOnly / TextSim |
| P3 Large-scale log observability (static baseline) | NASA HTTP log, 50k real records | Recall $= 0.237$ (keep $10\%$, $4xx/5xx$ retention) | Random $= 0.102$ (**$\times 2.3$**), without labels |
| P7 ML reproducibility meta (by-product) | 5 benchmarks × 5 scenarios | 4/5 exact predictions + 1 via alternate route | Released independently as the `bias-detector` crate |
| **P8 Distributed-execution bit-exactness (Claim 17)** | **LoCoMo 10 real graph (600+ nodes, 400+ edges)** | **max edge-weight diff $= 0.000 \text{e}0$** | `apply_edge_decay` (global) and `apply_edge_decay_local` (distributed) are completely bit-exact, F-069 |
| **P11 NASA streaming: Claim 14 decay benefit** | **NASA HTTP log 50k real records, replayed in time order (500 rec / 100 window)** | **C1 decay rare_recall $= 0.4898$ at keep $30\%$ (C0 static $0.4592$ → $+3.06$ pt)** | **First empirical anchor for the narrowed thesis that "streaming is the true use case", F-072** |

As additional verification in Phase X Step 1, all three mechanisms of Claim 1—metabolic control, rarity protection, and integrity discovery—are now empirically backed on realistic benchmarks (integrity discovery via F-068: $100\%$ on Gentner classics, $100\%$ on cross-domain git↔paper, $0\%$ false positives on the negative control). In Phase X Step 5 (F-072), Claim 14 exponential decay is confirmed to yield a $+3.06$-point benefit in a realistic streaming scenario, giving **empirical anchor to the narrowed thesis that "streaming is the true use case"**.

**日本語バックトランス要約**: P1-P11 の 6 件(LongMemEval ×7.7、Obsidian $F_1=0.747$ Wilcoxon $p=0.006$、NASA static ×2.3、bias-detector 4/5、LoCoMo distributed bit-exact、NASA streaming +3.06pt)。Phase X Step 1 で Claim 1 の 3 手段すべて realistic backed に到達、Step 5 で streaming use-case 仮説が empirical anchor を獲得。

---

### 5.2 Negative Results (Four Cases) — An Honest Record

For the integrity of this work, we report **both generalization failures found in testing** and **refutations of canonical values that we ourselves proposed**:

| Problem | Result | Interpretation |
|---|---|---|
| P6 OSS maintenance | KDF/Random ratios over three repos (`rust-lang/rust`, `tokio-rs/tokio`, `golang/go`) are $\times 1.13, \times 1.03, \times 0.85$; average **$\times 1.00$** | The $+15\%$ on `rust-lang` alone is a repository-specific local signal. The general-OSS applicability claim is **withdrawn** (F-038) |
| P5 Paper rediscovery | OpenAlex 200 papers × concept-sharing graph; KDF/Random $= \mathbf{\times 0.83}$ | Late-bloomer detection is a D5-type task (independent of structure); KDF is ill-suited to concept-graphs (F-039) |
| **P9 Redundancy of Claim 5/14 time signals on static tasks (validated on streaming)** | **LoCoMo 321 Q at keep $30\%$: KDF_static $= 0.5286$; adding time signals yields $0.43$–$0.53$ (static: all worse or tied). NASA streaming 50k records at keep $30\%$: C0 static $0.4592$; C1 Claim 14 decay $0.4898$ (**$+3.06$ pt benefit on streaming**)** | On static query tasks, structural rarity already subsumes temporal rarity, so the time signals are redundant. On streaming, decay discards stale normal traffic and relatively elevates rare resources → **task-structure-dependent conditional value** (F-069 static redundancy + F-072 streaming $+3.06$-pt validation) |
| **P10 Four-benchmark refutation of canonical $\theta_U = 0.80$ (Claims 47–48)** | **F-041 (Hopfield: $0\%$ detection) + F-068 (analogy scores $0.99+$ reject every positive) + F-070 Part A (F1 $0.000$ vs. $1.000$) + F-070 Part B (LoCoMo: $100\%$ RARE demoted)** | The 2-threshold *mechanism* is supported; the canonical specific values $(0.70, 0.80)$ are refuted across four benchmarks. Domain-specific calibration is required (F-041, F-070) |

**日本語バックトランス要約**: P6 OSS 3 repo 平均 ×1.00 → OSS 一般主張撤回、P5 論文再発見 ×0.83 → concept-graph 不向き、P9 Claim 5/14 時間信号は static 冗長だが streaming で +3.06pt benefit → task 構造依存の条件付き value(純陰性ではなく条件付き)、P10 canonical $(0.70, 0.80)$ は 4 benchmark 横断反証 → 機構支持 / 値は較正必要。

---

### 5.3 A Priori Applicability Metric: `bias-detector`

We released a zero-dependency Rust crate for determining KDF's applicability in advance ([`crates/bias-detector/`](../../crates/bias-detector/)). The metric $\text{bias\_score} = 0.3 \cdot I_1 + 0.7 \cdot I_4$ correctly predicted applicability on 4 of 5 benchmarks exactly, and on the 5th via an alternate route.

**日本語バックトランス要約**: 事前適用判別 `bias-detector` crate を公開、$\text{bias\_score} = 0.3 \cdot I_1 + 0.7 \cdot I_4$ で 5 件中 4 件完全予測一致、1 件は別経路一致。

---

## Translator's notes on §5

### Note A — §5.2 opening sentence: the honesty frame

The opening sentence of §5.2 — "For the integrity of this work, we report **both** generalization failures found in testing **and** refutations of canonical values that we ourselves proposed" — is the paper's **frame-setter for the negative-results section**. It is what transforms §5.2 from a section that could look like "places where KDF failed" into a section that reads as "what we systematically tested and transparently report, including refutations of our own prior claims".

**Translation choices**:
- "**For the integrity of this work**" translates "本研究の信頼性のため" — "integrity" is chosen over "credibility" because it conveys moral stance, not just reputation management.
- The **both/and** structure (two explicit clauses) mirrors the Japanese double-listing, so the two categories are visibly on equal footing: outside-of-KDF generalization failures (P5/P6) and inside-of-KDF self-refutations (P9/P10).
- "**refutations of canonical values that we ourselves proposed**" preserves "自ら提示した" — the "we ourselves" emphasis is load-bearing for review credibility.

**What we did not do**:
- We did not reorder the clauses (self-refutation first would signal priority, which the Japanese does not).
- We did not replace "integrity" with "completeness" (which would reduce moral stance to bookkeeping).
- We did not soften "refutations" to "revisions" (which would disguise self-refutation as polite tuning).

### Note B — P9 is a dual finding, not a pure negative

P9 is **structurally a negative result** (temporal signals are redundant on the LoCoMo static query task) **but also contains a validated positive** (Claim 14 decay benefit of +3.06 pt on NASA streaming). The paper chose to place P9 in §5.2 (negative results) because the **original claim under test** (that Claim 5/14 time signals help selection) was not validated on the static task; the streaming benefit is the narrowing that emerged.

**Translation choices**:
- The row title starts with "**Redundancy** of Claim 5/14 time signals on static tasks" — the negative finding is the primary label, with "(validated on streaming)" as a parenthetical clarifier.
- The result column reports both measurements (static redundancy + streaming $+3.06$ pt) with parallel phrasing, so neither is hidden.
- The interpretation column explicitly uses "**task-structure-dependent conditional value**" to name the nuanced stance — not "partial success" (too soft) or "mixed result" (too unstructured).

**Potential reviewer confusion**: a reader glancing only at the table might conclude P9 is a pure failure. The row is structured so that a second glance reveals the streaming validation. If the reviewer still objects, the fallback is to split P9 into two rows (static-P9 in §5.2, streaming-P11-in-context in §5.1), but this would fragment the narrowing narrative.

### Note C — P10 compactly summarizes §4.2.1's four-benchmark refutation

P10's cells are deliberately terse: four benchmark IDs, each with a one-phrase verdict. This is a **pointer**, not a replacement for §4.2.1 which contains the full tables. The translation preserves this density:

- "F-041 (Hopfield: $0\%$ detection) + F-068 (analogy scores $0.99+$ reject every positive) + F-070 Part A (F1 $0.000$ vs. $1.000$) + F-070 Part B (LoCoMo: $100\%$ RARE demoted)" — parallel structure, same verb-less style as the Japanese.
- The interpretation column summarizes the mechanism-supported / value-refuted two-layer stance in one sentence.

A reviewer who wants numerical detail is directed to §4.2.1; a reviewer skimming §5.2 gets the verdict at a glance.

### Note D — §5.1 P7 "4/5 exact predictions + 1 via alternate route"

The Japanese "4/5 完全予測一致 + 1 件別経路" is deliberately imprecise: the bias-detector predicted applicability exactly on 4 benchmarks, and on the 5th the prediction was correct but via a different decision path (the details are elsewhere). Preserving the imprecision is intentional honest reporting — it avoids overclaiming a 5/5 success.

**Translation choice**: "4/5 exact predictions + 1 via alternate route" — "via alternate route" is lightly informal but captures the Japanese "別経路". Academic alternatives like "on one via an alternate decision path" are more precise but wordier. For the table-cell context, the shorter form suffices.

### Note E — "withdrawn" (§5.2 P6) vs. "refuted" (§5.2 P10)

The Japanese uses two different verbs in §5.2:

- **P6 (external generalization failure)**: "一般への適用主張は**撤回**" → **withdrawn**
- **P10 (self-imposed canonical value)**: 4 benchmark で**反証** → **refuted**

The distinction is real and preserved:
- "Withdrawn" implies the claim was made and is now retracted (applies to general-OSS applicability after 3-repo average $\times 1.00$).
- "Refuted" implies the claim was tested against empirical data and contradicted (applies to canonical $(0.70, 0.80)$).

**Why the distinction matters**: a reviewer might object if both are called "refuted" (because "refuted" implies the kind of systematic testing that §4.2.1 documents; a 3-repo average is a single piece of counter-evidence, not a benchmark suite). Conversely, calling P10 "withdrawn" would imply we gave up on our canonical values without empirical pressure, which is inaccurate — the canonical refutation is backed by F-041 + F-068 + F-070 Part A + F-070 Part B.

### Note F — Terminology choices in §5

| Japanese | English used | Reason |
|---|---|---|
| 肯定的結果 | positive results | Direct rendering; standard academic term |
| 陰性結果 | negative results | Direct rendering; standard academic term |
| 誠実な記録 | an honest record | "Honest" over "candid" (which implies optional disclosure) |
| 業界既定の TTL | industry-default TTL | Direct rendering |
| PII マスク済 | PII-masked | Standard term |
| 時系列 replay | replayed in time order | "In time order" specifies temporal ordering, not random replay |
| empirical anchor | empirical anchor | Retained per style memory (metaphor is accepted academic English) |
| 局所 signal | local signal | Direct rendering |
| D5 型(構造非依存)| D5-type (independent of structure) | "D5" is a paper-internal classification; preserved |
| 冗長 | redundant | Standard term |
| 相対的に浮上 | relatively elevates | "Elevates" captures both the raising of rare items and their relative-to-others nature |
| task 構造依存の条件付き value | task-structure-dependent conditional value | Preserves the "conditional, not unconditional" epistemic stance |
| 事前適用判別 | a priori applicability | "A priori" matches the "before running" sense of "事前" and pairs naturally with "applicability" |
| zero-dependency | zero-dependency | Technical term; retained |
| 完全予測一致 | exact predictions | "Exact" over "perfect" (less promotional) |
| 別経路 | via alternate route | See Note D |

### Note G — What §5 does not claim

- §5 does **not** claim that the six positive results generalize beyond their tested domains — the paper's applicability matrix is in §7.
- §5 does **not** claim that bias-detector is a proven screening test — it is presented as a zero-dependency *metric* whose past-prediction record is 4/5.
- §5 does **not** classify P9 as either a pure positive or a pure negative — it is deliberately placed in §5.2 with the streaming positive noted inline, reflecting the "original claim refuted → narrowed claim validated" arc.
- §5 does **not** attempt to minimize P5/P6 by reducing their weight relative to P1/P2/P3. The negative results are presented at full magnitude, with the D5-type explanation giving a principled (not dismissive) account of why KDF fails on those tasks.
