# §3 Three-Pillar Structure Across Ten Independent Disciplines — English translation (Step 6.3, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 147–169.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

This section surveys the three-pillar structure's manifestations in ten disciplines. The central delicate passage is the hedge "**observed at a qualitative level, not claimed as strict equivalence**"—it must remain load-bearing in English, since the universality narrative of §1.2–§1.4 rests on this observation-not-proof framing. Additional delicate items (Hopfield's unsolved-problem row, KDF-row's $\theta_U$ placement, §3.1's "consistent-with-hypothesis-not-proof" wording) are flagged in translator's notes.

---

## 3. Three-Pillar Structure Across Ten Independent Disciplines

The table below enumerates the concrete realization of the three pillars in each discipline. The correspondence with KDF **can be observed at a qualitative level** (this is not a claim of strict equivalence).

| Discipline | Metabolism | Rarity protection | Recombination / integration | Representative references |
|---|---|---|---|---|
| Mammalian brain (1) | Synaptic pruning during sleep | Engram plasticity window | Hippocampus → neocortex replay | @mcclelland1995cls; @tartaglia2025twofactor |
| Immune system (2) | Naïve B-cell removal | Memory B-cell pool (restricted clonality) | Germinal-center affinity maturation | @mesin2020gc |
| Critical-phenomena physics (3) | SOC avalanche decay | Power-law tail (critical cluster) | Ginzburg-Landau quartic stabilization | @bak1987soc; @ginzburg1950superconductivity |
| Continual-learning ML (4) | Weight decay / dropout | EWC Fisher protection | Replay buffer | @kirkpatrick2017ewc |
| Coreset selection (5) | Adaptive pruning | ECS minority preservation | Class-sensitive partitioning | @sener2018coreset; @wang2024ecs |
| Hopfield associative memory (6) | Dynamic pattern decay | Stored-pattern attractor basin | **Spurious-attractor rejection remains unsolved** | @hopfield1982neural; @ramsauer2021hopfield |
| Markov processes on graphs (7) | $\exp(-\lambda \cdot dt)$ mixing | Slow-mixing mode ($\lambda_2$) | Spectral gap / Fiedler vector | @levin2017markov |
| Economics / finance (8) | Portfolio rebalance | Tail-event capital reserve | Pareto / extreme-value theory | @pareto1896economie; @embrechts1997extreme |
| Signal processing (9) | Dictionary pruning | Rare-atom preservation | K-SVD iterative update | @aharon2006ksvd |
| Graph ML (10) | Time-decayed line graph | Rare-node structural signal | Analogy discovery via fingerprint | @nguyen2018ctdne |
| **KDF (integrated)** | $\exp(-\lambda(C) \cdot dt)$-form exponential survival decay | $\deg \le 1$ absolute threshold + $\theta_U$ sandwich | Laplacian fingerprint analogy | — |

**日本語バックトランス要約**: 10 領域それぞれで 3 本柱(代謝・希少保護・組換え)の具体的実現を列挙。KDF との対応は **定性的レベルで観察可能**(厳密な同値性の主張ではない)。Hopfield 行の「spurious attractor 棄却は未解決」は他領域に無い gap として太字で明示。KDF 行は $\exp(-\lambda(C) \cdot dt)$、$\deg \le 1 + \theta_U$ sandwich、Laplacian fingerprint analogy の 3 要素で統合実装を示す。

---

### 3.1 Tartaglia et al. (PNAS 2025) as an Independently Parallel Discovery

Particularly noteworthy is Tartaglia et al., "Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling" [@tartaglia2025twofactor]. **Its publication overlaps in time with the KDF patent filing (2026-02), and the two works do not cite each other — exhibiting independent convergence.**

Their claim—"synapse as the product of two factors + replay + homeostatic scaling + Hebbian plasticity → prunes connections while preserving weak memories"—corresponds to our three pillars almost one-to-one (metabolism = homeostatic scaling; rarity protection = two-factor / weak-memory preservation; recombination = Hebbian replay). The fact that two independent teams converged on similar attractor points is **an observation consistent with the universality hypothesis** (not a proof of the hypothesis).

**日本語バックトランス要約**: Tartaglia et al. (PNAS 2025) は KDF 出願(2026-02)と時期的に重なり、相互参照の無い **独立収束** を示す事例。彼らの主張は本論文の 3 本柱とほぼ 1:1 対応(homeostatic scaling = 代謝、two-factor = 希少保護、Hebbian replay = 組換え)。独立チームが類似引力点に収束した事実は **普遍性仮説と整合する観察** であって仮説の証明ではない。

---

## Translator's notes on §3

### Note A — The "qualitative-level observation" hedge

The Japanese passage "定性的なレベルで観察できる(厳密な同値性の主張ではない)" is the load-bearing hedge that lets §3 present ten correspondences without overclaiming mathematical equivalence. This hedge coordinates with:

- **§1.2**: "structurally corresponds ... a qualitative observation of parallelism, not a proof of mathematical isomorphism"
- **§1.3 C2**: "This is a qualitative structural correspondence; a proof as mathematical isomorphism ... remains future work"
- **§6.4 Limitations**: "the correspondence with the ten disciplines is at the level of structural similarity; proof as mathematical isomorphism has not been done for any discipline"

**Translation choices**:
- "**can be observed at a qualitative level**" — the modal "can be observed" matches "観察できる" and keeps the stance descriptive rather than assertive.
- "**not a claim of strict equivalence**" — "strict equivalence" is chosen over "mathematical equivalence" to cover the broader notion (functional form match does not by itself establish either isomorphism or equivalence of dynamics).
- The hedge is placed **before** the table, so that a reader skimming the table sees it first — matching the Japanese ordering.

**What we did not do**: we did not move the hedge to a footnote, reword it as "approximately corresponds" (which would soften the Japanese "厳密な同値性の主張ではない"), or relegate it to §6.4.

### Note B — The Hopfield row deliberately admits an unsolved problem

The row for Hopfield associative memory lists the recombination column as "**Spurious-attractor rejection remains unsolved**" — with the Japanese source using bold emphasis. This is an **intentional admission** that the recombination pillar has no established solution in the Hopfield literature. Preserving the bold is important because:

1. It sets up §4.2, where KDF's original conjecture that its $\theta_U$ mechanism could resolve this 40-year-open problem is then empirically refuted — the narrative arc requires §3 to establish "this is unsolved" *before* §4.2 reports "our attempt did not solve it either (canonical values refuted)".
2. Without the bold, a reader could easily read "spurious-attractor rejection" as an existing technique rather than an open problem.

**Translation choice**: we kept the bold and used "**remains unsolved**" (active stance) rather than "is an open problem" (descriptive), matching the Japanese "**未解決**" which carries a slightly more urgent tone than "open".

### Note C — KDF row lists $\theta_U$ sandwich under rarity protection despite §1.3 C3 refutation

The KDF row in the table places "$\deg \le 1$ + $\theta_U$ sandwich" in the **rarity protection** column. This assignment reflects an **architectural design decision** (the sandwich acceptance condition operates on rare-node candidates flagged by $\deg_E(v) \le 1$), not a claim about the numerical validity of canonical $(\theta_L, \theta_U) = (0.70, 0.80)$.

**Potential reader confusion**: a reader seeing "$\theta_U$ sandwich" in the KDF architectural summary alongside the other nine disciplines' validated mechanisms might infer that the $\theta_U$ sandwich is equally validated. This inference is wrong — §1.3 C3 and §4.2 document the four-benchmark refutation of specific canonical values.

**Translation choice**: we kept the row faithful to the Japanese source (no added cross-reference inline), but note that the sandwich is an *architectural* placement, not a validation claim. If the reviewer objects, the table row could be augmented with "($\theta_L$, $\theta_U$ as mechanism; specific canonical values domain-dependent, see §4.2)" — deferred pending full-paper readthrough.

### Note D — The Tartaglia discovery as "consistent with" the hypothesis

§3.1 ends with the crucial formulation "**an observation consistent with the universality hypothesis** (not a proof of the hypothesis)". This is the architectural center of the entire §3 argument: the Tartaglia 2025 convergence is **evidence**, not **proof**. Preserving the strength relationship requires:

- "**consistent with**" (not "supports", which implies inferential direction) — matches "整合する".
- "**not a proof of the hypothesis**" (parenthetical, equal weight to the assertion) — matches "(仮説の証明ではない)".
- The phrase "converged on the same attractor point" is left as a lightly metaphorical rendering of "似た引力点に収束した"; a literal "similar attractor points" would be fine but slightly stilted. The metaphor echoes the dynamical-systems language used across the paper.

**What we did not do**: we did not write "provides evidence for" (too strong) or "is compatible with" (too weak). "Consistent with ... not a proof" is the precise epistemic stance.

### Note E — Terminology choices in §3

| Japanese | English used | Reason |
|---|---|---|
| 組換え / 統合 | recombination / integration | Column header kept as both terms because the pillar handles both pattern recombination (brain, immune) and structural integration (Ginzburg-Landau, Laplacian fingerprints) |
| 睡眠中の synaptic pruning | synaptic pruning during sleep | Direct rendering |
| engram plasticity window | Engram plasticity window | Capitalized sentence-initial; term itself standard |
| SOC avalanche decay | SOC avalanche decay | SOC = Self-Organized Criticality; abbreviation preserved per cross-domain convention |
| 保存パターン attractor basin | Stored-pattern attractor basin | "Stored" preserves the Hopfield-specific sense of long-term memory |
| spurious attractor の棄却は未解決 | **Spurious-attractor rejection remains unsolved** | Bold preserved (Note B) |
| slow-mixing mode ($\lambda_2$) | Slow-mixing mode ($\lambda_2$) | Spectral-gap terminology; $\lambda_2$ = second eigenvalue of transition matrix |
| tail event capital reserve | Tail-event capital reserve | Extreme-value-theory finance terminology |
| rare-atom preservation | Rare-atom preservation | K-SVD-specific; "atom" = dictionary element |
| time-decayed line graph | Time-decayed line graph | Graph-ML specific term |
| KDF(統合)| KDF (integrated) | Parenthetical noun-adjunct matches "統合" as "across-pillar integration" rather than "temporal integration" |
| 独立並行発見 | independently parallel discovery | Style-memory recommended; also used in §1.2 bullet 10 |
| 引力点 | attractor point | Dynamical-systems metaphor preserved; "attractor" is standard in the parallel-discovery literature |
| 普遍性仮説 | universality hypothesis | Matches §6.1 where the hypothesis is introduced formally |

### Note F — What §3 does not claim

- §3 does **not** claim that the ten correspondences are mutually independent pieces of evidence; some disciplines share citation roots (e.g., Tartaglia 2025 is rooted in the @mcclelland1995cls tradition).
- §3 does **not** claim the three-pillar framework is exhaustive for information-preserving systems; §6.1 explicitly frames it as a hypothesis, not a theorem.
- §3 does **not** claim discipline (10) Graph ML is a "discovery" in the same sense as disciplines (1)–(9); @nguyen2018ctdne is a research artifact rather than a convergent independent solution. A future revision may group (10) under "related engineering work" rather than "independent parallel disciplines".
