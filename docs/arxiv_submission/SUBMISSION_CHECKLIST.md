# arXiv Submission Checklist — KDF Preprint

This checklist covers the steps between "LaTeX builds cleanly" and "arXiv preprint is public." Items marked **[Claude can assist]** are automatable; items marked **[User action]** require the author's direct input.

## 1. Metadata

### arXiv categories **[User action]**

Recommended categories for KDF:

| Role | Category | Justification |
|---|---|---|
| **Primary** | `cs.LG` (Machine Learning) | Core contribution is a learning-adjacent graph-compression architecture; LongMemEval / LoCoMo benchmarks belong to ML memory research |
| **Secondary** | `cs.IR` (Information Retrieval) | Retention/selection against unknown future queries is an IR concern; BEIR / semantic-retrieval non-applicability result is IR-relevant |
| **Secondary** | `cs.DB` (Databases) | Graph-compression + archival retention is a database topic; NASA HTTP log / OSS git pruning |

Optional cross-list if desired:

- `cs.SI` (Social and Information Networks) — Structural Holes theoretical foundation section
- `cs.NE` (Neural and Evolutionary Computing) — Hopfield / associative-memory discussion in §4.2

arXiv submission UI at <https://arxiv.org/submit> lets you set one primary and multiple secondaries.

### Title & Abstract **[✅ done]**

Final versions are in [`title_and_abstract_v2.md`](title_and_abstract_v2.md). Both are already reflected in `paper.tex`.

### Authors **[User action]**

Current paper.tex has:

- Name: Chai (Kuroki Yasuhiro)
- Affiliation: Independent researcher, Japan
- Email: `garden.of.knowledge.chai@gmail.com`

Verify these are as you want them to appear. Add an ORCID if you have one: add `\email{orcid:0000-0000-0000-0000}` in `\author{}`.

### MSC / ACM classification codes **[User action, optional]**

arXiv allows MSC (mathematical) and ACM (computer-science) classification codes. Suggested:

- ACM: `I.2.6` (Learning) as primary; `H.3.3` (Information Search and Retrieval) as secondary.
- MSC: `68T05` (Learning and adaptive systems) as primary; `68R10` (Graph theory in computer science) as secondary.

These are optional — arXiv submissions are accepted without them.

---

## 2. Files to include in the submission

### Required **[✅ done]**

- `paper.tex` — LaTeX source
- `paper_body.tex` — body generated from `paper.md` via pandoc
- `references.bib` — BibTeX (29 entries)

### Not included (local-only)

- `paper.md` — markdown source (regeneration tooling; not uploaded)
- `build_paper.py` — assembly script (not uploaded)
- `section*_en.md` — per-section translations with notes (not uploaded)
- `title_and_abstract_v2.md`, `title_and_abstract_v1.md` — working documents (not uploaded)
- `BUILD.md`, `SUBMISSION_CHECKLIST.md`, `PLAN.md` — process documentation (not uploaded)

### arXiv upload format

Package as a tarball:

```bash
cd docs/arxiv_submission
tar czf kdf_arxiv_v0.3.tar.gz paper.tex paper_body.tex references.bib
```

Then upload `kdf_arxiv_v0.3.tar.gz` to arXiv's submission form.

---

## 3. Pre-submission verification **[User action + Claude can assist]**

### Build verification

- [ ] `python build_paper.py` completes without errors
- [ ] `pandoc paper.md -o paper_body.tex` completes without errors
- [ ] LaTeX compile completes (xelatex × 3 + bibtex × 1) without errors
- [ ] `paper.pdf` renders all nine main tables without overflow
- [ ] Page count is in the expected range (estimated 25–35 pages for arxiv style at 11pt)

### Content verification

- [ ] Abstract matches `title_and_abstract_v2.md` FINAL version (no older v1 leakage)
- [ ] §1.3 C3 four-benchmark canonical refutation appears verbatim in Abstract, §1.3, §4.2.1, §5.2 P10, §6.3, §6.4
- [ ] §5.1 shows six positive cases (P1/P2/P3/P7/P8/P11); §5.2 shows four negative cases (P5/P6/P9/P10)
- [ ] Mem0 vs KDF 2×2 matrix in §7 has correct values (0.672/0.434/0.722/0.452/0.206/0.312/0.090/0.324)
- [ ] F-060 Router paragraph in §7 concludes "complementary layer to Mem0, not a replacement"
- [ ] Theoretical Foundation's Limitations subsection is preserved (not soft-stripped)
- [ ] Appendix A Claim 47–48 row includes the inline refutation note
- [ ] No Japanese characters remain in `paper.md` (except as BibTeX titles if needed)

### Reference verification

- [ ] All in-text citations have matching BibTeX entries (if `[@key]` format is used)
- [ ] Patent JP 2026-027032 appears in refs
- [ ] Structural Holes refs (Burt 1992 / Myerson 1977 / Calvó-Armengol 2004 / Powell 2005) are present

---

## 4. Submission procedure **[User action]**

1. **Create an arXiv account** if you don't have one: <https://arxiv.org/user/register>
2. **Endorsement**: arXiv requires endorsement for first-time submissions in some categories. `cs.LG` typically does not require endorsement, but confirm at <https://arxiv.org/auth/show-endorsers>
3. **Start a new submission**: <https://arxiv.org/submit>
4. **Upload**: select "Compiled LaTeX" format, upload `kdf_arxiv_v0.3.tar.gz`
5. **Set metadata**: title, abstract, authors, categories (cs.LG primary + cs.IR + cs.DB)
6. **Set license**: recommended `CC BY 4.0` or `arXiv.org perpetual, non-exclusive license` (most permissive for preprints)
7. **Preview**: arXiv will compile your submission and generate a PDF preview. Verify it matches your local build.
8. **Submit**: you receive a temporary arXiv ID (e.g., `2604.xxxxx`). The paper is announced at the next business-day cutoff (14:00 ET / 18:00 UTC).
9. **Publication**: after announcement, the paper has a permanent arXiv URL (e.g., `https://arxiv.org/abs/2604.xxxxx`).

---

## 5. Post-submission actions **[User action]**

- [ ] Add the arXiv URL to the repo README (top of file, under "Paper" heading)
- [ ] Open a GitHub release tagged `paper-v0.3` with the arXiv URL in the release notes
- [ ] Optionally: create a Zenodo DOI for the GitHub repo at release time (<https://zenodo.org/>)
- [ ] Share on relevant channels (Twitter/X, relevant Slack/Discord communities, research-gate)
- [ ] Update [MEMORY.md](~/.claude/projects/C--work-kdf-perovskite/memory/MEMORY.md) with the arXiv URL as a project memory (for future sessions)

---

## 6. Version-update protocol

When a significant finding updates the paper (e.g., Phase XI or a new benchmark):

1. Update `paper_draft.md` (Japanese source of truth).
2. Update corresponding `section*_en.md` file(s).
3. Bump version in `paper.tex` and `paper.md` top matter.
4. Re-run `python build_paper.py` and rebuild PDF.
5. Submit a new arXiv version via the existing submission URL (arxiv supports version bumps).
6. Update the README reference + Zenodo DOI if any.

arXiv versioning preserves history, so v2 / v3 / etc. are expected and standard.

---

## 7. Known open items (flagged in translator's notes)

Items the author may want to resolve before final submission:

- [ ] **Appendix A Claim 47–48 inline refutation note** — currently reads "(canonical values — refuted in §4.2; see §1.3 C3)". Not in the original Japanese source; added for reader safety. Remove or keep per preference (Note D in [`back_matter_en.md`](back_matter_en.md)).
- [ ] **References numbering cascade** — the Structural Holes citations (Burt, Myerson, Calvó-Armengol, Powell) were added as [22]–[25], shifting Patil UPCORE to [26] and KDF-related materials to [27]/[28]. Verify the cascade matches your preference (Note B in [`back_matter_en.md`](back_matter_en.md)).
- [ ] **Citation format in body** — currently natural-language "(Burt 1992)". For numbered `\cite{}` integration, rewrite as `[@burt1992structuralholes]` in the section files and regenerate paper.md.
- [ ] **Emoji handling** — ✅/❌/⚠️/🔧/★ require XeLaTeX + emoji font. If pdflatex is mandatory, replace with textual markers (see BUILD.md).
- [ ] **Figures** — current paper has no figures, only tables. Consider adding: (a) architecture diagram showing three mechanisms; (b) 2×2 Mem0-vs-KDF matrix as a chart; (c) KDF trajectory in the 10-domain/3-pillar space.

None of these is blocking. arXiv accepts preprints without figures, and the current PDF is complete.

---

## Summary

- **Canonical sources**: `paper.md` (body) + `paper.tex` (wrapper) + `references.bib` (refs)
- **Build**: `python build_paper.py && pandoc paper.md -o paper_body.tex && xelatex paper.tex && bibtex paper && xelatex paper.tex && xelatex paper.tex`
- **Upload**: tarball of `paper.tex` + `paper_body.tex` + `references.bib` to arXiv
- **Categories**: `cs.LG` primary; `cs.IR` + `cs.DB` secondary
- **License**: CC BY 4.0 recommended

Ask Claude for help with any of these steps — most are automatable.
