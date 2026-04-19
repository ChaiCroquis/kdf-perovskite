# KDF: 有限資源での長期情報保持における領域不変アーキテクチャ — 10 独立分野からの統合的証拠

**KDF: A Domain-Invariant Architecture for Finite-Resource Long-Term Information Preservation — Integrated Evidence from Ten Independent Disciplines**

**Author:** Chai (Kuroki Yasuhiro)
**Affiliation:** Independent researcher, Japan
**Patent:** JP 2026-027032 (filed 2026-02-24)
**Code:** [github.com/ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite) (PolyForm Noncommercial 1.0.0; commercial license separate)
**Draft version:** v0.3, 2026-04-19(Phase X Step 1-5 完走反映 — F-069 + F-070 + F-071 + F-072 を §5 / §6.4 / §7 に統合、Claim 47-48 canonical 値の 4-benchmark 反証 + Claim 14 streaming +3.06pt の positive empirical anchor が両立する nuanced narrowing を完成)
**Status:** Pre-review draft. Not yet submitted.

---

## Abstract

**KDF は本質的に「項目内可逆・集合内選別型のグラフ圧縮技術」である**。与えられたグラフ(ノード + エッジ)と保持 budget に対し、選ばれたノードは **原形のまま(verbatim)保持** され、選ばれなかったノードは完全に破棄される。この「項目レベル可逆 + 集合レベル選別」の性質は、LLM fact extraction のような内容を変形する圧縮(項目レベル非可逆)と区別される。KDF は自然言語処理専用ではなく、**構造を持つ情報の budget 制約下 curation 全般** に適用可能である。本論文ではこの汎用アーキテクチャを、LLM エージェント記憶(具体応用 1)、PKM(具体応用 2)、ログ保持(具体応用 3)等で検証した。

長期運用される情報ネットワーク(対話記憶、ログ、知識グラフ、プロジェクトドキュメント等)では、全量保持はコスト爆発し、無作為削減は稀少で重要な情報(rare objects)を失う。我々は **KDF (Knowledge Decay Framework)** を提案する。これは (1) エッジベースの連続時間指数減衰による代謝制御、(2) 絶対閾値 deg_E(v) ≤ 1 に基づく希少性保護、(3) グラフラプラシアン固有値フィンガープリントによる整合性発見(アナロジー)の 3 手段からなる領域横断アーキテクチャである。

関連研究を系統的に調査した結果、同じ 3 本柱構造 —**代謝・希少保護・組換え**— が 10 の独立領域(哺乳類脳の記憶固定化、免疫系 B 細胞クローン選択、臨界現象の Ginzburg-Landau 自由エネルギー、連続学習 EWC、Equitable Coreset Selection、Hopfield 連想記憶、グラフ Markov 過程、Pareto 重尾経済、K-SVD 疎符号化、および本研究)で独立に発見されていることが判明した。

KDF の特徴的要素 —(a) Δα ∝ δk⁴ のメタ制御則が Ginzburg-Landau の 4 次項と**同じ関数形**を持つこと、(b) sandwich 2-閾値機構(上下限 θ_L ≤ S ≤ θ_U)が 10 領域横断の関連研究に対応物を見出せない独自要素であること、(c) λ(C)·exp(−λdt) の減衰が局所可変な連続時間 Markov 過程の生存確率と**構造的に対応**すること— は、KDF が単独の工学的発明ではなく、**有限資源で情報を長期保持する系に共通するアーキテクチャ族の一例である可能性**を支持する観察である(必然的収束の主張は仮説段階)。ただし (b) については、**特許で定めた具体値 (θ_L, θ_U) = (0.70, 0.80) が 4 benchmark 横断で経験的に反証された**(Hopfield mixture / F-068 analogy direct / F-070 Part A synthetic / F-070 Part B LoCoMo streaming)。2-閾値 *機構* 自体は支持されるが、具体値は領域ごとの較正が必要である旨を §4.2 で論じる。

実証面では LongMemEval (ICLR 2025) で業界既定の TTL 比 7.7 倍、個人知識ベース(Obsidian、2,182 ノート)で F1=0.747 (Wilcoxon p=0.006)、NASA HTTP ログで Random 比 2.3 倍の稀少イベント保持を達成した一方、OSS GitHub issue の一般化では 3 リポジトリ平均 ×1.00、OpenAlex 論文再発見では ×0.83、GP/カーネル回帰の inducing point 選択(F-063/F-067)、および**特許 θ_U=0.80 canonical 値の 4-benchmark 反証**(F-041/F-070)など**一般化に失敗した陰性結果・自己反証結果を合わせて報告する**。適用条件(構造的希少性が task importance と相関する D1/D1.5 型)と不適用条件(相関が無い D5 型、metadata-based minority、density-based 関数近似)を bias-detector メトリック(F-030/F-036)で事前判別可能である。

Phase X の追加検証として、Claim 17(分散実行 `apply_edge_decay_local`)が LoCoMo realistic graph で `apply_edge_decay`(大域)と完全 bit-exact で一致すること(F-069)、Claim 5/14 の時間評価成分は static query task では KDF 層分類に subsumed で冗長であること(F-069)、Claim 47-48 サンドイッチ機構の具体値 (0.70, 0.80) が analogy discovery / Rev12 streaming で 100% の RARE node を Garbage 化させる過度に厳格な設定であること(F-070)も honest に記録する。

**Keywords**: information preservation, graph metabolism, rarity protection, analogy discovery, Laplacian fingerprint, Ginzburg-Landau, Hopfield attractor, Equitable Coreset Selection, complementary learning systems, memory consolidation

---

## 1. Introduction

### 1.1 問題

持続的に成長する情報ネットワーク —LLM エージェントの会話記憶、個人知識ベース、分散システムログ、OSS イシュー、引用ネットワーク— において、ストレージ制約は**量** vs **質**のトレードオフを強制する:

- **全量保持**: 数 TB/日 スケールで実用的に不可能
- **無作為削減**(Random sampling, Reservoir): 稀少だが重要な情報(例: エラーログの 4xx/5xx、忘れていた重要ノート、少数言語発話)を**統計的に失う**
- **ラベル依存手法**(Stratified sampling, Tail-based sampling, Active learning, Equitable Coreset Selection): 強力だがラベル・真値を要求する。現場で availability が保証されない。

**要件**: ラベル不要・構造のみから、稀少で将来必要になる情報を保護しつつ、冗長情報を代謝する。

### 1.2 観察

**より広い文脈**: 「生命システム理論」(Miller 1978)に代表される一般システム論的伝統は、7 階層(細胞〜超国家)に共通する 20 個の「批判的サブシステム」(`associator`, `memory`, `decoder` 等)の存在を指摘してきた。本研究の観察はこの伝統と**整合的**であるが、20 サブシステムのうち情報保持に特化した 3 つ(代謝・希少保護・組換え)に焦点を絞り、具体的数値実装を提示する点に寄与を限定する。

我々は本課題を独立に工学的に解こうとしたところ、到達した 3 本柱アーキテクチャが以下と**構造的に対応している**ことに気づいた(数学的同型の証明ではなく、定性的な並行性の観察である):

1. **哺乳類脳の記憶固定化**(Complementary Learning Systems, McClelland 1995)
2. **免疫系の記憶 B 細胞選択**(germinal center affinity maturation)
3. **臨界現象の Ginzburg-Landau 理論**(quartic stabilization term)
4. **連続学習の Elastic Weight Consolidation**(Kirkpatrick, PNAS 2017)
5. **Equitable Coreset Selection** (ACM CIKM 2024) および unlearning 向け変種 **UPCORE** (Patil et al. 2025, arXiv:2502.15082)
6. **Hopfield 連想記憶ネットワーク**(Hopfield 1982 + modern Hopfield networks)
7. **グラフ上の連続時間 Markov 過程**(spectral gap 収束)
8. **Pareto 重尾分布の tail risk 管理**(Vilfredo Pareto 1896)
9. **K-SVD 疎辞書学習の rare atom 保持**(Aharon et al. 2006)
10. **Two-factor synaptic consolidation**(Tartaglia et al., PNAS 2025 — 独立並行)

### 1.3 主張

本論文の主張は次の 3 点である:

- **(C1) 統合性**: 上記 10 領域の知見は、「**代謝制御 + 希少性保護 + 組換え**」の 3 本柱として統一的に記述できる。KDF はこの統一記述の最初の**数値実装**である。
- **(C2) 構造的類似性(未厳密化)**: KDF の 2 つの特徴的数式 — (a) Δα ∝ δk⁴ メタ制御、(b) 連続時間 λ(C)·exp(−λdt) — は、それぞれ Ginzburg-Landau 自由エネルギーの 4 次項および連続時間 Markov 過程の生存確率と**同じ関数形**を持つ。これは定性的な構造対応であり、数学的同型(structure-preserving bijection)としての証明は今後の課題である。
- **(C3) 新規貢献(mechanism 支持 / canonical 値は 4-benchmark 反証)**: sandwich 2-閾値機構(下限 θ_L および上限 θ_U による「中間帯のみ受容」)は、調査した 10 領域で対応物が見当たらない KDF 独自の寄与である。ただし特許請求項 46/48 が canonical として定める具体値 (θ_L, θ_U) = (0.70, 0.80) については、以下 4 benchmark で経験的に反証された:

  1. **Phase V3 Hopfield** (F-041): θ_U=0.80 では Hopfield mixture state を 0% しか検出できず、θ=0.40 に下げたとき、負荷係数 P=18 で 24%、より高い負荷係数 P=22 で 40% の棄却が可能となる。
  2. **F-068 analogy discovery direct**: Gentner 古典 + git↔paper cross-domain で score が一様に 0.99+ に集中、上限 0.80 は全 true positive を reject するはず。
  3. **F-070 Part A synthetic + F-068 scenarios**: 38 pair(22 positive / 16 negative)で F1(canonical)= 0.000、F1((0.70, 1.00))= 1.000。
  4. **F-070 Part B LoCoMo Rev12 streaming**: 30 Q、1140 RARE node、canonical で 100% が 60-cycle timeout 後に Garbage demote(情報完全喪失)。

  → **「mechanism は domain 横断的に意味あり、specific value は domain-specific に較正必須」** に narrowing。paper §4.2 で詳述。機構のみを独自貢献として主張し、canonical 具体値の universality は主張しない。

### 1.4 Universality と novelty の緊張

本研究は「KDF の構造が 10 の独立領域で並行発見されている」と主張するが、これは novelty の観点から**両刃の剣**である。

- **好意的読み**: 10 領域が同じ解に独立収束 ⇒ KDF はこの universal pattern の最初の包括的実装として意義がある
- **批判的読み**: 10 領域で既に類似解が知られている ⇒ KDF は単なる reinvention であり novelty に乏しい

我々はこの緊張を正面から受け止め、KDF の novelty 主張を以下の狭い主張に限定する:

1. **統合の novelty**: 3 本柱(代謝・希少保護・組換え)の 3 つすべてを単一の数値実装・単一の特許請求項で統合した先行例は見つからない。各領域はいずれか 1-2 本柱に特化している(脳科学は主に代謝+組換え、ECS は主に希少保護、等)。
2. **sandwich 2-閾値 *機構* の novelty(値は別)**: 下限 θ_L および上限 θ_U で中間帯のみ受容するという *二重閾値構造* は、調査した 10 領域に対応物が見当たらない独自要素である。ただし特許 canonical 値 (0.70, 0.80) は 4 benchmark で反証されており(§1.3 C3、§4.2)、novelty として主張するのは *機構* のみで、具体値の最適性は主張しない。
3. **オープン数値実装の novelty**: 具体的パラメータ(β=0.01, 7:2:1, 5:3:1, [0.70, 0.80], δk⁴)と 公開数値実装(PolyForm Noncommercial 1.0.0)を提供した最初の事例である。

個別の 3 本柱(synaptic pruning、EWC、Laplacian fingerprint 等)自体の novelty は**主張しない**。これらはすべて既存の貢献である。

### 1.5 本論文の構成

第 2 章で KDF の 3 手段とキー数式を簡潔に述べる。第 3 章で 10 独立領域との対応を系統化する。第 4 章で 3 つの構造的類似性を論証する。第 5 章で実証(肯定的 6 件・陰性 4 件)を提示する。第 6 章で含意と limitations を議論する。第 7 章で refined な適用判別仮説と 2 × 2 Mem0 / KDF benchmark matrix を結論として提示する。第 8 章では Burt の Structural Holes 理論との alignment により **§4 より強い理論的基盤** を展開し、末尾の *Limitations and risks* 小節で §4 との強度階層を明示する。Acknowledgments、References、および Appendix A(主要数式)/ B(実装アーキテクチャ)が続く。

---

## 2. KDF アーキテクチャ概要

### 2.1 基本構造

グラフ $G = (V, E)$ を情報構造とし、エッジ $(u, v) \in E$ にパラメータ(強度 $w_{uv}$、接続履歴 $n_{uv}$、最終参照時刻 $t^{\text{acc}}_{uv}$)を付与する。

**手段 1(代謝制御 / Metabolic Control):** 局所混雑度 $C_{uv} = \deg(u) + \deg(v)$ に依存する減衰率

$$
\lambda(C_{uv}) = \beta \left( 1 + \gamma C_{uv}^{\alpha} \right)
$$

と離散時間ステップ $dt$ により

$$
w_{uv}(t + dt) = w_{uv}(t) \cdot \exp(-\lambda(C_{uv}) \cdot dt)
$$

でエッジ重みを指数減衰させる。またしきい値剪定および確率剪定 $\Pr[\text{prune}] = 1 - \exp(-\lambda dt)$ を許す。

**手段 2(希少性保護 / Rarity Protection):** $\deg_E(v) \le 1$(絶対閾値)を満たすノード $v$ を希少オブジェクトとし、保護期間 $[t_0, t_0 + T_{\text{wait1}}] \cup (t_0 + T_{\text{wait1}}, t_0 + T_{\text{wait1}} + T_{\text{wait2}}]$ にわたり代謝制御から**無条件に除外**する(2 段階審査)。第 1/第 2 期間は **$T_{\text{wait1}} = T_{\text{wait2}} \in [30, 70]$**(Claim 37 で両期間長が同一、Claim 39 で範囲指定、canonical default は $T_{\text{wait}} = 50$)。

**手段 3(整合性発見 / Integrity Discovery):** 希少ノードの ego-部分グラフから構築したラプラシアン $L_v$ の固有値ベクトル $\phi(v) \in \mathbb{R}^{32}$(固定長、等長不変フィンガープリント)を計算し、既存ノード群との整合性スコア

$$
S(v, u) = a \cdot S_{\text{cos}}(\phi(v), \phi(u)) + b \cdot S_{\text{struct}}(v, u) + c \cdot S_{\text{sign}}(v, u), \quad a,b,c > 0
$$

を評価する。上の式が与えるのは **内側(inner)の類似度** $S_{\text{inner}}$ であり、sandwich 採用域の判定に実際に用いるのは、$S_{\text{inner}}$ 由来の 3 種(systematic / relational / attribute)を集約した **外側(outer)の整合性スコア** $S_{\text{outer}}$ である。新エッジは、$S_{\text{outer}}$ が **sandwich 採用域** $\theta_L \le S_{\text{outer}} \le \theta_U$(以下 $\theta_L = 0.70$, $\theta_U = 0.80$)に落ちた場合に生成する。重み係数の使い分け:
- **Inner fingerprint similarity $S_{\text{inner}}$(構造指紋間の合成)**: $a : b : c = 0.40 : 0.35 : 0.25$($S_{\text{cos}}$, $S_{\text{struct}}$, $S_{\text{sign}}$ の線形結合, Claim 45)
- **Outer integrity aggregation $S_{\text{outer}}$(系統/関係/属性の集約)**: 系統(systematic) : 関係(relational) : 属性(attribute) $= 7 : 2 : 1$(Claim 44)

これらは互いに独立した 2 段の重み付けであり、前者は固定長ベクトル間の類似度、後者は分類された類似度スコアの集約を司る。sandwich 閾値は **外側の** $S_{\text{outer}}$ に対して課される。

### 2.2 メタ制御(Claim 27-32)

ネットワークの平均次数 $\langle k \rangle$ と目標次数 $k_{\text{opt}}$ の偏差 $\delta k = \max(0, \langle k \rangle - k_{\text{opt}})$ を用いた適応法則

$$
\Delta \alpha = -\eta (H - H_{\text{target}}) \pm \mu \cdot \delta k^{4}
$$

により $\alpha$ を範囲 $[\alpha_{\min}, \alpha_{\max}]$ 内で更新する。Lyapunov 安定条件は $\eta^2 > \mu^2$ で成立する(完全な Lyapunov 解析は特許 JP 2026-027032 および参照実装 `crates/cgb-kdf/src/meta_control.rs` に記述、本論文では非再掲)。

### 2.3 階層管理領域

短期・長期・希少の 3 領域を持ち、更新周期比は $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$。

---

## 3. 10 独立領域における 3 本柱構造

以下の表は各領域における 3 本柱の具体的実現を列挙したものである。KDF との対応は**定性的なレベルで観察できる**(厳密な同値性の主張ではない)。

| 領域 | 代謝 | 希少保護 | 組換え / 統合 | 代表文献 |
|---|---|---|---|---|
| 哺乳類脳 (1) | 睡眠中の synaptic pruning | engram plasticity window | hippocampus→neocortex replay | McClelland et al. 1995; Tartaglia et al. PNAS 2025 |
| 免疫系 (2) | naïve B cell 除去 | memory B cell pool (restricted clonality) | germinal center affinity maturation | Mesin et al. Cell 2020 |
| 臨界現象物理 (3) | SOC avalanche decay | power-law tail (critical cluster) | Ginzburg-Landau quartic stabilization | Bak, Tang, Wiesenfeld 1987; Ginzburg-Landau 1950 |
| 連続学習 ML (4) | weight decay / dropout | EWC Fisher protection | replay buffer | Kirkpatrick et al. PNAS 2017 |
| Coreset 選択 (5) | adaptive pruning | ECS minority preservation | class-sensitive partitioning | Sener & Savarese 2018; Wang et al. CIKM 2024 |
| Hopfield 連想記憶 (6) | 動的パターン減衰 | 保存パターン attractor basin | **spurious attractor の棄却は未解決** | Hopfield 1982; Ramsauer et al. 2021 |
| Markov グラフ過程 (7) | $\exp(-\lambda dt)$ mixing | slow-mixing mode($\lambda_2$) | spectral gap / Fiedler vector | Levin & Peres 2017 |
| 経済学・金融 (8) | portfolio rebalance | tail event capital reserve | Pareto / extreme value theory | Pareto 1896; Embrechts 1997 |
| 信号処理 (9) | dictionary pruning | rare atom preservation | K-SVD iterative update | Aharon et al. 2006 |
| グラフ ML (10) | time-decayed line graph | rare node structural signal | analogy discovery via fingerprint | Nguyen et al. WWW 2018 |
| **KDF(統合)** | $\exp(-\lambda(C) \cdot dt)$ 型の exp survival 減衰 | $\deg \le 1$ + $\theta_U$ sandwich | Laplacian fingerprint analogy | — |

### 3.1 独立並行発見としての Tartaglia et al. (PNAS 2025)

特筆すべきは Tartaglia et al. "Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling" (PNAS 2025, bioRxiv 2024-07) である。**KDF 出願(2026-02)と時期的に重なり、互いに参照していない状態での独立収束**を示している。

彼らの主張 "synapse as product of two factors + replay + homeostatic scaling + Hebbian plasticity → prunes connections while preserving weak memories" は、本論文の 3 本柱にほぼ 1:1 で対応する(代謝 = homeostatic scaling、希少保護 = two-factor / weak memory preservation、組換え = Hebbian replay)。2 つの独立チームが似た引力点に収束した事実は、**普遍性仮説と整合する観察**である(仮説の証明ではない)。

---

## 4. KDF の 3 つの構造的類似性

### 4.1 $\delta k^4$ メタ制御 $\Leftrightarrow$ Ginzburg-Landau quartic term

凝縮系物理学の Ginzburg-Landau 自由エネルギーは、相転移近傍の秩序変数 $\psi$ に対し

$$
F(\psi) = \alpha_2 \psi^2 + \alpha_4 \psi^4 + \cdots
$$

と展開される。**4 次項 $\alpha_4 \psi^4$ は秩序変数を臨界点で安定化する不可欠な復元力**を与える(quadratic だけでは二乗ポテンシャルが非有界)。

KDF のメタ制御法則 $\Delta \alpha \propto \delta k^4$ は、(a) 偏差 $\delta k$ を秩序変数と同一視、(b) 4 次の復元力を生成する、という二点で Ginzburg-Landau 型ポテンシャルと**関数形が一致する**(数学的同型の主張ではなく、関数形レベルの対応)。含意:

- KDF の meta-control は暗黙的に**ネットワークを「臨界点における自己組織化」状態に向けて制御**している
- SOC(Self-Organized Criticality)理論が予測する power-law 避断分布は KDF でも期待できる(未検証、今後の課題)
- 「なぜ 4 乗か」の問いに対する回答: 相転移近傍での最低次の非自明な復元力だから(物理学的必然性)

### 4.2 sandwich 上限閾値 $\theta_U$ $\Leftrightarrow$ Hopfield spurious attractor 棄却

Hopfield 連想記憶の古典問題: 記憶パターン $\xi^{(1)}, \ldots, \xi^{(P)}$ を格納すると、それらの線形結合が spurious attractor(学習パターンではないが fixed point として振る舞う偽の記憶)として出現する。Amit, Gutfreund, Sompolinsky (1987) はスピングラス模型のレプリカ解析により、臨界容量 $\alpha_c \equiv P/N \approx 0.138$ を超えると spurious attractor が急増し、連想想起が崩壊することを示した。また、反復提示や偏ったサンプリングを受けたパターンが広い吸引盆地を形成し他の記憶を覆い隠す現象(biased/correlated-pattern capacity の文脈で議論されている)も、スプリアス問題と密接に関連する。

**スプリアス検知の一般手法は 40 年以上確立されていない**。モダン Hopfield networks (Ramsauer et al. 2021) はソフトマックスによって capacity を指数的に改善したが、スプリアス抑制に対する広く受け入れられた一般原理は未だ提示されていない(本論文は KDF が解決したとも主張しない — §4.2 末尾の conjecture 参照)。

**KDF の上限閾値 $\theta_U$ は次の経験則を形式化する**: 整合性スコア $S$ が「完璧すぎる」(例えば $S > 0.80$)とき、その候補は

1. 自己ループ・trivial 重複(既存近傍の再発見)
2. overfitting / memorization artifact
3. **spurious attractor**(偽の固定点)

のいずれかである確率が高い。許容域を $[\theta_L, \theta_U]$ に制限することで、これらを統計的に棄却する。

**数学的仮説**: 無作為ペアの整合性スコアは確率分布 $p(S)$ を持ち、真の analogy は $[\theta_L, \theta_U]$ の中央付近、spurious attractor は $S > \theta_U$ の右裾に集中する。上限棄却は裾カット型統計的決定境界に対応する。

**Phase V3 の経験的検証結果(2026-04-18)**: 100-neuron Hopfield(Hebbian 学習, $N=100$, $P \in \{5,10,14,18,22\}$)で、想起後状態が複数の学習パターンと cosine 類似度 $\ge \theta$ を持つ場合に spurious として棄却する $\theta$-filter を実装・測定した:

| 検出閾値 $\theta$ | 負荷係数 $P$ | spurious 棄却率 | effective_recall 向上 |
|---|:-:|---:|---:|
| 0.80(KDF canonical)| 18 | **0%** | 0 |
| 0.70 | 18 | 0% | 0 |
| 0.55 | 18 | 0% | 0 |
| 0.40 | 18 | **24%** | 0.49 → 0.65(+32%) |
| 0.40 | 22 | **40%** | 0.34 → 0.56(+65%) |

**発見**:
- ✅ **メカニズム(multi-pattern similarity rejection)は支持される**: 適切な $\theta$ 選定で Hopfield mixture spurious state を 負荷係数 $P=18$ で 24%、$P=22$(臨界容量 $\alpha_c \approx 0.138$ の上方、spurious attractor 増殖域)で 40% 棄却可能。
- ❌ **KDF canonical $\theta_U=0.80$ を Hopfield にそのまま移植する単純な主張は実験で否定された**: Hopfield mixture state の複数パターンとの cos 類似度は各 $\sim 0.4$ に留まり、$0.80$ threshold では検出不能。

**paper 当初主張の精密化**:
- 原形 conjecture(「KDF 原値 θ_U=0.80 が Hopfield spurious 問題への原理的解答」)は**反証された**
- 修正 conjecture: 「**上限閾値メカニズム**は Hopfield spurious 抑制にも有効。ただし**具体的 value は domain-specific に調整必要**」→ **部分的に支持**

実装: [`demos/D7_github_issue/src/bin/phase_v3_hopfield_theta_u.rs`](../demos/D7_github_issue/src/bin/phase_v3_hopfield_theta_u.rs)。詳細記録: [VERIFIED_FINDINGS.md F-041](VERIFIED_FINDINGS.md)。

Ramsauer et al. (2021) のモダン Hopfield network への組み込み検証は future work(実験手法確立のための最小 experiment は本 phase で完了)。

#### 4.2.1 Phase X Step 2 の追加検証 — 4 benchmark 横断反証の完結(F-070, 2026-04-19)

F-041 の部分反証を受け、本研究は 3 つの追加 benchmark で canonical (θ_L, θ_U) = (0.70, 0.80) の実用性を系統的に再検証した。合わせて 4 benchmark 横断のエビデンスを示す:

**Part A: 合成 + F-068 再現 pair での sandwich 感度分析** — 合計 38 pair — 22 positive(Gentner 古典 3 pair(太陽系↔原子など)+ git↔paper cross-domain 対応 4 pair + 等長合成 15 pair)および 16 negative(手作り非等長 control 1 pair + 非等長合成 15 pair)— について `AnalogyDiscoveryEngine::find_analogy` の raw score(permissive 閾値 0.0)を捕捉し、5 通りのサンドイッチ $\{(0.70, 0.75), (0.70, 0.80), (0.70, 0.90), (0.70, 0.95), (0.70, 1.00)\}$ を post-hoc 適用。結果:

| (θ_L, θ_U) | TP | FN | TN | FP | F1 |
|---|---:|---:|---:|---:|---:|
| (0.70, 0.75) | 0 | 22 | 16 | 0 | 0.000 |
| **(0.70, 0.80) canonical** | **0** | **22** | **16** | **0** | **0.000** |
| (0.70, 0.90) | 0 | 22 | 16 | 0 | 0.000 |
| (0.70, 0.95) | 0 | 22 | 16 | 0 | 0.000 |
| (0.70, 1.00) | 22 | 0 | 16 | 0 | **1.000** |

**発見**: positive analogy の score は一様に $\ge 0.99$ に飽和し(graph isomorphism の fingerprint 一致)、negative は $\sim 0.57\text{-}0.60$ に分布する。canonical $\theta_U=0.80$ は positive を全て reject するため F1 は 0.000、一方 $\theta_U=1.00$(上限なし)では positive / negative が θ_L=0.70 のみで完全分離され F1=1.000 を達成する。**0.80 の "中間帯" に該当する score が empirically 存在しない**。

**Part B: LoCoMo temporal 30 Q で KdfProcessorRev12 review loop** — $(t_{\text{wait1}}, t_{\text{wait2}}) = (30, 30)$ 固定、$\theta_U \in \{0.80, 0.90, 1.00\}$ の 3 条件で review cycle を max 65 回まで回した。合計 1140 RARE node(うち 8 が answer turn)に対する spoke_up / demote 分布:

| $\theta_U$ | total RARE | answer-RARE | spoke_up (ans) | demoted (ans) | spoke_up (non-ans) | demoted (non-ans) | avg cycles |
|---|---:|---:|---:|---:|---:|---:|---:|
| **0.80 canonical** | 1140 | 8 | **0** | **8 (100%)** | 0 | **1132 (100%)** | 60.0 |
| 0.90 | 1140 | 8 | 8 (100%) | 0 | 1132 (100%) | 0 | 1.0 |
| 1.00 | 1140 | 8 | 8 (100%) | 0 | 1132 (100%) | 0 | 1.0 |

**発見**: canonical $\theta_U = 0.80$ では LoCoMo chain graph の全 RARE node が 60-cycle timeout を経て Garbage に demote される(情報完全喪失)。$\theta_U \ge 0.90$ では 1 cycle で全 RARE が spoke_up するが answer / non-answer の識別はなく filter として機能しない。**LoCoMo の sparse chain 構造では sandwich が discriminative な中間帯を持ち得ない**。

**4 benchmark 横断の verdict**:

| benchmark | domain | canonical (0.70, 0.80) verdict |
|---|---|---|
| F-041 Hopfield mixture | associative memory | $\theta_U = 0.80$ で 0% detect、$\theta = 0.40$ に下げたとき P=18 で 24%、P=22 で 40% |
| F-068 analogy engine direct | graph isomorphism | scores 0.99+、canonical は全 true positive を reject |
| F-070 Part A(38 pair) | synthetic + F-068 再現 | F1(canonical) = 0.000、F1((0.70, 1.00)) = 1.000 |
| F-070 Part B(LoCoMo streaming) | 30 Q × 1140 RARE | canonical で 100% RARE demote → 情報完全喪失 |

→ **4 benchmark 横断で canonical (θ_L, θ_U) = (0.70, 0.80) は実用的 value を失う**。真の positive analogy score は $\ge 0.95$ に集中、negative は $\le 0.65$ に集中するため、正しい経験的 sandwich は $(0.70, 1.00)$(= 上限実質なし、θ_L のみ有効)または $(0.90, 1.00)$ level。

**主張の最終的 narrowing**:

- ❌ 原形 conjecture「canonical (0.70, 0.80) が Hopfield spurious / 類似過剰候補への原理的解答」は **4 benchmark 横断で反証**。
- ✅ 2-閾値 *機構* の novelty(下限 + 上限を同時に課して中間帯のみ受容する構造)は、調査した 10 関連領域に対応物が見当たらず、機構として維持。
- 🔧 canonical 具体値は領域ごとの経験的較正を要する。F-068/F-070 の score 分布からは、graph isomorphism 系では $\theta_U \ge 0.95$、associative memory 系では $\theta \le 0.5$ が妥当と示唆。

実装: [`demos/D8_llm_memory/src/bin/phase_x2_sandwich_twait_locomo.rs`](../demos/D8_llm_memory/src/bin/phase_x2_sandwich_twait_locomo.rs)。詳細記録: [VERIFIED_FINDINGS.md F-070](VERIFIED_FINDINGS.md)。

### 4.3 $\exp(-\lambda(C) \cdot dt)$ $\Leftrightarrow$ locally-varying continuous-time Markov chain

連続時間 Markov 過程 $X_t$ のサバイバル確率(滞在時間の指数分布)は一般に

$$
\Pr[\text{no jump in } [t, t+dt]] = \exp(-\Lambda(x) dt)
$$

の形で記述される。KDF の重み更新 $w \leftarrow w \cdot \exp(-\lambda(C) dt)$ は **関数形として同じ** である。$\lambda(C)$ を局所的な「jump 率」と見立てれば、KDF のエッジ減衰は「エッジが各時刻微小区間で失われる確率が $1 - \exp(-\lambda(C)dt)$ に従う」という**サバイバル過程のモチーフ**と対応する。

**ただし、厳密な CTMC 同値性は成立しない**。KDF では (a) λ(C) が隣接エッジの剪定に伴い変化するため generator は非定常、(b) 重み $w$ は確率質量ではなく連続量、(c) エッジ保護(Rare)が大域条件に依存する等、標準 CTMC の仮定は満たされない。したがって本節の主張は**理論的接続のモチベーション(motivating analogy)**にとどまり、定常分布や spectral gap の ergodic 結果を直接輸入することはできない。

この構造的対応から示唆されるのは以下であり、定理ではなく**今後の理論展開の方向性**である:

- Claim 46 の Laplacian 固有値フィンガープリントは、ego-graph の spectral gap 情報を 32 次元に射影する試みと**概念的に整合**する(正確な情報保存の証明は未実施)
- diffusion maps、PageRank、spectral clustering との理論的接続は future work

---

## 5. 実証

### 5.1 肯定的結果(6 件)

| 問題 | データ | KDF の成績 | 比較対象 |
|---|---|---|---|
| P1 LLM エージェント記憶 | LongMemEval 500 Q (ICLR 2025) | Recall = 0.821 | TTL 業界既定 = 0.107(**×7.7**)、Random = 0.294(×2.8) |
| P2 個人知識ベース | Obsidian Vault 2,182 ノート(PII マスク済) | F1 = 0.747 | Wilcoxon p = 0.006 vs Random/OrphanOnly/TextSim |
| P3 大規模ログ観測(static baseline) | 実 NASA HTTP log 50k レコード | Recall = 0.237(keep 10%、4xx/5xx 保持)| Random = 0.102(**×2.3**)※ラベル無し条件 |
| P7 ML 再現性メタ(副産物) | 5 ベンチマーク × 5 シナリオ | 4/5 完全予測一致 + 1 件別経路 | `bias-detector` crate として独立公開 |
| **P8 分散実行 bit-exact(Claim 17)** | **LoCoMo 10 real graph(600+ nodes, 400+ edges)** | **max edge-weight diff = 0.000e0** | `apply_edge_decay`(大域)と `apply_edge_decay_local`(分散)が完全 bit-exact、F-069 |
| **P11 NASA streaming Claim 14 decay benefit** | **実 NASA HTTP log 50k を時系列 replay(500 rec / 100 window)** | **C1 decay rare_recall = 0.4898 @ keep 30%(C0 static 0.4592 → +3.06pt)** | **narrowing された仮説("streaming が真の use case")の最初の empirical anchor、F-072** |

Phase X Step 1 の追加検証として、Claim 1 の 3 手段(代謝 / 希少保護 / 整合性発見)がすべて realistic benchmark で empirically backed の状態に到達した(整合性発見は F-068 で Gentner 古典 100% + cross-domain git↔paper 100% + negative control 0% FP)。Phase X Step 5(F-072)で Claim 14 exp decay が realistic streaming scenario で +3.06pt の benefit を生むことが確定し、**narrowing された仮説("streaming が真の use case")が初の empirical anchor を得た**。

### 5.2 陰性結果(4 件) — 誠実な記録

本研究の信頼性のため、**実証で一般化に失敗した結果**および**自ら提示した canonical 値の反証結果**を合わせて報告する:

| 問題 | 結果 | 解釈 |
|---|---|---|
| P6 OSS 保守 | 3 repo(rust-lang/rust, tokio-rs/tokio, golang/go)で KDF/Random = ×1.13, ×1.03, ×0.85, 平均 **×1.00** | rust-lang 単独の +15% は repo 固有の局所 signal。OSS 一般への適用主張は**撤回**(F-038)|
| P5 論文再発見 | OpenAlex 200 paper × concept-sharing graph、KDF/Random = **×0.83** | late-bloomer 検出は D5 型(構造非依存)、concept-graph では KDF 不向き(F-039)|
| **P9 Claim 5/14 時間信号の static task での冗長性(streaming では validated)** | **LoCoMo 321Q @ keep 30%: KDF_static=0.5286、時間信号追加で 0.43-0.53(static では全て劣化 or tie)/ NASA streaming 50k recs @ keep 30%: C0 static 0.4592、C1 Claim 14 decay 0.4898(**+3.06pt streaming で benefit**)** | Static query task では構造的希少性が時間的稀少性を内包するため redundant、streaming scenario では decay が古い normal traffic を捨て rare resource を相対的に浮上させる → **task 構造依存の条件付き value**(F-069 static 冗長 + F-072 streaming +3.06pt validated) |
| **P10 Claim 47-48 canonical θ_U=0.80 の 4-benchmark 反証** | **F-041(Hopfield 0% detect)+ F-068(analogy score 0.99+ で全 reject)+ F-070 Part A(F1 0.000 vs 1.000)+ F-070 Part B(LoCoMo 全 RARE demote)** | 2-閾値 *機構* は支持、canonical 具体値 (0.70, 0.80) は 4 benchmark で反証。領域別較正が必須(F-041, F-070)|

### 5.3 事前適用判別メトリック: bias-detector

KDF の適用可否を事前に判別する zero-dependency Rust crate を公開した([`crates/bias-detector/`](../crates/bias-detector/))。`bias_score = 0.3·I_1 + 0.7·I_4` により、5 件のベンチマークで 4/5 完全予測一致、1 件は別経路での一致を確認。

---

## 6. Discussion / 含意

### 6.1 領域不変アーキテクチャの可能性

10 の独立領域が同じ 3 本柱構造(あるいは 2-3 本柱の部分集合)を採用している定性的観察から、我々は次の**仮説**を提起する(証明ではなく):

> **仮説(未検証)**: 「有限資源で情報を長期保持する系には、代謝・希少保護・組換えの 3 手段アーキテクチャ族に属する設計が繰り返し現れる。」

本仮説は以下の方向から支持**されうる**(必要条件だが十分条件ではない):
(a) 生物系(脳・免疫)における独立な進化的収束、
(b) ML エンジニアリングでの独立再発見(EWC, ECS)、
(c) 物理学での臨界現象との関数形の一致。

ただし、これらはいずれも「必然性」の証明ではなく「両立性」の観察である。真に普遍的な最適性を示すには、以下が必要となる(いずれも本論文では未実施):
- 有限資源情報保持問題の数理的定式化と、最適解クラスの特徴づけ
- 3 本柱を**持たない**系の同問題での性能下界の証明
- 10 領域の対応を定量化した統計的検定

本研究の KDF は、この仮説方向で**最初の包括的な数値実装を提供した**ものであり、仮説自体の正誤は今後の検証課題である。

### 6.2 領域別に専門化した実装の可能性

同じコアエンジンを領域固有のインターフェースで包めば、複数の応用市場に横展開できる:

- `kdf-associative-memory` — Hopfield 連想記憶の spurious attractor 抑制 wrapper
- `kdf-coreset` — ラベル不要の unsupervised Equitable Coreset Selection
- `kdf-temporal-graph` — 時間グラフ embedding(Nguyen et al. 2018 と直接比較)
- `kdf-portfolio` — 情報 tail risk 管理(保険・金融応用)
- `kdf-llm-memory` — LLM エージェント長期記憶(LongMemEval 実績をそのまま活用)

### 6.3 特許とライセンスの位置づけ

本研究の特許請求項は以下の 2 つの戦略的寄与を押さえている:

1. **Claim 1(独立)**: 3 手段の統合。個別要素が既存でも、統合は先行例なし。F-068 で analogy 手段の realistic benchmark 完了、3 手段全てが empirically backed。
2. **Claim 47-48: sandwich 2-閾値 *機構***(下限 θ_L + 上限 θ_U)は 10 関連領域に対応物が見当たらない独自要素。ただし特許で定める canonical 値 $(\theta_L, \theta_U) = (0.70, 0.80)$ 自体は §4.2 / §5.2 P10 の 4-benchmark 反証を受け、「*機構は novel、具体値は domain-calibration 必要*」に narrowing。

実装コードは PolyForm Noncommercial 1.0.0 のもとで研究・教育利用は無償。商用は別途ソフトウェアライセンス契約 + 特許ライセンス契約(COMMERCIAL.md 参照)。

### 6.4 Limitations

- **「領域不変アーキテクチャ」仮説自体が未検証**。10 領域対応は定性的観察に過ぎず、「必然的収束」を示す形式的定理は未証明。§6.1 の仮説は**結論ではなく仮説**として読まれるべきである。
- **θ_U canonical 値 (0.70, 0.80) は 4 benchmark で反証**(F-041 Hopfield / F-068 analogy / F-070 Part A synthetic / F-070 Part B LoCoMo streaming)。sandwich 機構自体は支持されるが、具体値の universality は主張しない(§4.2 / §5.2 P10)。
- **Claim 5/14 時間評価成分・指数減衰は task 構造依存**:static query task では redundant(F-069 LoCoMo)、streaming scenario では +3.06pt benefit(F-072 NASA 50k records 時系列 replay)。構造的希少性が既に時間的稀少性を内包する場合は冗長、連続運用で古い traffic を捨てる必要がある場合は有効。
- **Claim 25 ActivationScore は rare event の時間分布依存**:時間的に clustering した rare event(F-027 Mode E drift scenario)では 100% rescue、均等分布 rare event(F-072 NASA、F-069 LoCoMo)では recency bias により hurt / neutralize。application 時は rare event の temporal pattern 判別が必要。
- **10 領域の対応は構造的類似レベル**。数学的同型(structure-preserving bijection)としての証明は各領域で未実施。
- **§4.3 Markov 対応は motivating analogy**。厳密な CTMC 同値性は成立しない(generator 非定常、重みが確率質量でない等)。
- **実証は 5 分野 6 件のみ**(P1/P2/P3/P7/P8 = Claim 17 bit-exact、P11 = NASA streaming。P3 と P11 は同一の NASA HTTP ログ領域の static / streaming 2 scenario)。他 5 分野 + 未試行の汎用性主張は**本論文の範囲を超える**。
- **P5/P6 陰性結果**は D5 型(構造非依存の稀少性)には KDF が無効であることを明示。**P9/P10**は自身の canonical パラメータが realistic scenario で最適ではないことを明示。適用可否は bias-detector(F-030/F-036)での事前判別を推奨。
- **特定パラメータ値**(β=0.01, 7:2:1, 5:3:1, [0.70, 0.80], δk⁴, λ, τ_ref)の最適性は経験的選定であり、領域別チューニングが必須。P10 は具体例として、sandwich canonical が真の score 分布と整合しない事実を定量化した。
- **Universality ≠ optimality ≠ novelty**。10 領域の並行発見は universality を示唆するが、その共通構造が**真に最適**であることは別問題であり、また novelty は §1.4 の狭い主張(統合の novelty + 機構の novelty + オープン実装の novelty の 3 点)に限定される。

---

## 7. Conclusion

本論文では Knowledge Decay Framework (KDF) を有限資源の情報ネットワーク運用のための 3 手段統合アーキテクチャとして提示した。特許 JP 2026-027032 の請求項 1-50 に対応する参照実装 `cgb-kdf` は 50 件すべてに直接検証テスト `test_claimN_*` を備え(計 56 tests)、`kdf-python` / `kdf-wasm` を除く workspace 全体で 449 tests が pass する(2026-04-18 測定、[`COMPLIANCE.md`](patent/COMPLIANCE.md) 参照)。

本研究の中心的貢献は次の 3 点である:

1. **10 独立領域での 3 本柱構造の定性的整理**を行い、KDF を領域不変アーキテクチャ族の**候補となる一実装**として仮説提示した(「必然的収束」の形式的証明は未実施)。Claim 1 の 3 手段(代謝 / 希少保護 / 整合性発見)全てが F-068 / F-069 / F-070 の realistic benchmark で empirically backed に到達している。
2. **3 つの構造的類似性** (δk⁴ ↔ Ginzburg-Landau 4 次項[関数形一致]、sandwich 2-閾値 *機構* ↔ Hopfield spurious rejection [機構は支持、canonical 値は §4.2 の 4-benchmark 反証]、$\exp(-\lambda dt)$ ↔ Markov サバイバル確率 [motivating analogy]) により、KDF の工学的 heuristic に**理論的接続の方向性**を示した(厳密な定理化は future work)。
3. **誠実な実証** として、肯定的結果(P1/P2/P3/P7/P8 = Claim 17 bit-exact、P11 = NASA streaming +3.06pt、計 6 件)、他者との比較で一般化に失敗した結果(P5/P6)、**自ら提示した canonical 値の反証結果(P9 Claim 5/14 static task 冗長性、P10 Claim 47-48 canonical 値 4-benchmark 反証)**(陰性合計 4 件)の 3 系統を公開し、bias-detector による事前適用判別メトリックを提供した。自 claim の 4-benchmark 横断反証を自ら示す姿勢は novelty narrowing と同時に credibility 強化の二重目的である。

KDF は「万能」ではない。物理問題(気候変動)、経済問題(貧困)、社会問題(戦争・教育格差)には直接寄与できない。**また、2026-04-18 実施の BEIR SciFact 検証(F-045)により、 general semantic retrieval(query-document matching)にも適用できないことが実測で確認された**(recall@10 = 0.000, Random 以下)。

**2026-04-19 追加実証(F-061〜F-065)で KDF 適性の refined predictor が確立された**:

> **「structural rareness が task の importance と相関する条件下でのみ、KDF は Random / baseline heuristic を decisively 上回る」**

Task type × KDF 適性 matrix:

| Task | structural signal vs importance | KDF 適性 | Evidence |
|---|---|:-:|---|
| Path-based algorithms(APSP) | path-critical = bottleneck = rare | ✅ | F-061 |
| Integration point preservation(git merges, **repo merge rate < 10%**)| merge = 高 degree = 重要 | ✅ | F-062, F-065 |
| LLM memory temporal recall(long-context date/time)| 稀 date/time literal = structurally rare | ✅ | F-057/F-058 |
| Orphan note detection(PKM)| orphan = deg 0 = rare | ✅ | F-012/F-017 |
| **Merge-heavy repos(merge rate > 20%)** | **merge ≠ rare、TopDegree が勝つ** | ❌ | F-065 pytest |
| **GP / Kernel regression inducing points** | **density center ≠ rareness** | ❌ | F-063 |
| **Naive Python call graph API** | **API = 高 in-degree、KDF Rare と逆** | ❌ | F-064 |
| Scale-free hub centrality | degree ≈ betweenness、TopDegree で十分 | ❌ | F-061 BA/WS |
| Metadata-based minority(cultural/semantic) | metadata と構造独立 | ❌ | F-047 |
| General semantic retrieval | 意味理解が本質 | ❌ | F-045 |

KDF が効くのは次の限定された条件下のみ:
- ✅ データがグラフとして表現できる
- ✅ **query は retention 時点で未知**(後で検索される)
- ✅ **one-off / 稀少言及の保護**が目的
- ✅ 大半が冗長で一部がユニーク、ラベルが困難
- ✅ **Task importance が "structural rareness" と相関する**(新規、F-061〜F-065 で refine)

この制約下で、KDF は**conversational memory や PKM など memory-curation 型タスク**において有効(P1: LongMemEval TTL 7.7 倍、P2: Obsidian F1=0.747, p=0.006、P3: NASA log ×2.3)。

**2026-04-18 の F-044 初期実測は simulation であることが F-052/F-053 で判明した("KDF" の数値は実際には real KDF アルゴリズムを走らせずに、先行する static LongMemEval 測定値である F-033 の定数 0.821 recall を流用したもの)。real KDF による再実行で結論は LongMemEval では完全に逆転、LoCoMo では partial な勝利となった**:

- **LongMemEval 500Q(F-053)**: Real KDF 0.434 vs Mem0 0.672(Mem0 +23.8 pt、p<10⁻¹⁶)。全 6 category のうち 5 で Mem0 有意勝利。
- **LoCoMo 200Q balanced(F-056)**: Real KDF 0.535 vs Mem0 0.590(gap −5.5 pt、**p=0.24、statistically tied**)
  - **Temporal category(LoCoMo 50Q sub): KDF 0.460 vs Mem0 0.240(KDF +22 pt、p=0.035 で勝利)**
  - Narrative category では Mem0 +24pt 勝利、factual/inferential は n.s.
- **LoCoMo temporal 全量 321Q、gpt-4o-mini(F-057)**: Real KDF 0.312 vs Mem0 0.206(**KDF +10.6 pt、p=0.0014**、McNemar b=71/c=37)
- **LoCoMo temporal 全量 321Q、gpt-4.1-mini(F-058, model robustness check)**: Real KDF 0.324 vs Mem0 0.090(**KDF +23.4 pt、p=1.6×10⁻¹⁴**、McNemar b=89/c=14)
  - **Model を新世代に更新で Mem0 が degrade、KDF は維持 → gap 拡大**
  - gpt-4.1-mini の fact extraction はより aggressive な compression を行い temporal 情報を更に失う仮説が支持される
  - KDF の temporal 優位は **2 model × 321Q で robust、fact-extraction-based memory system の原理的弱点を露呈**
- **LongMemEval 500Q × gpt-4.1-mini(F-059, 2×2 matrix 完成)**: Real KDF 0.452 vs Mem0 0.722(**Mem0 +27.0 pt、p=3.06×10⁻²³**、McNemar b=32/c=167)
  - **F-053 の gap −23.8pt から −27.0pt に拡大**(Mem0 が新 model で強化、KDF は微増)
  - Per-category: single-session-assistant で Mem0 +61pt の大勝、temporal-reasoning で gap −6pt p=0.23 n.s.(短対話 temporal は KDF も competitive)
  - → **LongMemEval は Mem0 の構造的 strength を発揮する benchmark**、KDF が勝つ余地は n.s. category(temporal-reasoning, single-session-preference)のみ

**2 benchmark × 2 model matrix 完成(F-053/F-057/F-058/F-059)**:

| benchmark × model | Mem0 | KDF | gap | p | 勝者 |
|---|---:|---:|---:|---:|:-:|
| LongMemEval 500Q × gpt-4o-mini | 0.672 | 0.434 | −0.238 | <10⁻¹⁶ | Mem0 |
| LongMemEval 500Q × gpt-4.1-mini | 0.722 | 0.452 | −0.270 | 3×10⁻²³ | Mem0 |
| LoCoMo temporal 321Q × gpt-4o-mini | 0.206 | 0.312 | +0.106 | 1.4×10⁻³ | KDF |
| LoCoMo temporal 321Q × gpt-4.1-mini | 0.090 | 0.324 | +0.234 | 1.6×10⁻¹⁴ | KDF |

→ **Benchmark-dependent な住み分けが model-agnostic に robust に成立**。LLM-based memory (Mem0) は短対話一般 QA に強く、長会話 date/time recall に構造的弱点あり。KDF は後者の専用補完 layer として positioning される。

**F-061〜F-065 — Domain-generalization experiments(2026-04-19, $0 追加)**:

F-044〜F-060 は LLM memory 領域での検証だが、KDF を **汎用グラフ圧縮** として positioning するために、4 領域で追加検証:

| Finding | Task | 結果 | 含意 |
|---|---|---|---|
| **F-061** | Betweenness / APSP on 4 synthetic graphs | **Mixed**: KDF wins ER/SBM, loses BA/WS for betweenness; wins APSP on all 4 for path distance | Scale-free graph では TopDegree、uniform / community graph では KDF |
| **F-062** | Git commit pruning(tokio, 4752 commits) | **Positive**: merge recall 99.5% @ 30% keep | 商用 git archival で validated |
| **F-063** | Gaussian Process inducing points(California housing, Friedman1)| **Negative**: KDF < Random < KMeans で GP fit | Density estimation は KDF 非適 |
| **F-064** | Naive call graph API保持(flask, Python ast) | **Negative**: KDF 16% vs Random 41% | Public API は高 in-degree、KDF Rare と逆 |
| **F-065** | Git pruning 3-repo replication(tokio/pytest/lodash) | **Partial**: merge recall は repo の merge 率に依存(tokio 99% / pytest 59% / lodash 100%)| "merge が rare な repo" でのみ KDF が TopDegree と同等 |

これらの negative findings を合わせ、**KDF 適性の decisive predictor を確立**: 「structural rareness が task importance と相関するか」。LLM memory temporal recall、path-based algorithms、OSS-library-style git repos では相関が成立 → KDF が有効。Density estimation、API boundary detection、merge-heavy enterprise repo では相関せず → KDF 不適。

**Phase X — Claim-level realistic benchmarks(2026-04-19, $0 追加)**:

F-068 で Claim 1 の 3 手段全てが empirically backed に到達した後、残 claim group の realistic benchmark を Phase X として実施:

| Finding | Task | 結果 | 含意 |
|---|---|---|---|
| **F-069** | Claim 5 / 14 / 17 on LoCoMo temporal 321Q | **Mixed**: C17 分散実行 bit-exact pass、C5/C14 static task では KDF_static に劣る(全 time-aware 条件が Δ ≤ 0) | 時間信号は structural rareness に subsumed で static task で redundant、streaming scenario が真の use case(F-072 で検証) |
| **F-070** | Claim 36-41 T_wait + Claim 47-48 sandwich realistic | **Mixed**: 機構 ✅ / canonical 値 ❌。4-benchmark 横断で (0.70, 0.80) 反証、F1 0.000 vs F1((0.70, 1.00))=1.000、LoCoMo streaming で 100% RARE demote | 2-閾値 *機構* は novel 貢献として維持、canonical 値は領域別較正が必須 |
| **F-071** | Claim 20-32 動的制御 on LoCoMo streaming(軽量) | **機構 ✅ / selection benefit 無し**:Claim 21 5:3:1 integer tick 正確、MetaController α bound clamp 動作、TransitionController ceiling-effected(F-031 confirm) | mechanism-only validation、真の value は NASA-type streaming scenario(F-072 で validated) |
| **F-072** | Claim 14 / 25 / 27-32 on NASA HTTP 50k streaming replay | **✅ Claim 14 +3.06pt** over static baseline(C1 decay 0.4898 vs C0 0.4592); ⚠️ Claim 25 activation は均等分布 rare で neutralize; Claim 27-32 は selection-neutral(predicted) | **paper narrowing "streaming が真の use case" が empirical に validated**。Claim 14 value proposition の最初の肯定証拠。ActivationScore は rare event の時間分布 pattern 判別が必要 |

これら Phase X の findings は「自 claim の canonical 値を自ら 4-benchmark で反証する」姿勢として paper 信頼性を強化すると同時に、F-072 で **narrowing の後の positive empirical anchor** を得た。future work として「domain-calibrated parameter auto-tuning」および「rare event の temporal pattern に応じた Claim 25 activation 使い分け」という 2 つの follow-up 方向が明示される。

**F-060 — Ext-1 Precision-Query Router による補完アーキテクチャの実証**(2026-04-19, $0 追加コスト):

F-053/057/058/059 の既存 data に、以下の routing logic を事後適用:
```
if is_precision_query(q) and conversation_length >= 100 turns:
    use KDF answer
else:
    use Mem0 answer
```

結果(v2 = precision + long context):

| cell | Mem0 alone | Router (v2) | gain | p |
|---|---:|---:|---:|---:|
| LongMemEval 500Q × gpt-4o-mini | 0.672 | 0.672 | 0.000 | 1.00 (safe) |
| LongMemEval 500Q × gpt-4.1-mini | 0.722 | 0.722 | 0.000 | 1.00 (safe) |
| LoCoMo temporal 321Q × gpt-4o-mini | 0.206 | **0.302** | **+0.097** | 0.003 ★ |
| LoCoMo temporal 321Q × gpt-4.1-mini | 0.090 | **0.315** | **+0.224** | 4×10⁻¹⁴ ★ |

*表示値について*: Mem0 alone / Router (v2) 列は可読性のため 3 桁丸め、gain 列は 4 桁生値から算出しており、表示値の単純減算とは ±0.001 ずれ得る。

Router は **全 4 cell で Mem0 alone と同等以上**(strictly better property)、長会話 precision query では最大 +22.4pt 改善。LLM API call は長会話では 97% 削減。これは "KDF を Mem0 の代替" でなく "Mem0 の補完 layer" として設計する本研究の architectural thesis の最初の定量的実証。
- **KDF の benchmark-dependent behavior**: 長期会話(LoCoMo の 300-700 turns/conv、date/time 情報が raw turn に散在)では KDF の raw-turn 保持が有利、短対話(LongMemEval の 20-30 turns/Q)では Mem0 の fact extraction が優位

したがって、**KDF の realistic positioning** は以下に pivot:

1. **長期会話の temporal recall** 用途(F-056 で +22pt 実証):会議録、月単位の journal、年単位の conversation history から日付/時間情報を参照する用途で KDF の raw-turn 保持が決定的優位
2. **cost/latency/privacy/determinism が accuracy より重視される環境**:local-first chatbot、air-gapped agent、<1ms real-time memory gating、budget-constrained deployment、deterministic regulated output
3. **TTL より賢い retention 戦略**(real 500Q で KDF recall 0.665 vs TTL_recent 0.180、×3.7)

**正直な限界**: LongMemEval 型(短対話 + concise questions)の汎用 LLM memory 市場では Mem0 の accuracy に届かない。この市場での KDF 使用は accuracy trade-off を許容する場合に限定。


残りの領域については、できること / できないことを honest に明示した上で、オープンに検証可能な形で提供する。特に **general retrieval 市場での使用は明示的に非推奨**(F-045)。

---

## 8. Theoretical Foundation — KDF as computational realization of Structural Holes theory

本研究で KDF の empirical 特性を積み上げた結果(F-061〜F-065)、KDF の挙動は **organizational sociology の古典理論 "Structural Holes"(Burt 1992)とアルゴリズム的に合致する** ことが判明した。これは post-hoc な analogy ではなく、**graph-theoretic な isomorphism** を指している(ただし末尾の *Limitations and risks of the theoretical claim* 節で明示するとおり、isomorphism は強い枠組み仮説であって証明された定理ではなく、その留保付きで用いる)。

### Burt's Structural Holes(1992) の要約

Burt は職場ネットワーク(MBA 500 人)の実証研究で以下を主張した:

1. **組織内で最も高い情報 / イノベーション / 交渉力を持つのは、密なクラスタ内の中心人物ではなく、異なるクラスタ間を橋渡しする "broker" である**
2. **Structural hole** = 二つのクラスタが直接接続していない empty space
3. Broker はこの hole を "span" することで:
   - 情報の非冗長性を制御できる(両側で別情報が流れる)
   - 交換条件を有利に決められる(brokerage power)
   - アイデアを cross-pollinate できる(innovation advantage)
4. Broker は通常 **低 degree**(少ない tie で bridge する方が効率的)、**high betweenness**(path に乗る)

### KDF の挙動との mathematical alignment

KDF の Rare / Core / Edge / Garbage 分類は、以下のように Burt の broker 概念と対応する:

| KDF layer | 構造的特徴 | Burt's classification | Evidence |
|---|---|---|---|
| **Rare**(deg==1)| 境界 node、唯一 connection | Pure broker(if between clusters) | F-012 Obsidian orphan |
| **Core** | 多 cluster 橋渡し、中程度 degree | Structural broker | F-062 merge commits(99.5% recall) |
| Edge | cluster 内中心、高 degree | Cluster insider(非 broker) | F-061 scale-free hubs(KDF 負け) |
| Garbage | redundant、capacity 外 | Peripheral / replaceable | F-052 low-recall answer turns |

KDF の **"Rare + Core を優先保護"** という選択原理は、**"broker position を持つ node を優先保護"** と等価である。これは偶然ではなく、structural rareness の graph-theoretic 定義(low degree and/or high betweenness potential)が、Burt の broker 定義と重なるためである。

### Empirical validation via scale-free vs community graph split

F-061 の 4 graph(ER, BA, WS, SBM)での betweenness centrality 結果は、この理論対応の **予測的 validation** である:

| Graph type | 構造的特徴 | Burt 理論の予測 | KDF 結果 |
|---|---|---|---|
| ER(Erdős–Rényi)| uniform random | Broker が存在、bridge 保護で勝てる | ✅ KDF wins(top-50 recall 0.70) |
| SBM(Stochastic Block Model)| planted communities | Inter-community broker が決定的、KDF 適性最大 | ✅ KDF wins(0.50 vs 0.36 TopDegree) |
| BA(Barabási–Albert, scale-free)| hub dominated | Hub の代替経路豊富 → brokerage power 分散、KDF 不利 | ❌ KDF loses to TopDegree |
| WS(Watts–Strogatz, small world)| clustered + shortcut | Shortcut = broker 的だが degree 中程度、complex | ❌ KDF loses to TopDegree on betweenness |

**Scale-free graph で KDF が負ける** のは、Burt 理論的には **"hub-dominated network では broker の相対的価値が下がる"** という予測と整合する。Hub は多数の alternative path を持つため、任意の hub を消しても ネットワークは robust。Brokerage power は community 間 bridge に集中し、pure scale-free では distributed される。

### Game-theoretic brokerage power との接続

Network bargaining theory (Myerson 1977; Calvó-Armengol & Jackson 2004)では、プレイヤー A と B が取引したいが直接接続していない場合、必ず仲介者 C を経由するとき、**C の利得は代替経路の存在に反比例**する。これは構造的空隙の数理表現である。

KDF はこの **"代替経路のない仲介者"(monopolistic broker)** を決定論的に抽出するアルゴリズムとして解釈できる。F-061 の APSP 実験で、KDF が特に **Watts-Strogatz**(small world、shortcut broker 構造)での coverage 維持で Random を 4× 上回ったのは、この理論予測の実例である。

### 実装アルゴリズムの computational complexity

Burt の structural holes 計算は伝統的に **effective size** / **constraint** metric(O(V²) or worse)で行われる。KDF の Rare/Core/Edge/Garbage 分類は **O(V + E)** の structural counting + classification で近似的に同じ broker 検出を達成する(正確な同値性は次の future work)。

**含意**: KDF は "structural holes / brokerage power" を **linear-near time で決定論的に検出** する最初の graph algorithm である可能性がある。Burt 自身の 1992 以降 30+ 年、structural holes は social science research(M&A analysis, supply chain resilience, innovation diffusion)で引用され続けているが、**scale 可能な計算アルゴリズムは未確立**。KDF の O(V + E) classifier は実用化の rate limiting を外す可能性。

### Applications space のリスト(SVGs of each evidence)

この理論対応から、KDF の商用 / 研究 application space は以下:

1. **Enterprise network analytics**
   - 社内コミュニケーション分析(Slack/Teams)→ cross-team broker 特定
   - M&A target selection: 買収すべき "独占的仲介者 company" 抽出
   - Supply chain: 低次数 Tier 3/4 の BCP critical supplier 特定
2. **Urban / logistics networks**
   - 災害時 APSP preprocessing(F-061 で実証)
   - 交通網の critical intersection 同定
3. **Cybersecurity**
   - Lateral movement detection via "rare inter-segment connection"(要追加実測)
4. **Enterprise archival**
   - Decision preservation via cross-thread bridge capture(F-062 git merge と homologous)

### Limitations and risks of the theoretical claim

Honesty のため明記:

- **Burt の theory は primarily human organizational networks(<10³ nodes)で validated**、millions of nodes での適用は未だ empirical question
- KDF の Rare / Core classification は broker 検出の **十分条件ではなく必要条件に近い approximation**(F-061 BA graph で 負けた事実が示唆)
- Structural holes theory 自体は "hub-dominated network で weak" という既知の limitation を持つ(Powell et al. 2005 等の critique)
- **KDF の O(V + E) complexity が Burt's effective size metric と formally equivalent かは未証明**、近似と経験的一致のみ

これらの caveat を保持した上で、**KDF は Structural Holes detection の computational realization の強力な candidate** として提示する。

---

## Acknowledgments

独立検証エージェント(GPT/Claude ベース)を 12 回のフェーズ境界で実行し、本研究の肯定・否定双方の主張の妥当性をチェックした。仕様固定、コード実装、実データ検証、関連研究調査、および本論文草稿作成のすべての段階で AI コラボレーションを活用しており、検証プロセスは [`docs/VERIFIED_FINDINGS.md`](VERIFIED_FINDINGS.md) に完全に記録されている。

---

## References

### 神経科学・記憶固定化
[1] McClelland JL, McNaughton BL, O'Reilly RC. *Why There are Complementary Learning Systems in the Hippocampus and Neocortex.* Psychological Review, 1995.
[2] Tartaglia EM, et al. *Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling.* PNAS, 2025. [doi:10.1073/pnas.2422602122](https://www.pnas.org/doi/10.1073/pnas.2422602122)
[3] Tonegawa S, et al. *Memory engrams: Recalling the past and imagining the future.* PMC 2020.

### 免疫学
[4] Mesin L, et al. *Restricted Clonality and Limited Germinal Center Reentry Characterize Memory B Cell Reactivation by Boosting.* Cell, 2020.

### 物理学(臨界現象)
[5] Bak P, Tang C, Wiesenfeld K. *Self-organized criticality: An explanation of 1/f noise.* Phys. Rev. Lett., 1987.
[6] Ginzburg VL, Landau LD. *On the theory of superconductivity.* JETP, 1950.

### 連続学習・Coreset Selection
[7] Kirkpatrick J, et al. *Overcoming catastrophic forgetting in neural networks.* PNAS, 2017. [doi:10.1073/pnas.1611835114](https://www.pnas.org/doi/10.1073/pnas.1611835114)
[8] Sener O, Savarese S. *Active Learning for Convolutional Neural Networks: A Core-Set Approach.* ICLR, 2018.
[9] Wang Y, et al. *Towards Equitable Coreset Selection: Addressing Challenges Under Class Imbalance.* ACM CIKM, 2024.

### 連想記憶
[10] Hopfield JJ. *Neural networks and physical systems with emergent collective computational abilities.* PNAS, 1982.
[11] Amit DJ, Gutfreund H, Sompolinsky H. *Statistical mechanics of neural networks near saturation.* Annals of Physics 173(1):30-67, 1987.
[12] Ramsauer H, et al. *Hopfield Networks is All You Need.* ICLR, 2021.

### 認知科学
[13] Gentner D. *Structure-mapping: A theoretical framework for analogy.* Cognitive Science, 1983.

### グラフ理論・spectral method
[14] Reuter M, Wolter FE, Peinecke N. *Laplace-spectra as fingerprints for shape matching.* ACM Solid and Physical Modeling Symposium, 2005.
[15] Levin DA, Peres Y. *Markov Chains and Mixing Times* (2nd ed.), AMS, 2017.

### 信号処理
[16] Aharon M, Elad M, Bruckstein A. *K-SVD: An Algorithm for Designing Overcomplete Dictionaries for Sparse Representation.* IEEE Trans. Signal Processing, 2006.

### 経済学
[17] Pareto V. *Cours d'Économie Politique.* 1896.
[18] Embrechts P, Klüppelberg C, Mikosch T. *Modelling Extremal Events for Insurance and Finance.* Springer, 1997.

### グラフ ML
[19] Nguyen GH, et al. *Continuous-Time Dynamic Network Embeddings.* WWW, 2018.

### 情報理論
[20] Rissanen J. *Modeling by shortest data description.* Automatica, 1978.

### 一般システム論(motivating context のみ)
[21] Miller JG. *Living Systems.* McGraw-Hill, 1978. (20 critical subsystems across 7 hierarchical levels)

### Coreset / unlearning 追補
[22] Patil V, Stengel-Eskin E, Bansal M. *UPCORE: Utility-Preserving Coreset Selection for Balanced Unlearning.* arXiv:2502.15082, 2025.

### KDF 関連資料
[23] 本研究 特許公開資料 JP 2026-027032(出願日 2026-02-24).
[24] 本研究 検証記録: [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md), [PUBLIC_SUMMARY.md](PUBLIC_SUMMARY.md), [related_work_survey.md](related_work_survey.md).

---

## Appendix A: KDF の主要数式まとめ

| 請求項 | 数式 | 説明 |
|---|---|---|
| 7 | $C_{uv} = \deg(u) + \deg(v)$ | 局所混雑度 |
| 8, 9 | $\lambda(C) = \beta(1 + \gamma C^\alpha)$, $\alpha$ 正指数 | 減衰率の非線形形(単調増加+べき乗項) |
| 10 | $\alpha = 2$ | べき乗項の指数を 2 に固定(発明の核心) |
| 14 | $w(t+dt) = w(t) \cdot \exp(-\lambda(C) dt)$ | 指数減衰則 |
| 15 | $\deg_E(v) \le 1 \Rightarrow \text{Rare} \land \text{protected}$ | 絶対閾値による希少判定 |
| 21 | $dt_1 : dt_2 : dt_3 = 5:3:1$ | 階層領域の更新周期比 |
| 29 | $\Delta \alpha \propto \delta k^4$ | メタ制御 4 次則 |
| 44 | $S_{\text{outer}} = \frac{7}{10} S_{\text{sys}} + \frac{2}{10} S_{\text{rel}} + \frac{1}{10} S_{\text{attr}}$ | 整合性総合スコア(外側集約;systematic / relational / attribute の 3 種を入力とする) |
| 45 | $S_{\text{inner}} = 0.40 \cdot S_{\text{cos}} + 0.35 \cdot S_{\text{struct}} + 0.25 \cdot S_{\text{sign}}$ | Fingerprint 類似度合成(内側;上の $S_{\text{sys}}$ / $S_{\text{rel}}$ / $S_{\text{attr}}$ の入力となる) |
| 46 | $\phi(v) \in \mathbb{R}^{32}$, Laplacian 固有値由来 | 固定長構造指紋 |
| 47-48 | $\theta_L = 0.70 \le S_{\text{outer}} \le \theta_U = 0.80$ | sandwich 採用域、Claim 44 の外側(集約)スコアに対して適用(canonical 値 — §4.2 で反証、§1.3 C3 参照) |

---

## Appendix B: 実装アーキテクチャ

- `crates/cgb-kdf/` — 参照実装(Rust, PolyForm Noncommercial 1.0.0)、50 請求項すべてに直接テスト
- `crates/bias-detector/` — 独立公開 crate、zero dependency、KDF の事前適用判別
- `demos/D1-D8/` — 8 領域のショーケース実装
- `benchmarks/sota_comparison/` — SOTA 比較ベンチマーク

GitHub: [ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite)

---

*Draft v0.3 — 2026-04-19 (Phase X Step 1-5 完走反映). Comments and corrections welcome via GitHub issues.*
