# §4 Three Structural Similarities — English translation (Step 6.4, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 173–288.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

This is the **most delicate section** of the paper. Three structural correspondences are claimed between KDF's distinctive formulas and established results in physics, associative-memory theory, and continuous-time stochastic processes. For each, the paper:

1. **Asserts** a functional-form match.
2. **Hedges** that the match is qualitative (not a mathematical isomorphism).
3. **Self-refutes** — explicitly in §4.2 / §4.2.1 — where empirical tests contradict the original conjecture.

The translation preserves each of these three moves at its original strength. A pitch-style rewrite would collapse (2) and hide (3); we do neither. Translator's notes at the end flag the specific passages most at risk of being softened in casual reading.

---

## 4. Three Structural Similarities of KDF

### 4.1 $\delta k^4$ Meta-control ↔ Ginzburg-Landau Quartic Term

The Ginzburg-Landau free energy in condensed-matter physics, for an order parameter $\psi$ near a phase transition, is expanded as

$$
F(\psi) = \alpha_2 \psi^2 + \alpha_4 \psi^4 + \cdots .
$$

**The quartic term $\alpha_4 \psi^4$ provides the indispensable restoring force that stabilizes the order parameter at the critical point** (with only the quadratic term, the potential is unbounded below as a square).

KDF's meta-control law $\Delta \alpha \propto \delta k^4$ matches the Ginzburg-Landau-type potential in **functional form** on two points: (a) identifying the deviation $\delta k$ with the order parameter, and (b) generating a quartic restoring force. (This is a functional-form-level correspondence, not a claim of mathematical isomorphism.) Implications:

- KDF's meta-control is implicitly **steering the network toward self-organization at a critical point**.
- The power-law avalanche distribution predicted by SOC (Self-Organized Criticality) theory may also be expected in KDF (not yet verified; future work).
- Answer to "why the fourth power": it is the lowest-order non-trivial restoring force near a phase transition (a physical necessity).

**日本語バックトランス要約**: Ginzburg-Landau 自由エネルギー $F = \alpha_2 \psi^2 + \alpha_4 \psi^4 + \cdots$ の 4 次項が秩序変数を臨界点で安定化する不可欠な復元力を与える。KDF のメタ制御 $\Delta \alpha \propto \delta k^4$ は (a) 偏差を秩序変数と同一視、(b) 4 次復元力を生成、の 2 点で関数形一致(数学的同型の主張ではなく、関数形レベル)。含意: KDF は暗黙的に臨界点自己組織化へ制御、SOC の power-law 分布が期待される(未検証)、4 乗は臨界点近傍の最低次非自明復元力(物理的必然性)。

---

### 4.2 Sandwich Upper Threshold $\theta_U$ ↔ Hopfield Spurious-Attractor Rejection

**Classical problem in Hopfield associative memory.** Storing memory patterns $\xi^{(1)}, \ldots, \xi^{(P)}$ causes linear combinations of them to emerge as **spurious attractors**—fixed points that are not themselves learned patterns but behave as false memories. Amit, Gutfreund, and Sompolinsky [@amit1987replica], using the replica analysis of spin-glass models, showed that when the critical capacity $\alpha_c \equiv P/N \approx 0.138$ is exceeded, spurious attractors proliferate and associative recall collapses. Related phenomena—where repeatedly presented or biased-sampled patterns form broad basins of attraction and obscure other memories, discussed in the context of biased/correlated-pattern capacity—are also closely tied to the spurious problem.

**No general method for spurious detection has been established in over 40 years.** Modern Hopfield networks [@ramsauer2021hopfield] exponentially improved capacity via a softmax mechanism, but a widely accepted general principle for spurious suppression has not yet been presented (this paper does *not* claim that KDF has solved this; see the conjecture at the end of §4.2).

**KDF's upper threshold $\theta_U$ formalizes the following heuristic.** When the integrity score $S$ is "too perfect" (for example, $S > 0.80$), the candidate is most likely one of:

1. A self-loop or trivial duplicate (rediscovery of an existing neighbor),
2. An overfitting / memorization artifact, or
3. **A spurious attractor** (a false fixed point).

By restricting the acceptance region to $[\theta_L, \theta_U]$, these cases are statistically rejected.

**Mathematical hypothesis.** Integrity scores of random pairs follow a distribution $p(S)$; true analogies concentrate near the middle of $[\theta_L, \theta_U]$, while spurious attractors concentrate in the right tail $S > \theta_U$. Upper-bound rejection corresponds to a tail-cut statistical decision boundary.

**Phase V3 empirical verification (2026-04-18).** On a 100-neuron Hopfield network (Hebbian learning, $N = 100$, $P \in \{5, 10, 14, 18, 22\}$), we implemented and measured a $\theta$-filter that rejects the post-recall state as spurious when it has cosine similarity $\ge \theta$ to multiple learned patterns:

| Detection threshold $\theta$ | Load factor $P$ | Spurious-rejection rate | effective_recall improvement |
|---|:-:|---:|---:|
| 0.80 (KDF canonical) | 18 | **0%** | 0 |
| 0.70 | 18 | 0% | 0 |
| 0.55 | 18 | 0% | 0 |
| 0.40 | 18 | **24%** | 0.49 → 0.65 (+32%) |
| 0.40 | 22 | **40%** | 0.34 → 0.56 (+65%) |

**Findings:**

- [True] **The mechanism (multi-pattern similarity rejection) is supported**: an appropriately chosen $\theta$ can reject $24\%$ of Hopfield mixture spurious states at load factor $P = 18$, and up to $40\%$ at $P = 22$ (further above the critical capacity $\alpha_c \approx 0.138$, where spurious attractors proliferate).
- [False] **The simple claim of transplanting KDF's canonical $\theta_U = 0.80$ directly to Hopfield was experimentally refuted**: cosine similarities between a Hopfield mixture state and its constituent patterns each stay around $\sim 0.4$, undetectable at a $0.80$ threshold.

**Refinement of the paper's original claim.**

- The original conjecture—"KDF's canonical value $\theta_U = 0.80$ is the principled-level answer to the Hopfield spurious problem"—**is refuted**.
- The revised conjecture—"the **upper-threshold mechanism** is also effective for suppressing Hopfield spurious attractors, but the **specific value must be tuned domain-specifically**"—is **partially supported**.

Implementation: [`demos/D7_github_issue/src/bin/phase_v3_hopfield_theta_u.rs`](../../demos/D7_github_issue/src/bin/phase_v3_hopfield_theta_u.rs). Full record: [VERIFIED_FINDINGS.md F-041](../VERIFIED_FINDINGS.md).

Integration verification with modern Hopfield networks [@ramsauer2021hopfield] is future work (a minimal experiment for methodology calibration is completed in this phase).

**日本語バックトランス要約**: Hopfield 連想記憶の古典問題(spurious attractor、@amit1987replica、$\alpha_c \approx 0.138$、40 年以上未解決)を設定。KDF の上限閾値 $\theta_U$ は「$S$ が完璧すぎる candidate は spurious」経験則を形式化。Phase V3 実験: canonical $\theta_U = 0.80$ で 0% detect、$\theta = 0.40$ で 24-40% detect。発見: 機構は [True] 支持、canonical 値は [False] 反証。narrowing: 原形「canonical = 原理的解答」は反証、修正「機構は有効、値は domain-specific 調整必要」は部分的支持。

---

#### 4.2.1 Phase X Step 2 Additional Verification — Completing the Four-Benchmark Refutation (F-070, 2026-04-19)

Following the partial refutation of F-041, we systematically re-examined the practical applicability of canonical $(\theta_L, \theta_U) = (0.70, 0.80)$ across three additional benchmarks. Together, they establish four-benchmark cross-cutting evidence.

**Part A: Sandwich sensitivity analysis on synthetic + F-068 replicated pairs.** We captured the raw scores (with the permissive threshold 0.0) of `AnalogyDiscoveryEngine::find_analogy` on 38 pairs in total — 22 positive (3 Gentner classics including solar system ↔ atom, 4 git ↔ paper cross-domain correspondences, 15 synthetic isomorphic pairs) and 16 negative (1 hand-curated non-isomorphic control, 15 synthetic non-isomorphic pairs) — and post-hoc applied five sandwich conditions $\{(0.70, 0.75), (0.70, 0.80), (0.70, 0.90), (0.70, 0.95), (0.70, 1.00)\}$. Results:

| $(\theta_L, \theta_U)$ | TP | FN | TN | FP | F1 |
|---|---:|---:|---:|---:|---:|
| $(0.70, 0.75)$ | 0 | 22 | 16 | 0 | 0.000 |
| **$(0.70, 0.80)$ canonical** | **0** | **22** | **16** | **0** | **0.000** |
| $(0.70, 0.90)$ | 0 | 22 | 16 | 0 | 0.000 |
| $(0.70, 0.95)$ | 0 | 22 | 16 | 0 | 0.000 |
| $(0.70, 1.00)$ | 22 | 0 | 16 | 0 | **1.000** |

**Finding.** Positive-analogy scores saturate uniformly at $\ge 0.99$ (fingerprint identity for graph isomorphism), while negatives distribute around $\sim 0.57$–$0.60$. Because canonical $\theta_U = 0.80$ rejects every positive, F1 is 0.000; conversely, $\theta_U = 1.00$ (no effective upper bound) yields complete separation via $\theta_L = 0.70$ alone and reaches F1 = 1.000. **No empirical scores fall within the "middle band" that 0.80 would admit.**

**Part B: KdfProcessorRev12 review loop on LoCoMo temporal 30 Q.** Fixing $(t_{\text{wait1}}, t_{\text{wait2}}) = (30, 30)$, we ran the review cycle under three conditions $\theta_U \in \{0.80, 0.90, 1.00\}$ up to 65 cycles. The spoke_up / demote distribution across the total 1,140 RARE nodes (of which 8 belong to answer turns):

| $\theta_U$ | total RARE | answer-RARE | spoke_up (ans) | demoted (ans) | spoke_up (non-ans) | demoted (non-ans) | avg cycles |
|---|---:|---:|---:|---:|---:|---:|---:|
| **0.80 canonical** | 1140 | 8 | **0** | **8 (100%)** | 0 | **1132 (100%)** | 60.0 |
| 0.90 | 1140 | 8 | 8 (100%) | 0 | 1132 (100%) | 0 | 1.0 |
| 1.00 | 1140 | 8 | 8 (100%) | 0 | 1132 (100%) | 0 | 1.0 |

**Finding.** Under canonical $\theta_U = 0.80$, every RARE node in the LoCoMo chain graph is demoted to Garbage after the 60-cycle timeout (complete information loss). Under $\theta_U \ge 0.90$, all RARE nodes spoke up within one cycle, but there is no discrimination between answer and non-answer turns, so the filter does not function. **The sparse chain structure of LoCoMo cannot admit a discriminative middle band under the sandwich.**

**Cross-benchmark verdict:**

| Benchmark | Domain | Verdict on canonical $(0.70, 0.80)$ |
|---|---|---|
| F-041 Hopfield mixture | Associative memory | 0% detection at $\theta_U = 0.80$; $24\%$ at $P = 18$ and $40\%$ at $P = 22$ only after lowering to $\theta = 0.40$ |
| F-068 analogy-engine direct | Graph isomorphism | Scores at $\ge 0.99$; canonical rejects every true positive |
| F-070 Part A (38 pairs) | Synthetic + F-068 replication | F1(canonical) $= 0.000$; F1$((0.70, 1.00)) = 1.000$ |
| F-070 Part B (LoCoMo streaming) | 30 Q × 1,140 RARE | 100% RARE demoted under canonical → complete information loss |

→ **Across four benchmarks, canonical $(\theta_L, \theta_U) = (0.70, 0.80)$ loses practical value.** True positive-analogy scores concentrate at $\ge 0.95$ and negatives at $\le 0.65$, so the empirically correct sandwich is $(0.70, 1.00)$ (effectively no upper bound; only $\theta_L$ active) or at the $(0.90, 1.00)$ level.

**Final narrowing of the claim.**

- [False] Original conjecture — "canonical $(0.70, 0.80)$ is the principled-level answer to Hopfield spurious / over-similar candidates" — **is refuted across four benchmarks**.
- [True] Novelty of the 2-threshold *mechanism* (the structure that admits only the middle band by simultaneously imposing lower and upper bounds) has no counterpart among the ten related disciplines we surveyed; the novelty claim is **maintained at the mechanism level**.
- [Calibration Required] Specific canonical values require domain-specific empirical calibration. The score distributions from F-068 / F-070 suggest $\theta_U \ge 0.95$ for graph-isomorphism regimes and $\theta \le 0.5$ for associative-memory regimes.

Implementation: [`demos/D8_llm_memory/src/bin/phase_x2_sandwich_twait_locomo.rs`](../../demos/D8_llm_memory/src/bin/phase_x2_sandwich_twait_locomo.rs). Full record: [VERIFIED_FINDINGS.md F-070](../VERIFIED_FINDINGS.md).

**日本語バックトランス要約**: F-041 の部分反証を 3 追加 benchmark で完結。Part A 38 pair の sandwich 感度分析: canonical F1=0.000(全 true positive 棄却)、$(0.70, 1.00)$ F1=1.000、0.80 に該当する中間帯 score は empirically 不在。Part B LoCoMo 30 Q Rev12: canonical で全 1140 RARE が 60-cycle timeout 経て Garbage demote(情報完全喪失)、$\theta_U \ge 0.90$ で discrimination 消失。4 benchmark 横断 verdict で canonical は実用的 value を喪失。narrowing は 3 本立て: [False] 原形 conjecture 反証 / [True] 機構 novelty 維持 / [Calibration Required] 値は domain-specific 較正必要(graph iso で $\theta_U \ge 0.95$、associative memory で $\theta \le 0.5$)。

---

### 4.3 $\exp(-\lambda(C) \cdot dt)$ ↔ Locally-Varying Continuous-Time Markov Chain

The survival probability of a continuous-time Markov process $X_t$ (with exponentially distributed residence times) is generally written as

$$
\Pr[\text{no jump in } [t, t+dt]] = \exp(-\Lambda(x) \cdot dt) .
$$

KDF's weight update $w \leftarrow w \cdot \exp(-\lambda(C) \cdot dt)$ has **the same functional form**. If we interpret $\lambda(C)$ as a local "jump rate", KDF's edge decay corresponds to the **motif of a survival process**—"the probability that an edge is lost in an infinitesimal interval follows $1 - \exp(-\lambda(C) \cdot dt)$".

**However, strict CTMC equivalence does not hold.** In KDF, (a) $\lambda(C)$ changes as adjacent edges are pruned, so the generator is non-stationary; (b) the weight $w$ is a continuous quantity, not a probability mass; (c) edge protection (Rare) depends on global conditions; and so on — the standard CTMC assumptions are not satisfied. This section's claim therefore remains at the level of **motivation for a theoretical connection (a motivating analogy)**; ergodic results such as stationary distributions or spectral gaps cannot be imported directly.

What this structural correspondence suggests is the following — **directions for future theoretical development**, not theorems:

- Claim 46's Laplacian eigenvalue fingerprint is **conceptually consistent** with the attempt to project ego-graph spectral-gap information into a 32-dimensional space (an exact information-preservation proof has not been carried out).
- Theoretical connections to diffusion maps, PageRank, and spectral clustering are future work.

**日本語バックトランス要約**: CTMC のサバイバル確率 $\Pr[\text{no jump}] = \exp(-\Lambda(x) dt)$ と KDF の重み更新 $w \leftarrow w \cdot \exp(-\lambda(C) dt)$ は関数形として同じ。$\lambda(C)$ を局所 jump 率と見なせばサバイバル過程 motif と対応。ただし **厳密な CTMC 同値性は成立しない**(generator 非定常、$w$ は確率質量でない、Rare 保護が大域条件依存、等)。理論接続の motivating analogy にとどまり、stationary distribution や spectral gap の ergodic 結果は直接輸入不可。示唆: Claim 46 Laplacian 固有値フィンガープリントは ego-graph spectral gap 情報の 32 次元射影と概念的整合(正確な情報保存の証明は未実施)、diffusion maps / PageRank / spectral clustering との理論接続は future work。

---

## Translator's notes on §4

### Note A — The three-move structure of §4 (assert / hedge / self-refute)

Each of §4.1, §4.2 (+4.2.1), and §4.3 follows a three-move pattern that the translation preserves uniformly:

| Move | §4.1 | §4.2 / §4.2.1 | §4.3 |
|---|---|---|---|
| **1. Assert a match** | "match ... in functional form on two points" | "formalizes the following heuristic" / "mathematical hypothesis" | "has the same functional form" |
| **2. Hedge** | "(This is a functional-form-level correspondence, not a claim of mathematical isomorphism.)" | Implicit via the "heuristic" framing (and §1.2's blanket hedge) | "**However, strict CTMC equivalence does not hold.**" |
| **3. Self-refute (where applicable)** | None (SOC power-law "not yet verified") | Full: F-041 + §4.2.1 F-070 four-benchmark verdict | N/A — this section is up-front about being "motivating analogy only" |

**Why this matters for review acceptance.** A reviewer skimming only the "assert" moves could see §4 as overreach. A reviewer skimming only the "hedge" and "self-refute" moves could see §4 as under-confident. The three-move structure is the paper's central honesty mechanism: claim → qualify → test and report. The translation is careful to keep all three moves at equal visibility in the English — no hedges pushed into footnotes, no self-refutations reduced to parentheticals.

### Note B — "Principled-level answer" vs "principled solution" (§4.2, §4.2.1)

The Japanese "原理的解答" was softened in a past edit (commit `170354a Add AI verification bundle + soften 原理的解答 phrase`) — the original was stronger ("the answer"), and the softened form is "principled-level answer". The English translation uses **"principled-level answer"** (matching the softened Japanese), not "principled solution" (which in English carries a stronger "general solution" connotation).

This is important because:
- In §4.2 and §4.2.1, the original conjecture is already framed with this softened wording, acknowledging partial rather than total resolution.
- The refutation then targets the softened conjecture, not a strawman of a stronger claim.

"Principled-level answer" may read slightly awkward to a native English reader, but the awkwardness is load-bearing: it signals that even the original conjecture was epistemically modest, which makes the subsequent refutation more credible rather than cruel.

### Note C — "40 年以上確立されていない" as a claim of openness, not of KDF supremacy

§4.2 states that no general method for Hopfield spurious detection has been established in over 40 years, followed *immediately* by "(this paper does *not* claim that KDF has solved this; ...)". This juxtaposition is critical:

- The 40-year claim establishes the **importance** of the open problem, not the **stature** of KDF.
- Without the immediate "does *not* claim" disclaimer, a reader could interpret the paragraph as positioning KDF as the long-awaited solution.

**Translation choice**: the disclaimer is placed in the same sentence (not split across sentences or pushed to a later paragraph), as in the Japanese source. The italicized "*not*" emphasizes the disclaimer without over-formalizing it.

### Note D — §4.2.1 three-bullet final narrowing ([False] / [True] / [Calibration Required])

The tripartite bullet structure at the end of §4.2.1 is the paper's clearest statement of its final epistemic stance on the sandwich claim:

- **[False]** — original conjecture refuted
- **[True]** — mechanism novelty maintained
- **[Calibration Required]** — specific values require domain-specific calibration

Originally we used emoji markers (❌ / ✅ / 🔧) as a visual anchor; in Step 6.7 they were replaced with bracketed textual labels to avoid Unicode compile failures under arxiv's default pdflatex (see `harden_for_arxiv.py`). The trichotomy structure is preserved — refuted / maintained / conditional remain three distinct epistemic states rather than being flattened into prose. If a later revision re-enables XeLaTeX, the emoji markers can be restored by reverting the hardening pass; the bracket labels are forward-compatible with any compile target.

### Note E — "motivating analogy" treatment in §4.3

§4.3 is the only subsection that admits up-front that the correspondence is a **motivating analogy** rather than a provable theorem. The Japanese "motivating analogy" is already an English loanword in the source; the translation simply retains it. We did not try to render it as "heuristic analogy" or "suggestive analogy" — the "motivating" framing (used in the physics and math-methods literature) conveys that the analogy drives further inquiry rather than closing it.

**Important structural point**: §4.3 does not have a self-refutation subsection analogous to §4.2.1. This is deliberate: the Markov correspondence was never claimed to be more than analogical, so there is nothing to refute. A reviewer asking "why is §4.2 refuted but §4.3 not?" should read §4.3 as pre-hedged, not as spared from scrutiny.

### Note F — Terminology choices in §4

| Japanese | English used | Reason |
|---|---|---|
| 関数形が一致する | have the same functional form / match in functional form | Avoids "are equal" (which would imply isomorphism); "functional form" is the standard condensed-matter phrasing |
| 数学的同型 | mathematical isomorphism | Standard term; used consistently in §1.3 C2 |
| 臨界点における自己組織化 | self-organization at a critical point | Standard SOC terminology |
| 原形 conjecture | original conjecture | "Original" rather than "initial" to stress that it predates empirical testing |
| 精密化 | refinement | Used over "revision" because the original conjecture is not abandoned wholesale — the mechanism-level claim survives |
| サンドイッチ 採用域 | sandwich acceptance condition / sandwich mechanism | Style memory prohibits "band"; in §4 context "acceptance condition" fits the mathematical exposition |
| 中間帯 | middle band | Style-memory exception: "middle band" is used when explicitly describing *what* the sandwich admits (the range of scores), not when naming the *mechanism*; this is distinct from the prohibited "acceptance band" |
| 上限棄却 | upper-bound rejection | Direct rendering |
| 整合性スコア | integrity score | Consistent with §1 / §2 |
| 吸引盆地 | basin of attraction | Standard dynamical-systems term |
| レプリカ解析 | replica analysis | Standard spin-glass term |
| サバイバル過程のモチーフ | motif of a survival process | "Motif" preserves the analogical strength of "モチーフ" (design-pattern-like, not exact) |
| 概念的に整合 | conceptually consistent | Not "aligned with" (too strong) |
| 今後の理論展開の方向性 | directions for future theoretical development | Direct rendering; emphasizes *directions*, not results |

### Note G — Decision: "middle band" vs style-memory prohibition on "band"

The style memory prohibits "band" / "acceptance band" as renderings of "sandwich 採用域" (the mechanism name). However, when describing the *content* of the acceptance region — i.e., the scores in the range $[\theta_L, \theta_U]$ — the English phrase "middle band" is the most natural rendering of "中間帯" and is used in §4.2.1 ("no empirical scores fall within the middle band that 0.80 would admit", "cannot admit a discriminative middle band").

**Justification for the exception**: the style-memory prohibition targets the *name* of the mechanism (where "sandwich 2-threshold mechanism" is preferred to "sandwich acceptance band"). The term "middle band" used here is descriptive, not the mechanism's name. It appears only inside prose explanations of score distributions, never as the section heading or the primary term introduced.

If the user disagrees with this interpretation, the fallback is to replace "middle band" with "intermediate score range" or "between-threshold region" — both are correct but less fluent.

### Note H — What §4 does not claim

A compact checklist for reviewer assurance:

- §4 does **not** claim that any of the three correspondences is a proven isomorphism.
- §4 does **not** claim that KDF has solved the Hopfield spurious-attractor problem.
- §4 does **not** claim the Ginzburg-Landau correspondence predicts the SOC power-law distribution in KDF (this is marked "not yet verified").
- §4 does **not** claim that KDF inherits stationary-distribution or spectral-gap results from CTMC theory.
- §4 does **not** claim that canonical $(\theta_L, \theta_U) = (0.70, 0.80)$ is empirically correct — the four-benchmark refutation is the central self-refutation of the entire paper.
