# Contributing to KDF

## 特許準拠リポジトリとしての運用

本リポジトリは特許出願書類(特願 2026-027032)を **FROZEN な権威仕様** として運用しています。通常の OSS と異なる以下の規則に注意してください。

### 絶対ルール

1. **`docs/patent/` 配下のファイルを編集・削除・リネームしない**
   - `HASHES.sha256` で改ざん検知され、CI で PR がブロックされます
   - 新しい版の特許が出た場合は `docs/patent/v2/` 等へ**並列追加**すること
2. **実装と仕様の齟齬は「実装がバグ」と判定する**
   - 決して仕様を実装に合わせる方向で PR を出さない
3. **PR には準拠する請求項番号を必ず明記する**
   - 例: 「明細書§0022 (Claim 14) に従い …」

### 基本ワークフロー

```bash
# 1. ブランチ作成
git checkout -b feature/claim-N-something

# 2. 権威仕様を確認: [docs/patent/SPEC.md] (filed/ PDFs は JPO 自動公開
#    (2027-08 予定) までは公開しない。公開後 docs/patent/filed/ に追加)
# 3. 実装 + Claim テストを追加
cargo test -p cgb-kdf --release

# 4. clippy / fmt を通す
cargo clippy --workspace --exclude kdf-wasm --all-targets -- -D warnings
cargo fmt --all

# 5. ベンチマーク再現性を確認
cargo run --release -p sota-comparison

# 6. PR 送信時のタイトル例:
#    "Claim 29: δk^4 scaling proof test"
```

### PR チェックリスト

以下を PR description に含めること:

```markdown
## Related Patent Claim
Claim ##(明細書 §####)

## Changes
- [変更内容]

## Tests
- [追加/変更したテスト]
- `cargo test -p cgb-kdf --release` の結果
- `cargo clippy -D warnings` 合格

## Impact on TRACEABILITY.md / COMPLIANCE.md
- [更新の必要性、あれば行番号]

## Benchmark regression?
- [`cargo run --release -p sota-comparison` の結果比較]
```

### Claim トレーサビリティの維持

新しい実装機能を追加した場合、以下を更新:

1. [`docs/patent/TRACEABILITY.md`](docs/patent/TRACEABILITY.md): 該当 Claim 行に `file:line` を追加
2. [`docs/patent/COMPLIANCE.md`](docs/patent/COMPLIANCE.md): 判定(✓/△/✗)を更新
3. コード内 `/// # Patent claim mapping` docstring で Claim 番号を付与

### 品質ゲート

- `cargo test --release`: 全 pass
- `cargo clippy -- -D warnings`: 警告ゼロ
- `cargo fmt --check`: フォーマット遵守
- `sha256sum -c docs/patent/HASHES.sha256`: 改ざんなし

---

## ライセンス

本リポジトリは [PolyForm Noncommercial 1.0.0](LICENSE) のもとで提供されます。コントリビュートしたコードも同ライセンスで受理されます(Developer's Certificate of Origin 相当)。

商用ライセンスについては [COMMERCIAL.md](COMMERCIAL.md) を参照ください。特許権(特願 2026-027032)は著作権とは独立に管理され、PolyForm NC の patent license 条項により非商用利用に限って付与されます。
