# Phase 2 Retrospective — KDF の現在地(2026-04-29)

**期間カバレッジ**: F-073 〜 F-090(Phase 2 全体 + Phase 2.5 streaming replication)
**Phase 1 末時点の anchor**: F-072(NASA HTTP streaming +3.06pt)
**作成目的**: 公開後 reader が VERIFIED_FINDINGS.md の 70+ 件を全部読まなくても、KDF の **現在の正味 position** を一読で把握できるようにする

---

## 0. 一行でいうと

> **Phase 1 の "broad applicability" 仮説は Phase 2 で empirical に narrow され、現在の KDF は「4 つの structural-niche 製品 + domain-fit predictor」という narrow but durable 形に収束した。**

「狭くなった」と「弱くなった」は別。Phase 2 を経て **どこで効くか / どこで効かないか** が事前判別できる framework が手に入ったので、商用 deploy の確実性は **上がった**。失った主張のうち、研究者として最も誠実に向き合うべきは ① F-072 streaming benefit の汎用性、② bias-detector 商材 path の 2 件。

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
| **最新 baseline 比較** | 未測定 | Letta / Zep / GraphRAG / Anthropic memory tool との位置関係 |

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
2. PUBLIC_SUMMARY.md の P7 bias-detector 項目を「F-090 で撤回」マーカーに変更
3. F-088 / F-089 streaming(別 sprint で proper rare 定義設計)
4. Preprocessor thesis empirical demo(collaborator 確保時)
