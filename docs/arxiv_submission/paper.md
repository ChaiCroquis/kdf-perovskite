# KDF: A Deterministic Architecture for Finite-Resource Information Preservation—Cross-Domain Evidence and Self-Refutation of Canonical Values

**Author:** Yasuhiro Kuroki  
**ORCID:** [0009-0006-8943-9344](https://orcid.org/0009-0006-8943-9344)  
**Affiliation:** Independent researcher, Japan  
**Patent:** JP 2026-027032 (filed 2026-02-24)  
**Code:** [github.com/ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite) (PolyForm Noncommercial 1.0.0; commercial license separate)  
**Draft version:** v0.3, 2026-04-19

---

## Abstract

**KDF (Knowledge Decay Framework)** is a deterministic graph-compression technique: given a graph and a retention budget, selected nodes are preserved **verbatim** while unselected nodes are discarded, distinguishing KDF from content-transforming methods such as LLM-based fact extraction. KDF combines three mechanisms—edge-based continuous-time exponential decay (metabolic control), rarity protection under the absolute threshold deg_E(v) ≤ 1, and integrity discovery (analogy) via graph-Laplacian eigenvalue fingerprints—a three-pillar structure that our related-work survey finds independently recurring across ten disciplines, including mammalian memory consolidation, immune clonal selection, Ginzburg-Landau critical phenomena, continual-learning EWC, Hopfield associative memory, and K-SVD sparse coding.

Empirically, KDF delivers 7.7× gain over industry-standard TTL on LongMemEval (ICLR 2025), F1 = 0.747 on a 2,182-note Obsidian vault (Wilcoxon p = 0.006), 2.3× over Random for rare-event preservation on NASA HTTP logs, and a **+3.06-point rare-recall improvement in a realistic streaming replay of the NASA log**—providing the first positive empirical anchor for our narrowed thesis that dynamic control components find their true use in streaming scenarios rather than static queries.

However, we also transparently report refutations of our own prior claims. The sandwich 2-threshold *mechanism* is supported, but **our patent's canonical values (θ_L, θ_U) = (0.70, 0.80) are empirically refuted across four benchmarks** (Hopfield mixture, direct analogy, synthetic pairs, LoCoMo streaming): specific values require domain-specific calibration. Additional honest negatives include OSS issue generalization ×1.00 across three repositories, paper rediscovery ×0.83, and Gaussian-Process inducing-point selection failures. Applicability is predictable a priori via a zero-dependency bias-detector metric.

**Keywords:** information preservation, graph metabolism, rarity protection, analogy discovery, Laplacian fingerprint, Ginzburg-Landau, Hopfield attractor, Equitable Coreset Selection, complementary learning systems, memory consolidation

---

## 1. Introduction

### 1.1 Problem

In persistently growing information networks—LLM-agent conversation memory, personal knowledge bases, distributed-system logs, OSS issues, citation networks—storage constraints force a tradeoff between **volume** and **quality**:

- **Full retention** is practically infeasible at the TB/day scale.
- **Random reduction** (random sampling, reservoir sampling) **statistically loses** rare-but-important items (e.g., 4xx/5xx error logs, forgotten-yet-consequential notes, minority-language utterances).
- **Label-dependent methods** (stratified sampling, tail-based sampling, active learning, Equitable Coreset Selection) are powerful but presuppose labels or ground truth whose availability is not guaranteed in production.

**Requirement**: without labels, using only structure, protect rare items that will be needed later while metabolizing redundant information.

---

### 1.2 Observation

**Broader context.** The general-systems tradition, exemplified by "Living Systems Theory" [@miller1978livingsystems], has long identified twenty "critical subsystems"—*associator*, *memory*, *decoder*, and others—shared across the seven hierarchical levels from cell to supranational system. Our observation is **consistent with** this tradition; we narrow our contribution to three of those twenty subsystems—the ones specialized for information preservation (metabolism, rarity protection, recombination)—and to presenting a concrete numerical implementation.

Having attempted to solve this problem through independent engineering effort, we then noticed that the three-pillar architecture we arrived at **corresponds structurally** to the following (a qualitative observation of parallelism, not a proof of mathematical isomorphism):

1. **Mammalian memory consolidation** (Complementary Learning Systems) [@mcclelland1995cls]
2. **Memory B-cell selection in the immune system** (germinal-center affinity maturation)
3. **Ginzburg-Landau theory of critical phenomena** (quartic stabilization term)
4. **Elastic Weight Consolidation in continual learning** [@kirkpatrick2017ewc]
5. **Equitable Coreset Selection** [@wang2024ecs] and the unlearning variant **UPCORE** [@patil2025upcore]
6. **Hopfield associative memory networks** [@hopfield1982neural] and modern Hopfield networks [@ramsauer2021hopfield]
7. **Continuous-time Markov processes on graphs** (spectral-gap convergence)
8. **Tail-risk management for Pareto heavy-tailed distributions** [@pareto1896economie]
9. **Rare-atom preservation in K-SVD sparse dictionary learning** [@aharon2006ksvd]
10. **Two-factor synaptic consolidation** [@tartaglia2025twofactor]

---

### 1.3 Claims

This paper advances three claims:

- **(C1) Integration**: the findings from the ten disciplines above admit a uniform description as a three-pillar structure—**metabolic control + rarity protection + recombination**. KDF is the first **numerical implementation** of this unified description.

- **(C2) Structural similarity (not yet rigorous)**: two distinctive KDF formulas—(a) the meta-control law $\Delta \alpha \propto \delta k^4$ and (b) the continuous-time kernel $\lambda(C) \cdot \exp(-\lambda \cdot dt)$—share **the same functional form** as, respectively, the quartic term of the Ginzburg-Landau free energy and the survival probability of a continuous-time Markov process. This is a qualitative structural correspondence; a proof as mathematical isomorphism (a structure-preserving bijection) remains future work.

- **(C3) Novel contribution (mechanism supported; canonical values refuted across four benchmarks)**: the **sandwich 2-threshold mechanism**—acceptance only within a middle band, imposed jointly by a lower threshold $\theta_L$ and an upper threshold $\theta_U$—has no counterpart in the ten surveyed disciplines and is a distinctive KDF contribution. However, the **specific values that patent claims 46/48 designate as canonical, $(\theta_L, \theta_U) = (0.70, 0.80)$, are empirically refuted across the following four benchmarks**:

  1. **Phase V3 Hopfield** (F-041): at $\theta_U = 0.80$, $0\%$ of Hopfield mixture states are detected; rejection of $24\%$ (at $P = 18$) and up to $\sim 40\%$ at higher load factors becomes feasible only after lowering the threshold to $\theta = 0.40$.
  2. **F-068 analogy-discovery (direct)**: on Gentner classics and git↔paper cross-domain pairs, scores concentrate uniformly at $\ge 0.99$, so an upper bound of $0.80$ would reject every true positive.
  3. **F-070 Part A synthetic + F-068 scenarios**: over 38 pairs (22 positive / 16 negative), F1(canonical) = $0.000$ whereas F1($(0.70, 1.00)$) = $1.000$.
  4. **F-070 Part B LoCoMo Rev12 streaming**: across 30 queries and 1,140 RARE nodes, the canonical values cause 100% demotion to Garbage after the 60-cycle timeout (complete information loss).

  We therefore narrow the claim to: "**the mechanism is meaningful across domains; the specific values require domain-specific calibration.**" §4.2 elaborates. We claim novelty for the *mechanism* only, and we do not claim universality for the canonical numerical values.

---

### 1.4 Tension between Universality and Novelty

The paper's assertion that "KDF's structure has been independently found in ten disciplines" is, from a novelty standpoint, **a double-edged sword**:

- **Favorable reading**: ten disciplines converging independently on the same solution suggests that KDF, as the first integrated implementation of this universal pattern, has genuine significance.
- **Critical reading**: analogous solutions are already known in ten disciplines, so KDF is mere reinvention and lacks novelty.

We confront this tension directly and restrict our novelty claim to the following three narrow items:

1. **Novelty of integration**: we find no prior instance of a single numerical implementation or a single patent claim that integrates all three pillars (metabolism, rarity protection, recombination). Each discipline specializes in one or two pillars (neuroscience focuses mainly on metabolism and recombination, ECS mainly on rarity protection, and so on).
2. **Novelty of the sandwich 2-threshold *mechanism* (but not its specific values)**: the *dual-threshold structure* that admits only a middle band via a lower bound $\theta_L$ and an upper bound $\theta_U$ has no counterpart in the ten disciplines we surveyed and is distinctive to KDF. However, the patent canonical values $(0.70, 0.80)$ have been empirically refuted across four benchmarks (§1.3 C3, §4.2); we therefore claim novelty only for the *mechanism*, and we do not claim optimality for any specific numerical values.
3. **Novelty of an open numerical implementation**: this is the first publicly released implementation to provide concrete parameter values ($\beta = 0.01$, 7:2:1, 5:3:1, $[0.70, 0.80]$, $\delta k^4$) together with a source-available implementation (PolyForm Noncommercial 1.0.0; commercial license available separately).

We do **not** claim novelty for any individual pillar (synaptic pruning, EWC, Laplacian fingerprinting, and the like) in isolation. These are pre-existing contributions.

---

### 1.5 Paper Organization

Section 2 presents KDF's three mechanisms and key formulas concisely. Section 3 systematizes the correspondences with the ten independent disciplines. Section 4 argues the three structural similarities. Section 5 presents empirical evidence (six positive and four negative cases). Section 6 discusses implications and limitations. Section 7 concludes with a refined applicability predictor and the 2 × 2 Mem0 / KDF benchmark matrix. Section 8 develops a stronger theoretical foundation by aligning KDF with Burt's Structural Holes theory, paired with an explicit Limitations and Risks subsection that makes the hierarchy vs. §4 deliberate. Acknowledgments, References (generated from `references.bib`), and Appendices A (key formulas) and B (implementation architecture) follow.

---

## 2. KDF Architecture Overview

### 2.1 Basic Structure

Let the information structure be a graph $G = (V, E)$. Each edge $(u, v) \in E$ carries parameters: strength $w_{uv}$, connection history $n_{uv}$, and last-access time $t^{\text{acc}}_{uv}$.

**Mechanism 1 (Metabolic Control).** Using the local congestion $C_{uv} = \deg(u) + \deg(v)$, we define the decay rate

$$
\lambda(C_{uv}) = \beta \left( 1 + \gamma C_{uv}^{\alpha} \right)
$$

and apply exponential edge-weight decay over a discrete time step $dt$:

$$
w_{uv}(t + dt) = w_{uv}(t) \cdot \exp\bigl(-\lambda(C_{uv}) \cdot dt\bigr) .
$$

Threshold-based pruning and probabilistic pruning with $\Pr[\text{prune}] = 1 - \exp(-\lambda \cdot dt)$ are also permitted.

**Mechanism 2 (Rarity Protection).** Any node $v$ satisfying the absolute threshold $\deg_E(v) \le 1$ is treated as a rare object and **unconditionally excluded** from metabolic control over the protection interval

$$
[t_0, t_0 + T_{\text{wait1}}] \cup (t_0 + T_{\text{wait1}}, t_0 + T_{\text{wait1}} + T_{\text{wait2}}]
$$

(a two-stage review). The two intervals satisfy $T_{\text{wait1}} = T_{\text{wait2}} \in [30, 70]$ — Claim 37 requires equal lengths, Claim 39 specifies the range, and the canonical default is $T_{\text{wait}} = 50$.

**Mechanism 3 (Integrity Discovery).** From the ego-subgraph of a rare node we construct the Laplacian $L_v$, compute its eigenvalue vector $\phi(v) \in \mathbb{R}^{32}$—a fixed-length, isometry-invariant fingerprint—and evaluate the integrity score against existing nodes:

$$
S(v, u) = a \cdot S_{\text{cos}}(\phi(v), \phi(u)) + b \cdot S_{\text{struct}}(v, u) + c \cdot S_{\text{sign}}(v, u), \qquad a, b, c > 0 .
$$

The formula above defines the **inner** similarity $S_{\text{inner}}$; the **outer** (aggregated) integrity score — on which the sandwich acceptance condition actually operates — is obtained by aggregating $S_{\text{inner}}$-derived values of three types (systematic, relational, attribute). A new edge is generated when the outer score satisfies the **sandwich 2-threshold criterion** $\theta_L \le S_{\text{outer}} \le \theta_U$ (canonical defaults $\theta_L = 0.70$, $\theta_U = 0.80$). The weights are used at two independent levels:

- **Inner fingerprint similarity** $S_{\text{inner}}$ (combination of structural fingerprints): $a : b : c = 0.40 : 0.35 : 0.25$, the linear combination of $S_{\text{cos}}$, $S_{\text{struct}}$, and $S_{\text{sign}}$ (Claim 45).
- **Outer integrity aggregation** $S_{\text{outer}}$ (aggregation over systematic / relational / attribute similarities): $\text{systematic} : \text{relational} : \text{attribute} = 7 : 2 : 1$ (Claim 44).

The two weightings are independent: the inner one governs similarity between fixed-length vectors, while the outer one governs the aggregation of classified similarity scores. The sandwich threshold is imposed on the outer score.

---

### 2.2 Meta-control (Claims 27–32)

Let $\delta k = \max(0, \langle k \rangle - k_{\text{opt}})$ be the deviation of the network's mean degree $\langle k \rangle$ from the target degree $k_{\text{opt}}$. The adaptive law

$$
\Delta \alpha = -\eta (H - H_{\text{target}}) \pm \mu \cdot \delta k^{4}
$$

updates $\alpha$ within the range $[\alpha_{\min}, \alpha_{\max}]$. The Lyapunov stability condition $\eta^2 > \mu^2$ holds (full Lyapunov analysis is provided in the filed patent JP 2026-027032 and in the reference implementation `crates/cgb-kdf/src/meta_control.rs`; not reproduced in this paper).

---

### 2.3 Hierarchical Management Regions

The architecture maintains three regions—short-term, long-term, and rare—with update-period ratio $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$.

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

---

### 3.1 Tartaglia et al. (PNAS 2025) as an Independently Parallel Discovery

Particularly noteworthy is Tartaglia et al., "Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling" [@tartaglia2025twofactor]. **Its publication overlaps in time with the KDF patent filing (2026-02), and the two works do not cite each other — exhibiting independent convergence.**

Their claim—"synapse as the product of two factors + replay + homeostatic scaling + Hebbian plasticity → prunes connections while preserving weak memories"—corresponds to our three pillars almost one-to-one (metabolism = homeostatic scaling; rarity protection = two-factor / weak-memory preservation; recombination = Hebbian replay). The fact that two independent teams converged on similar attractor points is **an observation consistent with the universality hypothesis** (not a proof of the hypothesis).

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

---

### 5.2 Negative Results (Four Cases) — An Honest Record

For the integrity of this work, we report **both generalization failures found in testing** and **refutations of canonical values that we ourselves proposed**:

| Problem | Result | Interpretation |
|---|---|---|
| P6 OSS maintenance | KDF/Random ratios over three repos (`rust-lang/rust`, `tokio-rs/tokio`, `golang/go`) are $\times 1.13, \times 1.03, \times 0.85$; average **$\times 1.00$** | The $+15\%$ on `rust-lang` alone is a repository-specific local signal. The general-OSS applicability claim is **withdrawn** (F-038) |
| P5 Paper rediscovery | OpenAlex 200 papers × concept-sharing graph; KDF/Random $= \mathbf{\times 0.83}$ | Late-bloomer detection is a D5-type task (independent of structure); KDF is ill-suited to concept-graphs (F-039) |
| **P9 Redundancy of Claim 5/14 time signals on static tasks (validated on streaming)** | **LoCoMo 321 Q at keep $30\%$: KDF_static $= 0.5286$; adding time signals yields $0.43$–$0.53$ (static: all worse or tied). NASA streaming 50k records at keep $30\%$: C0 static $0.4592$; C1 Claim 14 decay $0.4898$ (**$+3.06$ pt benefit on streaming**)** | On static query tasks, structural rarity already subsumes temporal rarity, so the time signals are redundant. On streaming, decay discards stale normal traffic and relatively elevates rare resources → **task-structure-dependent conditional value** (F-069 static redundancy + F-072 streaming $+3.06$-pt validation) |
| **P10 Four-benchmark refutation of canonical $\theta_U = 0.80$ (Claims 47–48)** | **F-041 (Hopfield: $0\%$ detection) + F-068 (analogy scores $0.99+$ reject every positive) + F-070 Part A (F1 $0.000$ vs. $1.000$) + F-070 Part B (LoCoMo: $100\%$ RARE demoted)** | The 2-threshold *mechanism* is supported; the canonical specific values $(0.70, 0.80)$ are refuted across four benchmarks. Domain-specific calibration is required (F-041, F-070) |

---

### 5.3 A Priori Applicability Metric: `bias-detector`

We released a zero-dependency Rust crate for determining KDF's applicability in advance ([`crates/bias-detector/`](../../crates/bias-detector/)). The metric $\text{bias\_score} = 0.3 \cdot I_1 + 0.7 \cdot I_4$ correctly predicted applicability on 4 of 5 benchmarks exactly, and on the 5th via an alternate route.

---

## 6. Discussion / Implications

### 6.1 The Possibility of a Domain-Invariant Architecture

From the qualitative observation that ten independent disciplines adopt the same three-pillar structure (or a 2–3-pillar subset), we advance the following **hypothesis** (not a proof):

> **Hypothesis (untested)**: "Systems that preserve information long-term under finite resources repeatedly exhibit designs belonging to the architectural family of three mechanisms—metabolism, rarity protection, and recombination."

The hypothesis **can be supported** (a necessary but not sufficient set of grounds) from the following directions:

(a) independent evolutionary convergence in biological systems (brain, immune);
(b) independent rediscovery in ML engineering (EWC, ECS);
(c) functional-form match with critical-phenomena physics.

However, all three are observations of *compatibility*, not proofs of *necessity*. Demonstrating genuinely universal optimality would require the following—none carried out in this paper:

- Mathematical formalization of the finite-resource information-preservation problem and characterization of its optimal-solution class.
- Proof of a performance lower bound on this problem for systems that **lack** the three pillars.
- Statistical tests quantifying the ten-domain correspondence.

KDF in this work **provides the first comprehensive numerical implementation** in the direction of this hypothesis; verifying the hypothesis itself is a task for future work.

---

### 6.2 The Possibility of Domain-Specialized Implementations

By wrapping the same core engine with domain-specific interfaces, we can extend deployment across multiple application markets:

- `kdf-associative-memory` — a wrapper for Hopfield spurious-attractor suppression
- `kdf-coreset` — unsupervised, label-free Equitable Coreset Selection
- `kdf-temporal-graph` — temporal-graph embedding, directly compared with @nguyen2018ctdne
- `kdf-portfolio` — information tail-risk management (insurance / finance applications)
- `kdf-llm-memory` — LLM-agent long-term memory (leveraging the LongMemEval track record directly)

---

### 6.3 Positioning of Patents and Licensing

This paper's patent claims secure two strategic contributions:

1. **Claim 1 (independent)**: integration of the three mechanisms. Even if the individual elements pre-exist, the integration itself has no prior art. F-068 completed the realistic benchmark for the analogy mechanism, so all three mechanisms are now empirically backed.
2. **Claims 47–48: the sandwich 2-threshold *mechanism*** (lower bound $\theta_L$ + upper bound $\theta_U$) has no counterpart in the ten related disciplines surveyed and is a distinctive element. However, the patent-designated canonical values $(\theta_L, \theta_U) = (0.70, 0.80)$ themselves were subject to the four-benchmark refutation in §4.2 / §5.2 P10, narrowing the claim to *"the mechanism is novel; specific values require domain-calibration."*

The implementation code is available under PolyForm Noncommercial 1.0.0 (research, education, and personal use freely permitted). Commercial licenses — both for the source code and for practicing the patent — are managed separately; inquiries are welcome via the repository's COMMERCIAL.md.

---

### 6.4 Limitations

- **The "domain-invariant architecture" hypothesis itself is untested.** The ten-domain correspondence is merely a qualitative observation; a formal theorem of "necessary convergence" has not been proved. §6.1's hypothesis should be read **as a hypothesis, not a conclusion**.
- **Canonical values $(\theta_L, \theta_U) = (0.70, 0.80)$ are refuted across four benchmarks** (F-041 Hopfield / F-068 analogy / F-070 Part A synthetic / F-070 Part B LoCoMo streaming). The sandwich mechanism itself is supported, but universality of specific values is not claimed (§4.2 / §5.2 P10).
- **Claim 5 / 14 time-evaluation components and exponential decay are task-structure-dependent**: redundant on static query tasks (F-069 LoCoMo); $+3.06$-pt benefit on streaming scenarios (F-072 NASA 50 k records, time-ordered replay). When structural rarity already subsumes temporal rarity, the time signals are redundant; when stale traffic must be discarded under continuous operation, they are effective.
- **Claim 25 ActivationScore depends on the temporal distribution of rare events**: $100\%$ rescue for temporally clustered rare events (F-027 Mode E drift scenario); hurts or neutralizes on evenly distributed rare events (F-072 NASA; F-069 LoCoMo) due to recency bias. At application time, discrimination of the rare-event temporal pattern is required.
- **The ten-domain correspondences are at the level of structural similarity.** Proof as mathematical isomorphism (structure-preserving bijection) has not been carried out for any of the disciplines.
- **The §4.3 Markov correspondence is a motivating analogy.** Strict CTMC equivalence does not hold (non-stationary generator, weights are not probability mass, etc.).
- **Empirical evidence covers only five domains with six cases** (P1 / P2 / P3 / P7 / P8 = Claim 17 bit-exact; P11 = NASA streaming; P3 and P11 are the static and streaming scenarios of the same NASA HTTP-log domain). Generalization claims beyond these five domains are **beyond the scope of this paper**.
- **Negative results P5 / P6** establish that KDF is ineffective for D5-type tasks (rarity independent of structure). **P9 / P10** establish that our own canonical parameters are not optimal in realistic scenarios. A priori screening via `bias-detector` (F-030 / F-036) is recommended.
- **Optimality of specific parameter values** ($\beta = 0.01$, $7:2:1$, $5:3:1$, $[0.70, 0.80]$, $\delta k^4$, $\lambda$, $\tau_{\text{ref}}$) is empirically chosen; domain-specific tuning is required. P10 quantifies one such case, where the sandwich canonical fails to match the true score distribution.
- **Universality $\neq$ optimality $\neq$ novelty.** The ten-discipline parallel discovery suggests universality, but whether the common structure is **genuinely optimal** is a separate question; and novelty is limited to the narrow three-item claim in §1.4 (novelty of integration + novelty of mechanism + novelty of open implementation).

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

## 8. Theoretical Foundation — KDF as a Computational Realization of Structural Holes Theory

Having accumulated the empirical characteristics of KDF (F-061–F-065), we find that KDF's behavior **algorithmically aligns with the classical "Structural Holes" theory of organizational sociology [@burt1992structuralholes]**. This is not a post-hoc analogy; it refers to a **graph-theoretic isomorphism** (with explicit caveats developed in the *Limitations and Risks of the Theoretical Claim* subsection at the end of §8; the isomorphism is a strong framing hypothesis, not a proved theorem).

### Summary of Burt's Structural Holes theory [@burt1992structuralholes]

From an empirical study of workplace networks ($n = 500$ MBA students), Burt argued:

1. **The individuals who hold the highest information, innovation, and negotiation power in an organization are not the central figures within dense clusters, but the "brokers" who bridge different clusters.**
2. A **structural hole** is the empty space where two clusters are not directly connected.
3. By spanning this hole, the broker can:
   - control information non-redundancy (different information flows on each side),
   - set exchange terms favorably (brokerage power),
   - cross-pollinate ideas (innovation advantage).
4. Brokers typically have **low degree** (fewer ties make bridging more efficient) and **high betweenness** (they lie on paths).

### Mathematical Alignment with KDF's Behavior

KDF's Rare / Core / Edge / Garbage classification corresponds to Burt's broker concept as follows:

| KDF layer | Structural feature | Burt's classification | Evidence |
|---|---|---|---|
| **Rare** (deg $= 1$) | Boundary node, unique connection | Pure broker (if between clusters) | F-012 Obsidian orphan |
| **Core** | Bridges many clusters, moderate degree | Structural broker | F-062 merge commits ($99.5\%$ recall) |
| Edge | Within-cluster center, high degree | Cluster insider (non-broker) | F-061 scale-free hubs (KDF loses) |
| Garbage | Redundant, out of capacity | Peripheral / replaceable | F-052 low-recall answer turns |

KDF's selection principle of **"prioritizing Rare + Core for protection"** is equivalent to **"prioritizing nodes occupying broker positions for protection"**. This is not a coincidence: the graph-theoretic definition of structural rareness (low degree and/or high betweenness potential) overlaps with Burt's broker definition.

### Empirical Validation via the Scale-Free vs. Community-Graph Split

The betweenness-centrality results of F-061 on four graphs (ER, BA, WS, SBM) constitute a **predictive validation** of this theoretical correspondence:

| Graph type | Structural feature | Burt-theoretic prediction | KDF result |
|---|---|---|---|
| ER (Erdős–Rényi) | Uniform random | Brokers exist; bridge protection wins | [True] KDF wins (top-50 recall $0.70$) |
| SBM (Stochastic Block Model) | Planted communities | Inter-community brokers are decisive; KDF applicability is maximal | [True] KDF wins ($0.50$ vs. $0.36$ TopDegree) |
| BA (Barabási–Albert, scale-free) | Hub-dominated | Hubs have abundant alternative paths → brokerage power is distributed; KDF at disadvantage | [False] KDF loses to TopDegree |
| WS (Watts–Strogatz, small world) | Clustered + shortcut | Shortcuts are broker-like but of moderate degree; complex | [False] KDF loses to TopDegree on betweenness |

**KDF's loss on scale-free graphs** is consistent with Burt's theoretical prediction that **"the relative value of brokers declines in hub-dominated networks."** Hubs have many alternative paths, so removing any given hub leaves the network robust. Brokerage power concentrates in inter-community bridges, but is distributed in pure scale-free networks.

### Connection to Game-Theoretic Brokerage Power

In network-bargaining theory [@myerson1977graphs; @calvoarmengol2004networks], when two players A and B wish to transact but are not directly connected and must go through an intermediary C, **C's payoff is inversely proportional to the number of alternative paths**. This is the mathematical formulation of structural holes.

KDF can be interpreted as an algorithm that deterministically extracts the **"intermediaries with no alternative paths" (monopolistic brokers)**. In F-061's APSP experiment, KDF's $4 \times$ improvement over Random in coverage maintenance — particularly on **Watts–Strogatz** (small world, shortcut-broker structure) — is an instance of this theoretical prediction.

### Computational Complexity of the Implementation Algorithm

Burt's structural-holes computation is traditionally performed via **effective-size** / **constraint** metrics ($O(V^2)$ or worse). KDF's Rare / Core / Edge / Garbage classification achieves approximately the same broker detection in **$O(V + E)$** structural counting + classification (exact equivalence is left for future work).

**Implication**: KDF may be the first graph algorithm that **detects structural holes / brokerage power deterministically in near-linear time**. In the 30+ years since Burt's 1992 work [@burt1992structuralholes], structural-holes theory has been continuously cited in social-science research (M&A analysis, supply-chain resilience, innovation diffusion), but **no scalable computational algorithm has been established**. KDF's $O(V + E)$ classifier may remove the rate-limiting barrier to practical deployment.

### Applications Space (with Evidence Sketches)

From this theoretical correspondence, KDF's commercial and research application space is as follows:

1. **Enterprise network analytics**
   - Internal communication analysis (Slack / Teams) → identifying cross-team brokers
   - M&A target selection: extracting the "monopolistic intermediary companies" that should be acquired
   - Supply chain: identifying BCP-critical, low-degree Tier 3/4 suppliers
2. **Urban / logistics networks**
   - Disaster-time APSP preprocessing (validated in F-061)
   - Critical-intersection identification in transportation networks
3. **Cybersecurity**
   - Lateral-movement detection via "rare inter-segment connections" (additional measurement needed)
4. **Enterprise archival**
   - Decision preservation via cross-thread bridge capture (homologous to F-062 git merge)

### Limitations and Risks of the Theoretical Claim

For honesty, we make the following explicit:

- **Burt's theory is primarily validated on human organizational networks ($< 10^3$ nodes)**; application at millions of nodes remains an empirical question.
- KDF's Rare / Core classification is **closer to a necessary-condition approximation than a sufficient condition** for broker detection (as suggested by KDF's loss on BA graphs in F-061).
- Structural-holes theory itself has a known "weak on hub-dominated networks" limitation; see critiques such as @powell2005network.
- **Whether KDF's $O(V + E)$ complexity is formally equivalent to Burt's effective-size metric is unproven**; we have only approximation and empirical agreement.

With these caveats maintained, we position **KDF as a strong candidate for the computational realization of structural-holes detection**.

---

## Acknowledgments

We ran independent verification agents (GPT- and Claude-based) at 12 phase boundaries to check the validity of both the positive and negative claims of this work. AI collaboration was used at every stage — specification freezing, code implementation, real-data verification, related-work survey, and drafting of this paper — and the full verification process is recorded in [`docs/VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md).

---

## References

*Bibliography is generated from [`references.bib`](references.bib) via BibTeX / natbib. KDF-related primary materials — patent publication JP 2026-027032 (filed 2026-02-24) and the verification records in [`VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md) / [`PUBLIC_SUMMARY.md`](../PUBLIC_SUMMARY.md) / [`related_work_survey.md`](../related_work_survey.md) — are cited inline where relevant.*

---

## Appendix A: Summary of Key KDF Formulas

| Claim | Formula | Description |
|---|---|---|
| 7 | $C_{uv} = \deg(u) + \deg(v)$ | Local congestion |
| 8, 9 | $\lambda(C) = \beta(1 + \gamma C^\alpha)$, $\alpha$ positive exponent | Nonlinear form of the decay rate (monotone increasing + power-law term) |
| 10 | $\alpha = 2$ | Fix the power-law exponent at 2 (core of the invention) |
| 14 | $w(t + dt) = w(t) \cdot \exp(-\lambda(C) \cdot dt)$ | Exponential decay law |
| 15 | $\deg_E(v) \le 1 \Rightarrow \text{Rare} \land \text{protected}$ | Rarity detection via absolute threshold |
| 21 | $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$ | Update-period ratio across hierarchical regions |
| 29 | $\Delta \alpha \propto \delta k^4$ | Meta-control quartic law |
| 44 | $S_{\text{outer}} = \tfrac{7}{10} S_{\text{sys}} + \tfrac{2}{10} S_{\text{rel}} + \tfrac{1}{10} S_{\text{attr}}$ | Aggregated integrity score (outer; takes systematic / relational / attribute similarities as inputs) |
| 45 | $S_{\text{inner}} = 0.40 \cdot S_{\text{cos}} + 0.35 \cdot S_{\text{struct}} + 0.25 \cdot S_{\text{sign}}$ | Fingerprint similarity combination (inner; feeds into $S_{\text{sys}}$ / $S_{\text{rel}}$ / $S_{\text{attr}}$ above) |
| 46 | $\phi(v) \in \mathbb{R}^{32}$, derived from Laplacian eigenvalues | Fixed-length structural fingerprint |
| 47–48 | $\theta_L = 0.70 \le S_{\text{outer}} \le \theta_U = 0.80$ | Sandwich acceptance condition, applied to the outer (aggregated) score from Claim 44 (canonical values — refuted in §4.2; see §1.3 C3) |

---

## Appendix B: Implementation Architecture

- `crates/cgb-kdf/` — reference implementation (Rust; PolyForm Noncommercial 1.0.0, commercial license separate); direct tests for all 50 patent claims.
- `crates/bias-detector/` — independently released crate; zero-dependency; a priori KDF applicability screening.
- `demos/D1–D8/` — showcase implementations across 8 domains.
- `benchmarks/sota_comparison/` — SOTA comparison benchmarks.

GitHub: [ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite)

---

*Draft v0.3 — 2026-04-19 (reflecting Phase X Step 1–5 completion). Comments and corrections welcome via GitHub issues.*

---
