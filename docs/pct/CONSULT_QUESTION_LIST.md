# PCT Consult Question List(弁理士 consult input、Layer 1 / Layer 2 構造)

**Date**: 2026-04-29
**Status**: PCT consult appointment 持参資料、`CLAIM_STATUS_MATRIX.md` と並列
**Companion document**: [`CLAIM_STATUS_MATRIX.md`](CLAIM_STATUS_MATRIX.md)(claim landscape、本 list の technical anchor)
**Template**: [`_template_pre_reg.md`](../exploration/_template_pre_reg.md) §0 discipline 適用、第 2 test case(low-severity documentation work、F-099 後の運用継続)

---

## 0. Frozen task spec(本 list の scope を pre-empt)

### 0.1 Anchor constraint deep application(trigger 5 物理化)

- [x] **wall-clock / cost / scale 推定なし**(本 list は claim status matrix からの question 抽出のみ、estimation 不要)
- [x] **比較対象 finding と apples-to-apples**:F-060 paid v2 baseline + 6-pattern arc + Foreign baseline N=3 + Router design space frontier — 本 list は **既存 anchor からの question 抽出**、新規 measurement なし
- [x] **structural 前提 verify**:全 question は `CLAIM_STATUS_MATRIX.md` の §3-§5 に明示 anchored、独立 fabrication なし

### 0.2 Frozen scope

- [x] **Layer 1**(matrix-only から答えられる):弁理士 が `CLAIM_STATUS_MATRIX.md` 読んで feedback 可能な question 群
- [x] **Layer 2**(user-side input required):商用 path × 地理的市場 / budget / strategic preference 等、user 並行作業の output が必要な question 群
- [x] **excluded scope**:
  - 各 question への answer 提案(弁理士 judgment 領域)
  - 商用 path × 地理的市場 mapping 自体(user-side parallel work、本 doc に含まない)
  - PCT 出願費用 quotation(弁理士 input 領域)

### 0.3 Observation vs interpretation 分離

- [x] §2 / §3 の各 question は **observation form**(matrix data + 既存 finding cite)
- [x] interpretation(answer 候補 / strategic preference)は **書かない**、弁理士 / user の判断領域
- [x] **責任 framing 過度集約 警戒**(2026-04-29 self-application lesson 第 3 例):各 question は弁理士 judgment 領域、AI 側で 結論 inject しない

### 0.4 Segment split

- [x] 私 segment:question list drafting + matrix cross-reference verify
- [x] user segment:Layer 2 questions の user-side input(商用 path / 地理的市場 / budget)+ 弁理士 appointment + consult execution

### 0.5 完了 check

- [x] §1-§3 全 question fill 完了
- [x] §4 cross-references(matrix + filed PDFs + memory)完備
- [x] §0.1-§0.4 全 checkbox completed

---

## 1. 一行 summary

> **本 list は `CLAIM_STATUS_MATRIX.md`(50 Claim × 2 軸 tag、6-pattern arc、Foreign baseline N=3、Router design space frontier)を 弁理士 consult 用 question form に変換。Layer 1(matrix-only から答えられる、~17 questions)+ Layer 2(user-side parallel work output 必要、~10 questions)の 2 層 構造。Layer 1 を弁理士に提示、Layer 2 を user 側で並行整理して consult 着手。Answer 候補は本 doc に含まない(弁理士 / user judgment 領域)。**

---

## 2. Layer 1 — matrix-only から弁理士 が feedback 可能な questions

### A. Test backing strategy(U sub-split に基づく claim 階層化)

`CLAIM_STATUS_MATRIX.md` §3 に基づき、U(unit only)19 件を **trivial 15 + non-trivial 4** に split 済:

- **A1**:**U-trivial 15 件**(Claim 2-4, 6-9, 11-13, 15, 34-35, 49-50)— F-040 per-claim unit test backing で「trivial 認定 OK」と PCT 審査で扱われる可能性は?各 Claim ごとに realistic test を要請される risk と、preferred embodiment 格下げで対応可能な範囲を判定願いたい。
- **A2**:**U-non-trivial 4 件**:
  - Claim 26(意味的重要度):F-040 unit のみ、application context での realistic test 価値は?(関連する F-085 Obsidian / F-068 analogy で間接 covered の可能性 evaluate)
  - Claim 42-43(Rare → Core 昇格条件 / trigger):F-040 unit のみ、Claim 23 transition controller(F-027 + F-071 realistic backed)で間接 covered と扱える可能性?
  - Claim 45(0.40:0.35:0.25 specific value):F-040 unit のみ、Claim 44 7:2:1 重み(F-068 realistic backed)と pair で realistic test 設計可能、費用対効果評価?
- **A3**:U-non-trivial 4 件で realistic test を新規設計する場合、PCT 出願 (priority date 確保)前に間に合わせる必要性 vs continuation 出願で対応する trade-off?

### B. Application narrowing(R-N 13 件)の wording strategy

`CLAIM_STATUS_MATRIX.md` §4.1-§4.2 narrowing impact summary に基づく questions:

- **B1**:**Claim 10 α=2 canonical**(F-091 で α=2 NASA-recurring-rare specific、Apache では α=4 optimal):
  - PCT 出願時に specific value α=2 を preferred embodiment 格下げ + claim 範囲を α ∈ [0.5, 4.0] range 等に拡張する戦略は feasible?
  - JP 出願は既に α=2 canonical で claim 構成、PCT で範囲拡張すると JP との整合 / continuation 出願の必要性は?
- **B2**:**Claim 14 streaming benefit**(F-087 で one-shot rare narrow、F-094/F-097 で recurring rare cross-domain N=3 durable):
  - application precondition「temporally recurring structural rareness」を claim 本文に明示する wording is feasible?(現 claim は precondition 暗黙)
  - one-shot rare scope を explicit に exclude する形で書くべきか、それとも positive direction(recurring rare durable)のみ強調?
- **B3**:**Claim 25 activation**(F-097 sanity で BGL alphabet 8 templates のみで discrimination room なし、one-shot disposal は web log family specific):
  - precondition「rich resource alphabet」を application context として明示すべきか?(具体 alphabet size は別途定義)
  - 本 narrowing は F-097 sanity inconsistent に基づく structural caveat、PCT 出願時に開示すべきか defensible 範囲か?
- **B4**:**Claim 31 緊急介入**(F-092 adversarial で functional rare protection 完全崩壊、boundedness + recovery PASS):
  - precondition「non-adversarial input distribution」claim 本文明示、production deploy で rate limiting 等 defense layer 併用必須を embodiment 記述に追加 strategy?
  - F-092 の adversarial test 結果を pre-emptive disclosure として spec に書く戦略 vs claim 範囲を non-adversarial に narrow する戦略の trade-off?
- **B5**:**Claim 36-41 + 47-48 sandwich canonical (θ_L, θ_U)**(F-070 で 4-benchmark 横断反証、specific (0.70, 0.80) で 100% demote):
  - specific θ value を実施例(preferred embodiment)に格下げ + claim 本文は θ ∈ [0, 1] range のみ、feasible?
  - F-070 の 4-benchmark 反証(F-041 Hopfield + F-068 + F-070 Part A/B)は pre-emptive disclosure として PCT spec に書くべき(prior art 防御強化)/ claim narrow に止める どちらが strategic?

### C. Strategic alternatives + Cross-domain durability anchor activation

`CLAIM_STATUS_MATRIX.md` §4.3 cross-domain durability anchor + §5.2-§5.3 + §5.4 に基づく questions:

- **C1**:**狭い基本 claim + 広い続編 claim 階層化 strategy** が PCT phase で feasible?
  - 例:Claim 14 streaming benefit を「recurring rare 軸」narrow に固定(基本 claim)、cross-domain durability(F-094 web + F-097 HW)を続編 claim or continuation 出願 で広げる戦略
  - 同様に Claim 10 α-tunable を 基本 claim、α=2 specific を embodiment、α-domain calibration を 続編 claim、の階層化
- **C2**:**Pre-emptive disclosure of narrowing**(prior art 防御):
  - 4 narrow finding(F-070 / F-087 / F-091 / F-092)+ F-097 sanity inconsistent を PCT spec の "experimental verification" / "limitations" section に **pre-emptively disclose**、prior art rejection を防ぐ strategy が PCT 慣行に整合か?
  - 公開 GitHub commit + paper v2.6 で既に開示済、PCT spec で再 organize する場合の Best practice は?
- **C3**:**6-pattern arc(4 narrow + 2 positive)** を PCT credibility narrative としてどう活用?
  - "honesty-first stance" を PCT spec に明示する場合、出願 strategy として一般的か unusual か
  - F-094/F-097 cross-domain N=3 positive replication(NASA + Apache + BGL)を strongest evidence cluster として spec の "advantages over prior art" に integrate する presentation 戦略
- **C4**:**Foreign baseline anchor (Mem0 family N=3)** + **Infrastructure honest record (F-095/F-096)** + **Router design space frontier (F-099)** の PCT spec integration 戦略:
  - F-048 / F-053 / F-060 を "background of the invention" に組み込む際、setting-dependent position(local KDF / paid Mem0 alone / hybrid Router strictly better)を どう disclose するか
  - F-095/F-096 inconclusive infrastructure record は PCT spec に **書くべき / 不要**?
  - F-099 Router design space frontier(v1 paid hurt -11/-13pt、v2 length component necessity anchored)は PCT spec で どう activate?

---

## 3. Layer 2 — user-side input required(consult appointment 前に user 側並行整理)

`CLAIM_STATUS_MATRIX.md` §4.4 PCT cost-benefit observation flag に基づく user-side parallel work:

### D. 商用 path × 地理的市場 mapping(必須 input)

- **D1**:4 grounded products の **主要市場 priority**:
  - Obsidian plugin:US / EU / JP / その他?(Obsidian 本体は international community、user base distribution が anchor)
  - MovieLens niche genre surfacing:research demonstrator 主、商用化対象市場は?
  - Mem0 temporal hybrid Router(v2):LLM memory 市場、US / EU dominant?
  - Git Core preservation:OSS infrastructure、global/US dominant?
- **D2**:主要市場が **日本のみ → JP 出願で十分**、**US/EU 必要 → PCT 価値あり**(数百万円費用回収可能性)、user の判断 input?
- **D3**:特定国(China / Korea / India 等)の優先度?

### E. Budget + timeline(必須 input)

- **E1**:**PCT 出願費用 budget**(本体出願 + 国別 entry phase 数百万円〜)、user の優先度 / available budget?
- **E2**:国別 entry phase 費用見積 + total budget 想定(US, EU, JP それぞれ別費用)?
- **E3**:**商用化 timeline**:
  - short-term licensing(1-2 年内)— PCT 30 month 期限内に licensing 確定可能?
  - long-term commercial deploy(3-5 年)— PCT 30 month 期限超過 risk、国別 entry timing
  - PCT 出願 → 30 month → 国別 entry の 3 stage 費用配分との整合
- **E4**:**研究 + 商用化 + IP 維持** の同時並行 budget split、user の優先度?

### F. Strategic preferences(用 user judgment、consult 前準備)

- **F1**:**公開 vs 防衛**(disclosure vs trade secret)の preference?
  - 既に GitHub public + paper v2.6 で広範 disclosure 済、trade secret path は不可(observation 観点)
  - PCT 出願 + active patent enforcement の commitment level
- **F2**:**licensing-first**(ロイヤリティ収入)vs **commercial-deploy-first**(自社製品開発)の preference?
- **F3**:**Risk tolerance**:
  - PCT 出願 + 国別 entry で 5-7 年後 market disrupt されている risk
  - 出願なしで infringement 防御不可能 risk
  - どちらを許容?
- **F4**:**defensive patent**(自社防衛のみ)vs **offensive patent**(licensing / litigation 含む)の strategic stance?

### G. user-side preparation checklist(consult 着手前に揃えるべき)

弁理士 appointment 前に user-side で揃える成果物 list:

- [ ] D1-D3 の 商用 path × 地理的市場 mapping(1 page sheet 推奨)
- [ ] E1-E4 の budget / timeline 整理(数値 + scenario)
- [ ] F1-F4 の strategic preference 言語化
- [ ] consult agenda(本 doc Layer 1 + 2 を 弁理士 と review、~2-3 時間想定)

---

## 4. Cross-references

### Companion document

- [`CLAIM_STATUS_MATRIX.md`](CLAIM_STATUS_MATRIX.md)(claim landscape、本 list の technical anchor)

### Master spec(FROZEN)

- [`docs/patent/filed/`](../patent/filed/) 5 PDF
- [`docs/patent/COMPLIANCE.md`](../patent/COMPLIANCE.md)
- [`docs/patent/TRACEABILITY.md`](../patent/TRACEABILITY.md)
- [`docs/patent/SPEC.md`](../patent/SPEC.md) §3.1

### Empirical findings(F-xxx anchor)

- [`docs/VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md) F-001 〜 F-099(本 list で参照する全 F-xxx の trace)
- [`docs/PHASE_2_RETROSPECTIVE.md`](../PHASE_2_RETROSPECTIVE.md) §0-§15、4-category epistemic structure

### Memory operational protocols(私 segment 整合)

- `feedback_decision_framework`(誠実性優先)
- `feedback_observation_vs_interpretation`(本 list で適用、特に §0.3 責任 framing 過度集約警戒)
- `feedback_recommendation_boundary`(私 constraint vs user preference 分離、本 list は constraint side)
- `feedback_post_hoc_narrowing`(question list scope 後変更禁止)
- `feedback_tool_execution_verbal_claim_separation`(全 question matrix 直 anchor、独立 fabrication なし)

### Template

- [`_template_pre_reg.md`](../exploration/_template_pre_reg.md)(本 doc は第 2 test case、low-severity documentation work で運用継続、severity 高い test は F-098 candidate pending)

---

## 5. 完了 check

- [x] §0 frozen task spec 完了
- [x] Layer 1 = 17 questions(A:3, B:5, C:4)
- [x] Layer 2 = 10 questions + 1 checklist(D:3, E:4, F:4 + G checklist)
- [x] 全 question が `CLAIM_STATUS_MATRIX.md` の §3-§5 に明示 anchored
- [x] interpretation / answer 候補は **本 doc に含まない**(弁理士 / user judgment 領域、`feedback_observation_vs_interpretation` 適用)
- [x] cross-references 完備

**Deliverable status**:PCT consult question list 完成、`CLAIM_STATUS_MATRIX.md` と pair で 弁理士 appointment 持参可能。Layer 2 の user-side parallel work が consult 着手前に必須(§3 G checklist)。post-consult feedback で claim wording strategy paper / minimal-test design は別 finding(Axis 2 full)で対応。
