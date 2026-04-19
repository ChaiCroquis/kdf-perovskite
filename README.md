# KDF — Knowledge Decay Framework

[![Rust 1.70+](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License: PolyForm NC 1.0.0](https://img.shields.io/badge/License-PolyForm%20NC%201.0.0-yellow.svg)](LICENSE)
[![Commercial](https://img.shields.io/badge/Commercial-Contact%20author-red.svg)](COMMERCIAL.md)
[![Patent filed](https://img.shields.io/badge/Patent-2026--027032-informational)](docs/patent/SPEC.md)
[![Spec frozen](https://img.shields.io/badge/Spec-FROZEN-brightgreen)](docs/patent/SPEC.md)
[![DOI](https://zenodo.org/badge/DOI/10.5281/zenodo.19651035.svg)](https://doi.org/10.5281/zenodo.19651035)

**KDF** は、長期運用される情報ネットワーク(知識グラフ、ログ、学習データ等)を、**代謝的に削減しつつ希少情報を構造的に保護**する Rust 実装フレームワークです。

本実装は、特願 **2026-027032** の請求項1–50 を参照仕様として厳密に実装しています。特許出願書類 5 点(特許願 / 特許請求の範囲 / 明細書 / 要約書 / 図面)は **日本特許庁による自動公開(出願から 18 ヶ月、2027-08-24 頃)** まで本リポジトリには含めていません。それまでの間は、権威仕様の要約を [docs/patent/SPEC.md](docs/patent/SPEC.md)、請求項 × 実装の対応を [docs/patent/COMPLIANCE.md](docs/patent/COMPLIANCE.md) と [docs/patent/TRACEABILITY.md](docs/patent/TRACEABILITY.md) に記載しています。発明の理論・手法の詳細は **プレプリント論文(Zenodo DOI: [10.5281/zenodo.19651035](https://doi.org/10.5281/zenodo.19651035))** または [`docs/arxiv_submission/paper.pdf`](docs/arxiv_submission/paper.pdf) を参照してください。

## 引用(Citation)

```bibtex
@misc{kuroki2026kdf,
  author       = {Kuroki, Yasuhiro},
  title        = {{KDF: A Deterministic Architecture for Finite-Resource
                   Information Preservation---Cross-Domain Evidence and
                   Self-Refutation of Canonical Values}},
  year         = {2026},
  publisher    = {Zenodo},
  version      = {v0.3},
  doi          = {10.5281/zenodo.19651035},
  url          = {https://doi.org/10.5281/zenodo.19651035},
  note         = {Preprint. Patent: JP 2026-027032 (filed 2026-02-24).}
}
```

> 🔍 **検証済み知見集**: Phase 0〜8 Stage 2 で独立検証エージェントを通過した全知見を [**docs/VERIFIED_FINDINGS.md**](docs/VERIFIED_FINDINGS.md) に体系化。失敗モード / 解決策 / 未検証事項も honest に列挙。

---

## 主張できる性質(統計的裏付けあり)

[Phase 4 ベンチマーク](benchmarks/REPORT.md)(合成データ n∈{200,500,1000}, 10試行):

| 指標 | KDF(本実装) | Random | K-Medoids | CoreSet | Stratified |
|---|---|---|---|---|---|
| Rare Recall(ラベル不要) | **1.000 ± 0.000** | 0.29-0.34 | 0.000 | 0-0.04 | 1.000 (ラベル必要) |
| Compression | 0.558 | 0.70 | 0.70 | 0.70 | 0.665 |
| 決定論性(SE) | 0.000(bit-exact) | — | — | — | — |

**KDF は「ラベル不要で Rare 100% 保持」を達成する唯一の手法**(合成データ条件, n∈{200,500,1000})。実データでの挙動は条件依存で、[PUBLIC_SUMMARY.md](docs/PUBLIC_SUMMARY.md) と [VERIFIED_FINDINGS.md](docs/VERIFIED_FINDINGS.md) に分野別の verified / unverified 結果を誠実に記録している:
- ✅ LongMemEval: KDF=0.821 vs TTL=0.107(×7.7)
- ✅ Obsidian: F1=0.747, Wilcoxon p=0.006
- ✅ NASA log: ×2.3 Random
- ❌ OSS GitHub issues: 3-repo 平均 ×1.00(一般化失敗。rust-lang 単独では +15% だが golang では -15%)
- ❌ OpenAlex 論文再発見: ×0.83(D5 型で KDF は Random に負ける)

---

## アーキテクチャ

```
kdf-perovskite/
├── docs/patent/         特許準拠文書 + トレーサビリティ
│   ├── SPEC.md          権威宣言(filed/ は JPO 自動公開後に追加予定)
│   ├── TRACEABILITY.md  Claim × 実装行マッピング
│   ├── COMPLIANCE.md    準拠判定レポート
│   └── HASHES.sha256    整合性ハッシュ
├── docs/math/           数理解析(exp減衰、Lyapunov、複雑度証明)
├── crates/
│   └── cgb-kdf/         参照実装 ★ (Claim 1-50 対応)
├── kdf-lib/             Rev.10 Basic サブセット
├── kdf-cli/             CLI フロントエンド
├── kdf-python/          PyO3 バインディング
├── kdf-wasm/            WebAssembly
└── benchmarks/
    ├── sota_comparison/ 対SOTA比較ベンチ
    └── REPORT.md        結果・誠実な制限
```

## クレート別対応請求項

| クレート | 請求項 | コンプライアンス |
|---|---|---|
| [cgb-kdf](crates/cgb-kdf/) | **Claim 1–50 すべてに直接テスト**(`test_claimN_*` × 56、workspace 449 tests pass)| [COMPLIANCE.md](docs/patent/COMPLIANCE.md) |
| [kdf-lib](kdf-lib/) | Rev.10 サブセット(Claim 2-10, 15, 18-19) | 同上 |

---

## クイックスタート

### Rust API(cgb-kdf, 請求項1対応の参照実装)

```rust
use cgb_kdf::{KdfProcessorRev12, NodeClassifier, Layer};

// 1. 代謝制御 + 希少性保護 + 整合性発見 = Claim 1 の3手段
let mut processor = KdfProcessorRev12::default();
// t_wait1=50, t_wait2=50, θ_L=0.75, θ_U=0.80 (Claim 39/46/47/48 準拠)

let edges = vec![(0, 1, 1.0), (1, 2, 1.0), /* ... */];
processor.initialize(/*node_count=*/ 100, &edges);

// 2. Rev.12 多段審査サイクル (Claim 36-41)
for _ in 0..100 {
    for (node, action) in processor.process_review_cycle() {
        match action {
            "promote" => processor.apply_promotion(node),  // Claim 40 spoke_up
            "demote" => processor.apply_demotion(node),    // Claim 41
            _ => {}
        }
    }
}

// 3. メタ制御(Claim 27-32)
use cgb_kdf::MetaController;
let mc = MetaController::default();
assert!(mc.check_lyapunov_stability());  // Claim 28/29 の数理安定性
```

### ベンチマーク再現

```bash
cargo run --release -p sota-comparison
# → benchmarks/results/sota_comparison.json に書き出し
```

### テスト

```bash
cargo test --release -p cgb-kdf
# 324 unit + 10 math + 7 property + 1 doc = 342 tests
# Claim 5 / 17 テストを含む workspace 全体: 412 pass (kdf-python/kdf-wasm 除く)
```

---

## 実データ実験の再現手順

外部ネットから取得するデータ(workspace 内にはコミットしていない):

| 実験 | データ | 取得方法 | バイナリ |
|---|---|---|---|
| P1 LLM memory | LongMemEval 500q | `python -c "from datasets import load_dataset; load_dataset('xiaowu0162/longmemeval-cleaned').save_to_disk('demos/D8_llm_memory/data')"` | `cargo run --release -p demo-d8-llm-memory` |
| P2 Obsidian | 発明者 vault(非公開, PII masked) | プロジェクトで公開予定の合成代替 sample を使う | `cargo run --release -p demo-d1-obsidian` |
| P3 NASA log | NASA HTTP 1995/7 | `curl -sL https://ita.ee.lbl.gov/traces/NASA_access_log_Jul95.gz \| gunzip \| head -50000 > demos/D2_nasa_log/data/access.log` | `cargo run --release -p demo-d2-nasa-log` |
| P6 GitHub issues | rust-lang/rust 500 issues | バイナリが GitHub REST API から直接取得、`GITHUB_TOKEN` を環境変数で渡す | `cargo run --release -p demo-d7-github-issue --bin phase_delta_real_issues` |
| FB15K-237 (D5) | FreebaseK-237 24MB | `curl -sL https://www.microsoft.com/en-us/download/details.aspx?id=52312 …` (詳細は [demos/D5_fb15k237/README.md](demos/D5_fb15k237/README.md)) | `cargo run --release -p demo-d5-fb15k237` |

データファイルは `demos/**/data/` 以下に展開され、リポジトリ側では `.gitignore` で除外している。サイズは合計で約 100MB。

---

## 数理仕様(抜粋)

- **減衰方程式(Claim 14)**: $\lambda(C) = \beta(1 + \gamma C^\alpha)$, $w \leftarrow w \cdot e^{-\lambda\Delta t}$
- **4乗則(Claim 29)**: $\Delta\alpha \propto (\max(0, \langle k\rangle - k_{opt}))^4$
- **Lyapunov 条件(Rev.11 §7.4)**: $\eta^2 > \mu^2$(デフォルトで検証済)
- **整合性発見(Claim 44-46)**: 構造フィンガープリント = グラフラプラシアン固有値、類似度 $S = 0.40 S_\text{cos} + 0.35 S_\text{struct} + 0.25 S_\text{sign}$, サンドイッチ採用域 $S \in [0.70, 0.80]$

詳細は [docs/math/decay_analysis.md](docs/math/decay_analysis.md)。

---

## 開発・品質保証

| カテゴリ | 状態 |
|---|---|
| `cargo test`(cgb-kdf)| **342 tests pass, 0 fail** |
| `cargo clippy -D warnings`(cgb-kdf)| **クリーン** |
| proptest(property tests)| 7 プロパティ × 256 ケース = 1,792 生成入力で検証 |
| 決定論性 | HashMap 挿入順非依存を [`apply_decay_is_insertion_order_invariant`](crates/cgb-kdf/tests/math_properties.rs) で bit-exact 検証 |
| 特許完全性 | [`HASHES.sha256`](docs/patent/HASHES.sha256) で改ざん検知、CI で自動検証 |

---

## 仕様根拠の優先順位

1. KDF の仕様根拠は **特許出願書類(特願 2026-027032 の filed/ 5 書類)**。本公開リポジトリには JPO 自動公開(2027-08 頃)まで現物は含めず、要約として [docs/patent/SPEC.md](docs/patent/SPEC.md) を提供
2. `docs/patent/` 配下は特許庁提出物に対する参照資料であり、**実装と仕様の齟齬は実装がバグ**
3. 発明の数学的背景・検証結果は [プレプリント論文](docs/arxiv_submission/paper.pdf) + [docs/VERIFIED_FINDINGS.md](docs/VERIFIED_FINDINGS.md)(67 F-xxx 検証記録)に記載

---

## ライセンス

本実装コードは **[PolyForm Noncommercial 1.0.0](LICENSE)** のもとで提供されます。

- ✅ **無料で使える場合**: 学術研究・教育・個人研究・非営利組織・非本番評価
- ❌ **別途商用ライセンスが必要な場合**: 商用製品への組込み、本番稼働、SaaS 提供等

商用利用をご検討の場合は [COMMERCIAL.md](COMMERCIAL.md) をご参照いただき、author までお問い合わせください(`garden.of.knowledge.chai@gmail.com`)。

特許権(特願 2026-027032)は PolyForm NC の patent license 条項により **非商用利用に限って付与** されます。商用特許ライセンスは商用ソフトウェアライセンス契約と併せて発行します。
