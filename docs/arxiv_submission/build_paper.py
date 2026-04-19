#!/usr/bin/env python3
"""
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
