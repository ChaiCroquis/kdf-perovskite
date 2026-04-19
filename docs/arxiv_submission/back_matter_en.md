# Back Matter (Acknowledgments + References + Appendix A + Appendix B) — English translation (Step 6.6, 2026-04-19)

Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3, lines 590–676.
Translation policy: [`feedback_translation_style.md`](~/.claude/projects/C--work-kdf-perovskite/memory/feedback_translation_style.md).

Back-matter translation is mostly rote. The one note of care is in **Acknowledgments**, where the paper openly states AI-agent collaboration — a disclosure that some reviewers value positively and others may question. The wording is preserved faithfully. BibTeX formatting is provided in the sibling file [`references.bib`](references.bib).

---

## Acknowledgments

We ran independent verification agents (GPT- and Claude-based) at 12 phase boundaries to check the validity of both the positive and negative claims of this work. AI collaboration was used at every stage — specification freezing, code implementation, real-data verification, related-work survey, and drafting of this paper — and the full verification process is recorded in [`docs/VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md).

**日本語バックトランス要約**: 12 回のフェーズ境界で GPT/Claude ベース独立検証エージェントを実行、肯定・否定双方の主張の妥当性をチェック。仕様固定〜論文草稿作成のすべてで AI 協働、検証 process は VERIFIED_FINDINGS.md に完全記録。

---

## References

*Bibliography is generated from [`references.bib`](references.bib) via BibTeX / natbib. KDF-related primary materials — patent publication JP 2026-027032 (filed 2026-02-24) and the verification records in [`VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md) / [`PUBLIC_SUMMARY.md`](../PUBLIC_SUMMARY.md) / [`related_work_survey.md`](../related_work_survey.md) — are cited inline where relevant.*

---

## Appendix A: Summary of Key KDF Formulas

| Claim | Formula | Description |
|---|---|---|
| 7 | $C_{uv} = \deg(u) + \deg(v)$ | Local congestion |
| 8, 9 | $\lambda(C) = \beta(1 + \gamma C^\alpha)$, $\alpha$ positive exponent | Nonlinear form of the decay rate (monotone increasing + power-law term) |
| 10 | $\alpha = 2$ | Fix the power-law exponent at 2 (core of the invention) |
| 14 | $w(t + dt) = w(t) \cdot \exp(-\lambda(C) \cdot dt)$ | Exponential decay law |
| 15 | $\deg_E(v) \le 1 \Rightarrow \text{Rare} \land \text{protected}$ | Rarity detection via absolute threshold |
| 21 | $dt_1 : dt_2 : dt_3 = 5 : 3 : 1$ | Update-period ratio across hierarchical regions |
| 29 | $\Delta \alpha \propto \delta k^4$ | Meta-control quartic law |
| 44 | $S_{\text{outer}} = \tfrac{7}{10} S_{\text{sys}} + \tfrac{2}{10} S_{\text{rel}} + \tfrac{1}{10} S_{\text{attr}}$ | Aggregated integrity score (outer; takes systematic / relational / attribute similarities as inputs) |
| 45 | $S_{\text{inner}} = 0.40 \cdot S_{\text{cos}} + 0.35 \cdot S_{\text{struct}} + 0.25 \cdot S_{\text{sign}}$ | Fingerprint similarity combination (inner; feeds into $S_{\text{sys}}$ / $S_{\text{rel}}$ / $S_{\text{attr}}$ above) |
| 46 | $\phi(v) \in \mathbb{R}^{32}$, derived from Laplacian eigenvalues | Fixed-length structural fingerprint |
| 47–48 | $\theta_L = 0.70 \le S_{\text{outer}} \le \theta_U = 0.80$ | Sandwich acceptance condition, applied to the outer (aggregated) score from Claim 44 (canonical values — refuted in §4.2; see §1.3 C3) |

---

## Appendix B: Implementation Architecture

- `crates/cgb-kdf/` — reference implementation (Rust; PolyForm Noncommercial 1.0.0, commercial license separate); direct tests for all 50 patent claims.
- `crates/bias-detector/` — independently released crate; zero-dependency; a priori KDF applicability screening.
- `demos/D1–D8/` — showcase implementations across 8 domains.
- `benchmarks/sota_comparison/` — SOTA comparison benchmarks.

GitHub: [ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite)

---

*Draft v0.3 — 2026-04-19 (reflecting Phase X Step 1–5 completion). Comments and corrections welcome via GitHub issues.*

---

## Translator's notes on back matter

### Note A — Acknowledgments AI-collaboration disclosure

The Acknowledgments paragraph discloses AI-agent collaboration at **every stage** of the work. This is unusual in academic papers but reflects the honesty stance of this work. The disclosure is kept intact and unsoftened.

**Translation choices**:
- "**independent verification agents (GPT- and Claude-based)**" — preserves the specific vendor-agnostic listing.
- "**at every stage**" — preserves "すべての段階で" without softening to "throughout" or "at various stages".
- "**the full verification process is recorded**" — preserves "完全に記録されている"; points the reader to VERIFIED_FINDINGS.md for reproducibility.

### Note B — References: two new entries beyond the Japanese source

The Theoretical Foundation section cites @burt1992structuralholes, @myerson1977graphs, @calvoarmengol2004networks, and @powell2005network. None of these appear in the Japanese source's References list. We have added them as a new category — "Structural Holes and Network Bargaining" — with IDs [22]–[25]. This renumbering cascades: ref [22] in Japanese (Patil UPCORE) becomes [26], KDF-related moves from [23]/[24] to [27]/[28].

If the user prefers to keep the original Japanese numbering ([22] = Patil, [23] = patent, [24] = verification records) and omit the Structural Holes citations from References, we can move those four refs to an inline citation-only treatment in the Theoretical Foundation section. Flagging for user decision.

### Note C — Category-heading translations

| Japanese category heading | English equivalent |
|---|---|
| 神経科学・記憶固定化 | Neuroscience and Memory Consolidation |
| 免疫学 | Immunology |
| 物理学(臨界現象)| Physics (Critical Phenomena) |
| 連続学習・Coreset Selection | Continual Learning and Coreset Selection |
| 連想記憶 | Associative Memory |
| 認知科学 | Cognitive Science |
| グラフ理論・spectral method | Graph Theory and Spectral Methods |
| 信号処理 | Signal Processing |
| 経済学 | Economics |
| グラフ ML | Graph ML |
| 情報理論 | Information Theory |
| 一般システム論(motivating context のみ)| General Systems Theory (Motivating Context Only) |
| Coreset / unlearning 追補 | Coreset / Unlearning (Supplementary) |
| KDF 関連資料 | KDF-Related Materials |
| — (newly added) | Structural Holes and Network Bargaining |

### Note D — Appendix A: canonical-value disclaimer added inline

Appendix A row for Claims 47–48 lists $\theta_L = 0.70 \le S \le \theta_U = 0.80$ as a formula summary. To prevent a reader from treating these canonical values as validated, we **added the inline note** "(canonical values — refuted in §4.2; see §1.3 C3)". The Japanese source does not include this inline note in Appendix A, but the rest of the paper (§4.2 / §5.2 P10 / §6.3) clearly refutes the canonical values; the inline note prevents a reader who jumps to Appendix A from leaving with the wrong impression. Flagging for user decision — the note can be removed if the user prefers pure formula-only style in the appendix.

### Note E — Appendix B: GitHub URL preserved

The GitHub URL is preserved verbatim. The `demos/D1–D8/` rendering uses an en-dash to match the Japanese "D1-D8" hyphenation without ambiguity.

### Note F — Draft-footer preservation

The final italicized line ("*Draft v0.3 — 2026-04-19 ... Comments and corrections welcome via GitHub issues.*") is preserved as a footer. It signals to reviewers that the document is under active development and accepts external feedback — consistent with the paper's honesty stance.
