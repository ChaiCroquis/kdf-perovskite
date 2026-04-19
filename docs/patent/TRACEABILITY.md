# 特許請求項 ⇔ 実装 トレーサビリティ行列

**生成日:** 2026-04-17
**根拠:** 特願 2026-027032 の特許請求の範囲・明細書(JPO 自動公開まで本公開リポジトリには含めず、[`filed/README.md`](filed/README.md) 参照)
**目的:** 請求項50項 × 実装箇所 × 検証テスト の全射マッピング。Phase 0 時点のスナップショット。

---

## 凡例

| 記号 | 意味 |
|---|---|
| ✓ | 実装あり、請求項の要件を満たす |
| △ | 実装あり、要件の一部のみ満たす(数値/形式の逸脱) |
| ✗ | 実装なし |
| — | 該当なし(方法・プログラムクレーム等) |

**実装クレート略称:**
- `K` = [kdf-lib/src/lib.rs](../../kdf-lib/src/lib.rs)
- `C` = [crates/cgb-kdf/](../../crates/cgb-kdf/)
- `P` = [crates/kdf-perovskite-py/src/lib.rs](../../crates/kdf-perovskite-py/src/lib.rs)

---

## Claim 1 (独立請求項) — 3手段の同時具備

| サブ要素 | 明細書段落 | 実装 (K) | 実装 (C) | 判定 |
|---|---|---|---|:---:|
| 代謝制御手段 | §0067 | [lib.rs:1285](../../kdf-lib/src/lib.rs) `Kdf::process()` | [framework/decay.rs:206](../../crates/cgb-kdf/src/framework/decay.rs) `apply_edge_decay()` | K:△ / C:✓ |
| 希少性保護手段 | §0068 | [lib.rs:315](../../kdf-lib/src/lib.rs) `classify_layers` + [lib.rs:1322](../../kdf-lib/src/lib.rs) | [framework/rev12.rs:95](../../crates/cgb-kdf/src/framework/rev12.rs) `KdfProcessorRev12` | K:△ / C:✓ |
| 整合性発見手段 | §0069 | **未実装** | [analogy.rs:139](../../crates/cgb-kdf/src/analogy.rs) `AnalogyDiscoveryEngine` + [fingerprint/engine.rs:62](../../crates/cgb-kdf/src/fingerprint/engine.rs) | **K:✗** / C:✓ |

**Claim 1 総合**: K=**✗** (整合性発見手段が欠落)/ C=**✓**

---

## Claim 2-10 — グラフ構造・局所混雑度・非線形関数

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 2 | ノード/エッジのグラフ構造 | §0010 | [lib.rs:1296](../../kdf-lib/src/lib.rs) 類似度行列 | [framework/decay.rs:134](../../crates/cgb-kdf/src/framework/decay.rs) `initialize_with_edges` | K:△ / C:✓ |
| 3 | 関連パラメータ(強度/頻度/信頼度) | §0011 | [lib.rs:437](../../kdf-lib/src/lib.rs) `edge_weights` (未使用) | [framework/decay.rs:91](../../crates/cgb-kdf/src/framework/decay.rs) `edge_weights` | K:△ / C:✓ |
| 4 | 時間系メタデータ | §0012 | [lib.rs:2104](../../kdf-lib/src/lib.rs) `TemporalKdf` (ノード時刻のみ) | [framework/decay.rs:87](../../crates/cgb-kdf/src/framework/decay.rs) `access_counts` | K:△ / C:△ |
| 5 | 時間評価成分 | §0013 | [lib.rs:2274](../../kdf-lib/src/lib.rs) | [decay.rs:`compute_time_component` / `compute_evaluation_value`](../../crates/cgb-kdf/src/framework/decay.rs) | K:△ / **C:✓** |
| 6 | 局所混雑度指標(接続量/密度/分布) | §0014 | [lib.rs:354](../../kdf-lib/src/lib.rs) `compute_edge_congestion` | [framework/decay.rs:187](../../crates/cgb-kdf/src/framework/decay.rs) | K:△ / C:✓ |
| **7** | **C=deg(u)+deg(v)** | §0015 | [lib.rs:354](../../kdf-lib/src/lib.rs) 定義あり、[lib.rs:339](../../kdf-lib/src/lib.rs) `process()` では **C=deg(i)** 単一ノード | [framework/decay.rs:187](../../crates/cgb-kdf/src/framework/decay.rs) `deg_u + deg_v` ✓ | **K:✗** / C:✓ |
| 8 | 単調増加の非線形関数 | §0016 | [lib.rs:347](../../kdf-lib/src/lib.rs) `β(1+γC^α)` | [framework/decay.rs:202](../../crates/cgb-kdf/src/framework/decay.rs) 同形式 | K:✓ / C:✓ |
| 9 | べき乗項を含む | §0017 | [lib.rs:347](../../kdf-lib/src/lib.rs) `c.powf(alpha)` | [framework/decay.rs:202](../../crates/cgb-kdf/src/framework/decay.rs) `congestion.powf(alpha)` | K:✓ / C:✓ |
| 10 | 指数=2 (従属) | §0018 | デフォルト `alpha_core=2.0` のみ | 同左 | K:△ / C:△ |

---

## Claim 11-15 — 剪定方式・指数減衰・孤立ノード

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 11 | 閾値剪定 or 確率剪定 | §0019 | [lib.rs:1325](../../kdf-lib/src/lib.rs) `weights[i] >= theta_edge` (閾値のみ) | [framework/decay.rs:217](../../crates/cgb-kdf/src/framework/decay.rs) 重み減衰のみ | K:△ / C:△ |
| 12 | 排除確率と乱数比較(確率剪定) | §0020 | **未実装** | **未実装** (rand比較がない) | K:✗ / C:✗ |
| 13 | 指数関数的減衰 | §0021 | [lib.rs:395](../../kdf-lib/src/lib.rs) 線形 `w*=(1-p)` を100回反復 | [framework/decay.rs:217](../../crates/cgb-kdf/src/framework/decay.rs) 線形 `*weight *= 1.0 - decay_prob` | **K:✗** / **C:✗** |
| **14** | **λ(C)=β(1+γC^α), w←w·exp(-λ·dt)** | §0022 | [lib.rs:347](../../kdf-lib/src/lib.rs) λの形式はOK、**exp 不使用、dt 不在** | [framework/decay.rs:202](../../crates/cgb-kdf/src/framework/decay.rs) 同左 | **K:✗** / **C:✗** |
| 15 | 孤立ノード剪定・保護時保持 | §0023 | [lib.rs:1322](../../kdf-lib/src/lib.rs) Rare即選択 | [framework/classifier.rs:98](../../crates/cgb-kdf/src/framework/classifier.rs) Garbage vs Rare 分離 | K:△ / C:✓ |

---

## Claim 16-19 — 局所統計・分散・保護属性・記録

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 16 | 局所統計のみで評価 | §0024 | **全ペア走査** | [framework/decay.rs:206](../../crates/cgb-kdf/src/framework/decay.rs) エッジ局所のみ | K:✗ / C:✓ |
| 17 | 複数処理主体による分散実行 | §0025 | `rayon` feature はあるが分散主体化していない | [decay.rs:`apply_edge_decay_local`](../../crates/cgb-kdf/src/framework/decay.rs) — 局所情報のみで排除決定、`test_claim17_local_decay_matches_global` で global と bit-一致確認 | K:△ / **C:✓** |
| 18 | 保護属性付与 | §0026 | [lib.rs:315](../../kdf-lib/src/lib.rs) `Layer::Rare` による暗黙保護 | [framework/decay.rs:166](../../crates/cgb-kdf/src/framework/decay.rs) `is_protected()` | K:△ / C:✓ |
| 19 | 記録・出力手段 | §0027 | [lib.rs:437](../../kdf-lib/src/lib.rs) `KdfResult` | [framework/rev12.rs:64](../../crates/cgb-kdf/src/framework/rev12.rs) `Rev12Stats` | K:✓ / C:✓ |

---

## Claim 20-22 — 階層管理構造(短期・長期・希少)

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 20 | 第1(短期) + 第2(長期)領域 | §0028 | **未実装** | **未実装** (Layer分類のみ) | K:✗ / C:✗ |
| **21** | **dt1:dt2:dt3 = 5:3:1** | §0029 | **未実装** | **未実装** | **K:✗** / **C:✗** |
| 22 | 第1/第2で評価条件を異ならせる | §0030 | **未実装** | 層別 γ,α はある(領域でなく層) | K:✗ / C:△ |

---

## Claim 23-26 — 遷移制御(活性度・意味的重要度)

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 23 | 遷移制御手段 | §0031 | **未実装** | [framework/rev12.rs:298](../../crates/cgb-kdf/src/framework/rev12.rs) `apply_promotion` のみ(層間遷移) | K:✗ / C:△ |
| 24 | 接続量/活性度/意味的重要度の遷移スコア | §0032 | **未実装** | **未実装** (スコア計算不在) | K:✗ / C:✗ |
| 25 | 活性度の時間減衰・イベント増加 | §0033 | **未実装** | [framework/decay.rs:176](../../crates/cgb-kdf/src/framework/decay.rs) `record_access`(増加のみ、時間減衰なし) | K:✗ / C:△ |
| 26 | 意味的重要度(基準集合/外部モデル) | §0034 | **未実装** | [analogy.rs:276](../../crates/cgb-kdf/src/analogy.rs) `generate_semantic_vector` (別用途) | K:✗ / C:△ |

---

## Claim 27-32 — メタ制御手段(健全性・4乗則・緊急介入)

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 27 | メタ制御手段 | §0035 | **未実装** | **未実装** | K:✗ / C:✗ |
| 28 | 健全性指標(平均接続量−目標接続量) | §0036 | **未実装** | **未実装** | K:✗ / C:✗ |
| **29** | **Δパラメータ ∝ δk^4** (偏差4乗) | §0037 | **未実装** | **未実装** | **K:✗** / **C:✗** |
| 30 | 目標値と比較した双方向更新 + 上限下限 | §0038 | **未実装** | **未実装** | K:✗ / C:✗ |
| 31 | 緊急介入 | §0039 | **未実装** | **未実装** | K:✗ / C:✗ |
| 32 | 動作モード切替 | §0040 | **未実装** | **未実装** | K:✗ / C:✗ |

**メタ制御手段群(27-32)は両実装とも完全欠落。**

---

## Claim 33-42 — 希少性保護の詳細

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 33 | 孤立度指標(強度/頻度/接続量/時間推移) | §0041 | [lib.rs:321](../../kdf-lib/src/lib.rs) degree のみ | [framework/classifier.rs:101](../../crates/cgb-kdf/src/framework/classifier.rs) degree+neighbor_count | K:△ / C:△ |
| 34 | 保護用管理状態への設定 | §0042 | [lib.rs:315](../../kdf-lib/src/lib.rs) `Layer::Rare` 付与 | [framework/rev12.rs:21](../../crates/cgb-kdf/src/framework/rev12.rs) `RareNodeState` | K:△ / C:✓ |
| 35 | 解除条件(期間経過/整合性発見/組合せ) | §0043 | **未実装** | [framework/rev12.rs:276](../../crates/cgb-kdf/src/framework/rev12.rs) Phase遷移 | K:✗ / C:✓ |
| **36** | **多段審査(第1期間 + 第2期間)** | §0044 | **未実装** | [framework/rev12.rs:10](../../crates/cgb-kdf/src/framework/rev12.rs) `ReviewPhase::{Phase1, Phase2}` | **K:✗** / C:✓ |
| 37 | 第1期間と第2期間の期間長同一 | §0045 | **未実装** | `t_wait1, t_wait2` 独立変数(同一にする強制なし) | K:✗ / C:△ |
| 38 | 期間終了時の状態切替 | §0046 | **未実装** | [framework/rev12.rs:276](../../crates/cgb-kdf/src/framework/rev12.rs) `ReviewPhase::Complete` へ遷移 | K:✗ / C:✓ |
| **39** | **期間 30≦t_wait≦70** | §0047 | **未実装** | [framework/rev12.rs:123](../../crates/cgb-kdf/src/framework/rev12.rs) **デフォルト t_wait1=3, t_wait2=5 (仕様違反)** | **K:✗** / **C:✗** |
| 40 | 接続獲得フラグ(spoke_up) | §0048 | **未実装** | [framework/rev12.rs:25](../../crates/cgb-kdf/src/framework/rev12.rs) `spoke_up: bool` | K:✗ / C:✓ |
| 41 | 第2期間末 + spoke_up=false + 関連情報なし → 排除 | §0049 | **未実装** | [framework/rev12.rs:281](../../crates/cgb-kdf/src/framework/rev12.rs) `apply_demotion` | K:✗ / C:△ |
| 42 | 希少範囲外の候補除外 | §0050 | **未実装** | [framework/classifier.rs:100](../../crates/cgb-kdf/src/framework/classifier.rs) `Layer::Garbage` | K:✗ / C:✓ |

---

## Claim 43-48 — 整合性発見手段の詳細

| Claim | 要件 | 明細書 | 実装 (K) | 実装 (C) | 判定 |
|:---:|---|---|---|---|:---:|
| 43 | 構造表現と整合性スコア | §0051 | **未実装** | [analogy.rs:340](../../crates/cgb-kdf/src/analogy.rs) `find_analogy` | K:✗ / C:✓ |
| **44** | **第1:第2:第3 = 7:2:1** | §0052 | **未実装** | [analogy.rs:209-211](../../crates/cgb-kdf/src/analogy.rs) `systematic=0.7, relational=0.2, attribute=0.1` | **K:✗** / **C:✓** |
| 45 | S_cos+S_struct+S_sign 正係数和 | §0053 | **未実装** | [fingerprint/engine.rs:318](../../crates/cgb-kdf/src/fingerprint/engine.rs) `0.40*cos+0.35*struct+0.25*sign` | K:✗ / C:✓ |
| **46** | **ラプラシアン固有値 + S=a·S_cos+b·S_struct+c·S_sign + theta_L∈[0.70,0.80] + 簡易スクリーニング** | §0054 | **未実装** | [fingerprint/engine.rs:156](../../crates/cgb-kdf/src/fingerprint/engine.rs) `symmetric_eigen` + [analogy.rs:212](../../crates/cgb-kdf/src/analogy.rs) `discovery_threshold=0.75` + [prescreening.rs](../../crates/cgb-kdf/src/prescreening.rs) | **K:✗** / **C:✓** |
| 47 | 上限閾値 theta_U を追加 | §0055 | **未実装** | **未実装** (下限閾値のみ) | K:✗ / C:✗ |
| **48** | **theta_L=0.70, theta_U=0.80** | §0056 | **未実装** | theta_L=0.75 (範囲内だが上限閾値不在) | **K:✗** / **C:△** |

---

## Claim 49-50 — 方法・プログラム

| Claim | 要件 | 明細書 | 実装 | 判定 |
|:---:|---|---|---|:---:|
| 49 | 情報処理方法(工程として) | §0057 | `C` のシステム構成を method 化すれば等価 | K:△ / C:✓ |
| 50 | プログラム | §0058 | Rust クレート全体 | K:✓ / C:✓ |

---

## 準拠サマリ

| 実装 | ✓ | △ | ✗ | 適合率(✓ / 全体) |
|---|:---:|:---:|:---:|:---:|
| **kdf-lib (K)** | 4 | 12 | 34 | **8%** |
| **cgb-kdf (C)** | 27 | 13 | 10 | **54%** |

**Claim 1 (独立請求項)**: K=✗ / C=✓ (ただし従属請求項の重要な数値制約で失格あり)

---

## 数値制約違反の具体

以下は Claim が**数値**で強制している条項で、現実装が外れているもの:

| Claim | 要求 | 現実装 | 差 |
|:---:|---|---|---|
| 14 | `exp(-λdt)` | `(1 - p)` 線形 | 形式違反 |
| 21 | dt1:dt2:dt3 = 5:3:1 | 未実装 | — |
| 29 | δk の **4乗** | 未実装 | — |
| 39 | t_wait ∈ **[30, 70]** | cgb-kdf デフォルト 3, 5 | **10倍小さい** |
| 44 | 7:2:1 | 0.7:0.2:0.1 | **一致** ✓ |
| 45 | 正係数3項の和 | 0.40:0.35:0.25 | **一致** ✓ |
| 48 | theta_L=0.70, theta_U=0.80 | 0.75単一閾値 | **theta_U 未実装** |

---

## 次フェーズの着手ポイント

Phase 1(完全準拠)で埋めるべき実装ギャップ優先度:

### P0 (Claim 1 の基本充足に必須)
1. `kdf-lib` に整合性発見手段を組込、Claim 1 サブ要素3つ揃える
2. Claim 14 の `exp(-λdt)` を両実装で正規化
3. Claim 39 の `t_wait` デフォルト値を 50 に修正

### P1 (数値違反の矯正)
4. Claim 48 の上限閾値 theta_U=0.80 を追加
5. Claim 21 の dt1:dt2:dt3=5:3:1 階層領域を新設
6. Claim 27-32 のメタ制御手段ブロックを新規実装(健全性指標・4乗則・緊急介入)

### P2 (要件の洗練)
7. Claim 12 の確率剪定(rand比較)
8. Claim 23-26 の遷移制御手段
9. Claim 17 の分散実行対応

---

## 更新規則

- 本ファイルは手編集ではなく、`scripts/gen_traceability.py`(Phase 1 で作成)で自動生成する想定。
- 各行は(Claim番号, 明細書段落, 実装file:line, テストfile:line, 判定)のタプル。
- PR 時に `cargo run --bin compliance-check` が `COMPLIANCE.md` の判定と本行列の整合を検査する。
