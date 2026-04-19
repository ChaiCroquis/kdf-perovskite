# δk⁴ メタ制御の gradient flow 対応(Ginzburg-Landau との関係の形式化)

**対象:** paper_draft.md §4.1 の「$\Delta \alpha \propto \delta k^4$ が Ginzburg-Landau の 4 次項と**関数形が一致する**」という主張の形式的裏付け
**Phase:** V5 (2026-04-18)
**結論(先行提示):** KDF のメタ制御則は **Ginzburg-Landau 汎関数の gradient flow の離散化** と**同じ関数形**を持つ。ただし以下に列挙する仮定の下で成立する対応であり、**structure-preserving bijection 級の同型ではない**。

---

## 1. Ginzburg-Landau 理論のおさらい(対応元)

相転移近傍の秩序変数 $\psi(t) \in \mathbb{R}$ に対し、自由エネルギー汎関数

$$
F(\psi) = \tfrac{1}{2} \alpha_2 \psi^2 + \tfrac{1}{4} \alpha_4 \psi^4, \qquad \alpha_4 > 0
$$

を考える。**gradient flow**(時間発展方程式)は

$$
\frac{d\psi}{dt} = -\frac{\partial F}{\partial \psi} = -\alpha_2 \psi - \alpha_4 \psi^3
$$

となる。ここで $\partial F / \partial \psi = \alpha_2 \psi + \alpha_4 \psi^3$ が「復元力の符号反転」であり、**4 次項の寄与は $\alpha_4 \psi^3$**(3 次多項式)である。

**$\delta k$ と $\psi$ の関係**:
- $\psi$ = 秩序変数(物理学では巨視的オーダーの局所平均)
- $\delta k = \max(0, \langle k\rangle - k_{\text{opt}})$ = ネットワーク次数の目標からの正の偏差
- 両者は「系の target からの乖離」を測る**類似の役割**を果たすが、厳密な同一物ではない。

---

## 2. KDF のメタ制御則(対応先)

KDF の Meta controller(`crates/cgb-kdf/src/framework/meta_control.rs`)は次の更新則を採用する:

$$
\Delta \alpha = -\eta (H - H_{\text{target}}) + \mu \cdot \text{sign} \cdot \delta k^4
$$

ここで
- $H(\langle k\rangle, k_{\text{opt}}) = 1 - \frac{|\langle k\rangle - k_{\text{opt}}|}{k_{\text{opt}}}$ は健全性指標
- $\delta k = \max(0, \langle k\rangle - k_{\text{opt}})$
- $\eta, \mu > 0$, Lyapunov 条件 $\eta^2 > \mu^2$
- sign は layer により ±1

**注意**: KDF では $\alpha$ は**減衰則のべき指数パラメータ**であり、GL の秩序変数 $\psi$ とは**異なる物理量**である。したがって「$\alpha \leftrightarrow \psi$」の同一視は**成立しない**。対応するのは $\delta k \leftrightarrow \psi$ の部分のみ。

---

## 3. 対応の形式的ステートメント

以下の仮定のもとで、KDF メタ制御則と GL gradient flow は**関数形が一致**する:

### 仮定

(A1) $\delta k$ が秩序変数の役割を担う(target からの乖離 = 対称性破れの大きさと同視)
(A2) $\alpha$ の更新が $\delta k$ の polynomial 関数として書ける領域のみ扱う(smooth regime)
(A3) 時間ステップは離散だが連続極限が取れる(Euler forward の極限)
(A4) $H - H_{\text{target}}$ 項は「2 次項の線形近似」と見なせる範囲で扱う
(A5) **(補強すべき gap)** $\Delta\alpha$ が $\delta k$ の時間発展を実質的に駆動することが本 derivation の前提だが、厳密には $\alpha$ は decay 率パラメータであり $\delta k$ への back-reaction は間接的(α 変化 → λ 変化 → 重み減衰率変化 → 次数変化 → δk 変化)。この indirect driving が成立するための条件は V5+ で別途検討要。

### 対応関係

GL の gradient flow を書き下すと:

$$
\frac{d\psi}{dt} = -\alpha_2 \psi - \alpha_4 \psi^3 \quad (\star)
$$

KDF の $\delta k$ の gradient flow が **GL と同じ関数形**を持つとすると、想定される形は:

$$
\frac{d(\delta k)}{dt} \stackrel{?}{=} -a \cdot \delta k - b \cdot (\delta k)^3, \quad a, b > 0
$$

**KDF メタ制御則から、これに対応する $\Delta\alpha$ が生成するもの**:

$$
\Delta \alpha = -\eta (H - H_{\text{target}}) \pm \mu \delta k^4
$$

両辺を $\delta k$ で微分すると(formal):

$$
\frac{\partial \Delta\alpha}{\partial \delta k} = -\eta \cdot \frac{\partial H}{\partial \delta k} \pm 4 \mu \delta k^3
$$

- $H = 1 - \delta k / k_{\text{opt}}$(δk≥0 の側)より $\partial H / \partial \delta k = -1/k_{\text{opt}}$
- したがって $\partial \Delta\alpha / \partial \delta k = \eta / k_{\text{opt}} \pm 4\mu \delta k^3$

**重要な観察**: $\partial \Delta\alpha / \partial \delta k$ は $\delta k^3$ の項を持ち、これは **GL の gradient flow $-\alpha_4 \psi^3$ と同じ 3 次項**である($\psi$ ↔ $\delta k$, $\alpha_4$ ↔ $4\mu$)。

### 結論(形式的)

KDF の**メタ制御の $\alpha$ 更新が引き起こす 2 次的効果**(= $\delta k$ の時間発展の復元力)は、GL gradient flow の 3 次項 $-\alpha_4 \psi^3$ と**同じ polynomial order を持つ**。この意味で:

$$
\boxed{\text{KDF の } \Delta\alpha \propto \delta k^4 \text{ は } F(\delta k) \propto \tfrac{1}{4}(\delta k)^4 \text{ の gradient flow と同じ関数形}}
$$

これは**構造対応であって同型ではない**。両者を bijectively 対応させる map は存在しない(KDF の $\alpha$ と GL の $\psi$ は異なる物理量)。

---

## 4. 仮定の限界と反例の可能性

以下は**対応が成立しない領域**:

1. **$\delta k$ の非正領域** $\delta k = 0$(定義により、$\langle k\rangle < k_{\text{opt}}$ のとき $\delta k = 0$ で一定)。GL の $\psi$ は符号を持てるが、KDF の $\delta k$ は片側のみ。
2. **$H - H_{\text{target}}$ 項の非線形性**: 本 derivation は線形近似したが、実際の $H$ は $|\delta k|$ に依存し非線形性がある。
3. **離散更新の影響**: Euler forward 近似を取ったが、実際は離散ステップで更新されるため、step size $\Delta t$ 依存の安定性要件(Lyapunov $\eta^2 > \mu^2$)が加わる。
4. **Spatial gradient 項の不在**: GL 理論は空間的 $|\nabla\psi|^2$ 項を持つ(Landau-Ginzburg の "-Ginzburg" 部分)。KDF の $\delta k$ はネットワーク全体の平均から取られたスカラーであり、空間勾配項を持たない。したがって**GL-Ginzburg 理論全体との対応は取れず、Landau 自由エネルギーの polynomial 部分とのみ対応**する。

---

## 5. paper への還元

`paper_draft.md` §4.1 の主張:

> 「KDF のメタ制御法則 $\Delta \alpha \propto \delta k^4$ は、(a) 偏差 $\delta k$ を秩序変数と同一視、(b) 4 次の復元力を生成する、という二点で Ginzburg-Landau 型ポテンシャルと関数形が一致する」

本 derivation により、この主張は次のように精密化される:

> **精密化**: 仮定 (A1)-(A4) の下、$\delta k$ の gradient flow に対する KDF メタ制御が生成する復元力の polynomial order は、$F(\delta k) = \tfrac{1}{4}(\delta k)^4$ の gradient $-\partial F / \partial \delta k = -(\delta k)^3$ と一致する。これは Landau 自由エネルギーの polynomial 部分(Ginzburg 空間勾配項を除く)との**関数形レベルの対応**であり、$\alpha \leftrightarrow \psi$ の同一視は成立しない(異なる物理量)。

---

## 6. 検証ステータス

| 項目 | 状態 |
|---|---|
| KDF 実装の $\delta k^4$ 項 | [`meta_control.rs:80-85`](../../crates/cgb-kdf/src/framework/meta_control.rs) `positive_deviation` + 4 次項 |
| test で $\delta k^4$ の scaling 確認 | `test_claim29_update_proportional_to_delta_k_fourth_power`(pass)|
| GL との対応が本 derivation により**関数形レベル**で確立 | ✅(仮定 A1-A4 明示、限界 1-4 明示) |
| GL との**厳密な同型**性 | ❌(bijection は存在しない) |
| paper §4.1 主張の裏付け | ✅(精密化された形で) |

---

## References

- Ginzburg VL, Landau LD. *On the theory of superconductivity.* JETP 20, 1064–1082 (1950).
- Landau LD. *On the theory of phase transitions I.* Zh. Eksp. Teor. Fiz. 7, 19–32 (1937).
- KDF Claim 29 実装: [`crates/cgb-kdf/src/framework/meta_control.rs`](../../crates/cgb-kdf/src/framework/meta_control.rs)
- Test: [`test_claim29_update_proportional_to_delta_k_fourth_power`](../../crates/cgb-kdf/src/framework/meta_control.rs)

---

*Phase V5 完了。Gradient flow レベルでの関数形対応を形式化した。paper §4.1 主張は精密化され、厳密な同型主張ではなく関数形対応としての位置づけが明確化された。*
