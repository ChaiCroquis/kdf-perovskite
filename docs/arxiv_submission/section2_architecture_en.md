# §2 KDF Architecture Overview — English translation (Step 6.3, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 97–143.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

This section is a **specification-level description** of the patent-filed architecture. It states the canonical parameter values as filed; the empirical refutation of specific canonical values is not repeated here but is treated in §1.3 C3 and §4.2. Translator's notes at the end flag where the self-refutation context matters.

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

**日本語バックトランス要約**: グラフ $G=(V,E)$ のエッジにパラメータ(強度・接続履歴・最終参照時刻)を付与。手段 1 代謝制御は混雑度依存の $\lambda(C) = \beta(1+\gamma C^\alpha)$ による指数減衰、手段 2 希少性保護は $\deg_E(v) \le 1$ の絶対閾値で 2 段階審査($T_{\text{wait1}} = T_{\text{wait2}} \in [30, 70]$、canonical $= 50$)、手段 3 整合性発見は Laplacian 固有値 $\phi(v) \in \mathbb{R}^{32}$ フィンガープリントと sandwich 2-閾値条件 $\theta_L \le S \le \theta_U$(canonical $(0.70, 0.80)$)。重み係数は内側($a:b:c = 0.40:0.35:0.25$, Claim 45)と外側(系統:関係:属性 $= 7:2:1$, Claim 44)の 2 段。

---

### 2.2 Meta-control (Claims 27–32)

Let $\delta k = \max(0, \langle k \rangle - k_{\text{opt}})$ be the deviation of the network's mean degree $\langle k \rangle$ from the target degree $k_{\text{opt}}$. The adaptive law

$$
\Delta \alpha = -\eta (H - H_{\text{target}}) \pm \mu \cdot \delta k^{4}
$$

updates $\alpha$ within the range $[\alpha_{\min}, \alpha_{\max}]$. The Lyapunov stability condition $\eta^2 > \mu^2$ holds (full Lyapunov analysis is provided in the filed patent JP 2026-027032 and in the reference implementation `crates/cgb-kdf/src/meta_control.rs`; not reproduced in this paper).

**日本語バックトランス要約**: 平均次数 $\langle k \rangle$ と目標次数 $k_{\text{opt}}$ の偏差 $\delta k$ を用いた適応法則 $\Delta \alpha = -\eta(H - H_{\text{target}}) \pm \mu \cdot \delta k^4$ により $\alpha$ を更新。Lyapunov 安定条件は $\eta^2 > \mu^2$。

---

### 2.3 Hierarchical Management Regions

The architecture maintains three regions—short-term, long-term, and rare—with update-period ratio $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$.

**日本語バックトランス要約**: 短期・長期・希少の 3 領域、更新周期比 $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$。

---

## Translator's notes on §2

### Note A — Canonical values are stated without cross-reference to refutation

The canonical parameter values—$T_{\text{wait}} = 50$, $(\theta_L, \theta_U) = (0.70, 0.80)$, $7:2:1$, $5:3:1$, $\delta k^4$—are stated here as **the patent specification**, without in-section qualification. The reader has already encountered the four-benchmark refutation of $(0.70, 0.80)$ in §1.3 C3, and §4.2 elaborates. We deliberately did not inject a cross-reference into §2 itself, because:

- §2's role is to **state the filed architecture** so that §3 (domain survey) and §4 (structural analysis + empirical benchmarks) have a fixed referent.
- Inserting "(but see §4.2 for refutation)" each time a canonical value appears would double-state the self-refutation already handled by §1.3 C3, and would make §2 read as an editorialized critique rather than a spec.
- Readers arriving at §2 directly without reading §1.3 C3 are expected to continue to §4.2 in due course; the narrowing is load-bearing but distributed, not repeated.

If a reviewer flags this as obscuring self-refutation, the fix is to add a single sentence at the top of §2.1 (e.g., "The canonical values listed below are as filed; §4.2 documents their empirical refutation across four benchmarks and narrows the novelty claim to the 2-threshold mechanism.") — but we prefer the current structure where §2 = spec, §4 = critique.

### Note B — Terminology choices applied in §2

| Japanese | English used | Reason |
|---|---|---|
| sandwich 採用域 | sandwich 2-threshold criterion | Style memory prohibits "band" / "acceptance band"; "criterion" is mathematically natural and preserves the 2-threshold framing |
| 絶対閾値 $\deg_E(v) \le 1$ | absolute threshold $\deg_E(v) \le 1$ | Direct rendering; "absolute" distinguishes from relative/adaptive rarity definitions |
| 2 段階審査 | two-stage review | Style memory |
| 整合性スコア | integrity score | Consistent with "integrity discovery (analogy)" in §1 |
| 固有値ベクトル | eigenvalue vector | "Laplacian eigenvalue fingerprint" used in §1 abstract; here "eigenvalue vector" is mathematically precise |
| 固定長、等長不変フィンガープリント | fixed-length, isometry-invariant fingerprint | "Isometry-invariant" conveys the permutation-and-reflection-insensitivity that "等長不変" encodes |
| 線形結合 | linear combination | Standard term |
| 系統 / 関係 / 属性 | systematic / relational / attribute | Direct rendering with Claim 44 reference preserved |
| 混雑度 $C_{uv}$ | local congestion $C_{uv}$ | Direct rendering; "congestion" captures the load-on-edge semantic |
| 離散時間ステップ | discrete time step | Standard |
| Lyapunov 安定条件 | Lyapunov stability condition | Standard |

### Note C — What §2 does not claim

Several properties are stated here (Lyapunov stability, isometry-invariance, Claim-level provenance) without proof in this section. §2 is a **specification** section; the proofs and empirical validations live elsewhere:

- **Lyapunov stability $\eta^2 > \mu^2$** is asserted per the patent. Full Lyapunov analysis is not reproduced in this paper.
- **Isometry invariance of $\phi(v)$** follows from the Laplacian spectrum being invariant under graph isomorphism; we do not restate this standard result.
- **Claim numbers (37, 39, 44, 45)** reference the patent claim structure. Full claim text is in the filed patent (JP 2026-027032).

A reviewer seeking proofs or validation evidence for any specific claim is directed to `docs/VERIFIED_FINDINGS.md` for the F-IDs corresponding to each validation.
