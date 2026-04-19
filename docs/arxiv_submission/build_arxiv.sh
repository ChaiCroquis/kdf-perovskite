#!/usr/bin/env bash
# Build the arxiv-submission PDF from paper.md via pandoc + XeLaTeX (TeX Live 2026).
#
# Pipeline:
#   1. Extract body-only markdown from paper.md (from "## 1. Introduction" onward)
#   2. pandoc → paper_body.tex with --shift-heading-level-by=-1 and --natbib
#   3. Post-process: strip References placeholder, unnumber terminal matter
#   4. xelatex → bibtex → xelatex × 2 (inside Docker by default)
#
# Usage:
#   ./build_arxiv.sh           # full rebuild inside Docker (requires docker + pandoc)
#   ./build_arxiv.sh --no-docker  # assume xelatex/bibtex on PATH (local install)
#
# Requires: pandoc on PATH (or PANDOC env var), docker (default) or local xelatex.
set -euo pipefail

cd "$(dirname "$0")"

PANDOC="${PANDOC:-pandoc}"
USE_DOCKER=1
for arg in "$@"; do
    case "$arg" in
        --no-docker) USE_DOCKER=0 ;;
    esac
done

echo "[1/4] Extracting body from paper.md..."
sed -n '/^## 1\. Introduction/,$p' paper.md > _body.md

echo "[2/4] Running pandoc (--natbib, heading-shift -1)..."
"$PANDOC" _body.md -o paper_body.tex \
    --shift-heading-level-by=-1 \
    --wrap=preserve \
    --natbib
rm _body.md

echo "[3/4] Post-processing paper_body.tex..."
python3 postprocess_body.py

echo "[4/4] Compiling with XeLaTeX + BibTeX..."
if [[ $USE_DOCKER -eq 1 ]]; then
    WINPWD="$(pwd -W 2>/dev/null || pwd)"
    MSYS_NO_PATHCONV=1 docker run --rm -v "${WINPWD}:/work" -w /work texlive/texlive:latest bash -c "
        xelatex -interaction=nonstopmode paper.tex > /dev/null 2>&1
        bibtex paper > /dev/null 2>&1
        xelatex -interaction=nonstopmode paper.tex > /dev/null 2>&1
        xelatex -interaction=nonstopmode paper.tex 2>&1 | tail -3
    "
else
    xelatex -interaction=nonstopmode paper.tex > /dev/null
    bibtex paper > /dev/null
    xelatex -interaction=nonstopmode paper.tex > /dev/null
    xelatex -interaction=nonstopmode paper.tex | tail -3
fi

if [[ -f paper.pdf ]]; then
    pages=$(pdfinfo paper.pdf 2>/dev/null | awk '/^Pages:/ {print $2}')
    size=$(wc -c < paper.pdf)
    echo "Built paper.pdf: ${pages} pages, ${size} bytes"
else
    echo "ERROR: paper.pdf not produced" >&2
    exit 1
fi
