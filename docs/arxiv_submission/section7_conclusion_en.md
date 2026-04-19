# §7 Conclusion — English translation (Step 6.6, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 378–504.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

§7 is the longest section of the paper. It binds together: (a) three central contributions, (b) the refined applicability predictor matrix, (c) the 2-benchmark × 2-model empirical matrix, (d) the F-061–F-065 domain-generalization verdicts, (e) the Phase X claim-level realistic benchmarks, (f) F-060's empirical validation of the complementary-architecture thesis via the Precision-Query Router, and (g) the realistic positioning pivot with honest limits. Translator's notes call out the self-refutation re-statement, the three-way tension between "F-044 was a simulation artifact" / "real KDF loses LongMemEval" / "real KDF wins LoCoMo temporal", and the final positioning pivot wording.

---

## 7. Conclusion

We presented the Knowledge Decay Framework (KDF) as a three-mechanism integrated architecture for operating information networks under finite resources. The reference implementation `cgb-kdf`, corresponding to claims 1–50 of patent JP 2026-027032, carries direct verification tests `test_claimN_*` for all 50 claims (56 tests total); the workspace excluding `kdf-python` / `kdf-wasm` passes 449 tests (measured 2026-04-18; see [`COMPLIANCE.md`](../patent/COMPLIANCE.md)).

Our central contributions are three:

1. **A qualitative systematization of the three-pillar structure across ten independent disciplines**, with KDF advanced as **a candidate implementation** of a domain-invariant architectural family (formal proof of "necessary convergence" not carried out). All three mechanisms of Claim 1 (metabolic control / rarity protection / integrity discovery) reach empirically backed status on realistic benchmarks via F-068 / F-069 / F-070.
2. **Three structural similarities** ($\delta k^4$ ↔ Ginzburg-Landau quartic term [functional-form match]; sandwich 2-threshold *mechanism* ↔ Hopfield spurious rejection [mechanism supported, canonical values refuted in §4.2 on four benchmarks]; $\exp(-\lambda \cdot dt)$ ↔ Markov survival probability [motivating analogy]) indicate **directions of theoretical connection** for KDF's engineering heuristics (rigorous theorem formulation is future work).
3. **Honest empirical reporting** across three streams: positive results (P1 / P2 / P3 / P7 / P8 = Claim 17 bit-exact; P11 = NASA streaming $+3.06$ pt; six total); generalization failures on comparison benchmarks (P5 / P6); and **refutations of canonical values we proposed ourselves (P9 Claim 5/14 static-task redundancy; P10 Claim 47–48 four-benchmark refutation of canonical values)** (four negatives total). We also provide the `bias-detector` a priori applicability metric. Self-refuting our own claim across four benchmarks serves the dual purpose of novelty narrowing and credibility strengthening.

KDF is **not a universal solution**. It cannot directly contribute to physical problems (climate change), economic problems (poverty), or social problems (war, educational inequality). **Moreover, the BEIR SciFact verification on 2026-04-18 (F-045) empirically confirmed that KDF is not applicable to general semantic retrieval (query–document matching)** (recall@10 $= 0.000$, below Random).

**The refined applicability predictor, established with the additional verifications of 2026-04-19 (F-061–F-065), is:**

> **"KDF decisively outperforms Random and baseline heuristics only under the condition that structural rareness correlates with task importance."**

Task type × KDF applicability matrix:

| Task | structural signal vs. importance | KDF applicability | Evidence |
|---|---|:-:|---|
| Path-based algorithms (APSP) | path-critical = bottleneck = rare | [True] | F-061 |
| Integration-point preservation (git merges, **repo merge rate $< 10\%$**) | merge = high degree = important | [True] | F-062, F-065 |
| LLM-memory temporal recall (long-context date/time) | rare date/time literal = structurally rare | [True] | F-057 / F-058 |
| Orphan-note detection (PKM) | orphan = deg $0$ = rare | [True] | F-012 / F-017 |
| **Merge-heavy repos (merge rate $> 20\%$)** | **merge $\neq$ rare; TopDegree wins** | [False] | F-065 pytest |
| **GP / kernel-regression inducing points** | **density center $\neq$ rareness** | [False] | F-063 |
| **Naive Python call-graph API** | **API = high in-degree; opposite of KDF Rare** | [False] | F-064 |
| Scale-free hub centrality | degree ≈ betweenness; TopDegree suffices | [False] | F-061 BA/WS |
| Metadata-based minority (cultural / semantic) | metadata independent of structure | [False] | F-047 |
| General semantic retrieval | semantic understanding is essential | [False] | F-045 |

KDF is effective only under the following restricted conditions:

- [True] the data can be represented as a graph;
- [True] **the query is unknown at retention time** (to be searched for later);
- [True] **protection of one-off / rarely-mentioned items** is the goal;
- [True] most of the data is redundant, a minority is unique, and labeling is difficult;
- [True] **task importance correlates with "structural rareness"** (newly refined via F-061–F-065).

Under these constraints, KDF is effective for **memory-curation tasks such as conversational memory and PKM** (P1: LongMemEval TTL $\times 7.7$; P2: Obsidian F1 $= 0.747$, $p = 0.006$; P3: NASA log $\times 2.3$).

**The initial F-044 measurement on 2026-04-18 turned out to be a simulation (the "KDF" numbers used a constant 0.821 recall value inherited from F-033, an earlier static LongMemEval measurement, rather than running the real KDF algorithm), as determined by F-052 / F-053; re-running with the real KDF completely reversed the conclusion on LongMemEval and yielded partial wins on LoCoMo:**

- **LongMemEval 500 Q (F-053)**: Real KDF $0.434$ vs. Mem0 $0.672$ (Mem0 $+23.8$ pt, $p < 10^{-16}$). Mem0 wins significantly in 5 of 6 categories.
- **LoCoMo 200 Q balanced (F-056)**: Real KDF $0.535$ vs. Mem0 $0.590$ (gap $-5.5$ pt, **$p = 0.24$, statistically tied**).
  - **Temporal category (LoCoMo 50 Q sub)**: KDF $0.460$ vs. Mem0 $0.240$ (**KDF $+22$ pt, $p = 0.035$ win**).
  - Mem0 wins narrative by $+24$ pt; factual / inferential are n.s.
- **LoCoMo temporal full 321 Q, gpt-4o-mini (F-057)**: Real KDF $0.312$ vs. Mem0 $0.206$ (**KDF $+10.6$ pt, $p = 0.0014$**, McNemar $b = 71$ / $c = 37$).
- **LoCoMo temporal full 321 Q, gpt-4.1-mini (F-058, model-robustness check)**: Real KDF $0.324$ vs. Mem0 $0.090$ (**KDF $+23.4$ pt, $p = 1.6 \times 10^{-14}$**, McNemar $b = 89$ / $c = 14$).
  - **Updating to the newer-generation model degrades Mem0 while KDF is preserved, widening the gap.**
  - Hypothesis: gpt-4.1-mini's fact extraction performs more aggressive compression and loses even more temporal information.
  - KDF's temporal advantage is **robust across 2 models × 321 Q, exposing a principled weakness of fact-extraction-based memory systems**.
- **LongMemEval 500 Q × gpt-4.1-mini (F-059, completing the 2×2 matrix)**: Real KDF $0.452$ vs. Mem0 $0.722$ (**Mem0 $+27.0$ pt, $p = 3.06 \times 10^{-23}$**, McNemar $b = 32$ / $c = 167$).
  - **The F-053 gap of $-23.8$ pt widens to $-27.0$ pt** (Mem0 strengthens on the new model; KDF marginally increases).
  - Per-category: Mem0 $+61$-pt landslide on single-session-assistant; temporal-reasoning gap $-6$ pt, $p = 0.23$ n.s. (KDF remains competitive on short-dialogue temporal queries).
  - → **LongMemEval is a benchmark that showcases Mem0's structural strength**; KDF's room to win is limited to n.s. categories (temporal-reasoning, single-session-preference).

**Completed 2 benchmarks × 2 models matrix (F-053 / F-057 / F-058 / F-059)**:

| benchmark × model | Mem0 | KDF | gap | $p$ | winner |
|---|---:|---:|---:|---:|:-:|
| LongMemEval 500 Q × gpt-4o-mini | 0.672 | 0.434 | $-0.238$ | $< 10^{-16}$ | Mem0 |
| LongMemEval 500 Q × gpt-4.1-mini | 0.722 | 0.452 | $-0.270$ | $3 \times 10^{-23}$ | Mem0 |
| LoCoMo temporal 321 Q × gpt-4o-mini | 0.206 | 0.312 | $+0.106$ | $1.4 \times 10^{-3}$ | KDF |
| LoCoMo temporal 321 Q × gpt-4.1-mini | 0.090 | 0.324 | $+0.234$ | $1.6 \times 10^{-14}$ | KDF |

→ **The benchmark-dependent division of labor is robust across models.** LLM-based memory (Mem0) is strong on short-dialogue general QA but has a structural weakness for long-conversation date/time recall. KDF is positioned as a dedicated complementary layer for the latter.

**F-061–F-065 — Domain-generalization experiments (2026-04-19, zero added cost)**:

F-044–F-060 validates only the LLM-memory domain. To position KDF as a **general-purpose graph compression technique**, we verified on four additional domains:

| Finding | Task | Result | Implication |
|---|---|---|---|
| **F-061** | Betweenness / APSP on 4 synthetic graphs | **Mixed**: KDF wins on ER / SBM, loses BA / WS for betweenness; wins APSP on all 4 for path distance | TopDegree wins on scale-free graphs; KDF wins on uniform / community graphs |
| **F-062** | Git-commit pruning (tokio, 4752 commits) | **Positive**: merge recall $99.5\%$ at $30\%$ keep | Validated for commercial git archival |
| **F-063** | Gaussian Process inducing points (California housing, Friedman1) | **Negative**: KDF < Random < KMeans on GP fit | Density estimation is not applicable to KDF |
| **F-064** | Naive call-graph API preservation (flask, Python `ast`) | **Negative**: KDF $16\%$ vs. Random $41\%$ | Public APIs have high in-degree — opposite of KDF Rare |
| **F-065** | Git-pruning 3-repo replication (tokio / pytest / lodash) | **Partial**: merge recall depends on the repo's merge rate (tokio $99\%$ / pytest $59\%$ / lodash $100\%$) | KDF matches TopDegree only when "merges are rare" in the repo |

Combining these negative findings, **we establish the decisive predictor of KDF applicability**: "does structural rareness correlate with task importance?". LLM-memory temporal recall, path-based algorithms, and OSS-library-style git repos satisfy the correlation → KDF is effective. Density estimation, API-boundary detection, and merge-heavy enterprise repos do not → KDF is not applicable.

**Phase X — Claim-level realistic benchmarks (2026-04-19, zero added cost)**:

After F-068 brought all three mechanisms of Claim 1 to empirically-backed status, we ran realistic benchmarks for the remaining claim groups as Phase X:

| Finding | Task | Result | Implication |
|---|---|---|---|
| **F-069** | Claim 5 / 14 / 17 on LoCoMo temporal 321 Q | **Mixed**: C17 distributed-execution bit-exact pass; C5 / C14 inferior to KDF_static on static tasks (all time-aware conditions have $\Delta \le 0$) | Time signals are subsumed by structural rareness and redundant on static tasks; streaming is the true use case (verified in F-072) |
| **F-070** | Claim 36–41 T_wait + Claim 47–48 sandwich, realistic | **Mixed**: mechanism [True] / canonical values [False]. Four-benchmark refutation of $(0.70, 0.80)$, F1 $0.000$ vs. F1$((0.70, 1.00)) = 1.000$; LoCoMo streaming $100\%$ RARE demoted under canonical | The 2-threshold *mechanism* is maintained as novel; specific canonical values require domain-specific calibration |
| **F-071** | Claim 20–32 dynamic control on LoCoMo streaming (lightweight) | **Mechanism [True] / no selection benefit**: Claim 21 $5:3:1$ integer tick exact; MetaController $\alpha$-bound clamp working; TransitionController ceiling-effected (F-031 confirmed) | Mechanism-only validation; the true value shows on NASA-type streaming (verified in F-072) |
| **F-072** | Claim 14 / 25 / 27–32 on NASA HTTP 50 k streaming replay | **[True] Claim 14 $+3.06$ pt** over static baseline (C1 decay $0.4898$ vs. C0 $0.4592$); [Warning] Claim 25 activation neutralizes on evenly distributed rare; Claim 27–32 are selection-neutral (as predicted) | **The paper narrowing "streaming is the true use case" is empirically validated.** The first positive evidence for Claim 14's value proposition. ActivationScore requires discrimination of the rare-event temporal pattern |

These Phase X findings both strengthen paper credibility by "self-refuting our own canonical values on four benchmarks" and, via F-072, provide a **positive empirical anchor after the narrowing**. Two follow-up directions are explicitly indicated as future work: "domain-calibrated parameter auto-tuning", and "conditional use of Claim 25 activation based on the rare-event temporal pattern".

**F-060 — Empirical validation of a complementary architecture via the Ext-1 Precision-Query Router (2026-04-19, zero added cost)**:

Applying the following routing logic post-hoc to the existing data of F-053 / 057 / 058 / 059:

```
if is_precision_query(q) and conversation_length >= 100 turns:
    use KDF answer
else:
    use Mem0 answer
```

Result (v2 = precision + long context):

| cell | Mem0 alone | Router (v2) | gain | $p$ |
|---|---:|---:|---:|---:|
| LongMemEval 500 Q × gpt-4o-mini | 0.672 | 0.672 | 0.000 | 1.00 (safe) |
| LongMemEval 500 Q × gpt-4.1-mini | 0.722 | 0.722 | 0.000 | 1.00 (safe) |
| LoCoMo temporal 321 Q × gpt-4o-mini | 0.206 | **0.302** | **$+0.097$** | 0.003 [Significant] |
| LoCoMo temporal 321 Q × gpt-4.1-mini | 0.090 | **0.315** | **$+0.224$** | $4 \times 10^{-14}$ [Significant] |

*Note on precision: the columns "Mem0 alone" and "Router (v2)" are 3-decimal rounded for readability; the "gain" column is computed from the 4-decimal raw scores and may therefore differ from naive subtraction of the displayed values by ±0.001.*

The Router is **greater than or equal to Mem0 alone across all four cells** (strictly-better property); on long-conversation precision queries, the improvement reaches $+22.4$ pt. LLM-API call count is reduced by $97\%$ on long conversations. This is the first quantitative validation of this paper's architectural thesis that KDF should be designed as **a complementary layer to Mem0, not a replacement**.

- **KDF's benchmark-dependent behavior**: in long conversations (LoCoMo's 300–700 turns/conv, with date/time information scattered across raw turns), KDF's raw-turn retention is advantageous; in short dialogues (LongMemEval's 20–30 turns/Q), Mem0's fact extraction dominates.

Therefore, **KDF's realistic positioning** pivots to the following:

1. **Long-conversation temporal-recall use cases** (F-056: $+22$ pt validated) — meeting minutes, month-scale journals, year-scale conversation histories where date/time information must be referenced; KDF's raw-turn retention is decisively advantageous.
2. **Environments where cost / latency / privacy / determinism take priority over accuracy** — local-first chatbot, air-gapped agent, $< 1$-ms real-time memory gating, budget-constrained deployment, deterministic regulated output.
3. **Retention strategies smarter than TTL** (real 500 Q: KDF recall $0.665$ vs. TTL_recent $0.180$, $\times 3.7$).

**Honest limits**: in the general LLM-memory market of LongMemEval type (short dialogue + concise questions), KDF does not reach Mem0's accuracy. KDF use in this market is limited to cases that tolerate the accuracy trade-off.

For other domains, we honestly declare what KDF can and cannot do, and provide it in an openly verifiable form. In particular, **KDF use in the general-retrieval market is explicitly discouraged** (F-045).

---

## Translator's notes on §7

### Note A — Self-refutation re-statement and its positional weight

§7 restates the self-refutation in contribution (3), using essentially the same wording as Abstract P3 and §1.3 C3. This is deliberate redundancy: a reviewer who reads only the Abstract or only §7 must encounter the self-refutation.

**Translation choices**:
- "**refutations of canonical values we proposed ourselves**" matches Abstract's wording exactly ("refute ... canonical values ... we ourselves").
- "Self-refuting our own claim ... serves the dual purpose of novelty narrowing and credibility strengthening" — this closing sentence of contribution (3) is load-bearing and is rendered in active voice without softening.

### Note B — The three-way tension: "F-044 was simulation" / "LongMemEval lost" / "LoCoMo won"

This is the paper's most intricate credibility passage. §7 must simultaneously:

1. **Admit** that F-044 (the initial positive headline result) was a simulation artifact, not real KDF.
2. **Report** that real KDF loses LongMemEval by $-23.8$ pt → $-27.0$ pt (two models).
3. **Report** that real KDF wins LoCoMo temporal by $+10.6$ pt → $+23.4$ pt (two models).
4. **Frame** the result so that (2) is not dismissed and (3) is not overclaimed.

**Translation choice**: the paragraph opens with the **admission** ("turned out to be a simulation ... as determined by F-052 / F-053"), then the **reversal** ("completely reversed the conclusion on LongMemEval"), then the **partial win** ("yielded partial wins on LoCoMo"). This sequencing — admission → negative reversal → partial positive — preserves the Japanese source's structure and keeps the reader from being "sold" a rescue narrative before seeing the loss.

The subsequent per-finding bullets maintain strict parallelism (winner, gap, p-value, per-category where applicable). The summary 2×2 matrix table gives a single-glance verdict. The interpretation ("Mem0 is strong on short-dialogue; KDF is a complementary layer for long-conversation date/time") appears **after** the numerical reporting, never before.

### Note C — "The first quantitative validation of ... the architectural thesis"

The F-060 Router paragraph ends with the sentence: "This is the first quantitative validation of this paper's architectural thesis that KDF should be designed as a complementary layer to Mem0, not a replacement."

**Why this wording matters**: the Japanese "補完 layer として設計する本研究の architectural thesis の最初の定量的実証" is the paper's **positioning pivot sentence**. The entire paper's commercial-positioning arc flows from this sentence:

- Before F-060: KDF is a replacement (which it is not, per F-053).
- After F-060: KDF is a complementary layer (which F-060 empirically validates).

**Translation choice**: we preserved "first quantitative validation" (最初の定量的実証), "architectural thesis" (architectural thesis), and "complementary layer ... not a replacement" (補完 layer として設計する ... not a replacement) because each phrase is load-bearing and has matching phrasing elsewhere in the paper.

### Note D — "Honest limits" section

The final paragraphs — "Honest limits" + "For other domains ... honestly declare ... openly verifiable form" + "general-retrieval market is explicitly discouraged" — close §7 on a honesty note rather than a pitch note.

**Translation choice**: we rendered "正直な限界" as "**Honest limits**" (bold phrase) rather than "frank limitations" or "acknowledged limitations" because "honest" is the paper's consistent stance vocabulary (feedback_decision_framework.md: 誠実性 / honesty). Ending §7 on "explicitly discouraged" — rather than a softened "may not be optimal" — matches the Japanese "明示的に非推奨" at its original stringency.

### Note E — Table preservation fidelity

§7 contains **five tables** (applicability matrix, 2×2 Mem0-vs-KDF matrix, F-061–F-065, Phase X F-069–F-072, F-060 Router) plus multiple numerical bullets. All numerical values, p-values, McNemar $b/c$ counts, and $\pm$ percentages are preserved exactly. The tables are arxiv-format markdown; a review-time conversion to LaTeX `tabular` is straightforward (planned in Step 6.7).

Applicability markers in the matrix tables use **bracketed textual labels** ([True] / [False] / [Warning] / [Significant]). Originally emoji (✅ / ❌ / ⚠️ / ★), they were hardened to text in Step 6.7 to avoid Unicode compile failures under arxiv's default pdflatex pipeline (see `harden_for_arxiv.py` and BUILD.md). The labels are semantically equivalent to the original emoji and function as identical at-a-glance verdicts.

### Note F — Terminology choices in §7

| Japanese | English used | Reason |
|---|---|---|
| 領域不変アーキテクチャ族 | domain-invariant architectural family | Consistent with §1 / §6 usage |
| 候補となる一実装 | a candidate implementation | Not "the" implementation — the paper explicitly frames KDF as one of a family |
| 万能でない | not a universal solution | Standard academic English phrasing of "万能" |
| 明示的に非推奨 | explicitly discouraged | Direct rendering; matches "明示的に" stringency |
| structural rareness | structural rareness | Consistent with §1 / refined-predictor usage |
| benchmark-dependent | benchmark-dependent | Standard ML term |
| refined predictor | refined predictor | Retained per paper's naming |
| strictly-better property | strictly-better property | Standard game-theory / optimization term |
| raw-turn retention | raw-turn retention | Distinguishes KDF's approach from fact-extraction |
| fact-extraction-based memory | fact-extraction-based memory | Paper's term for the Mem0-style approach |
| LLM-API call count $97\%$ 削減 | LLM-API call count is reduced by $97\%$ | "Reduced by" is the standard phrasing |
| 正直な限界 | Honest limits | Bold retained; "honest" over "candid" (see Note D) |
| budget-constrained deployment | budget-constrained deployment | Standard term |
| deterministic regulated output | deterministic regulated output | Compliance-relevant positioning term |

### Note G — What §7 does not claim

- §7 does **not** claim that KDF is universal. The "not a universal solution" disclaimer is reiterated, with F-045 BEIR SciFact as specific supporting evidence.
- §7 does **not** claim that the refined applicability predictor is exhaustive — 10 task types are enumerated, but the matrix is marked as current-best-knowledge, not final.
- §7 does **not** claim that KDF wins LongMemEval. The 2×2 matrix clearly shows Mem0 wins both LongMemEval cells.
- §7 does **not** claim that F-060's Router is a proprietary algorithm — it is presented as post-hoc routing over existing data, demonstrating the complementary architecture rather than introducing a new system.
- §7 does **not** soften the "short-dialogue accuracy trade-off" — the "KDF does not reach Mem0's accuracy" phrasing is kept explicit.
