#!/usr/bin/env python3
"""
[DEPRECATED 2026-05-10] このスクリプトを実行すると paper.md が v1 状態に巻き戻る。

paper.md は v2.7 (Phase 2 Retrospective integrated, Zenodo DOI 10.5281/zenodo.20104836,
2026-05-10 publish) 以降、section_*_en.md を経由せず直接編集されている。section_*_en.md
は v1 時点 (2026-04-19) で固定されており、本 script を実行すると v2.7 の Addendum / §5
table F-097/F-099/F-100 行 / Abstract v2 update / §6.4 Limitations v2.5/v2.7 / §7
Conclusion router F-099 段落 / Reproducibility cross-ref が **すべて消失** する。

実行禁止条件:
  - paper.md が現在 v2.7 以降 (head に "## Version 2 Addendum" がある)
  - section_*_en.md が v1 時点のまま (date < 2026-04-30)

回復用途のみ許可 (例: 万一 paper.md が破損した場合、v1 状態に戻して再 v2 作業を始める
緊急 fallback): 明示確認後にのみ手動実行。

正規 build pipeline (v2.7 以降):
  1. paper.md は人手で直接編集 (section_*_en.md は触らない)
  2. build_arxiv.sh で pandoc + xelatex 経由で paper.pdf 生成
     (extraction 起点は "## Version 2 Addendum"、Abstract update + Addendum も PDF
     にレンダされる)
  3. arxiv_submission.tar.gz は build_arxiv.sh が paper.tex + paper_body.tex +
     references.bib をまとめる

deprecated 経緯: project_kdf_phases.md memory line "post-publish housekeeping" entry
(2026-05-10) 参照。

---

(原文 docstring、historical reference として保持)

Assemble the arxiv-ready paper.md by concatenating section_*_en.md files,
stripping translator's notes and Japanese back-translation blocks.

Usage:
    python build_paper.py
    -> produces paper.md (body-only, pandoc-ready)

Output: docs/arxiv_submission/paper.md

The output file is the canonical English source for:
- pandoc conversion to LaTeX: pandoc paper.md -o paper.tex -s --bibliography=references.bib
- pandoc direct PDF build: pandoc paper.md -o paper.pdf --pdf-engine=xelatex --bibliography=references.bib
- Overleaf upload: copy paper.md content into Overleaf's pandoc-enabled project
"""

import re
import sys
from pathlib import Path


BASE = Path(__file__).parent

SECTION_FILES = [
    "section1_intro_en.md",
    "section2_architecture_en.md",
    "section3_domains_en.md",
    "section4_structural_similarities_en.md",
    "section5_evaluation_en.md",
    "section6_discussion_en.md",
    "section7_conclusion_en.md",
    "theoretical_foundation_en.md",
    "back_matter_en.md",
]


def strip_backtranslations(text: str) -> str:
    """Remove **日本語バックトランス要約**: ... paragraph blocks.

    Each block is a single paragraph starting with the marker and ending at the
    next blank line. Trailing blank lines are collapsed to avoid doubling.
    """
    lines = text.split("\n")
    out = []
    in_block = False
    for line in lines:
        stripped = line.strip()
        if stripped.startswith("**日本語バックトランス要約**"):
            in_block = True
            continue
        if in_block:
            if stripped == "":
                in_block = False
                # Skip the terminating blank line to avoid a double blank.
                continue
            # Still inside the block; discard.
            continue
        out.append(line)
    return "\n".join(out)


def extract_body(content: str) -> str:
    """Extract the paper-body portion from a section file.

    Start: first ``## `` heading (actual paper content begins here).
    End: ``## Translator's notes`` heading (exclusive) or end-of-file.
    Also removes trailing ``---`` separators.
    """
    first_h2 = re.search(r"^## .+$", content, re.MULTILINE)
    if not first_h2:
        return ""
    start = first_h2.start()

    rest = content[start:]
    notes = re.search(r"^## Translator's notes", rest, re.MULTILINE)
    if notes:
        rest = rest[: notes.start()]

    rest = strip_backtranslations(rest)
    rest = re.sub(r"\n---\s*$", "\n", rest)
    return rest.rstrip() + "\n"


def extract_title_and_abstract(path: Path) -> tuple[str, str]:
    """Return (title, abstract_plain) from title_and_abstract_v2.md."""
    content = path.read_text(encoding="utf-8")

    title_m = re.search(
        r"## Title\s*\(FINAL\)\s*\n+>\s*\*\*(.+?)\*\*",
        content,
        re.DOTALL,
    )
    title = title_m.group(1).strip() if title_m else "KDF paper"

    abs_m = re.search(
        r"## Abstract\s*\(FINAL[^)]*\)\s*\n+((?:> .*\n|>\s*\n)+)",
        content,
    )
    if abs_m:
        raw = abs_m.group(1)
        stripped = []
        for ln in raw.splitlines():
            if ln.startswith("> "):
                stripped.append(ln[2:])
            elif ln.startswith(">"):
                stripped.append(ln[1:])
            else:
                stripped.append(ln)
        abstract = "\n".join(stripped).strip()
    else:
        abstract = "(Abstract extraction failed — see title_and_abstract_v2.md)"

    return title, abstract


def main() -> int:
    # [DEPRECATED 2026-05-10] paper.md v2.7 状態を保護する Stop Gate
    # 全文 scan でマーカ検出 (head[:N] だと multi-byte char + 3000 char 超の
    # 位置に "## Version 2 Addendum" がある case を見逃す、2026-05-10 確認済 bug)
    paper_md_path = BASE / "paper.md"
    if paper_md_path.exists():
        full = paper_md_path.read_text(encoding="utf-8")
        v27_markers = [
            "## Version 2 Addendum",
            "v2 update (2026-04",
            "7-pattern empirical arc",
            "F-100",
            "F-097",
        ]
        hits = [m for m in v27_markers if m in full]
        if hits:
            print("=" * 70, file=sys.stderr)
            print("ABORT: build_paper.py is DEPRECATED as of 2026-05-10.", file=sys.stderr)
            print(f"paper.md contains v2.x markers: {hits}", file=sys.stderr)
            print("Running this script would OVERWRITE v2.x content with v1", file=sys.stderr)
            print("content from section_*_en.md (which are frozen at 2026-04-19).", file=sys.stderr)
            print("", file=sys.stderr)
            print("If you really want to regenerate paper.md from sections (e.g. as an", file=sys.stderr)
            print("emergency fallback after corruption), run with FORCE=1:", file=sys.stderr)
            print("    FORCE=1 python build_paper.py", file=sys.stderr)
            print("", file=sys.stderr)
            print("See docstring at top of this file for deprecation rationale.", file=sys.stderr)
            print("=" * 70, file=sys.stderr)
            import os
            if os.environ.get("FORCE") != "1":
                return 2

    parts: list[str] = []

    title, abstract = extract_title_and_abstract(BASE / "title_and_abstract_v2.md")

    parts.append(f"# {title}\n")
    parts.append("")
    parts.append("**Author:** Yasuhiro Kuroki  ")
    parts.append("**ORCID:** [0009-0006-8943-9344](https://orcid.org/0009-0006-8943-9344)  ")
    parts.append("**Affiliation:** Independent researcher, Japan  ")
    parts.append("**Patent:** JP 2026-027032 (filed 2026-02-24)  ")
    parts.append(
        "**Code:** [github.com/ChaiCroquis/kdf-perovskite]"
        "(https://github.com/ChaiCroquis/kdf-perovskite) (PolyForm Noncommercial 1.0.0; commercial license separate)  "
    )
    parts.append("**Draft version:** v0.3, 2026-04-19")
    parts.append("")
    parts.append("---")
    parts.append("")
    parts.append("## Abstract")
    parts.append("")
    parts.append(abstract)
    parts.append("")
    parts.append(
        "**Keywords:** information preservation, graph metabolism, rarity protection, "
        "analogy discovery, Laplacian fingerprint, Ginzburg-Landau, Hopfield attractor, "
        "Equitable Coreset Selection, complementary learning systems, memory consolidation"
    )
    parts.append("")
    parts.append("---")
    parts.append("")

    for fname in SECTION_FILES:
        path = BASE / fname
        if not path.exists():
            print(f"WARNING: {fname} not found", file=sys.stderr)
            continue
        body = extract_body(path.read_text(encoding="utf-8"))
        parts.append(body)
        parts.append("---")
        parts.append("")

    paper_md = "\n".join(parts)
    # Collapse 3+ consecutive blank lines into 2 for cleanliness.
    paper_md = re.sub(r"\n{3,}", "\n\n", paper_md)

    out = BASE / "paper.md"
    out.write_text(paper_md, encoding="utf-8")

    # Print stats
    n_lines = paper_md.count("\n") + 1
    n_words = len(paper_md.split())
    print(f"Wrote {out.relative_to(BASE.parent.parent)} ({n_lines} lines, {n_words} words)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
