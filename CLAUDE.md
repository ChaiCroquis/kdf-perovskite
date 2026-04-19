# KDF — AI Agent Contributing Notes

このファイルは本リポジトリで作業する AI コーディングアシスタント向けの運用指示です。

## 仕様根拠の優先順位

1. **KDF の仕様根拠**: 特願 2026-027032 の出願書類一式。`filed/` PDFs は日本特許庁(JPO)の自動公開(2027-08 頃)まで本公開リポジトリには含めていません(詳細: [docs/patent/filed/README.md](docs/patent/filed/README.md))。それまでの参照先は:
   - [docs/patent/SPEC.md](docs/patent/SPEC.md) — 権威宣言と優先順位ルール
   - [docs/patent/COMPLIANCE.md](docs/patent/COMPLIANCE.md) — 請求項 1-50 × 実装マッピング
   - [docs/patent/TRACEABILITY.md](docs/patent/TRACEABILITY.md) — Claim × 実装行マッピング
   - [docs/arxiv_submission/paper.pdf](docs/arxiv_submission/paper.pdf) — プレプリント論文(発明の数学的定義 + 実証結果 + 自己反証)

2. **実装と仕様の齟齬は実装がバグ**。仕様を実装に合わせる方向の PR は受け付けません。

3. **PR は該当請求項番号を明記**(例: 「Claim 14 に従い `apply_edge_decay` を修正」)。詳細は [CONTRIBUTING.md](CONTRIBUTING.md)。

## 現在の実装状態(概要)

| 実装 | 請求項対応 |
|---|---|
| [`crates/cgb-kdf/`](crates/cgb-kdf/) | Claim 1-50 すべてに直接テスト(`test_claimN_*` × 56) — **参照実装** |
| [`kdf-lib/`](kdf-lib/) | Rev.10 Basic サブセット([ADR-0001](docs/adr/0001-cgb-kdf-is-reference-impl.md)) — 整合性発見機構は意図的に欠く |

詳細は [COMPLIANCE.md](docs/patent/COMPLIANCE.md) 参照。

## 検証知見

67 件の F-xxx 検証記録を [docs/VERIFIED_FINDINGS.md](docs/VERIFIED_FINDINGS.md) にまとめています。肯定結果 / 陰性結果 / 自己反証のすべてを誠実に記録しています。

主な知見の要約:
- [docs/kdf_characteristics.md](docs/kdf_characteristics.md) — KDF の proven 特性 6 カテゴリ
- [docs/design_philosophy.md](docs/design_philosophy.md) — 発明者の設計思想
- [docs/validation_strategy.md](docs/validation_strategy.md) — 未検証 candidate の優先度マトリクス
- [docs/domain_validation.md](docs/domain_validation.md) — 応用領域の validation マトリクス
- [docs/classical_algorithm_revival.md](docs/classical_algorithm_revival.md) — 古典アルゴリズム復権候補

## ビルド / テスト

```bash
cargo build --release -p kdf                              # コアライブラリ
cargo test  --release -p kdf                              # 単体テスト
cargo test  --release -p cgb-kdf                          # 参照実装全 50 claim テスト
cargo run   --release -p kdf --example kdf_quantitative_validation
```

Python / WASM バインディングは pyo3 / wasm-bindgen のネイティブ依存が CI で揃わない場合ビルドに失敗し得るため、workspace 全体を一括検証する場合は除外可:

```bash
cargo build --release --workspace --exclude kdf-python --exclude kdf-wasm
cargo test  --release --workspace --exclude kdf-python --exclude kdf-wasm
```

## KDF の適用可否(要点)

> **「structural rareness が task importance と相関する条件下でのみ、KDF は Random / baseline を decisively 上回る」**

- ✅ 相関あり: git merges、path-critical bottleneck、LLM memory の date/time literal、orphan note
- ❌ 相関なし: feature-space density(GP/fraud/SVM)、high in-degree API、scale-free hub、metadata minority

詳細は [paper.pdf](docs/arxiv_submission/paper.pdf) の §7 Conclusion 参照。
