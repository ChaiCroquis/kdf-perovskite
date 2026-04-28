#!/usr/bin/env bash
# Configure git to use the repository-tracked .githooks/ directory.
#
# Why this approach (not .git/hooks/):
#   .githooks/ is committed to the repo. `git config core.hooksPath .githooks`
#   tells git to look there instead of .git/hooks/. Result: each developer
#   runs this script once, and any new hook (or update to an existing hook)
#   added to .githooks/ takes effect on the next commit — no manual re-sync,
#   no symlinks needed (which require admin/dev-mode on Windows).
#
# Usage: ./tools/install-hooks.sh
# Disable: git config --local --unset core.hooksPath

set -e
cd "$(dirname "$0")/.."

if [ ! -d ".git" ]; then
    echo "[install-hooks] ERROR: .git not found — run from repo root" >&2
    exit 1
fi

if [ ! -d ".githooks" ]; then
    echo "[install-hooks] ERROR: .githooks/ not found in repo" >&2
    exit 1
fi

# Migrate from legacy .git/hooks/pre-commit (symlink/copy era)
if [ -e ".git/hooks/pre-commit" ]; then
    rm -f ".git/hooks/pre-commit"
    echo "[install-hooks] removed legacy .git/hooks/pre-commit"
fi

git config --local core.hooksPath .githooks
chmod +x .githooks/* 2>/dev/null || true

echo "[install-hooks] core.hooksPath = .githooks"
echo "[install-hooks] active hooks:"
ls -1 .githooks/ | sed 's/^/  /'
echo "[install-hooks] done"
