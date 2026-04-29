# Phase 2 Retrospective — KDF の現在地(2026-04-29、F-091〜F-096 追記)

**期間カバレッジ**: F-073 〜 F-094(Phase 2 全体 + Phase 2.5 streaming replication + α/Lyapunov + anchor sharpening + cross-domain positive replication empirical)
**Phase 1 末時点の anchor**: F-072(NASA HTTP streaming +3.06pt)
**作成目的**: 公開後 reader が VERIFIED_FINDINGS.md の 70+ 件を全部読まなくても、KDF の **現在の正味 position** を一読で把握できるようにする

---

## 0. 一行でいうと

> **Phase 1 の "broad applicability" 仮説は Phase 2 で empirical に narrow され、現在の KDF は「4 つの structural-niche 製品 + domain-fit predictor + 4 narrow + 1 positive epistemic anchor + F-072 anchor の 3 軸高解像 specificity」という narrow but durable(durable 側は cross-domain 物理証拠あり)形に arc 完結した。**

「狭くなった」と「弱くなった」は別。Phase 2 を経て **どこで効くか / どこで効かないか** が事前判別できる framework が手に入り、4 件の self-refutation(F-070 sandwich / F-087 streaming / F-091 α=2 / F-092 Claim 31 functional)が paper の honesty-first stance を支える epistemic anchor として揃い、F-093 で F-072 anchor の真の dependency が 3 軸(domain / α / rare type)で sharpen され、**F-094 で recurring rare 軸が cross-domain N=2(NASA + Apache)で positive replication された**ので、商用 deploy の確実性は **上がった**。失った主張のうち、研究者として最も誠実に向き合うべきは ① F-072 streaming benefit の汎用性、② bias-detector 商材 path、③ Claim 10 / Claim 31 の universal claim form の 3 件。F-093 は別 category(narrowing でなく anchor 解像度向上)、F-094 は更に別 category(narrowing でなく durable 側の cross-domain 物理証拠 = positive direction)。

---

## 1. Phase 1 末時点の position(F-072 anchor)

[F-072](VERIFIED_FINDINGS.md) NASA HTTP streaming で Claim 14 decay が +3.06pt benefit を出した時点で、暗黙的に以下を期待していた:

- **Direct SOTA 勝負 path** が viable(structural rareness が widely 効く)
- **Streaming benefit** が広い log domain で再現する(NASA は representative example)
- **bias-detector** が KDF applicability の事前判別 tool として機能する(F-030 で 5/5 hit、F-074 BGL の 1 miss を含めて 7/8 = 87.5%)

これは **未検証の希望的観測** で、Phase 2 はこの 3 仮説を systematic に test する design だった。

---

## 2. Phase 2 で何を test したか

| Track | Test | F-xxx anchor |
|---|---|---|
| **A. Direct SOTA 勝負** | Wikipedia orphan / BGL anomaly / Citation interdisciplinary bridge の 3 task で KDF を baseline と比較 | F-073 / F-074 / F-075 |
| **B. Cross-domain replication** | F-072 streaming を別 log source(Apache / 候補 HPC / Linux)で再現 | F-087(Apache 完了、HPC/Linux deferred) |
| **C. bias-detector 系統的検証** | 既存 graph data 21 dataset で predict vs actual を集計 | F-090 |
| **D. Niche product replication** | F-082 MovieLens を 6 genre で再現 / Obsidian prototype / Mem0 hybrid / Git Core preservation の verification | F-082 / F-085 / F-086 α |
| **E. Domain-fit predictor refinement** | Hub-peripheral vs hub-biased / peer-network の境界線確定(meta-family sweep N=5) | F-086 γ |

---

## 3. 何が残ったか(empirically strong、Phase 2 後も影響なし)

| 製品候補 | Anchor | 限定条件 |
|---|---|---|
| **Obsidian plugin** | F-071 + F-085 Task 3 | personal note vault、orphan 保護(structural rareness 直接適用)。F1=1.0 synthetic 実証 |
| **MovieLens niche genre surfacing** | F-082 + F-085 Task 1(6 genres) | bipartite user-movie、rare genre = low-popularity content。γ-check 100% で 6 genre 全 PASS |
| **Mem0 temporal hybrid (Router v2)** | F-060 | 長期会話 date/time literal recall(+10〜23pt、2 model robust)。precision-query 分岐で strictly better |
| **Git Core preservation** | F-062 / F-077 / F-086 α | OSS-style repo、merge rate < 10%。**Rare layer でなく Core layer preservation product** として framework 正確化(F-086 α)|

加えて、**F-086 γ domain-fit predictor**(hub-peripheral / hub-biased / peer-network の事前判別 framework)は本質的に新しい知識資産として残った。これは bias-detector の sister tool ではなく、**γ-check correlation rate** という別 framework の predictor で、F-090 撤回の影響を受けない。

### Foreign baseline anchor (Mem0 family, N=3 cells、Phase 2 開始前に establish 済)

KDF vs Mem0 family の foreign baseline 比較は **N=3 empirical setting** で Phase 2 開始前(2026-04-18 〜 2026-04-19)に確立済:

| F-xxx | setting | result | implication |
|---|---|---|---|
| **F-048** | Local Mem0-style proxy(Qwen2.5-0.5B + BGE)、無料 | KDF 0.8210 vs Mem0-style 0.5083(**KDF +31.27pt**)| Local / budget / privacy 制約下では KDF retrieval が decisively 優位 |
| **F-053** | Paid Mem0 framework(gpt-4o-mini)、LongMemEval 500 Q | Mem0 0.672 vs KDF 0.434(**Mem0 +23.8pt**, p<10⁻¹⁶、Mem0 wins 5/6 categories)| Standard paid LLM-memory workload では Mem0 alone が強い |
| **F-060** | KDF + Mem0 hybrid Router(precision-query 分岐)、無料 post-hoc | **Router > Mem0 alone +10〜23pt LoCoMo、never worse on LongMemEval** | Hybrid deployment で **strictly better than Mem0 alone**(4 cells × 2 models 横断)|

→ KDF と Mem0 family の関係は **setting-dependent**:Local では KDF 優位、Paid では Mem0 alone が優位、Hybrid では KDF + Mem0 Router 戦略で strictly better。これは KDF を「Mem0 replacement」でなく「Mem0 complementary layer」として position する empirical 根拠で、F-060 Router implementation が validated commercial path。

**Cat 5 残り 5 baseline**(Letta / Zep / GraphRAG / Anthropic memory / OpenAI memory / LangChain)の cross-replication は **paid-budget multi-session sprint** に deferred。本 retrospective 時点で foreign baseline anchor は Mem0 family の N=3 cells で empirically anchored、PCT 判定 input としては「Mem0 family N=3 setting-dependent evidence + 残り 5 baseline deferred 状態」を組み合わせて読む。

---

## 4. 何が narrow になったか

### F-072 streaming benefit → "temporally recurring rare" 限定(F-087)

Apache error log で同 framework を再現した結果、**−13.04pt の逆向き** が出た。

| 環境 | rare の structural property | streaming benefit |
|---|---|---|
| NASA HTTP(F-072) | HTTP 4xx/5xx を返す resource。**同じ resource が時系列で何度も error し続ける**(persistent failure) | **+3.06pt**(decay が "最近 error した resource" を保持) |
| Apache error log(F-087) | freq ≤ 10 の resource path。**攻撃者が一度だけ probe して二度と来ない**(one-shot reconnaissance) | **−13.04pt**(decay が one-shot 信号を消す、activation が common path を上に押し上げて rare を弾き出す) |

→ paper §"streaming は真の use case" は **半分正しい**。streaming benefit は「rare が時系列上で recurring」な domain に specific、one-shot rare では actively harmful。

**商業 implication**: SOC realtime anomaly detection / SIEM 系 positioning は NASA-style status-coded recurring error log のみ candidate。一般 log streaming(syslog format / Apache error / Linux audit)は scope 外と見なすべき。

### Claim 10(α=2 「発明の核心」)→ NASA-recurring-rare specific(F-091)

α_core ∈ {0.5, 1.0, 2.0, 3.0, 4.0} sweep を NASA + Apache streaming + MovieLens null control で実行した結果:

| domain | best α | α=2.0 vs best | implication |
|---|:---:|---:|---|
| NASA HTTP(recurring rare) | **α=2.0** | diff 0.00pt | F-072 anchor +3.06pt 完全再現、α=2 は NASA で empirical 最適 |
| Apache error log(one-shot rare) | **α=4.0** | diff −17.39pt | aggressive decay で streaming が static を +4.35pt 上回る、α=2 は最適でない |
| MovieLens(static null) | (α 不感)| range 0.00pt | null control PASS、α は decay path 経由でのみ影響 |

→ Claim 10 機構は支持、ただし **canonical α=2.0 は domain-universal でない**。one-shot rare では α=4.0 が optimal、F-087 narrowing(streaming は recurring rare 限定)を **further refined**:streaming は α tuning で one-shot rare にも benefit 出せる可能性、ただし α は domain-specific calibration 必要。

### Claim 31(緊急介入 mechanism)→ 非 adversarial settings 限定(F-092)

NASA HTTP streaming に **window 50 で rare-target 1000 events** を adversarial burst として注入、controller stability を測定:

| metric | result | verdict |
|---|---|:---:|
| **boundedness** α_edge ∈ [1.0, 2.5] 全 100 window | 違反 0 | ✅ |
| **recovery** w55 で baseline ±0.3 内 | diff 0.0043 | ✅ |
| **functional** recall_perturbed / baseline ≥ 0.80 | 0.000 / 0.4592 | ❌ |

→ controller stability mechanism(α bound + adaptive recovery)は real-data 摂動下で **empirical 支持**。ただし adversarial degree inflation で natural rare resources が Rare → Core layer demote → top-30% selection から漏れる。**functional rare protection 主張は narrow**:production deploy では rate limiting / provenance filter 等の defense layer が必要。

---

## 4-pattern self-refutation 蓄積(本 retrospective の epistemic anchor)

F-087 + F-091 + F-092 の追加で、paper の "honesty-first" stance を支える self-refutation 例が **4 件 同型 pattern** で揃った:

| F-xxx | 機構 | specific value / application |
|---|:---:|:---:|
| F-070 | Sandwich 2-threshold ✓ | canonical (θ_L, θ_U) = (0.70, 0.80) refuted |
| F-087 | Streaming framework ✓ | "universal" claim narrowed to recurring rare |
| F-091 | Claim 10 power-law decay ✓ | canonical α=2.0 narrowed to NASA-specific |
| F-092 | Claim 31 controller stability ✓ | functional rare protection narrowed to non-adversarial |

すべて **mechanism supported / specific application robustness narrowed** という同型構造。これは KDF の発明 core(Claim 8-9-10 power-law decay、Claim 14 streaming benefit、Claim 31 emergency intervention、Claim 47-48 sandwich)が **機構として novel / 実装 verified** だが、**universal optimal value としては domain-conditional**、という整合的な scope statement。

### F-093: 別 category(anchor sharpening、self-refutation でない)

F-093 は self-refutation 4-pattern とは **質的に異なる発見**で、F-072 anchor の **真の dependency 解像度を 1 段深く露わにした**。表面的記述「rare = 4xx/5xx 8 codes」は実質的に「rare = 404-pattern driven」であり、NASA dataset に 5xx response が皆無、4xx subset は rare resource set が 404-only と完全一致。

- F-091 で domain narrowing(NASA-recurring-rare specific)
- F-093 で rare type narrowing(404-pattern driven)
- F-087 で context narrowing(recurring vs one-shot)

の 3 軸で F-072 anchor の真の generality が **2 軸 narrow + 1 軸 sharpening** で解像。V1/V2/V4 完全同値 (+3.06pt) は KDF が「rare codes 集合」でなく「rare resource pattern」を捉えている証拠で、機構 supported を data-driven 確証する形。

これは narrowing でなく **anchor の解像度向上**:paper §6.4 限界節の strengthening でなく §5 P11 row の caveat 強化に属し、外部 reader が anchor の真の scope を misread しない構造を作る。Sister でなく独立 category として記録。

### F-094: 第 3 category(positive replication、durable 側の cross-domain 物理証拠)

F-094 は self-refutation 4-pattern とも anchor sharpening(F-093)とも **質的に異なる発見**で、**durable 側に物理証拠を物理的に追加した**。F-072 anchor の真の axis(F-093 で structural reading により抽出した「recurring rare 構造」)を、Apache 同 dataset(F-087 と同 file、31,062 records)で **rare def を flip**(one-shot freq ≤ 10 → recurring freq ≥ 5)した時に benefit が再現するかを pre-reg + replication template で test:

| variant | rare def | n_rare | Δ (pt) | verdict |
|---|---|---:|---:|:---:|
| V_one-shot(F-087 reproduce sanity) | freq ≤ 10 | 23 | **−13.04**(F-087 と完全一致 ±0.0pt) | ✅ Sanity PASS |
| V_recurring(F-094 main) | freq ≥ 5 | 109 | **+3.67**(> +1.0pt threshold) | ✅ **PASS** |

**Cross-domain anchor table(post F-094)**:

| dataset | rare def | α | Δ_streaming (pt) | source |
|---|---|---:|---:|---|
| NASA HTTP | 4xx/5xx 8 codes(404-pattern driven by F-093)| 2.0 | **+3.06** | F-072 anchor |
| Apache | one-shot freq ≤ 10 | 2.0 | −13.04 | F-087 |
| Apache | one-shot freq ≤ 10 | 4.0 | +4.35(副産物 evidence)| F-091 |
| **Apache** | **recurring freq ≥ 5** | **2.0** | **+3.67** | **F-094 本** |

→ F-072 anchor の真の axis(recurring rare 構造)が **NASA + Apache の 2 dataset で empirical 支持**。narrative arc の structure:

- **4 narrowing patterns**(F-070 / F-087 / F-091 / F-092):mechanism ✓ / specific application robustness narrowed
- **1 anchor sharpening**(F-093):F-072 anchor の真の dependency 3 軸解像
- **1 positive replication**(F-094):durable 側 cross-domain 物理証拠

これら 6 finding(4 narrow + 1 sharpening + 1 positive)で arc 完結。Claim 14 streaming benefit の scope:**recurring rare 構造 × α=2.0 × NASA-Apache homotype log dataset で empirical durable**、log domains beyond access logs / one-shot rare / α 別値 / 別 anchor dataset 構造への generalization は future work。

これは narrowing arc が「scope が縮み続ける研究」と外部 reader に misread されるリスクへの **structural 答え**:durable 側に物理証拠が複数 dataset で backed されている narrative。Sister でなく **第 3 独立 category** として記録。pre-reg + self-replication template が誠実性 framework の operational 実装として偶然でなく設計通り機能した実証(memory `feedback_pre_reg_self_replication_template`)。

---

## 5. 何を撤回したか

| 撤回項目 | 撤回 anchor | 撤回内容 |
|---|---|---|
| **Direct SOTA 勝負 path** | F-073 / F-074 / F-075 | Phase 2 Top 3 で **3/3 LOSS**。Wikipedia orphan(−4.07pt)/ BGL anomaly(−12.92pt)/ Citation interdisciplinary(完敗 recall 0%)。"low degree ≠ important" の real-world task で逆向きに働くことが確定 |
| **Reddit community anomaly** | F-085 Task 4 | replication 失敗、製品 candidate 撤回 |
| **Academic citation meta-family** | F-086 β | peer-network structure(citing papers 自体も低 deg)で REJECT |
| **Biological hub-biased label** | F-084 | PPI cancer gene が hub-biased(TP53/BRCA1/MYC 等の signaling hub)、γ-check 74% < strong threshold |
| **bias-detector 87.5% predictor 商材化** | F-090 | N=21 systematic test で certain prediction accuracy **45.5%** ≪ 70% threshold。87.5% は初期 5 synthetic + 3 simple cases の sampling bias artifact |

bias-detector 撤回の structural reading: `bias_score = 0.3·I1 + 0.7·I4` の formula は **「rare items are deg=1」前提** で、実 data で rare items が moderate degree(2-10、e.g. HDFS template / MovieLens niche / Obsidian small graph)の case を systematic に miss する。商材化には features 拡張(bipartite ratio、hub-distance、structural betweenness 等)必須、しかしそれは **新 predictor の derive** であり F-090 を救う tweak ではない。

---

## 6. 現在の核領域(narrow but durable)

KDF の position を 3 つの否定形 + 3 つの肯定形 で記述する:

**NOT**:
1. ❌ Flagship task の SOTA-beating direct competitor(Phase 2 で empirical 確定)
2. ❌ Universal rare-event preservation tool(F-087 で one-shot rare には逆効果)
3. ❌ Universal applicability predictor の供給元(F-090 で bias-detector 撤回)

**IS**:
1. ✅ 4 件の **structural-holes intuition + temporal decay** が両立する niche product 群(Obsidian / MovieLens / Mem0 hybrid / Git Core)
2. ✅ Burt's Structural Holes(1992)と数学的整合する **brokerage 役 deterministic 実装**(audit-grade、Claim 15 bit-exact)
3. ✅ F-086 γ domain-fit predictor を持つ — 「hub-peripheral structure な domain」と「hub-biased / peer-network な domain」を事前定性判別可能、**deploy 確実性が上がった**

発明者(Chai)の比喩で言えば:**KDF は物語の主人公や行商人**。最強 fighter でも最賢者でもないが、分断された村々を繋ぐ structural broker。Phase 2 を経て、**どの村なら橋渡しの value を出せるか / どの村では出せないか** が事前に分かるようになった。

---

## 7. Open frontiers(未検証、parking lot)

| 候補 | 状態 | 期待 |
|---|---|---|
| **Preprocessor thesis** "Without KDF 0% → With KDF Y%" | collaborator 待ち(Slack/Discord 10 年 archive 必要) | 最大 narrative impact、direct SOTA 勝負 path 失敗後の natural reposition |
| **Lyapunov stability**(Claim 31 real-data) | コード実装済、empirical test 未着手 | 特許 Claim の empirical coverage 補完 |
| **Phase 1 deferred 6 candidates** | parked | Power grid / BGP / SO low-answer / Code silent pivot / Slack broker / EHR |
| **F-088 HPC / F-089 Linux streaming** | 別 sprint へ deferred | N=2 で narrowing 確定済、再 inventive 余地 low |
| **Cross-domain transfer**(Domain A 調整 → Domain B 評価) | 未測定 | 商業 deploy の transferability test |
| **Cat 5 残り 5 baseline 比較** | Mem0 family は F-048 / F-053 / F-060 で **N=3 anchored**(§3 Foreign baseline anchor 参照)、残り 5 baseline は paid-budget 必要で deferred | Letta / Zep / GraphRAG / Anthropic memory / OpenAI memory / LangChain との位置関係 |

---

## 8. 公開 documents との整合

本 retrospective は以下の公開文書と整合する形で位置づけられる:

- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — F-073〜F-090 の raw entry(本 retrospective が要約する詳細)
- [paper_draft.md](paper_draft.md) — §1.4 "Universality と novelty の緊張" で同 narrowing が paper 形式で表現済
- [arxiv_submission/paper.md](arxiv_submission/paper.md) — §F-072 paragraph(line 502-505)は **F-087 narrowing 反映が必要**、別 corrigendum / v2 upload で対応予定
- [PUBLIC_SUMMARY.md](PUBLIC_SUMMARY.md) — 2026-04-17 時点 snapshot、P7 bias-detector 項目は F-090 で撤回反映が必要(別 commit で patch)
- [kdf_characteristics.md](kdf_characteristics.md) — F-086 γ predictor を含む domain-fit framework は本 retrospective と整合
- [extension_ideas.md](extension_ideas.md) — Selection predictor 5 質問 self-check は F-086 γ の sister tool として残る

---

## 9. 今後の編集判断(maintenance log)

本 retrospective は **公開後の固定 snapshot ではなく**、Phase 3 以降の追加 finding によって最適化を続ける live document として運用する。重要な追加が出た場合は `## §N (date) <event>` 形式で末尾に追記し、§1〜§7 の本文は **追記時点での最良要約** に更新する(差分は git history で trace 可能)。

直近 next step 候補(優先度順):

1. arxiv_submission/paper.md §F-072 段落の F-087 narrowing 反映(corrigendum or v2)
2. PUBLIC_SUMMARY.md の P7 bias-detector 項目を「F-090 で撤回」マーカーに変更(完了)
3. F-088 / F-089 streaming(別 sprint で proper rare 定義設計)
4. Preprocessor thesis empirical demo(collaborator 確保時)

## §10(2026-04-29 追記)F-091 + F-092 反映後の追加 maintenance log

- F-091 Claim 10 (α=2) cross-domain robustness 結果を §4 に追加(NASA で robust、Apache で α=4 が optimal、canonical narrowing)
- F-092 Claim 31 Lyapunov real-data perturbation 結果を §4 に追加(controller mechanism robust、functional rare protection narrow)
- 4-pattern self-refutation(F-070 / F-087 / F-091 / F-092)を §4 末尾に新設、paper epistemic anchor として明示
- §0 「一行でいうと」を 4-pattern self-refutation 込みに更新
- paper.md v2 Addendum + §5 Phase 2/2.5 subsection に F-091/F-092 行追加(commit 別)、Zenodo paper v3 候補

## §11(2026-04-29 追記)F-093 反映後の追加 maintenance log

- F-093 NASA F-072 anchor robustness to rare code subset 結果を §4 末尾に「**anchor sharpening category**(self-refutation でない)」として追加
- F-072 anchor の真の dependency が 3 軸(F-087 domain / F-091 α / F-093 rare type)で sharpen された narrative を §0 「一行でいうと」に反映
- paper.md §5.1 P11 row に `[v2.1: F-093 で実質 404-pattern driven]` caveat 追加、v2 Addendum changelog に F-093 entry 追加(commit 別)、§5 Phase 2/2.5 table に F-093 row 追加(14 → 15 rows)、combined picture に "anchor sharpening category" 明記
- VERIFIED_FINDINGS tail を 2026-04-29 + F-073-F-093 form に更新
- memory project_kdf_phases.md に F-093 + anchor sharpening category 反映

## §12(2026-04-29 追記)F-094 反映後の追加 maintenance log

- F-094 Apache recurring-rare positive replication 結果を §4 末尾に「**positive replication category**(narrative arc の durable 側 cross-domain 物理証拠)」として追加(narrowing でも sharpening でもない第 3 category)
- §0「一行でいうと」を「4 narrow + 1 positive epistemic anchor」「durable 側 cross-domain 物理証拠あり」に拡張、F-094 を arc 完結 anchor として位置づけ
- paper.md v2.2:§5.1 P11 row に `[v2.2: F-094 で Apache 別 dataset を recurring rare 定義 (freq ≥ 5) で再 test、+3.67pt 再現、cross-domain N=2 で recurring-rare 軸が durable]` caveat 追加、v2 Addendum changelog に F-094 entry 追加(commit 別)、§5 Phase 2/2.5 table に F-094 row 追加(15 → 16 rows)、combined picture に "F-094 (v2.2) completes the arc on the positive direction" 明記、F-093 commit に紛れていた typo「실질 → 実質」も併せ修正
- VERIFIED_FINDINGS に F-094 entry 挿入(F-093 entry の前)+ 最終更新行 update(F-073〜F-094、5-pattern (4 narrow + 1 positive) anchor + anchor sharpening category)
- public sync の副次的 bug fix:VERIFIED_FINDINGS で「(旧) F-044 Mem0 entry + 第 33-35 部」が public 側で MIDDLE と END に重複していた状態を、dev-side の clean state で overwrite し duplicate 解消(意図せずだが結果として整合性回復)
- memory project_kdf_phases.md に F-094 + 5-pattern anchor 反映

## §13(2026-04-29 追記)F-095 / F-096 反映後の追加 maintenance log — Foreign baseline anchor の local replication 試行 = infrastructure honest record category

**新 category 確立**:**Infrastructure honest record category**(self-refutation でも anchor sharpening でも positive replication でもない第 4 category)。F-060(paid Mem0 + KDF Router で LoCoMo +9.7-22.4pt)を **完全 local 環境で replicate 試行** したが、test environment infrastructure 不備で **inconclusive** に終わった honest 記録。F-060 paid finding は本 chain で **refute されてもいない、support もされていない**。

**F-095(g7 attempt)— infrastructure-infeasible at 5Q stop**:

- 設定: llama3.1:8b-instruct-q4_K_M(Ollama)+ Mem0 framework default ingest batching(batch=4)+ HuggingFace BGE-small + Qdrant local
- 実行 5Q checkpoint で eta=803.9 min(13.4h LongMemEval のみ、合計 ~33-36h 連続実行)判明、infrastructure-infeasible で stop
- 原因: Mem0 framework `mem.add()` は per add 内部で 2 LLM call(fact extraction + ADD/UPDATE/DELETE/NONE 判定)、22-turn Q で 6 add × 2 = 12 LLM call/Q × 8B Q4 GPU per-call ~10s = ~3 min/Q。Pre-reg drafting で per-add LLM call multiplicity を grep / source-read せず undercount(参照した F-057 paid anchor の per-call latency ~100ms vs local 8B ~10s で 50-100x 違うことを caveat 化せず = anchor の表面引用)
- 私の Direction A 型失敗 4回目として記録、`feedback_tool_execution_verbal_claim_separation` memory に occurrence 4 + trigger 5 deep application(anchor の constraint = latency / call multiplicity を caveat 化)refinement 追記、本 occurrence 内で self-correct し F-096 へ移行

**F-096(g8 attempt)— qwen2.5:3b で wall-clock 圧縮、ただし sub-noise floor で inconclusive**:

- 設定変更: LLM swap to qwen2.5:3b-instruct-q4_K_M、ingest batching 最適化(LongMemEval は single mem.add(all_msgs) per Q、LoCoMo は batch_size=50 per conv)、wall-clock ~33h → **~2.5h に圧縮**、LongMemEval n=479 + LoCoMo n=321 完走
- H_R primary verdict: **mechanical PARTIAL**(Δ_router = +0.62pt、p=0.5)、ただし LoCoMo Mem0 alone = **0/321 正解**、KDF alone = 2/321 正解で **両者 sub-noise floor**(意味のある相対差を測れる土俵に達していない)
- Sanity 2 baseline shift: 全 cell で **|Δ| = 20〜64pt**(local 0.000-0.157 vs paid F-053/F-057 baseline 0.206-0.672)、frozen 5pt threshold 大幅超過、qwen2.5-3b が paid gpt-4o-mini 比で **answer-gen + judge 両方 threshold 以下**と判明
- F-060 paid finding は **intact**(本 chain は test environment infrastructure 不備、F-060 を refute せず)、「local-only Mem0 hybrid Router を第 5 grounded product 候補」claim は **PASS criterion 未達 + sub-noise floor で本 chain では立証 不可**
- 「local replication は **stronger local LLM (7B+) が必要**」が actionable conclusion、F-098 候補

**Descriptive contribution**(framework cross-LLM-size 不変性の anchor 化):

- `mem0_recall_substring`(turn 生 substring と retrieved Mem0 facts の overlap)が F-095 8B 5Q + F-096 3B 479Q **両方で 0.000-0.006** 測定
- F-048 旧解釈「**weak LLM が Mem0 風 retrieval を悪化**」は LLM size 効果を仮定していたが、F-095/F-096 で 8B / 3B どちらでも recall ~0 と判明 → **「Mem0 framework の batched fact compression 戦略そのものが 99%+ substring を保存しない」**(LLM size の問題でなく framework の問題)に sharpen
- F-048 hand-rolled per-turn extraction(1 turn → 1 fact、生 substring 保持)と Mem0 framework batched compression の **methodology 差** が apples-to-apples 不能と判明、F-095 で frozen していた H_LLM hypothesis(F-048 + 0.10 PASS)は ill-formed として撤回(F-096 では H_LLM dropped、recall は descriptive のみ)

**Exploratory observation**(F-097 候補):

- LongMemEval v1(precision-only、length filter 無)で Δ_router = **+6.26pt、p=1.86 × 10⁻⁹**(highly significant)観測。pre-reg primary は v2(precision + length≥100)のため LoCoMo cell が判定対象で、本 v1 観測は **exploratory として並記**
- 短 context での precision-query routing benefit を独立 pre-reg(F-097 候補)で frozen 化して formal finding にする path、既存データ流用で追加 wall-clock 不要

**反映 changes**:

- VERIFIED_FINDINGS に F-095(historical record)+ F-096(verdict)entries 挿入(F-094 entry の前)、最終更新行 update(F-073〜F-096、5-pattern (4 narrow + 1 positive) anchor + anchor sharpening category + **infrastructure honest record category**)
- paper.md v2.4:§5 Foreign baseline anchor section に "Attempted local replication of F-060 (F-095 / F-096)" paragraph 追加(F-060 intact 明記、stronger local LLM future work で frame、descriptive cross-LLM-size finding を F-048 caveat sharpen として記述)、Addendum changelog に v2.3(Foreign baseline anchor v2.3 retroactive log)+ v2.4(F-095/F-096)entries 追加
- 本 retrospective に §13 として infrastructure honest record category を establish、F-095/F-096 chain を arc に追加(narrowing / sharpening / positive / **honest infrastructure** の 4 category 体系)
- memory `feedback_tool_execution_verbal_claim_separation` に occurrence 4 + trigger 5 deep application refinement 追記(私の self-correction protocol の operational 強化)
- memory `project_kdf_phases.md` に F-095 / F-096 status + future work(F-097 / F-098 候補)+ remaining tasks 反映
- pre-reg + self-replication template が誠実性 framework の operational 実装として偶然でなく設計通り機能した実証として記録(F-087 sanity reproduce ±0.0pt は preprocessing/build env consistent の independent verification としても効いている、memory `feedback_pre_reg_self_replication_template` 適用)
