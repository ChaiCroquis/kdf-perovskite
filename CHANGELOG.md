# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added — Phase 1 (Patent Claim 1-50 compliance)

- **Claim 14**: `apply_edge_decay` now uses `exp(-λ·dt)` exactly (prior implementation used linear `(1 - p)` approximation). See [decay.rs](crates/cgb-kdf/src/framework/decay.rs).
- **Claim 12**: `probabilistic_prune` compares per-step P_decay with caller-supplied rng (Bernoulli trial with Rare protection).
- **Claim 20-22**: New `HierarchicalRegionManager` with short-term / long-term / rare regions, periods dt1:dt2:dt3 = 5:3:1.
- **Claim 23-26**: New `TransitionController`, `ActivationScore` (event-increment + time-decay), `SemanticImportance` (reference set + external model).
- **Claim 27-32**: New `MetaController` with health index, δk^4 update, bidirectional clamp, emergency intervention, mode toggle.
- **Claim 39**: `t_wait ∈ [30, 70]` validation in constructors; `T_WAIT_DEFAULT = 50`.
- **Claim 47-48**: Added upper threshold `θ_U`, sandwich acceptance band `θ_L ≤ S ≤ θ_U` with defaults (0.70, 0.80).
- **Claim 17**: `apply_edge_decay_local` for distributed processing.

### Added — Phase 2 (Mathematical rigor)

- `docs/math/decay_analysis.md`: 8-section analytical document covering ODE closed form, N-step error bound, convergence time, Lyapunov stability proof sketch, complexity matrix, numerical stability.
- `crates/cgb-kdf/tests/math_properties.rs`: 10 integration tests validating exp-decay closed form, Lyapunov bound (5000-step simulation), δk^4 ratio, insertion-order invariance.
- `crates/cgb-kdf/src/framework/invariants.rs`: Runtime checker for α ordering, β/γ/dt positivity.

### Added — Phase 3 (Implementation quality)

- `crates/cgb-kdf/tests/properties.rs`: proptest-based property tests (256 cases × 7 = 1792 generated inputs) for monotonicity, survival ∈ (0,1], 4th-power ratio, α clamping, disabled-noop.
- Determinism: `apply_edge_decay` sorts edges before iteration; denormal flushing at `|w| < 1e-290`.
- `clippy -D warnings` clean.
- `Rev12Error` enum with `TwaitOutOfRange`, `ThetaLowerOutOfRange`, `ThetaUpperNotAbove`.

### Added — Phase 4 (Benchmark)

- New workspace member `benchmarks/sota_comparison`: compares KDF vs Random / Stratified / K-Medoids / CoreSet / PageRank on seeded synthetic data.
- 10 trials × 3 sizes × 6 methods = 180 runs reproducibly.
- `benchmarks/REPORT.md` with honest limitations (synthetic data, rare=deg-1 structure-isomorphic to classifier).

### Added — Phase 0 (Foundation)

- `docs/patent/SPEC.md`: Authority declaration (MASTER / REFERENCE / NON-AUTHORITATIVE hierarchy).
- `docs/patent/TRACEABILITY.md`: Claim × code-line × test mapping for all 50 claims.
- `docs/patent/COMPLIANCE.md`: Pass/fail judgment report with evidence.
- `docs/patent/HASHES.sha256`: SHA-256 manifest for all 17 patent artifacts.
- `.github/workflows/patent-spec.yml`: CI hash verification, blocks `filed/` modification.
- `.github/workflows/rust-quality.yml`: fmt / clippy / test matrix across Linux/macOS/Windows.
- `CLAUDE.md`: AI-agent binding instructions.

### Fixed

- `kdf-python/src/lib.rs`: Removed duplicate `#[pymethods] impl KdfResult` block that prevented compilation.
- `kdf-perovskite-py/src/lib.rs`: Adapted to new `KdfProcessorRev12::new() -> Result<_, _>` signature with PyValueError bridging.

### Changed

- `KdfProcessorRev12::new()` now returns `Result<Self, Rev12Error>` (validates Claim 39 range).
- `kdf-lib/src/lib.rs` top-level docstring declares itself as "Rev.10 Basic subset" and points to cgb-kdf for Claim 1 compliance.

### Deprecated

- `KdfProcessorRev12::new_unchecked_for_tests` (intentionally non-compliant, for internal test use only).

### Known Limitations (not regressions)

- Claim 5 (time evaluation component) and Claim 17 distributed processing have partial support; full-scale distributed examples are deferred.
- Claim 10 (指数=2 限定) only satisfied in Core layer's default (α=2.0); other layers use other exponents per Master Formulas.
- Benchmark validity proven on synthetic Zipf+redundancy data only; real-world dataset validation pending.
