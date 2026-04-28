#!/usr/bin/env bash
# Pre-commit hook: enforce CI quality gate locally.
#
# Mirrors .github/workflows/rust-quality.yml jobs (fmt + clippy).
# Designed to prevent the kind of accumulated violations that surfaced
# on 2026-04-28 (rustfmt 169 files + clippy 208 lints from past commits
# only detected when CI ran on a markdown-only push).
#
# To install:  ./tools/install-hooks.sh
# To bypass:   git commit --no-verify   (use sparingly)

set -e

echo "[pre-commit] cargo fmt --all -- --check"
if ! cargo fmt --all -- --check; then
    echo "[pre-commit] FAIL: rustfmt violations. Run 'cargo fmt --all' to fix." >&2
    exit 1
fi

echo "[pre-commit] cargo clippy -D warnings (workspace, excluding kdf-python / kdf-wasm)"
if ! cargo clippy --workspace --exclude kdf-python --exclude kdf-wasm --all-targets -- -D warnings; then
    echo "[pre-commit] FAIL: clippy violations." >&2
    exit 1
fi

echo "[pre-commit] OK"
