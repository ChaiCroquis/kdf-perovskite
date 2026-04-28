#!/usr/bin/env bash
# Install repository git hooks.
#
# Sets .git/hooks/pre-commit to point at tools/pre-commit.sh
# (symlink where supported, copy fallback for Windows where symlinks
# require admin / developer mode).
#
# Usage: ./tools/install-hooks.sh

set -e

cd "$(dirname "$0")/.."

HOOK_SRC_REL="../../tools/pre-commit.sh"
HOOK_SRC_ABS="tools/pre-commit.sh"
HOOK_DEST=".git/hooks/pre-commit"

if [ ! -d ".git" ]; then
    echo "[install-hooks] ERROR: .git not found — run from repo root or via ./tools/install-hooks.sh" >&2
    exit 1
fi

if [ ! -f "$HOOK_SRC_ABS" ]; then
    echo "[install-hooks] ERROR: $HOOK_SRC_ABS not found" >&2
    exit 1
fi

# Remove any existing hook
[ -e "$HOOK_DEST" ] && rm -f "$HOOK_DEST"

# Try symlink first (Unix-like), fall back to copy (Windows without dev mode)
if ln -sf "$HOOK_SRC_REL" "$HOOK_DEST" 2>/dev/null; then
    echo "[install-hooks] symlinked $HOOK_DEST -> $HOOK_SRC_REL"
else
    cp "$HOOK_SRC_ABS" "$HOOK_DEST"
    echo "[install-hooks] copied $HOOK_SRC_ABS to $HOOK_DEST (symlink unsupported)"
fi

chmod +x "$HOOK_SRC_ABS" "$HOOK_DEST" 2>/dev/null || true
echo "[install-hooks] done — pre-commit hook is now active"
