#!/usr/bin/env python3
"""Safety-harden the section files for arxiv AutoTeX compilation.

Performs two transformations in-place on section_*_en.md and
theoretical_foundation_en.md / back_matter_en.md:

1. Emoji → textual markers  (✅ → [True], ❌ → [False], etc.)
   Avoids Unicode compile failures under arxiv's default pdflatex.

2. Natural-language citations → pandoc-style [@key] where unambiguous.
   Enables proper \cite{} output via pandoc --citeproc / natbib.

Run AFTER edits and BEFORE build_paper.py, so paper.md picks up the changes.

Usage:
    python harden_for_arxiv.py
"""

import re
import sys
from pathlib import Path

BASE = Path(__file__).parent

FILES = [
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

# ----------------------------------------------------------------------------
# Emoji → textual fallback
# Order matters: "⚠️" (U+26A0 + U+FE0F VS16) must be matched before bare "⚠".
# ----------------------------------------------------------------------------

EMOJI_MAP = [
    ("\u26A0\uFE0F", "[Warning]"),   # ⚠️ with variation selector
    ("\u26A0", "[Warning]"),           # ⚠ bare
    ("\u2705", "[True]"),              # ✅
    ("\u274C", "[False]"),             # ❌
    ("\U0001F527", "[Calibration Required]"),  # 🔧
    ("\u2605", "[Significant]"),       # ★
]

# ----------------------------------------------------------------------------
# Citation map: natural-language → pandoc [@key]
# Entries apply in order; earlier patterns take precedence.
# Use @key for narrative (author is subject); [@key] for parenthetical.
# ----------------------------------------------------------------------------

CITATION_MAP: list[tuple[str, str]] = [
    # Tartaglia — parenthetical in body
    (r"\(Tartaglia et al\.,\s*PNAS 2025 — independently parallel\)",
     "[@tartaglia2025twofactor]"),
    (r"Tartaglia et al\.,\s*\"Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling\" \(PNAS 2025, bioRxiv 2024-07\)",
     'Tartaglia et al., "Two-factor synaptic consolidation reconciles robust memory with pruning and homeostatic scaling" [@tartaglia2025twofactor]'),
    (r"Tartaglia et al\.\s*\(PNAS 2025, bioRxiv 2024-07\)",
     "Tartaglia et al. [@tartaglia2025twofactor]"),
    (r"Tartaglia et al\.\s*PNAS 2025", "@tartaglia2025twofactor"),

    # McClelland 1995
    (r"\(Complementary Learning Systems, McClelland 1995\)",
     "(Complementary Learning Systems) [@mcclelland1995cls]"),
    (r"McClelland 1995", "@mcclelland1995cls"),

    # Miller 1978
    (r"\"Living Systems Theory\" \(Miller 1978\)",
     '"Living Systems Theory" [@miller1978livingsystems]'),
    (r"\(Miller 1978\)", "[@miller1978livingsystems]"),

    # Hopfield
    (r"\(Hopfield 1982\)", "[@hopfield1982neural]"),
    (r"Hopfield 1982 \+ modern Hopfield networks",
     "@hopfield1982neural + modern Hopfield networks"),

    # Amit
    (r"Amit, Gutfreund, and Sompolinsky \(1987\)",
     "Amit, Gutfreund, and Sompolinsky [@amit1987replica]"),
    (r"Amit et al\. 1987", "@amit1987replica"),

    # Ramsauer modern Hopfield
    (r"modern Hopfield networks \(Ramsauer et al\. 2021\)",
     "modern Hopfield networks [@ramsauer2021hopfield]"),
    (r"Ramsauer et al\. \(2021\)", "Ramsauer et al. [@ramsauer2021hopfield]"),
    (r"Ramsauer et al\. 2021", "@ramsauer2021hopfield"),

    # Kirkpatrick EWC
    (r"\(Kirkpatrick, PNAS 2017\)", "[@kirkpatrick2017ewc]"),
    (r"Kirkpatrick et al\. PNAS 2017", "@kirkpatrick2017ewc"),

    # Nguyen
    (r"\(Nguyen et al\. 2018\)", "[@nguyen2018ctdne]"),
    (r"Nguyen et al\. 2018", "@nguyen2018ctdne"),

    # Patil UPCORE
    (r"\(Patil et al\. 2025, arXiv:2502\.15082\)", "[@patil2025upcore]"),
    (r"UPCORE\s*\(Patil et al\. 2025, arXiv:2502\.15082\)",
     "UPCORE [@patil2025upcore]"),

    # Aharon K-SVD
    (r"\(Aharon et al\. 2006\)", "[@aharon2006ksvd]"),
    (r"Aharon et al\. 2006", "@aharon2006ksvd"),

    # Pareto
    (r"\(Vilfredo Pareto 1896\)", "[@pareto1896economie]"),
    (r"Pareto 1896", "@pareto1896economie"),

    # Burt Structural Holes
    (r'"Structural Holes" theory of organizational sociology \(Burt 1992\)',
     '"Structural Holes" theory of organizational sociology [@burt1992structuralholes]'),
    (r"Burt's Structural Holes \(1992\)",
     "Burt's Structural Holes theory [@burt1992structuralholes]"),
    (r"Burt \(1992\)", "Burt [@burt1992structuralholes]"),
    (r"since Burt 1992,",
     "since Burt's 1992 work [@burt1992structuralholes],"),
    (r"Burt 1992", "@burt1992structuralholes"),

    # Myerson
    (r"\(Myerson 1977;\s*Calvó-Armengol\s*&\s*Jackson 2004\)",
     "[@myerson1977graphs; @calvoarmengol2004networks]"),
    (r"Myerson 1977", "@myerson1977graphs"),
    (r"Calvó-Armengol\s*&\s*Jackson 2004", "@calvoarmengol2004networks"),

    # Powell
    (r"\(Powell et al\. 2005\)", "[@powell2005network]"),
    (r"critiques such as Powell et al\. 2005",
     "critiques such as @powell2005network"),
    (r"Powell et al\. 2005", "@powell2005network"),

    # §3 table entries — added 2026-04-19 after Loop 1 reviewer feedback.
    # These were previously left as plain text inside the ten-domain table,
    # producing a mixed-style (plain text + @key) bibliography after pandoc.
    (r"McClelland et al\. 1995", "@mcclelland1995cls"),
    (r"Mesin et al\. Cell 2020", "@mesin2020gc"),
    (r"Bak, Tang, Wiesenfeld 1987", "@bak1987soc"),
    (r"Ginzburg-Landau 1950", "@ginzburg1950superconductivity"),
    (r"Sener\s*&\s*Savarese 2018", "@sener2018coreset"),
    (r"Wang et al\. CIKM 2024", "@wang2024ecs"),
    (r"Hopfield 1982", "@hopfield1982neural"),
    (r"Levin\s*&\s*Peres 2017", "@levin2017markov"),
    (r"Embrechts 1997", "@embrechts1997extreme"),
    (r"Nguyen et al\. WWW 2018", "@nguyen2018ctdne"),
]


def harden_file(path: Path) -> tuple[int, int]:
    """Return (emoji_replacements, citation_replacements) for this file."""
    text = path.read_text(encoding="utf-8")
    original = text

    emoji_count = 0
    for needle, repl in EMOJI_MAP:
        n = text.count(needle)
        if n:
            text = text.replace(needle, repl)
            emoji_count += n

    cite_count = 0
    for pattern, replacement in CITATION_MAP:
        n = len(re.findall(pattern, text))
        if n:
            text = re.sub(pattern, replacement, text)
            cite_count += n

    if text != original:
        path.write_text(text, encoding="utf-8")

    return emoji_count, cite_count


def main() -> int:
    totals = {"emoji": 0, "citation": 0}
    for fname in FILES:
        path = BASE / fname
        if not path.exists():
            print(f"WARNING: {fname} not found", file=sys.stderr)
            continue
        e, c = harden_file(path)
        totals["emoji"] += e
        totals["citation"] += c
        print(f"  {fname}: {e} emoji, {c} citation replacements")

    print()
    print(f"Total: {totals['emoji']} emoji, {totals['citation']} citations")
    print("Run `python build_paper.py` next to regenerate paper.md")
    return 0


if __name__ == "__main__":
    sys.exit(main())
