# KDF 減衰方程式の解析

**Phase 2 成果物 / Claim 14 数理裏付け**

---

## 1. 定義

本節を通じて、以下の記号を用いる。

| 記号 | 意味 |
|---|---|
| $w_{ij}(t)$ | エッジ $(i,j)$ の重み、時刻 $t$ |
| $C_{ij}$ | 局所混雑度 $\deg(i) + \deg(j)$ (Claim 7) |
| $\alpha, \beta, \gamma$ | 層別減衰パラメータ(Claim 9, 14) |
| $\lambda(C)$ | 減衰率 $= \beta(1 + \gamma C^\alpha)$ (Claim 14) |
| $\Delta t$ | 層別離散時間刻み(Claim 14) |
| $\ell \in \{E, R, C, M\}$ | 層インデックス(Edge/Rare/Core/Meta) |

---

## 2. 連続時間方程式

Claim 14 は以下の ODE を規定する:

$$
\frac{dw_{ij}}{dt} = -\lambda(C_{ij})\, w_{ij}
\tag{2.1}
$$

$C_{ij}$ が時間に対して短時間で定数とみなせる区間(準静的近似)では、式(2.1)は線形1階ODE であり、閉形式解:

$$
w_{ij}(t) = w_{ij}(0)\, e^{-\lambda(C_{ij})\, t}
\tag{2.2}
$$

**物理的意味**: $\tau \equiv 1/\lambda$ を特性時定数とすると、$w$ は $e^{-1} \approx 0.368$ 倍になるまでの時間。

---

## 3. 離散化

Claim 14 は離散化として $w(t+\Delta t) = w(t)\cdot e^{-\lambda\Delta t}$ を指定する。この離散化は式(2.1)の**厳密解**(C 定数区間)であり、有限差分近似ではない点に注意。

### 3.1 N ステップ後の解

$N$ ステップ適用後:

$$
w(N\Delta t) = w(0) \prod_{k=0}^{N-1} e^{-\lambda(C^{(k)})\,\Delta t}
= w(0)\, e^{-\Delta t \sum_{k=0}^{N-1}\lambda(C^{(k)})}
\tag{3.1}
$$

$C$ が一定なら $w(N\Delta t) = w(0)\, e^{-N\lambda\Delta t}$。

### 3.2 線形近似との誤差

実装以前のプロトタイプは $w \leftarrow w(1-\lambda\Delta t)$ という一次 Taylor 近似を使っていた。誤差は:

$$
|e^{-\lambda\Delta t} - (1 - \lambda\Delta t)| = \frac{(\lambda\Delta t)^2}{2} + O((\lambda\Delta t)^3)
\tag{3.3}
$$

キャノニカル値 $\lambda = 0.01 \cdot (1 + 0.015 \cdot 10^{1.5}) \approx 0.0147$, $\Delta t = 0.005$ では $(\lambda\Delta t)^2/2 \approx 2.7 \times 10^{-9}$ であり数値的には僅少だが、形式的にはClaim違反となる。Phase 1 で厳密 exp 形式に統一した。

---

## 4. 定常解・収束性

### 4.1 定常状態

外部流入 $I_{in}(t)$ なしでは式(2.1)の唯一の定常解は $w^* = 0$(消去状態)。λ > 0 である限り $w(t) \to 0$ は指数収束。

### 4.2 収束速度

$t \geq T_\varepsilon \equiv (1/\lambda)\ln(1/\varepsilon)$ で $w(t) \leq \varepsilon\, w(0)$。すなわちエッジの「消去閾値」 $\theta$ に到達する時間は:

$$
T_\theta = \frac{1}{\lambda(C)} \ln\!\left(\frac{w_0}{\theta}\right)
\tag{4.1}
$$

これが混雑度 $C$ に**減少関数**であることから、混雑度が高いエッジほど早く消去される(Rev.10 の「代謝」設計意図)。

### 4.3 層別依存性

層パラメータ(Master 仕様):

| 層 | $\alpha$ | $\beta$ | $\gamma$ | $\Delta t$ | $\theta$ |
|---|---|---|---|---|---|
| Edge | 1.5 | 0.010 | 0.015 | 0.005 | 0.15 |
| Rare | 0.3 | 0.010 | 0.010 | 0.001 | 0.01 |
| Core | 2.0 | 0.003 | 0.008 | 0.003 | 0.05 |
| Meta | 0.5 | 0.001 | 0.005 | 0.001 | 0.01 |

$C=10$ での $\lambda_\ell \Delta t$:

- Edge: $0.01(1 + 0.015 \cdot 10^{1.5}) \cdot 0.005 \approx 7.4 \times 10^{-5}$
- Core: $0.003(1 + 0.008 \cdot 10^{2.0}) \cdot 0.003 \approx 1.6 \times 10^{-5}$
- Rare: $0.01(1 + 0.010 \cdot 10^{0.3}) \cdot 0.001 \approx 1.2 \times 10^{-5}$

**Rare は Edge の約 6 倍長い時定数** → 孤立情報の保護強度が定量化される。

---

## 5. 安定性(Lyapunov 条件)

### 5.1 適応制御系の安定性

メタ制御手段(Claim 27-32)で $\alpha$ が動的更新される場合、全体系は非線形に拡張される。Rev.11 §7.4 では以下の Lyapunov 条件を要求:

$$
\eta_E \eta_C > \mu_E \mu_C
\tag{5.1}
$$

ここで $\eta$ = 健全性感度、$\mu$ = 不均衡感度。デフォルトの $\eta = 0.15, \mu = 0.08$ では:

$$
0.15 \times 0.15 = 0.0225 > 0.08 \times 0.08 = 0.0064\ \checkmark
\tag{5.2}
$$

### 5.2 証明(スケッチ)

適応制御の誤差 $e_\ell = \alpha_\ell - \alpha_\ell^*$ について、候補 Lyapunov 関数:

$$
V(e) = \frac{1}{2}(e_E^2 + e_C^2)
$$

を取ると、近似線形化下で:

$$
\dot V = -\eta_E e_E^2 - \eta_C e_C^2 + \mu_E e_E e_C - \mu_C e_C e_E
$$

Cauchy–Schwarz により $\mu_E e_E e_C \leq \frac{1}{2}(\mu_E^2 e_C^2 + e_E^2)$ とすると、$\dot V < 0$ の十分条件が(5.1)に相当する。

### 5.3 実装上の保証

- [`MetaController::check_lyapunov_stability`](../../crates/cgb-kdf/src/framework/meta_control.rs) が $(\eta^2 > \mu^2)$ を実行時チェック
- デフォルトコンストラクタは(5.2)を満たす値を供給

---

## 6. 複雑度解析

| 操作 | 理論複雑度 | 実装確認 |
|---|---|---|
| `lambda(C, layer)` | $O(1)$ | [decay.rs:lambda](../../crates/cgb-kdf/src/framework/decay.rs) |
| `compute_edge_decay_probability` | $O(1)$ | 同上 |
| `apply_edge_decay` (E エッジ) | $O(|E|)$ | イテレート×O(1) |
| `probabilistic_prune` | $O(|E|)$ | 同上 |
| `classify` (n ノード, m エッジ) | $O(n + m)$ | degree 集計のみ |
| `find_analogy` (N 候補) | $O(N \log N + N \cdot d)$ | top-k prescreening + d=32次元類似度 |
| `fingerprint eigenvalue` | $O(k^3)$ (k = ego 2-hop サイズ) | symmetric_eigen, nalgebra |
| `step` (Rev12 1サイクル) | $O(|R|\cdot|V|_{core})$ | rare × 候補 |

### 6.1 O(n log n) への経路

明細書 §0003-0005 で謳われる **O(n²) → O(n log n)** は現状 `find_analogy` の top-K prescreening (K=5%) + サブ線形類似度で実現可能。**ただし Phase 7 スケーリング実測では KDF セレクタ全体の empirical 計算量は O(n^1.20)** であり、O(n log n) を厳密に満たすには NodeClassifier の `is_meaningful_rare` 経路の線形化が必要(Phase 8 候補)。詳細は [benchmarks/PHASE7_REPORT.md §5](../../benchmarks/PHASE7_REPORT.md)。

---

## 7. 数値安定性

### 7.1 アンダーフロー

`exp(-λ·dt)` は常に $(0, 1]$ なので、重み列は単調非増加。$w < f_{64}$ の denormal に近づいた場合、Phase 3 で明示的 `max(w, ε)` フラッシング予定。

### 7.2 固有値計算の条件数

`DMatrix::symmetric_eigen` は Householder + QR。2-hop エゴグラフの Laplacian は疎かつ実対称、典型的 spectral radius は $\leq 2 \cdot \max\deg$。条件数 $\kappa \sim 10^2$ 程度で、f64 精度(仮数 52 bit)では相対誤差 $< 10^{-13}$。fingerprint 次元 32 に切り詰めるため、数値安定上も十分。

### 7.3 プラットフォーム決定性

`.exp()` は IEEE 754 準拠 libm でプラットフォーム毎に数ULPの違い得る。Phase 3 で `libm` クレートへの切替を検討。

---

## 8. 参考

- Master Formulas: [../patent/technical/kdf_rev12_complete_jp.md](../patent/technical/kdf_rev12_complete_jp.md)
- 実装: [../../crates/cgb-kdf/src/framework/decay.rs](../../crates/cgb-kdf/src/framework/decay.rs)
- 対応テスト: `test_exp_decay_analytic_solution`, `test_claim12_probabilistic_pruning_rand_comparison`
