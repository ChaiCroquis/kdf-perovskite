# ADR-0002: 減衰更新は必ず exp(-λ·dt) 形式を使う

**Status:** Accepted
**Date:** 2026-04-17
**Phase:** 1

## Context

Phase 0 以前、`apply_edge_decay` は `weight *= 1.0 - decay_prob` という**線形近似**を使っていた。

請求項14は以下を明文化している:
> 前記関連パラメータwは、所定時間刻みdtごとに、w ← w×exp(－λ(C)×dt) に基づいて更新される

`(1 - λdt)` と `exp(-λdt)` の差は $(\lambda dt)^2 / 2 + O((\lambda dt)^3)$ で、キャノニカル値では $10^{-9}$ 程度だが、**請求項の文言は exp を明記**している。特許侵害判定では形式違反とみなされる可能性が高い。

## Decision

- 全ての減衰更新で `(-lambda * dt).exp()` を使用する
- 線形近似への切替フラグは設けない(Claim 違反の抜け道を残さない)
- `test_exp_decay_analytic_solution` で 1000 step の反復を解析解 $e^{-N\lambda\Delta t}$ と 1e-10 精度で一致することを CI で常時検証

## Consequences

- Claim 14 形式的準拠
- 数値的には線形近似と等価レベルの精度
- 性能オーバーヘッド無視できる(exp は CPU の libm でネイティブ)
- Phase 2 数理解析 §2-3 で閉形式解との一致が保証される

## References

- [docs/math/decay_analysis.md §3](../math/decay_analysis.md)
- [crates/cgb-kdf/src/framework/decay.rs](../../crates/cgb-kdf/src/framework/decay.rs)
