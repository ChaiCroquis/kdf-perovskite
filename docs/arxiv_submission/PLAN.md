# arxiv preprint 準備計画(Step 6)

**目的**: `docs/paper_draft.md` v0.3(日本語)を arxiv preprint として公開可能な状態に持っていく。

**前提**:
- 本体主張・empirical evidence は v0.3 で確定済(誠実性原則に照らしてすべての主張に empirical anchor あり)
- arxiv は LaTeX 推奨、Markdown source + pandoc LaTeX 変換で作業
- 1-2 週ブロックを許容、複数 session で iterative に進める

## Sub-steps

### 6.1 — タイトル + Abstract(first iteration)✅ 完了 2026-04-19
- [x] arxiv_submission ディレクトリ作成
- [x] PLAN.md 作成(本ファイル)
- [x] タイトル候補 3-4 本作成 + 選定理由 → **candidate D 採用**
- [x] Abstract 英訳(v0.3 日本語から、245 words 事実骨格版)
- [x] 翻訳スタイル指針の確定(user 合意)
- [x] 翻訳スタイル memory 化([feedback_translation_style.md](../../../memory/feedback_translation_style.md))
- [x] v2 final draft 発行([title_and_abstract_v2.md](title_and_abstract_v2.md))

### 6.2 — §1 Introduction 英訳 ✅ 完了 2026-04-19
- [x] 1.1 問題設定
- [x] 1.2 観察(10 領域並行発見)
- [x] 1.3 主張(C1 統合 / C2 構造的類似 / C3 機構 novel + canonical 値反証)
- [x] 1.4 Universality と novelty の緊張
- [x] 1.5 本論文構成
- [x] count 統一 sweep: §1.5 / §6.4 / §7 を 肯定 6・陰性 4 に更新、LaTeX deprecation マーク追加
- [x] 成果物: [section1_intro_en.md](section1_intro_en.md)(commit 4a32878)

### 6.3 — §2 KDF アーキテクチャ + §3 10 領域対応 ✅ 完了 2026-04-19
- [x] 2.1 基本構造(3 手段 + 数式)
- [x] 2.2 メタ制御(Claim 27-32)
- [x] 2.3 階層管理領域(Claim 20-22)
- [x] 3 領域対応表と 3.1 Tartaglia 独立並行発見
- [x] 成果物: [section2_architecture_en.md](section2_architecture_en.md) + [section3_domains_en.md](section3_domains_en.md)

### 6.4 — §4 構造的類似性(最重要節)✅ 完了 2026-04-19
- [x] 4.1 δk⁴ ↔ Ginzburg-Landau 4 次項
- [x] 4.2 sandwich θ_U(canonical 値 4-benchmark 反証を含む長い節)
- [x] 4.2.1 Phase X Step 2 追加検証(F-070)
- [x] 4.3 exp(-λdt) ↔ Markov サバイバル確率(motivating analogy 扱い)
- [x] 成果物: [section4_structural_similarities_en.md](section4_structural_similarities_en.md)(237 行、translator's notes A-H)

### 6.5 — §5 実証 ✅ 完了 2026-04-19
- [x] 5.1 肯定 6 件(P1/P2/P3/P7/P8/P11)
- [x] 5.2 陰性 4 件(P5/P6/P9/P10)
- [x] 5.3 bias-detector
- [x] 成果物: [section5_evaluation_en.md](section5_evaluation_en.md)(translator's notes A-G)

### 6.6 — §6 Discussion + §7 Conclusion + Theoretical Foundation + References + Appendix ✅ 完了 2026-04-19
- [x] 6.1 領域不変アーキテクチャ仮説
- [x] 6.2 領域別専門化
- [x] 6.3 特許とライセンス位置づけ
- [x] 6.4 Limitations(narrowing 完成形、9 bullet)
- [x] 7 Conclusion + 2 benchmark × 2 model matrix + Phase X findings 表 + F-060 Router + positioning pivot
- [x] Theoretical Foundation(Burt's Structural Holes 対応、graph-theoretic isomorphism 主張 + limitations)
- [x] References を BibTeX に整形(29 entries、Structural Holes 関連 4 件新規追加)
- [x] Appendix A 数式まとめ + Appendix B 実装アーキテクチャ
- [x] 成果物:
  - [section6_discussion_en.md](section6_discussion_en.md)(translator's notes A-E)
  - [section7_conclusion_en.md](section7_conclusion_en.md)(translator's notes A-G)
  - [theoretical_foundation_en.md](theoretical_foundation_en.md)(translator's notes A-G)
  - [back_matter_en.md](back_matter_en.md)(translator's notes A-F)
  - [references.bib](references.bib)(BibTeX 29 entries)

### 6.7 — 最終仕上げ ⏳ 大部分完了、user action 待ち 2026-04-19
- [x] 全体通し読み + スタイル統一(paper.md で body-only 統合、structure sanity check pass、§1-§7 + Theoretical Foundation + Appendix の heading 構造正しい、日本語バックトランス / Translator's notes の leak 0)
- [ ] Figures(もし追加する場合):アーキテクチャ図、2×2 matrix、trajectory — **user 判断待ち、現状 table のみで submission 可能**
- [x] LaTeX template 作成(arxiv 形式): [paper.tex](paper.tex)、article 11pt、XeLaTeX 推奨
- [x] Assembly script 作成: [build_paper.py](build_paper.py)(section_*_en.md → paper.md で translator's notes + 日本語バックトランス strip)
- [ ] Pandoc 変換 → PDF ビルド確認 — **pandoc/LaTeX ローカル未インストール、BUILD.md に 4 option(local / Overleaf / Tectonic / Docker)記載、user 環境で実行**
- [x] References BibTeX 完成: [references.bib](references.bib)(29 entries、6.6 で完了)
- [x] arxiv カテゴリ選定: cs.LG primary、cs.IR + cs.DB secondary、(optional) cs.SI + cs.NE — SUBMISSION_CHECKLIST に根拠記載
- [ ] ORCID / Affiliation / Contact 確認 — **user action**(現状 paper.tex に email と affiliation 記載、ORCID は user が追加)
- [ ] GitHub repo 公開状態確認(Zenodo DOI 取得も考慮)— **user action**(SUBMISSION_CHECKLIST §5 post-submission に記載)
- [ ] arxiv 提出(user が最終実行)— **user action**(SUBMISSION_CHECKLIST §4 に submission 手順 9 step)
- [x] 成果物:
  - [paper.md](paper.md)(740 行、9155 words、body-only assembled)
  - [paper.tex](paper.tex)(LaTeX wrapper、title/abstract/bibliography 設定済、body は `\input{paper_body.tex}`)
  - [build_paper.py](build_paper.py)(Python script、section files → paper.md 変換)
  - [BUILD.md](BUILD.md)(4 build option + troubleshooting)
  - [SUBMISSION_CHECKLIST.md](SUBMISSION_CHECKLIST.md)(metadata / pre-sub verification / submission procedure / post-sub actions / open items)

### User action 残項目(submission までの 3 ステップ)
1. **ビルド環境準備**: BUILD.md の option A〜D から 1 つ選択、pandoc + LaTeX 環境を用意
2. **PDF build 実行**: `python build_paper.py && pandoc paper.md -o paper_body.tex && xelatex paper.tex && bibtex paper && xelatex paper.tex && xelatex paper.tex`
3. **arxiv 提出**: SUBMISSION_CHECKLIST §4 の 9 step(endorsement → metadata → upload → preview → submit)

## 見積もり

| Sub-step | 予想 session 数 | 累積 |
|---|---:|---:|
| 6.1 | 1(本) | 1 |
| 6.2 | 1-2 | 2-3 |
| 6.3 | 1 | 3-4 |
| 6.4 | 2(§4.2 が重い) | 5-6 |
| 6.5 | 1 | 6-7 |
| 6.6 | 1-2 | 7-9 |
| 6.7 | 2-3 | 9-12 |

合計 9-12 session。各 session は 1-3 時間と想定。

## 原則

- **誠実性は譲らない** — narrowing(C3 canonical 値反証)を pitch 用に薄める改訂はしない
- **英訳は意味優先** — 日本語のニュアンスが英語で自然に出ない部分は、日本語の原文意図に沿った意訳を採用、直訳固執しない
- **技術用語は業界標準に合わせる** — "サンドイッチ採用域" は "sandwich acceptance band" など
- **引用文献の整合性** — 既に Japanese paper で引用した文献は同一 BibTeX key を使う

## 参照

- Source: [`docs/paper_draft.md`](../paper_draft.md) v0.3
- Specification authority: [`docs/patent/filed/`](../patent/filed/) 5 書類(矛盾時は filed/ が正)
- Evidence: [`docs/VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md) F-001〜F-072
