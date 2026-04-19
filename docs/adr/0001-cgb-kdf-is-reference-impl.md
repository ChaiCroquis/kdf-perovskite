# ADR-0001: cgb-kdf を参照実装、kdf-lib を Rev.10 サブセットとする

**Status:** Accepted
**Date:** 2026-04-17
**Phase:** 1

## Context

プロジェクトには歴史的に2つの実装が存在した:

- `kdf-lib/src/lib.rs` (2800 行、README に露出、広告対象)
- `crates/cgb-kdf/` (Rev.12 フル実装、analogy/fingerprint/rev12 サブモジュール持ち)

特許出願書類の請求項1は「代謝制御手段 + 希少性保護手段 + **整合性発見手段**」の3手段を独立クレームとして必須要件にしている。`kdf-lib` 単体では整合性発見手段が欠落しており Claim 1 を充足できない。一方 `cgb-kdf` は3手段すべてを実装済み。

## Decision

- **`cgb-kdf` を特許 Claim 1-50 の参照実装(reference implementation)とする**
- `kdf-lib` は Rev.10 Basic のサブセット実装と位置付け、Claim 1 準拠は主張しない
- README/CLAUDE.md/Cargo.toml の metadata にこの責務分離を明示

## Consequences

**Positive**
- Claim 1 準拠が明確に特定のクレートで達成可能
- 新機能(メタ制御・遷移制御・領域管理)は `cgb-kdf` に集約、二重メンテナンス不要
- 外部ユーザーは「KDF = cgb-kdf」と明確に認識できる

**Negative**
- 既存の `kdf-lib` ユーザーは `cgb-kdf` への移行が必要
- サブセット実装を残すことで、コード重複が生じる

**Mitigation**
- `kdf-lib` の lib.rs docstring に明示的な注意書き
- Cargo.toml の workspace.metadata.patent に reference-implementation = "cgb-kdf" を宣言

## References

- [docs/patent/SPEC.md](../patent/SPEC.md)
- [docs/patent/TRACEABILITY.md](../patent/TRACEABILITY.md)
