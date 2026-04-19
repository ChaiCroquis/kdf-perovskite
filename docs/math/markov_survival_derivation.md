# KDF エッジ減衰の非定常 CTMC サバイバル解釈(形式的 derivation)

**対象:** paper_draft.md §4.3 の「$w \leftarrow w \cdot \exp(-\lambda(C) dt)$ は連続時間 Markov 過程のサバイバル確率と**構造的に対応**する」主張の形式的裏付け
**Phase:** V4 (2026-04-18)
**結論(先行提示):** KDF のエッジ重み減衰則は、**非定常生成作用素を持つ片側-CTMC のサバイバル関数**の関数形と一致する。ただし、重み $w$ は確率質量ではなく連続量であるため、**厳密な Markov 過程としての確率論的解釈は成立しない**。本文書はこの対応が成立する条件と成立しない条件を明示する。

---

## 1. 古典 CTMC のサバイバル関数(対応元)

**一様(homogeneous)CTMC**: 状態 $X_t$ が速度 $\lambda$(定数)で "jump" するとき、時刻 $t$ まで jump が起きない確率(サバイバル関数)は

$$
S(t) = \Pr[\text{no jump in } [0, t]] = \exp(-\lambda t). \quad (1)
$$

**非一様(non-homogeneous)CTMC**: 速度が時刻依存 $\lambda(s)$ のとき、

$$
S(t) = \exp\left(-\int_0^t \lambda(s) \, ds\right). \quad (2)
$$

**状態依存(state-dependent rate)CTMC**: 速度が状態 $X$(KDF の場合、congestion $C$)に依存 $\lambda(C_s)$ のとき、同じ形で

$$
S(t) = \exp\left(-\int_0^t \lambda(C_s) \, ds\right). \quad (3)
$$

離散時間近似(Euler forward, step size $dt$)では、各 step の単独サバイバル確率は:

$$
S(t + dt \mid t) = \exp(-\lambda(C_t) \cdot dt). \quad (4)
$$

---

## 2. KDF のエッジ減衰則(対応先)

KDF の実装([`decay.rs:320-326`](../../crates/cgb-kdf/src/framework/decay.rs)):

```rust
let lambda = self.master_params.lambda(self.compute_edge_congestion(u, v), layer);
let dt = self.master_params.dt_for_layer(layer);
let survival = (-lambda * dt).exp();
*weight *= survival;
```

数式で書くと:

$$
w_{uv}(t + dt) = w_{uv}(t) \cdot \exp(-\lambda(C_{uv}(t)) \cdot dt). \quad (5)
$$

式 (4) と (5) の**関数形は完全に同一**。しかし、以下の点で厳密な CTMC 解釈は成立しない。

---

## 3. 対応が成立する条件

### 成立する対応(関数形レベル)

- **一 step ごとの更新係数** $\exp(-\lambda(C) dt)$ は、CTMC (4) のサバイバル確率と同形。
- **$\lambda(C) = \beta (1 + \gamma C^\alpha)$** は layer ごとに決まる状態依存 rate で、CTMC の state-dependent rate の特殊形と見なせる。

### 成立しない厳密同値

以下の 4 点のいずれかで、KDF は Markov 過程の枠組みから外れる:

**(L1) 重み $w$ は確率質量ではない**
CTMC のサバイバル $S(t) \in [0, 1]$ は「survival 確率」。KDF の $w$ は初期値が任意の正数であり、**正規化されていない連続量**。したがって $w$ は確率解釈を持たない。CTMC との対応が取れるのは「相対的な重み劣化率」の関数形のみ。

**(L2) Generator が非定常で、rare protection が介入する**
CTMC の生成作用素 $Q(s)$ が「滑らか」に時刻依存する場合の理論(式 (2), (3))は成立するが、KDF では Rare 判定により一部エッジが保護され、**generator の jump discontinuity** が発生する(Claim 15, 18)。これは連続時間 Markov の標準仮定に反する。

**(L3) $\lambda(C)$ の feedback loop**
$C_{uv}$ はグラフ構造依存で、**エッジが pruning されると変化する**。つまり生成作用素 $\lambda$ は「CTMC の遷移確率」に対して**依存の方向が逆向き**になる(通常 CTMC では state が rate に依存するが、ここでは state と rate が相互依存する)。これは古典 CTMC の枠を超えた stochastic reaction network に近い。

**(L4) Protected edges + probabilistic pruning の混在**
`apply_edge_decay` と `probabilistic_prune` の 2 経路が同じ $\lambda(C)$ を使うが、前者は**決定論的な連続減衰**、後者は**Bernoulli 試行による離散的削除**。両者の混在は一つの Markov chain として定式化できない。

**(L5) 多エッジ同時更新の非独立性**(Phase V3 audit で指摘された追加項目)
KDF では各エッジが $\lambda(C_{uv})$ で並列に減衰するが、$C_{uv} = \deg(u) + \deg(v)$ は**共有端点を通じてエッジ間に依存性**を持つ。あるエッジが剪定されれば隣接エッジの $C$ が変化するため、各エッジを独立な Markov chain とは見なせず、**joint CTMC としての product-form 生存確率は成立しない**。これは L3(rate-state feedback)と関連するがより具体的な joint-distribution 成立不能性を指す。

---

## 4. 何が正当化されるか

本対応から**直接的に輸入できる結果**:

- **安定性**: サバイバル関数 $\exp(-\lambda dt) \in (0, 1]$ は monotone decreasing in $t$ ← これは定理レベルで保証される。
- **関数形の正当性**: KDF の exp-形は数値的に安定(under/overflow は $\lambda dt$ の範囲で制御可能)。
- **Lyapunov 解析との接続**: 非定常 rate CTMC の Lyapunov 関数として $V = \sum w$ の monotone 性が期待できる(未完了、future work)。

**輸入できない結果**:

- Mixing time / spectral gap / stationary distribution ← これらは **(L1)-(L4) により厳密に適用できない**。"motivating analogy" に留まる。
- Ergodic theorem ← 同上。
- PageRank, diffusion maps との厳密同値 ← 成立しない。

---

## 5. paper §4.3 主張の精密化

paper_draft.md §4.3 の現在の主張:

> 「KDF の重み更新 $w \leftarrow w \cdot \exp(-\lambda(C) dt)$ は関数形として [CTMC サバイバル確率と] 同じである。... ただし、厳密な CTMC 同値性は成立しない。」

本 derivation により:

> **精密化**: KDF の重み更新則は、**非定常・状態依存 rate を持つ片側 CTMC のサバイバル確率 (式 (4))** と**関数形が一致する**。しかし、(L1) 重みが確率質量でないこと、(L2) Rare 保護による generator 不連続、(L3) rate-state 相互依存、(L4) 決定論的減衰と確率的削除の混在、という 4 点により、**標準 CTMC としての確率論的結果(定常分布・spectral gap・ergodic theorem 等)は KDF には直接適用できない**。この対応は motivating analogy であり、KDF 固有の動力学的性質は別途解析する必要がある。

---

## 6. 検証ステータス

| 項目 | 状態 |
|---|---|
| KDF 実装が survival form exp(-λ dt) を使用 | ✅ [`decay.rs:324`](../../crates/cgb-kdf/src/framework/decay.rs) で確認、`*weight *= (-lambda * dt).exp()` |
| PDF 形 λ·exp(-λ dt) multiplier は不使用 | ✅ Round 9 audit で `CONSISTENT` 判定 |
| 関数形が CTMC サバイバルと一致 | ✅ 本 derivation 第 2 節 |
| 厳密な CTMC 同値性 | ❌ 本 derivation 第 3 節(L1-L4)で不成立を明示 |
| paper §4.3 主張の位置づけ | ✅ motivating analogy として精密化 |
| **結論**: KDF の関数形選択は確率論的に妥当、ただし確率論的結果を輸入する主張は avoid すべき |

---

## References

- Norris JR. *Markov Chains.* Cambridge University Press (1997). Ch. 2 (pure jump processes).
- Levin DA, Peres Y. *Markov Chains and Mixing Times* (2nd ed.). AMS (2017).
- Bortolussi L, Hillston J. *Stochastic process algebras and related approaches*. 非一様・状態依存 rate モデルの文献サーベイ。
- KDF 実装: [`crates/cgb-kdf/src/framework/decay.rs`](../../crates/cgb-kdf/src/framework/decay.rs)
- Test: `test_exp_decay_analytic_solution`(1000 step 反復 = exp(-Nλdt) を 1e-10 精度で一致)

---

*Phase V4 完了。CTMC との関数形対応の境界を明示した。paper §4.3 の "motivating analogy" ステータスは、このレベルの形式的裏付けを備えた状態になった。厳密な CTMC 定理の輸入は依然不可であり、paper は正しくその限界を記述している。*
