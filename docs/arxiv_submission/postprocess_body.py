"""Post-process pandoc-generated paper_body.tex for arxiv compile."""
import re

with open("paper_body.tex", "r", encoding="utf-8") as f:
    txt = f.read()

# 1. Remove the manual References placeholder block (heading + italic note).
#    natbib at end of paper.tex will generate the real References.
start = txt.find(r"\section{References}\label{references}")
if start >= 0:
    # End is the next \section{...} after the placeholder
    end = txt.find(r"\section{", start + 1)
    if end >= 0:
        txt = txt[:start] + txt[end:]
        print(f"  removed References placeholder block ({end - start} chars)")

# 2. Unnumber terminal matter (Acknowledgments / Appendix A / Appendix B).
replacements = [
    (r"\section{Acknowledgments}", r"\section*{Acknowledgments}"),
]
for old, new in replacements:
    if old in txt:
        txt = txt.replace(old, new)
        print(f"  unnumbered {old!r}")

# Appendix A / B have extra title text in braces; use regex fallback.
pat_a = re.compile(r"\\section\{(Appendix A[^}]*)\}")
txt, n_a = pat_a.subn(r"\\section*{\1}", txt)
if n_a:
    print(f"  unnumbered Appendix A heading")
pat_b = re.compile(r"\\section\{(Appendix B[^}]*)\}")
txt, n_b = pat_b.subn(r"\\section*{\1}", txt)
if n_b:
    print(f"  unnumbered Appendix B heading")

with open("paper_body.tex", "w", encoding="utf-8") as f:
    f.write(txt)
print("post-process complete.")
