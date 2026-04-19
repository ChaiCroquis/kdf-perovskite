# ADR-0003: 多段審査期間 t_wait のデフォルトは 50、範囲 [30, 70] を強制

**Status:** Accepted
**Date:** 2026-04-17
**Phase:** 1

## Context

請求項39: 前記第1期間および前記第2期間は、それぞれ30以上70以下の範囲に設定される。

Phase 0 時点のデフォルト:
- `t_wait1 = 3`
- `t_wait2 = 5`

これは下限30を **10倍下回る** 値で、デフォルトコンストラクタで処理を走らせた場合 Claim 39 を満たせない。

## Decision

1. `T_WAIT_MIN = 30`, `T_WAIT_MAX = 70`, `T_WAIT_DEFAULT = 50` を定数化
2. `Default::default()` は `t_wait1 = t_wait2 = 50` を使用
3. `KdfProcessorRev12::new()` は範囲外を `Rev12Error::TwaitOutOfRange` で拒否
4. テスト目的で 30 以下の値が必要な場合は `new_unchecked_for_tests`(`#[doc(hidden)]`)を用意
5. `with_upper_threshold` / `new` の両方で検証を走らせる

## Consequences

**Positive**
- Claim 39 がデフォルト挙動で自動準拠
- 誤用時に早期エラー(build 時ではなく instantiate 時)
- Rev.11 §45「第1期間 = 第2期間」の canonical 値として 50/50 を採用できる

**Negative**
- 既存のテストが低値期間に依存していた場合、`new_unchecked_for_tests` への移行が必要
- 範囲 [30, 70] を変えたいユーザーは `Rev12Error::TwaitOutOfRange` を見て仕様を再確認する必要

**Mitigation**
- 既存テスト4箇所を Phase 1 で `new_unchecked_for_tests` に移行(テスト意図を維持)
- 新規テスト `test_rev12_default_claim_compliant` と `test_rev12_new_rejects_twait_out_of_range` で両側を保証

## References

- [crates/cgb-kdf/src/framework/rev12.rs](../../crates/cgb-kdf/src/framework/rev12.rs)
