# KDF 請求項準拠レポート

**生成日:** 2026-04-17 (Phase 1 完了)
**前回:** Phase 0 ベースライン (2026-04-17)
**検証対象:** [kdf-lib](../../kdf-lib/) / [crates/cgb-kdf](../../crates/cgb-kdf/)
**権威仕様:** 特願 2026-027032 の特許請求の範囲(JPO 自動公開まで本公開リポジトリには含めず、[`filed/README.md`](filed/README.md) 参照)
**トレーサビリティ:** [TRACEABILITY.md](TRACEABILITY.md)

---

## エグゼクティブサマリ

| 実装 | Claim 1(独立) | 従属請求項適合率 | 総合評価 |
|---|:---:|:---:|:---:|
| **kdf-lib** | ✗ **不適合** (整合性発見手段が完全欠落) | 8% (4/50) | **Rev.10 Basic サブセット**([ADR-0001](../adr/0001-cgb-kdf-is-reference-impl.md)) |
| **cgb-kdf** | ✓ Claim 1 独立要件すべて | **50/50 の直接テスト付き(`test_claimN_*` × 56)** | **参照実装** |

**headline(独立検証エージェント PASS)**: cgb-kdf は Claim 1 から Claim 50 まで **50 件すべてに直接テスト**(`test_claimN_*` 形式)が付いている。計 56 tests、workspace 全体 449 tests all pass。独立検証エージェントによる re-audit で 7 件の弱いテストを指摘 → 全件 STRONG/ADEQUATE に強化済。

**現時点で「KDFが実装されている」と外部に宣言できるのは cgb-kdf のみ**。kdf-lib は ADR-0001 により意図的なサブセットとして位置付け、Claim 1 準拠を主張しない。

---

## 判定根拠の全リスト

各 Claim について「要件 / 証拠 / 判定 / 理由」を記載。

### Claim 1【独立・システム】

**要件**: 代謝制御手段 + 希少性保護手段 + 整合性発見手段 の3つを具備する情報処理システム

- **kdf-lib**:
  - 代謝制御: [lib.rs:1285 `process()`](../../kdf-lib/src/lib.rs) — 動作はするが Claim 7,14 の形式要件違反
  - 希少性保護: [lib.rs:1322](../../kdf-lib/src/lib.rs) — `Layer::Rare` を無条件選択、期間概念なし
  - 整合性発見: **該当コードなし**
  - **判定: ✗ 不適合** (3手段目が欠落)

- **cgb-kdf**:
  - 代謝制御: [framework/decay.rs:206](../../crates/cgb-kdf/src/framework/decay.rs)
  - 希少性保護: [framework/rev12.rs:95](../../crates/cgb-kdf/src/framework/rev12.rs) `KdfProcessorRev12`
  - 整合性発見: [analogy.rs:340](../../crates/cgb-kdf/src/analogy.rs) `find_analogy`
  - **判定: ✓ 適合**

### Claim 7【従属】C = deg(u) + deg(v)

**要件**: 局所混雑度指標 = エッジ両端ノードの次数の和

- **kdf-lib**: [lib.rs:354 `compute_edge_congestion`](../../kdf-lib/src/lib.rs) では deg(u)+deg(v) だが、主処理 [lib.rs:339](../../kdf-lib/src/lib.rs) では C=deg(i) 単一ノード。**判定: ✗**
- **cgb-kdf**: [framework/decay.rs:187](../../crates/cgb-kdf/src/framework/decay.rs) 両端次数和。**判定: ✓**

### Claim 14【従属】λ(C)=β(1+γC^α), w←w·exp(-λ·dt)

**要件**: (1) 形式 λ(C)=β(1+γC^α) (2) 更新式 w·exp(-λdt) 離散化

- **kdf-lib**: [lib.rs:347](../../kdf-lib/src/lib.rs) 線形近似。**判定: ✗** (Rev.10 Basic サブセットとして範囲外)
- **cgb-kdf (Phase 1 以降)**: [framework/decay.rs:244-256 `apply_edge_decay`](../../crates/cgb-kdf/src/framework/decay.rs) が `(-λ*dt).exp()` で survival factor を計算、weight に乗算。[`compute_edge_decay_probability`](../../crates/cgb-kdf/src/framework/decay.rs) も `1 - exp(-λdt)`。**判定: ✓**
- 根拠テスト: `test_edge_decay_probability_exp_form`, `test_lambda_master_form`, `test_exp_decay_analytic_solution` (1000ステップ反復 = 閉形式 exp(-Nλdt) を 1e-10 精度で一致)

### Claim 39【従属】第1・第2期間は 30 以上 70 以下

**要件**: `t_wait1 ∈ [30, 70]` かつ `t_wait2 ∈ [30, 70]`

- **cgb-kdf (Phase 1 以降)**: [framework/rev12.rs](../../crates/cgb-kdf/src/framework/rev12.rs) にて
  - 定数 `T_WAIT_MIN=30, T_WAIT_MAX=70, T_WAIT_DEFAULT=50` を定義
  - `Default::default()` が `t_wait1 = t_wait2 = 50` を設定
  - `new()` / `with_upper_threshold()` が `[30, 70]` 外を `Rev12Error::TwaitOutOfRange` で拒否
  - テスト用エスケープハッチとして `new_unchecked_for_tests`(doc hidden)を提供
- **判定: ✓**
- 根拠テスト: `test_rev12_default_claim_compliant`, `test_rev12_new_rejects_twait_out_of_range`

### Claim 44【従属】7:2:1 重み比

**要件**: 系統的類似性 : 関係類似性 : 属性類似性 = 7:2:1

- **cgb-kdf**: [analogy.rs:209-211](../../crates/cgb-kdf/src/analogy.rs) `0.7:0.2:0.1`。**判定: ✓**

### Claim 45【従属】S_cos+S_struct+S_sign 正係数重み付き和

**要件**: 第1スコア = a·S_cos + b·S_struct + c·S_sign (a,b,c > 0)

- **cgb-kdf**: [fingerprint/engine.rs:318](../../crates/cgb-kdf/src/fingerprint/engine.rs) `0.40*cos + 0.35*struct + 0.25*sign`。全係数正。**判定: ✓**

### Claim 46【従属】ラプラシアン固有値 + 閾値 + 簡易スクリーニング

**要件**:
1. 固定長ベクトル = グラフラプラシアン固有値列に基づく
2. S = a·S_cos + b·S_struct + c·S_sign
3. theta_L ∈ [0.70, 0.80]
4. 簡易距離によるスクリーニング

- **cgb-kdf**:
  1. [fingerprint/engine.rs:156 `eigenvalue_fingerprint`](../../crates/cgb-kdf/src/fingerprint/engine.rs) `symmetric_eigen().eigenvalues` ✓
  2. 同上 `full_similarity` 0.40/0.35/0.25 ✓
  3. [analogy.rs:212](../../crates/cgb-kdf/src/analogy.rs) `discovery_threshold=0.75` ∈ [0.70,0.80] ✓
  4. [prescreening.rs](../../crates/cgb-kdf/src/prescreening.rs) + [analogy.rs:215](../../crates/cgb-kdf/src/analogy.rs) `top_k_percent=0.05` ✓
  - **判定: ✓**

### Claim 48【従属】theta_L=0.70, theta_U=0.80

**要件**: 下限閾値 0.70、上限閾値 0.80、両方を採用基準に使用

- **cgb-kdf (Phase 1 以降)**:
  - [framework/rev12.rs](../../crates/cgb-kdf/src/framework/rev12.rs) に `discovery_threshold` (θ_L) と `discovery_threshold_upper` (θ_U) の2値を持つ
  - 定数 `DISCOVERY_THRESHOLD_DEFAULT=0.75`, `DISCOVERY_THRESHOLD_UPPER_DEFAULT=0.80`
  - 構築時に θ_L ∈ [0.70, 0.80] と θ_U > θ_L をバリデーション
  - [`attempt_discovery`](../../crates/cgb-kdf/src/framework/rev12.rs) で採用条件 `θ_L ≤ S ≤ θ_U` (サンドイッチ)
- **判定: ✓**
- 根拠テスト: `test_rev12_default_claim_compliant`, `test_rev12_new_rejects_theta_out_of_range`

---

## 全 Claim 判定リスト (50項)

| Claim | 独立/従属 | kdf-lib | cgb-kdf | Phase 1 実装箇所 |
|:---:|:---:|:---:|:---:|---|
| 1 | 独立 | ✗ | ✓ | 既存(cgb-kdf は Rev12Processor) |
| 2 | 従属 | △ | ✓ | — |
| 3 | 従属 | △ | ✓ | — |
| 4 | 従属 | △ | ✓ | [transition.rs](../../crates/cgb-kdf/src/framework/transition.rs) activation 追加 |
| 5 | 従属 | △ | ✓ | [decay.rs:compute_evaluation_value / compute_time_component](../../crates/cgb-kdf/src/framework/decay.rs) |
| 6 | 従属 | △ | ✓ | — |
| 7 | 従属 | ✗ | ✓ | kdf-lib は Rev.10 subset として除外 |
| 8 | 従属 | ✓ | ✓ | — |
| 9 | 従属 | ✓ | ✓ | — |
| 10 | 従属 | △ | ✓ | [decay.rs](../../crates/cgb-kdf/src/framework/decay.rs) `alpha_core=2.0` default + `test_claim10_alpha_equals_two` |
| 11 | 従属 | △ | ✓ | [decay.rs:probabilistic_prune](../../crates/cgb-kdf/src/framework/decay.rs) |
| **12** | 従属 | ✗ | **✓** | **Phase 1: [decay.rs:probabilistic_prune](../../crates/cgb-kdf/src/framework/decay.rs)** |
| **13** | 従属 | ✗ | **✓** | **Phase 1: exp 減衰化** |
| **14** | 従属 | ✗ | **✓** | **Phase 1: [decay.rs:apply_edge_decay](../../crates/cgb-kdf/src/framework/decay.rs) `(-λdt).exp()`** |
| 15 | 従属 | △ | ✓ | — |
| 16 | 従属 | ✗ | ✓ | — |
| 17 | 従属 | △ | ✓ | [decay.rs:apply_edge_decay_local](../../crates/cgb-kdf/src/framework/decay.rs) Claim 17 準拠 API + local==global 同値テスト |
| 18 | 従属 | △ | ✓ | — |
| 19 | 従属 | ✓ | ✓ | — |
| **20** | 従属 | ✗ | **✓** | **Phase 1: [region.rs:RegionKind](../../crates/cgb-kdf/src/framework/region.rs)** |
| **21** | 従属 | ✗ | **✓** | **Phase 1: [region.rs:tick_period()](../../crates/cgb-kdf/src/framework/region.rs) 5:3:1** |
| **22** | 従属 | ✗ | **✓** | **Phase 1: [region.rs:RegionConfig](../../crates/cgb-kdf/src/framework/region.rs)** |
| **23** | 従属 | ✗ | **✓** | **Phase 1: [transition.rs:TransitionController](../../crates/cgb-kdf/src/framework/transition.rs)** |
| **24** | 従属 | ✗ | **✓** | **Phase 1: [transition.rs:TransitionScore](../../crates/cgb-kdf/src/framework/transition.rs)** |
| **25** | 従属 | ✗ | **✓** | **Phase 1: [transition.rs:ActivationScore](../../crates/cgb-kdf/src/framework/transition.rs)** |
| **26** | 従属 | ✗ | **✓** | **Phase 1: [transition.rs:SemanticImportance](../../crates/cgb-kdf/src/framework/transition.rs)** |
| **27** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:MetaController](../../crates/cgb-kdf/src/framework/meta_control.rs)** |
| **28** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:health_index](../../crates/cgb-kdf/src/framework/meta_control.rs)** |
| **29** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:alpha_update](../../crates/cgb-kdf/src/framework/meta_control.rs) δk⁴** |
| **30** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:step](../../crates/cgb-kdf/src/framework/meta_control.rs) clamp** |
| **31** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:emergency_intervention](../../crates/cgb-kdf/src/framework/meta_control.rs)** |
| **32** | 従属 | ✗ | **✓** | **Phase 1: [meta_control.rs:set_enabled](../../crates/cgb-kdf/src/framework/meta_control.rs)** |
| 33 | 従属 | △ | ✓ | [classifier.rs](../../crates/cgb-kdf/src/framework/classifier.rs) 加重次数で strength+connection-count、`test_claim33_isolation_metric_uses_strength_and_connection_count` |
| 34 | 従属 | △ | ✓ | — |
| 35 | 従属 | ✗ | ✓ | — |
| 36 | 従属 | ✗ | ✓ | — |
| 37 | 従属 | ✗ | ✓ | Phase 1: constructor 受理の Default=50,50 |
| 38 | 従属 | ✗ | ✓ | — |
| **39** | 従属 | ✗ | **✓** | **Phase 1: [rev12.rs:T_WAIT_DEFAULT=50 + 範囲バリデーション](../../crates/cgb-kdf/src/framework/rev12.rs)** |
| 40 | 従属 | ✗ | ✓ | — |
| 41 | 従属 | ✗ | ✓ | Phase 1: 既存 `apply_demotion` |
| 42 | 従属 | ✗ | ✓ | — |
| 43 | 従属 | ✗ | ✓ | — |
| 44 | 従属 | ✗ | ✓ | — |
| 45 | 従属 | ✗ | ✓ | — |
| 46 | 従属 | ✗ | ✓ | — |
| **47** | 従属 | ✗ | **✓** | **Phase 1: [rev12.rs:discovery_threshold_upper](../../crates/cgb-kdf/src/framework/rev12.rs)** |
| **48** | 従属 | ✗ | **✓** | **Phase 1: [rev12.rs:sandwich band](../../crates/cgb-kdf/src/framework/rev12.rs) 0.70/0.80** |
| 49 | 方法 | — | ✓ | — |
| 50 | プログラム | ✓ | ✓ | — |

**2026-04-18 時点(per-claim 直接テスト追加後):**

| カテゴリ | Claim 数 | 内訳 |
|---|:---:|---|
| 直接 `test_claimN_*` テスト付き ✓ | **50** | Claim 1 ~ Claim 50 すべて |
| 完全未実装 | 0 | — |

**表記**: 「cgb-kdf は Claim 1-50 すべてに対し `test_claimN_*` 形式の直接テストを整備(計 56 tests)、workspace 全体で 449 tests all pass」。独立検証エージェントが 2 回の re-audit(2026-04-18)で全テストを STRONG/ADEQUATE 判定済。

**直接テスト一覧(全 50 Claim):**

Claim 1 の独立テスト(`test_claim1_three_means_present`)+ 4 従属 Claim 別ごとに 1 つ以上の `test_claimN_*`(計 56 tests)。主要モジュール別:

- **decay.rs**: Claim 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 17, 18, 19, 33
- **region.rs**: Claim 20, 21, 22
- **transition.rs**: Claim 23, 24, 25, 26
- **meta_control.rs**: Claim 27, 28, 29, 30, 31, 32
- **framework/tests.rs**: Claim 1, 14, 34, 35, 36, 37, 38, 39, 40, 41, 42, 47, 48, 49, 50
- **analogy.rs**: Claim 43, 44, 45, 46

個別テスト関数名は `grep -rhn "fn test_claim" crates/cgb-kdf` で列挙可能。

**kdf-lib (K 列)** は Rev.10 basic subset のため意図的に Claim 1 系の不適合部分を残す([ADR-0001](../adr/0001-cgb-kdf-is-reference-impl.md))。

**補足(CLAUDE.md の乖離リストとの整合):**
`CLAUDE.md` §"現在の乖離サマリ"に列挙されていた「Meta 層未実装」「昇格/抑制/二段階審査/摩擦関数 W_eff 未実装」「Rare 入場条件が相対閾値」等は:
- Meta 層 / 昇格 / 二段階審査 / 希少性保護の期間制御は Phase 1 で `region.rs` / `transition.rs` / `rev12.rs` / `meta_control.rs` に実装済(Claim 20-32, 36-41 の ✓ 対応)
- 摩擦関数 $W_{eff}$ は特許 filed/特許請求の範囲 にはなく、technical/発明提案書 のみの記載(filed/ 優先ルールにより対象外)
- Rare 入場条件 $\deg_E(v)\le 1$ は **kdf-lib** の話(cgb-kdf には該当しない)

この経緯は読み取りやすさのため [CLAUDE.md](../../CLAUDE.md) を更新予定。

**テスト緑状況 (Phase 1-5 完了時):**
- `cgb-kdf` クレート: 324 unit + 10 math_properties + 7 proptest + 1 doc = **342 tests pass**
- Workspace 集計 (kdf-wasm / kdf-python 除外): **370 tests pass, 0 fail**
- proptest: 7 プロパティ × 256 ケース = 1,792 生成入力で検証
- 新規 Claim 直接検証テスト: Claim 12, 14, 17, 20-32, 39, 47-48 (20 件超)

**ベンチマーク (Phase 4):**
- 6 methods × 3 sizes × 10 trials = 180 runs
- KDF: Rare Recall = 1.000 ± 0.000 (ラベル不要)、Compression = 0.558 (seed固定・決定論)

**品質ゲート (Phase 3):**
- cgb-kdf: `cargo clippy -D warnings` クリーン
- 決定論性: HashMap 挿入順非依存 + denormal flushing で bit-exact 保証

---

## Phase 0 ベースライン合意事項

1. 本レポートの判定は `docs/patent/filed/*.pdf` を**唯一の根拠**とする。
2. 実装の動作が優れていても、Claim の文言に合わなければ ✗ と判定する。
3. Phase 1 完了後、本ファイルを更新し、`git diff` で進捗を可視化する。
4. 最終目標: **kdf-lib = cgb-kdf = 全50項 ✓** (Claim 10 のような「α=2 限定」など選択的従属は「条件付き適合」とする)。

---

## 自動生成予定

Phase 1 で `cargo run --bin compliance-check` を実装し、本ファイルと [TRACEABILITY.md](TRACEABILITY.md) を CI 実行時に再生成する:

```bash
cargo run --bin compliance-check -- --output docs/patent/COMPLIANCE.md
cd docs/patent && sha256sum -c HASHES.sha256
```

失敗した場合は PR マージをブロック。
