# Patent Claim 1-50 Status Matrix(Axis 2 minimal、PCT consult prep)

**Date**: 2026-04-29
**Scope**: PCT consult 持参資料、claim status の **2 軸 tag table**(test backing × mechanism/application)。**minimal-test design 抜き**(post-consult feedback 反映後に別 finding で追加予定)。
**Status**: Axis 2 minimal、scope 確定優先 path α 経由
**作成**: F-094 + F-097 + F-099 完走後 timestamp、6-pattern arc(4 narrow + 2 positive)+ Foreign baseline N=3 + Infrastructure honest record + Router design space frontier 完成時点

---

## 0. Frozen task spec(本 document の scope を pre-empt)

### 0.1 Scope frozen(post-hoc 拡張禁止)

- [x] **対象**: cgb-kdf 全 50 Claim(Claim 1-50)、kdf-lib subset(ADR-0001 Rev.10 Basic)は scope 外
- [x] **2 軸 tag**:
  - **Test backing level**:`U` = unit only(F-040)/ `R` = realistic backed / `R-narrow` = realistic backed but application narrowed by F-xxx finding
  - **Mechanism/Application**:`M` = claim mechanism 自体(broad) / `A` = claim の specific application instance / `M+A` = both
- [x] **excluded scope**(本 document に含まない):
  - minimal-test design(unit-only claim を realistic に格上げするための test 設計)
  - PCT 出願時の claim 範囲修正提案
  - 弁理士 への質問 list(別 doc)
- [x] **post-consult expansion candidate**(別 finding):
  - minimal-test design for unit-only claims
  - claim wording narrow proposals based on consult feedback
  - PCT vs JP 出願 strategic分岐の弁理士 input integration

### 0.2 Observation vs interpretation 分離

- [x] §2 master matrix は **observation form**(claim status from existing artifacts、F-xxx anchor 引用)
- [x] §4 narrowing impact + §5 gaps は **interpretation form**(observation を 弁理士-readable に再構成、narrative 修正 不可)
- [x] gap inventory に「test 設計提案」を含めない(scope frozen per §0.1)

### 0.3 Segment split

- [x] 私 segment:matrix drafting + cross-reference verify + commit
- [x] user segment:弁理士 consult question list 作成 + consult appointment + scope 確定後の minimal-test design priority 判断

### 0.4 Document scope 完了 check

- [x] §1-§6 全 fill 完了
- [x] §0.1-§0.3 全 checkbox completed
- [x] master matrix 50/50 entries(空 row なし)

---

## 1. 一行 summary(弁理士 5 分把握用)

> **cgb-kdf 全 50 Claim は F-040 で per-claim unit test backed(基底)、うち主要 18 Claim(Claim 1, 5, 10, 14, 16-19, 20-22, 23-26, 27-32, 33, 36-41, 44, 46, 47-48)が realistic benchmark backed。Application narrowing 5 件(F-070 sandwich canonical / F-087 streaming one-shot / F-091 α=2 NASA-specific / F-092 Claim 31 functional non-adversarial / F-097 Claim 25 BGL alphabet caveat)、Cross-domain positive replication 2 件(F-094/F-097 recurring-rare N=3、Claim 14 streaming benefit を web log + HW kernel log family に anchor)。Mechanism は 全 Claim 不変 supported、narrow されたのは specific application instances のみ。**

---

## 2. Master matrix(Claim 1-50、2 軸 tag)

**Legend**:
- **Test**: `U` unit only / `R` realistic backed / `R-N` realistic backed but specific application narrowed
- **M/A**: `M` mechanism / `A` application / `M+A` both
- **F-xxx**: empirical anchor(realistic backed のみ列挙、unit-only は F-040 共通基底)
- ✓ supported / ⚠️ narrowed / ❌ refuted / — N/A

| Claim | 内容(短縮)| Test | M/A | F-xxx anchor | Application narrow | Notes |
|:---:|---|:---:|:---:|---|---|---|
| **1** | 整合性発見手段(3 柱:代謝 / 希少 / 整合性)| **R** | M+A | F-068 + F-052 + F-012 | — | broad umbrella、3 柱全 backed |
| 2 | basic data structure(node) | U | M | F-040 | — | structural primitive |
| 3 | basic data structure(edge) | U | M | F-040 | — | structural primitive |
| 4 | classifier interface | U | M | F-040 | — | structural primitive |
| **5** | 時間評価成分 | **R-N** | M+A | F-040 + F-069 | F-069 で **static task では冗長**(streaming benefit context) | 機構 ✓ / static 応用 ❌、streaming で価値(Claim 14 family)|
| 6 | 減衰関数 family(λ) | U | M | F-002 analytic | — | math foundation |
| 7 | 局所混雑度 C = deg(u)+deg(v)| U | M | F-040 | — | basic formula |
| 8 | β スケーリング | U | M | F-040 | — | basic parameter |
| 9 | γ 比例係数 | U | M | F-040 | — | basic parameter |
| **10** | **α=2 canonical(発明の核心)**| **R-N** | M+A | F-037 + **F-091** | F-091 で **α=2 は NASA-recurring-rare specific**、Apache では α=4 optimal | 機構 ✓ / canonical value 反証(domain-dependent)|
| 11 | 確率剪定(probabilistic prune) | U | M | F-007 proptest | — | statistical foundation |
| 12 | exp 減衰確率関数 | U | M | F-040 | — | math form |
| 13 | exp 化(Phase 1)| U | M | F-040 | — | implementation form |
| **14** | **指数減衰 w←w·exp(-λ·dt)(streaming benefit anchor)**| **R-N + R+** | M+A | F-002 + F-069 + F-072 + F-087 + F-091 + **F-094 + F-097** | F-087 で **one-shot rare では hurt**、F-094/F-097 で **recurring rare cross-domain N=3 durable**(web + HW family)| 機構 ✓ / streaming application narrow but positive cross-family。**6-pattern arc primary anchor** |
| 15 | bit-exact determinism | U | M | F-005 determinism | — | reproducibility primitive |
| **16** | Rare 保護(orphan preservation)| **R** | M+A | F-012 Obsidian + F-085 Task 3 | — | Obsidian product anchor、F-085 で realistic application supported |
| **17** | 分散実行(local==global bit-exact)| **R** | M | F-037 + F-069 LoCoMo(max diff 0.0)| — | strongly backed、production-grade distributed |
| 18 | Rare 維持(retention rule)| R | A | F-012 + F-040 | — | application of Claim 16 |
| 19 | Rare 維持(promotion path)| R | A | F-012 + F-040 | — | application of Claim 16 |
| **20** | 階層領域 5:3:1(integer tick)| **R** | M | F-071 LoCoMo | — | integer tick 正確 verified |
| **21** | tick_period() ratio | **R** | M | F-071 | — | mechanism backed |
| **22** | RegionConfig kind | **R** | M | F-071 | — | mechanism backed |
| **23** | TransitionController(promotion)| **R** | M | F-027 Mode E rescue + F-071 | — | mechanism + synthetic rescue |
| **24** | TransitionScore | **R** | M | F-027 + F-071 | — | mechanism backed |
| **25** | activation(temporal recurrence sense)| **R-N** | M+A | F-097 BGL recurring-rare + F-072 NASA + F-094 Apache | F-097 sanity:**BGL alphabet 小(8 templates)で discrimination room なし**、recurring rare で help だが one-shot disposal は web log family specific | 機構 ✓ / one-shot disposal application は web log family specific(F-097 sanity inconsistent narrow)|
| 26 | 意味的重要度(semantic importance)| U | M | F-040 | — | unit-only(realistic test 候補)|
| **27** | meta α 適応(2 方向更新)| **R** | M | F-004 proptest 16× + F-027 + F-071 | — | mechanism backed |
| **28** | meta bound clamp | **R** | M | F-071 bound clamp | — | mechanism backed |
| **29** | δk⁴ tracking | **R** | M | F-004 + F-071 | — | mechanism backed |
| **30** | meta error correction | **R** | M | F-071 | — | mechanism backed |
| **31** | **緊急介入(emergency intervention)**| **R-N** | M+A | F-092 adversarial perturbation | F-092 で **functional rare protection が adversarial degree inflation で完全崩壊**(boundedness + recovery PASS、functional FAIL)| 機構 ✓(controller stability)/ application narrow:non-adversarial settings、production deploy で rate limiting 等 defense layer 必要 |
| **32** | meta robust bound | **R** | M | F-071 + F-092 | — | mechanism backed |
| **33** | 複合孤立度指標 | **R** | M+A | F-024 D6 + F-037 | — | composite metric supported |
| 34 | data format(serialization)| U | M | F-040 | — | unit-only(format primitive)|
| 35 | data format(persistence)| U | M | F-040 | — | unit-only(persistence primitive)|
| **36-41** | **二段階審査 T_wait(canonical sandwich)**| **R-N** | M+A | F-040 unit + F-070 LoCoMo Part B | F-070 で **canonical (θ_L, θ_U) で 100% demote**(canonical value 反証)| 機構 ✓(T_wait period 動作)/ canonical value 反証(specific θ で 機構 ineffective)|
| 42 | Rare → Core 昇格条件 | U | A | F-040 | — | unit-only(promotion rule)|
| 43 | Rare → Core 昇格 trigger | U | A | F-040 | — | unit-only(promotion rule)|
| **44** | 7:2:1 重み | **R** | A | F-040 + F-068 | — | specific value backed by realistic |
| 45 | 0.40:0.35:0.25 合成 | U | A | F-040 | — | unit-only(specific value、realistic test 候補)|
| **46** | 32-dim fingerprint | **R** | M+A | F-040 + F-068 | — | dimensional realistic backed |
| **47-48** | **sandwich θ_L/θ_U(canonical 0.70/0.80)**| **R-N** | M+A | F-040 + F-041 Hopfield + F-068 + F-070 Part A/B(4-benchmark cross)| F-041 + F-068 + F-070 で **canonical (0.70, 0.80) 4-benchmark 横断反証**| 機構 ✓ / canonical value 反証(non-trivial 4-benchmark cross-domain refutation)|
| 49 | library entry | U | M | F-040 | — | API surface |
| 50 | program form(deployment)| U | M | F-040 | — | deployment form |

---

## 3. Statistics

### Test backing distribution

| Test level | count | claim numbers |
|---|---:|---|
| **U** unit only | **22** | 2-4, 6-9, 11-13, 15, 26, 34-35, 42-43, 45, 49-50 |
| **R** realistic backed(narrow なし)| **15** | 1, 16-19(=4)、20-24(=5)、27-30(=4)、32, 33, 44, 46 |
| **R-N** realistic backed but application narrowed | **13** | 5, 10, 14, 25, 31, 36-41(=6)、47-48(=2) |
| **計** | **50** | (うち主要 28 Claim が realistic backed = 28/50 = 56%)|

### Mechanism / Application split

| M/A | count | description |
|---|---:|---|
| **M only** | **22** | data structures、math foundations、controller mechanisms(Claim 2-4, 6-9, 11-13, 15, 17, 20-24, 26-30, 32, 34-35, 49-50)|
| **A only** | **5** | application-specific(Claim 18-19, 42-43, 44, 45)|
| **M+A** | **23** | umbrella + application instances(Claim 1, 5, 10, 14, 16, 25, 31, 33, 36-41, 46, 47-48)|

### Application narrowing(F-xxx → claim)

| F-xxx | direction | 影響 claim | narrow type |
|---|:---:|---|---|
| **F-070** | narrow | Claim 36-41(T_wait sandwich)+ Claim 47-48(sandwich θ)| canonical value(specific (θ_L, θ_U) 4-benchmark 反証)|
| **F-087** | narrow | Claim 14(streaming benefit)| application(one-shot rare で hurt)|
| **F-091** | narrow | Claim 10(α=2 canonical) | canonical value(NASA-recurring-rare specific)|
| **F-092** | narrow | Claim 31(緊急介入) | application(adversarial で functional FAIL)|
| **F-097 sanity** | narrow | Claim 25(activation)| application(one-shot disposal は web log family specific、HW kernel log alphabet 小で discrimination なし)|

### Cross-domain positive replication(F-xxx → claim)

| F-xxx | direction | 影響 claim | scope |
|---|:---:|---|---|
| **F-094** | positive | Claim 14(streaming benefit、recurring rare)| Apache web log、cross-domain N=2 |
| **F-097** | positive | Claim 14 + Claim 25(streaming + activation 連携) | BGL HW kernel log、cross-domain N=3(NASA + Apache + BGL)|

---

## 4. Recent narrowing impact summary(2026-04-29 時点)

### 4.1 Canonical value 反証(機構 不変、specific value 反証)

- **Claim 10 α=2.0**:F-091 で α_core sweep 結果、α=2 は NASA-recurring-rare で optimal、Apache では α=4.0 optimal(diff -17.39pt)。**機構(α-tunable decay)は不変、canonical α=2 は domain-universal でない**。PCT 出願時の wording 検討候補:「α は domain-specific calibration」追加 or「α=2 は preferred embodiment」格下げ。
- **Claim 36-41 + Claim 47-48 canonical sandwich (θ_L, θ_U) = (0.70, 0.80)**:F-041 Hopfield + F-068 analogy + F-070 LoCoMo Part A/B で 4-benchmark 横断反証(canonical で 100% demote / sandwich 機構 ineffective)。**機構(二段階審査 + sandwich)は不変、canonical (0.70, 0.80) value は invalid**。PCT 出願時の wording 検討候補:specific θ value を「実施例」に格下げ、claim は θ ∈ [0, 1] 範囲のみ。

### 4.2 Application robustness narrow(機構 不変、specific application narrow)

- **Claim 14 streaming benefit**:F-087 Apache one-shot で hurt(-13.04pt)、F-091 で α-domain dependency、F-094/F-097 で recurring rare cross-domain N=3 durable。**機構 不変、application は recurring rare 限定**。PCT 出願時の wording 検討候補:「temporally recurring structural rareness」を application precondition として明示、one-shot rare scope は exclude。
- **Claim 25 activation**:F-097 sanity で BGL alphabet 8 templates のみで one-shot disposal mechanism 不発(top-30% selection が naturally one-shot 含む)。**機構 不変、one-shot disposal application は rich resource alphabet specific**(web log family)。PCT 出願時の wording 検討候補:application precondition「resource alphabet ≥ N templates」(具体 N 値は別途定義)。
- **Claim 31 緊急介入**:F-092 で adversarial degree inflation で functional rare protection 完全崩壊(recall 0.000 / 0.4592)、boundedness + recovery は PASS。**機構(controller stability)不変、functional protection は non-adversarial settings 限定**。PCT 出願時の wording 検討候補:「non-adversarial input distribution」を application precondition として明示、production deploy では rate limiting / provenance filter 等の defense layer 併用必須。

### 4.3 Cross-domain durability anchor(機構 + application 両 supported)

- **Claim 14 + Claim 25 streaming benefit、recurring rare 軸**:F-072 NASA HTTP +3.06pt + F-094 Apache recurring +3.67pt + F-097 BGL recurring +33.33pt、cross-domain N=3 で durable(web access log family + HW kernel log family の両 family across)。**機構 + application 両 supported、PCT 出願時の strongest evidence cluster**。

---

## 5. Gaps inventory(PCT consult discussion items、minimal-test design は本 doc scope 外)

### 5.1 Unit-only Claims(realistic test 未実施、22 件)

以下の Claim は F-040 per-claim unit test backed のみ、realistic benchmark 未実施。**Trivial 認定可能か / realistic test 設計が必要か は 弁理士 judgment**:

| Claim | 内容 | trivial 認定可能性(私の constraint-derived 推定)|
|:---:|---|---|
| 2-4 | basic data structures | trivial(structural primitive、unit test で十分)|
| 6-9 | 減衰関数 family、parameter math | trivial(math foundation、analytic verification F-002 で部分 covered)|
| 11-13 | 確率剪定 + exp 化 | trivial(statistical primitive、F-007 proptest で部分 covered)|
| 15 | bit-exact | trivial(determinism primitive、F-005 unit で十分)|
| 26 | 意味的重要度 | **non-trivial?**(application context で realistic test 価値あり、弁理士 confirm 候補)|
| 34-35 | data format | trivial(serialization primitive)|
| 42-43 | Rare → Core 昇格 rule | **non-trivial?**(promotion mechanism は Claim 23 と関連、realistic backed の Claim 23 で間接 covered の可能性、弁理士 confirm 候補)|
| 45 | 0.40:0.35:0.25 specific value | **non-trivial**(specific value、Claim 44 7:2:1 と pair で realistic test 候補)|
| 49-50 | library entry / program form | trivial(API/deployment primitive)|

→ **弁理士 consult discussion items**:non-trivial 候補(Claim 26, 42-43, 45)で realistic test 設計の費用対効果評価。post-consult feedback 反映後、別 finding(Axis 2 full)で minimal-test design。

### 5.2 Application narrow Claims の PCT wording strategy(13 件)

R-N tag claim(Claim 5, 10, 14, 25, 31, 36-41, 47-48)について、PCT 出願時の claim wording 修正 strategy を 弁理士 と discuss:

- (i) **specific value を実施例に格下げ**:Claim 10 α=2 / Claim 36-41 + 47-48 canonical sandwich (0.70, 0.80)など、specific value は preferred embodiment、claim は range
- (ii) **application precondition を claim に明示追加**:Claim 14 「temporally recurring structural rareness」、Claim 25 「rich resource alphabet」、Claim 31 「non-adversarial input distribution」
- (iii) **狭い claim + 広い続編 claim の階層化**:basic claim を機構 only に narrow、broader claim は specific application の continuation 出願

→ **弁理士 consult discussion items**:strategy (i)/(ii)/(iii) の どれが PCT phase で feasible か、JP 出願との整合性、prior art 防御の trade-off。

### 5.3 Cross-domain durability anchor の strongest evidence presentation(2 件)

F-094 / F-097 cross-domain N=3 positive replication(Claim 14 + Claim 25)を **strongest evidence cluster** として PCT 出願時の証拠提示順序を 弁理士 と discuss。

→ **弁理士 consult discussion items**:6-pattern arc(4 narrow + 2 positive)を PCT 出願 narrative の credibility anchor として活用する戦略、既存 narrowing の pre-emptive disclosure(prior art 防御)。

### 5.4 Foreign baseline anchor(Mem0 family N=3)

KDF vs Mem0 family の 3 cells(F-048 local proxy / F-053 paid alone / F-060 paid hybrid)+ infrastructure honest record(F-095/F-096 local replication inconclusive)+ Router design space frontier(F-099 v1 router、length component necessity anchored)を PCT 出願時の prior art / state of the art 整理として活用。

→ **弁理士 consult discussion items**:Foreign baseline N=3 + infrastructure honest record + Router design frontier を PCT specification の "background of the invention" / "advantages over prior art" に integrate する presentation strategy。

---

## 6. Cross-references

### 関連文書

- **Master spec(FROZEN)**:[`docs/patent/filed/`](../patent/filed/) 5 PDF(特許願 / 特許請求の範囲 / 明細書 / 要約書 / 図面)
- **Compliance report(Phase 1 完了時点)**:[`docs/patent/COMPLIANCE.md`](../patent/COMPLIANCE.md) Claim 1-50 詳細判定根拠
- **Traceability**:[`docs/patent/TRACEABILITY.md`](../patent/TRACEABILITY.md) Claim → 実装 module mapping
- **SPEC + 既知乖離**:[`docs/patent/SPEC.md`](../patent/SPEC.md) §3.1
- **Empirical findings 全件**:[`docs/VERIFIED_FINDINGS.md`](../VERIFIED_FINDINGS.md) F-001〜F-099(本 matrix の F-xxx anchor 全 trace 可能)
- **Phase 2 retrospective**:[`docs/PHASE_2_RETROSPECTIVE.md`](../PHASE_2_RETROSPECTIVE.md) §0-§15、6-pattern arc + 4-category epistemic structure
- **arxiv paper**:[`docs/arxiv_submission/paper.md`](../arxiv_submission/paper.md) v2.6 + Addendum changelog v2-v2.6

### F-xxx anchor cluster(本 matrix で参照)

- **Mechanism backed**:F-002, F-004, F-005, F-007, F-012, F-024, F-027, F-037, F-040, F-041, F-052, F-068, F-069, F-071, F-072
- **Application narrow**:F-070, F-087, F-091, F-092, F-097(sanity)
- **Cross-domain positive**:F-094, F-097
- **Foreign baseline / Router**:F-048, F-053, F-060, F-095, F-096, F-099

### post-consult expansion candidate(別 finding、本 doc scope 外)

- Axis 2 full:minimal-test design for non-trivial unit-only claims(Claim 26, 42-43, 45)
- Claim wording strategy paper(post-consult feedback 反映)
- 弁理士 question list(本 doc §5 inventory を質問 form に変換)
- F-098 stronger local LLM での H_R 再 test(F-095/F-096 inconclusive resolution)
- F-100+ candidate(post-PCT expansion)

---

## 7. 完了 check(本 doc scope vs deliverable)

- [x] §0 frozen task spec 完了(scope frozen、segment split、observation/interpretation 分離)
- [x] §2 master matrix 50/50 entries(空 row なし)
- [x] §3 statistics(test level + M/A + narrowing F-xxx + positive replication)
- [x] §4 narrowing impact summary(canonical value / application robustness / cross-domain durability の 3 category)
- [x] §5 gaps inventory(unit-only / R-N wording / cross-domain anchor presentation / foreign baseline)— **minimal-test design を含まない**(scope frozen)
- [x] §6 cross-references(filed PDFs + COMPLIANCE + TRACEABILITY + VERIFIED_FINDINGS + retrospective + paper)

**deliverable status**:Axis 2 minimal 完了、PCT consult 持参資料として standalone readable、弁理士 5 分 把握 format 達成。post-consult feedback で minimal-test design / claim wording strategy の 拡張は別 finding(Axis 2 full)で対応。
