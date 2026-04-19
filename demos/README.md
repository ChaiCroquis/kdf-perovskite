# KDF Showcase Portfolio

特許 2026-027032(KDF: Knowledge Decay Framework)の**実施例ギャラリー**。
各デモは、既存手法との**比較**と**正直な trade-off** を併記した独立パッケージです。

---

## ギャラリー(Stage 1 + Stage 2 完了: 7 demos)

| Demo | 領域 | 特許§ | KDF 勝利 | KDF 敗北 | 型分類 | ステータス |
|---|---|---|---|---|:---:|:---:|
| [D1 Obsidian](D1_obsidian/) | 個人知識ベース | §0002 ナレッジ | F1, precision, compression | (なし) | **D1** | ✅ **圧勝** |
| [D2 NASA log](D2_nasa_log/) | Web ログ管理 | §0002 ログ管理 | **label-free で** 3x Random | ラベル有り手法 | **D2** | ⚠️ 拡張必要 |
| [D3 ML long-tail](D3_ml_longtail/) | 学習データ | §0002 学習データ | (なし) | Stratified に大敗、拡張も無効 | **D5**(真) | ◐ marginal |
| [D4 MovieLens](D4_movielens/) | 検索/推薦 | §0002 検索又は推薦 | **RelDensity 首位 (0.359)** | baseline 単体では敗 | **D1.5** | ⚠️→✅ 拡張で救済 |
| [D5 FB15K-237](D5_fb15k237/) | 知識グラフ | §0002 知識グラフ | +8.6% over TransE | 差小、RelDensity は逆効果 | **D5**(真) | ◐ 僅差 |
| [D6 forum dedup](D6_text_dedup/) | SNS/フォーラム | §0002 SNS | (なし) | **ExactDup に完敗** | **D1.5** | ❌ 予想外敗 |
| [D7 GitHub issue](D7_github_issue/) | アーカイブ管理 | §0002 アーカイブ | **Recall 0.486 最強** | (なし) | **D1** | ✅ **圧勝** |

### 横断分析ドキュメント

- [META_ANALYSIS.md](META_ANALYSIS.md) — Stage 1 時点の 3-demo 横断パターン + ユーザ思考パターン
- [STAGE2_REPORT.md](STAGE2_REPORT.md) — Stage 2 完了時の 7-demo 横断分析、新カテゴリ D1.5 の発見

### 型分類の意味

- **D1 型**: 構造が rareness を直接 encode → baseline KDF で勝てる
- **D1.5 型**: 構造は長尾を持つが絶対閾値では判定不可 → Phase 7 S2 RelDensity 必須(本 Stage 2 で発見した新カテゴリ)
- **D2 型**: 構造から rareness を近似可(ラベル独立)→ Phase 7 拡張で中位
- **D5 型**: ラベルと構造が独立、構造シグナルなし → KDF marginal

### 未着手

| Demo | 領域 | 状態 |
|---|---|---|
| D8 LLM 持続記憶 | 広義ナレッジ | 未着手(Stage 3 候補、LLM cost 要検討) |

---

## 各 demo の共通構造

```
demos/D<N>_<name>/
├── Cargo.toml
├── README.md              ← ピッチ(5分で読める)
├── src/main.rs            ← cargo run で実行
├── data/                  ← 公開データ取得先(未取得でも合成で動作)
└── out/                   ← 実行後に生成
    ├── report.json        ← 生データ(再現検証用)
    ├── report.md          ← 自動生成 Markdown
    ├── bar_comparison.svg
    ├── kdf_axis_diagram.svg
    └── tradeoff_scatter.svg
```

---

## 全デモ共通の評価方針 — 3軸フレーム

「KDF が全てで勝つ」は**主張しない**。代わりに:

```
軸A: KDF の強み  — KDF 固有性(3手段)が効く指標
軸B: 同等        — 既存手法との互換性(退行しない)
軸C: KDF の弱み  — 正直な trade-off(透明性軸)
```

各 demo の表で **軸 A でどれだけ勝つか / 軸 C でどれだけ譲るか** を定量化。
完全勝利の主張は軸 C でのトレードオフ出しが弱いので、**KDF を売り込む誠実な形** を選んでいます。

---

## 再現

全 7 demos は `cargo run` で動作、実行時間は各 demo 秒オーダー(D3 の kNN のみ数秒)。
Seed は各 demo で **42** に固定(dataset 合成用) + trial seed は **4000 + trial_idx** (D1), **5000+** (D2), ..., **10000+** (D4)。

```bash
# 1. workspace ビルド
cd kdf-perovskite
cargo build --release --workspace --exclude kdf-wasm --exclude kdf-python

# 2. 全 7 demos を実行
cargo run --release -p demo-d1-obsidian       # 知識ネットワーク
cargo run --release -p demo-d2-nasa-log       # HTTP ログ管理
cargo run --release -p demo-d3-ml-longtail    # ML 学習データ長尾
cargo run --release -p demo-d4-movielens      # 推薦 long-tail
cargo run --release -p demo-d5-fb15k237       # 知識グラフ
cargo run --release -p demo-d6-text-dedup     # forum 重複削減
cargo run --release -p demo-d7-github-issue   # issue archive

# 3. SVG 可視化(Python + matplotlib)
pip install matplotlib
for demo in D1_obsidian D2_nasa_log D3_ml_longtail D4_movielens D5_fb15k237 D6_text_dedup D7_github_issue; do
    python demos/scripts/render_visualizations.py demos/$demo/out/report.json
done
```

### 統計検定について

各 demo は N=10 trials の平均+標準誤差を報告。一次比較は**効果量の差**(KDF - baseline)を目視するのが中心で、Wilcoxon signed-rank 検定は Phase 7 の `adversarial_bench::wilcoxon_signed_rank` を使って `report.json` の `raw_trials` から追加可能(本 demo セットではレポート冗長化を避けるため省略)。

### Phase Verification Binary の対応表

各 Phase 検証で走らせる補助バイナリ:

| Phase | Entry point | 内容 |
|---|---|---|
| A | `demos/scripts/verify_wilcoxon.py` | 全 7 demos の Wilcoxon paired test |
| B | `cargo run -p adversarial-bench --bin phase_b_robustness` | 多重 seed ロバストネス(5×3 runs) |
| C | `cargo run -p adversarial-bench --bin phase_c_ablation` | KDF 成分別 ablation |
| D | `cargo test -p cgb-kdf --test phase_d_lyapunov_long` | 100k step Lyapunov |
| E | `cargo test -p cgb-kdf --test phase_e_numerical_precision` | 極値パラメータ数値安定性 |
| F | `cargo run -p demo-d6-text-dedup --bin phase_f_deepdive` | D6 失敗深掘り(7 シナリオ) |
| G | `cargo run -p demo-d5-fb15k237` (with data) | FB15K-237 実データ評価 |
| K | `cargo run -p demo-d6-text-dedup --bin phase_k_text_hybrid` | D6 TextHybrid 反証試行 |
| L | `cargo run -p demo-d2-nasa-log` (with data) | NASA 実ログ reality check |
| M | `cargo run -p adversarial-bench --bin phase_m_large_scale` | n=10k..500k scaling |
| N | `cargo run -p adversarial-bench --bin phase_n_dynamic_loop` | 動的制御 loop 救済実証 |
| O | `cargo run -p demo-d8-llm-memory` | LLM メモリ curation |

---

## データポリシー

- **Obsidian Vault**: 発明者個人データ、PII マスキング + FNV-1a 匿名化、集計のみ公開
- **NASA HTTP log**: 取得元 [ita.ee.lbl.gov](https://ita.ee.lbl.gov/html/contrib/NASA-HTTP.html)、再配布しない、`data/access.log` に配置で自動使用
- **FB15K-237**: 取得元 [Microsoft Research](https://www.microsoft.com/en-us/download/details.aspx?id=52312)、再配布しない、`data/fb15k-237/` に配置で自動使用
- **合成データ**: いずれも seed 固定で再現可能、decimal SHA-256 記録予定

---

## Stage 1 サマリ主張

各デモで **KDF が何に強く、何に弱いか** を数値で出し切っています。主張は**誇張しない**ことを第一に:

### KDF が強いと言える範囲
- **ラベル不要**で rare / 長尾を構造的に検出(D1, D2+RelDensity 確認)
- 知識ベースの **"忘れていた"ノート** の高 precision 発見(D1)
- 圧縮率と recall の同時最適化(D1)

### KDF が苦手な範囲
- **ラベルが得られる場合**は Stratified / Tail-based のほうが強い(D2)
- 構造的に distinguishable でない rare には無力(D5 合成データで +8.6% 止まり)
- リアルタイム処理(graph 構築コストあり)

### 未検証
- 大規模(n ≥ 10^5)、動的制御(TransitionController/MetaController ループ)、LLM 統合

これは **「KDF は汎用の万能ツールではなく、ある特定の trade-off profile を持つツール」** という位置付けを、**定量的に** 示すギャラリーです。

---

## ライセンス

各 demo は [PolyForm Noncommercial 1.0.0](../LICENSE) のもとで提供されます(商用利用は [COMMERCIAL.md](../COMMERCIAL.md) 参照)。
特許権は独立管理(特願 2026-027032)、コード利用 ≠ 特許実施許諾。
