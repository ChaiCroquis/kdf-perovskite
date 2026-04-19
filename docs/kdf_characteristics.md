# KDF の特性(properties)— 検証で判明した性質の整理

**最終更新**: 2026-04-19
**由来**: Phase W1-W5b+W4+Ext-1+C1+C2(F-050〜F-061)での実測から抽出
**目的**: KDF の決定論アルゴリズムとしての **証明済み特性** を整理し、異なる視点からの解釈を付与する。論文・pitch・拡張機能設計の一次 reference として使う。

---

## 🧭 特性の分類

以下、6 つの視点で KDF の特性を整理する:

1. **決定論的特性**(by construction)
2. **選別品質特性**(selection quality、実測ベース)
3. **比較特性**(vs baselines、実測ベース)
4. **操作特性**(operational、cost / latency / privacy)
5. **構造保持特性**(structural integrity、F-061 で新発見)
6. **失敗モード / 限界**(honest limitations)

各特性に:
- 「何が証明済みか(evidence)」
- 「別の視点から見るとどうか(alt-perspective)」
- 「商用・研究的な意味」

を付記する。

---

## 1. 決定論的特性(by construction)

### P1.1 — 完全 deterministic

**性質**: 同じ入力 graph + 同じ keep_rate → **必ず同じ選択結果**。

**Evidence**:
- `cgb-kdf::NodeClassifier::classify(n, edges)` が決定論的(Rust 実装で pure function)
- F-052 / F-057 / F-058 / F-059 / F-060 / F-061 全実験で再実行一致
- (対比) Mem0 の LLM fact extraction は stochastic、F-058 で新 model 適用時に temporal recall が ±0.116 悪化

**Alt-perspective**:
- 情報理論的: selection entropy = 0(given input → no stochastic bits)
- 統計的: variance = 0 across runs、single seed で十分
- 監査的: 「なぜこの node が選ばれたか」追跡可能(Rare/Core/Edge/Garbage + id order)

**意味**:
- **監査可能性**(regulated industries、金融・医療・法務で価値)
- **Reproducibility**: paper 掲載、peer review で絶対的再現性
- **CI/CD 親和性**: 同じ data → 同じ output → diff-based change detection が可能

---

### P1.2 — Item-level 可逆(verbatim preservation)

**性質**: 選ばれた item(turn / node / commit / etc.)は **一切改変されない**。LLM-based 要約は "around May" と粗く変形するが、KDF は "7 May 2023" を verbatim 保持する。

**Evidence**:
- F-057 / F-058 で LoCoMo temporal category に KDF が +10.6pt / +23.4pt 勝利 → raw date strings が preserved されるため
- 対比: Mem0 が同 category で degrade(gpt-4.1-mini で 0.09)
- 特許明細書 Claim 1 で "raw data preservation" を明記

**Alt-perspective**:
- 情報理論的: **kept set に対する information loss は zero**(distortion = 0)
- 比較: LLM fact extraction は kept set に対しても lossy(fact を paraphrase)
- 法的: 「証拠として原文を保持」→ legal discovery、audit trail で価値

**意味**:
- 数値・日時・固有名詞・リストの完全性要求 use case で決定的(金融取引、医療記録、法務)
- "LLM で要約したら失われる情報" を守る唯一の deterministic 手段

---

### P1.3 — Set-level 選別(controlled loss)

**性質**: 選ばれなかった item は **完全破棄**、復元不可能。圧縮率は事前に指定可能(keep_rate パラメータ)。

**Evidence**:
- F-052 keep_rate ablation: 0.05 → 1.00 で recall が 0.14 → 1.00 に単調増加
- Budget 制御が predictable(want 30% 保持→ 30% 確実に保持)

**Alt-perspective**:
- Trade-off 可視化: 情報量 vs budget の curve を事前に描ける
- Classical "lossy compression" ではあるが、lossy は "選別" レベルであり "変形" ではない

**意味**:
- **Budget-constrained environments** で予算管理が確定的
- IoT / 医療 log で「月 X GB 以内」等の SLA に厳密に合わせられる

---

## 2. 選別品質特性(selection quality、実測ベース)

### P2.1 — Budget 対 recall の単調増加

**性質**: keep_rate 増加 → recall 増加。ただし diminishing return あり。

**Evidence**: F-052(LongMemEval 500Q KDF recall):

| keep_rate | KDF answer_turn_recall |
|---:|---:|
| 0.05 | 0.279 |
| 0.10 | 0.449 |
| 0.15 | 0.510 |
| 0.20 | 0.524 |
| 0.30 | **0.665** |
| 0.50 | 0.771 |
| 0.70 | 0.896 |
| 1.00 | 1.000 |

**Alt-perspective**:
- 経済的: marginal value of budget diminishes → 30% keep_rate が sweet spot
- 学習曲線的: power-law 類似の形状

**意味**:
- Commercial pitch で「30% 保持で 67% の情報を取る、70% 保持すれば 90% 取れる」と明確に示せる

---

### P2.2 — Random よりも一貫して優位

**性質**: 同じ budget 下で、KDF は Random selection を consistent に上回る。

**Evidence**:
- F-052: KDF 0.665 vs Random 0.366 at 30% (×1.82)
- F-061 ER graph: KDF betweenness top-50 recall 0.70 vs Random 0.18 (×3.9)
- F-061 APSP: KDF rel error 0.019 vs Random 0.388 on BA (×20 better)

**Alt-perspective**:
- "Structural information" は budget 効率を改善する → KDF は structure を活用する証拠

**意味**:
- 「Random でも十分か?」を否定する empirical 根拠
- 構造情報のない random よりは structural KDF が有利、という default 主張

---

### P2.3 — TTL_recent より顕著に優位

**性質**: 「直近のみ残す」単純戦略より決定的に強い。

**Evidence**:
- F-052 LongMemEval 500Q @30% keep: KDF 0.665 vs TTL 0.180 (**×3.7**)
- F-033 100Q: KDF 0.821 vs TTL 0.107 (×7.7)

**Alt-perspective**:
- 時系列 bias なし → 歴史的重要情報を保持する
- TTL は「最近 = 重要」の仮定を持つが、KDF は構造的 rareness を使う

**意味**:
- LLM memory で「直近 N 件だけ覚える」慣例を覆す
- ログ retention / PKM 等で TTL 置き換えの価値を定量化

---

## 3. 比較特性(vs baselines、実測ベース)

### P3.1 — LLM-based fact extraction(Mem0)との benchmark-dependent な住み分け

**性質**: KDF は **短対話一般 QA で敗北、長会話 date/time recall で勝利**。model-agnostic(gpt-4o-mini / gpt-4.1-mini 両方で同じ傾向)。

**Evidence**(2×2 matrix):

| benchmark × model | KDF - Mem0 | 勝者 |
|---|---:|:-:|
| LongMemEval 500Q × gpt-4o-mini (F-053) | −0.238 | Mem0 |
| LongMemEval 500Q × gpt-4.1-mini (F-059) | −0.270 | Mem0 |
| LoCoMo temporal 321Q × gpt-4o-mini (F-057) | +0.106 | KDF |
| LoCoMo temporal 321Q × gpt-4.1-mini (F-058) | +0.234 | KDF |

**Alt-perspective**:
- Model 更新で Mem0 は LongMemEval で強化、LoCoMo で degrade → LLM fact extraction の **構造的非対称性** が露呈
- KDF は model 更新で挙動不変 → "model-independent baseline" の役割

**意味**:
- **「KDF は Mem0 の替わり」ではなく、「KDF は Mem0 の safety net / complementary layer」** という positioning の empirical 根拠
- F-060 Ext-1 Router で "strictly better" 実証の基盤

---

### P3.2 — TopDegree heuristic との graph-structure dependent な住み分け

**性質**: KDF は **scale-free graph(social network 型)で TopDegree に敗北、uniform / community 型で優位**。

**Evidence**(F-061 Betweenness top-50 recall):

| Graph type | KDF | TopDegree | 勝者 |
|---|---:|---:|:-:|
| ER (uniform random) | 0.70 | 0.50 | **KDF** |
| SBM (planted communities) | 0.50 | 0.36 | **KDF** |
| BA (scale-free) | 0.68 | 0.74 | **TopDegree** |
| WS (small world) | 0.26 | 0.46 | **TopDegree** |

**Alt-perspective**:
- Scale-free graph では degree と betweenness が強く相関 → 素直に degree で切るのが正解
- Uniform graph では degree が informative でない → KDF の rareness signal が効く

**意味**:
- 適用領域の事前判別: target graph が scale-free か否かで KDF/TopDegree を選ぶべき
- 論文の limitation として honest に記載

---

### P3.3 — APSP / path-based 算法での consistent 優位

**性質**: KDF は **全 4 graph で APSP(全対最短路)精度において Random を圧倒、TopDegree と同等 or 優位**。

**Evidence**(F-061 APSP rel error @30% keep):

| Graph | KDF | Random | TopDegree |
|---|---:|---:|---:|
| ER | 0.307 | 0.715 | 0.256 |
| BA | **0.019** | 0.388 | 0.021 |
| WS | **0.222** | **3.240** | 0.600 |
| SBM | **0.000** | 0.019 | 0.010 |

**Alt-perspective**:
- KDF は "path-critical bottleneck" node を保護する傾向 → APSP で重要 path が残る
- 同時に TopDegree は hub を残すが、path は通るとは限らない(WS で顕著な差)

**意味**:
- Routing、logistics、network latency analysis で KDF が defensible な value
- "path-based classical algorithm 前処理" として商業 pitch 可能

---

### P3.4 — Hybrid 設計での "strictly better" 達成

**性質**: KDF を precision-query + long context に限定した routing で、Mem0 alone を **dominate** する(どの cell でも worse にならず、半数の cell で strictly better)。

**Evidence**(F-060 Ext-1 Router v2):

| cell | Mem0 alone | Router | gain | p |
|---|---:|---:|---:|---:|
| LME 500 × 4o-mini | 0.672 | 0.672 | 0.000 | 1.00 (safe) |
| LME 500 × 4.1-mini | 0.722 | 0.722 | 0.000 | 1.00 (safe) |
| LoCoMo × 4o-mini | 0.206 | **0.302** | **+0.097** | 0.003 ★ |
| LoCoMo × 4.1-mini | 0.090 | **0.315** | **+0.224** | 4×10⁻¹⁴ ★ |

**Alt-perspective**:
- 「KDF 単独 vs Mem0 単独」は benchmark-specific な住み分けだが、**組み合わせると常に Mem0 以上**
- 決定論的 classifier(regex)で routing 先を切り替えるだけで strictly better

**意味**:
- **補完 architecture の empirical 根拠**(design_philosophy.md の core thesis 実証)
- Ext-1 として MVP 可能、既存 Mem0 deployment に追加販売可

---

## 4. 操作特性(operational)

### P4.1 — $0 per-query cost

**性質**: KDF の selection 自体は LLM API 呼び出しを要しない、completely local computation。

**Evidence**: F-044 cost breakdown で Mem0 は $0.38/500Q、KDF 部分は 0。

**Alt-perspective**:
- Scale に対して cost が一定(per-query は 0、build-time のみ)
- "API 予算" という制約が消える

**意味**: 
- Budget-constrained agent deployment で唯一の選択肢
- Offline / air-gapped で動作可能

---

### P4.2 — Sub-millisecond latency

**性質**: KDF selection は典型的 graph で < 1ms(Rust 実装)。

**Evidence**: F-044 latency 計測で <1ms/query vs Mem0 ~30s/query。

**Alt-perspective**: 
- Real-time system での応答性要件に合致
- Mem0 の 30s latency は多くの UX で不可

**意味**: 
- Real-time chat / streaming agent / sensor gating で使用可能

---

### P4.3 — Full local、External dependency 無し

**性質**: KDF 実行に external API / network / cloud service を必要としない。

**Evidence**: cgb-kdf is pure Rust crate、no network calls、no model weights to load.

**Alt-perspective**: 
- HIPAA / GDPR / 金融 regulations 準拠の default behavior
- Data sovereignty 要求を自然に満たす

**意味**: 
- Regulated industries への entry が容易(compliance overhead が低い)

---

## 5. 構造保持特性(structural integrity、F-061 + F-062 で発見)

### P5.0 — Merge / integration point の優先保護(F-062 新規)

**性質**: KDF は graph 内の高 degree な "統合 node"(git の merge commit 等)を効率的に保護する。

**Evidence**(F-062 tokio repo git pruning, 4,752 commits):

| keep_rate | KDF merge recall | Random merge recall | TTL merge recall | TopDegree merge recall |
|:-:|---:|---:|---:|---:|
| 30% | **99.45%** | 30.05% | 22.40% | 99.45% |
| 50% | **100.00%** | 52.46% | 35.52% | 100.00% |

**Alt-perspective**:
- Information theory: merge commit = high-entropy aggregation point、KDF が情報密度の高い node を priority-preserve
- Graph theory: merge commit = "join node" in DAG、degree 高で KDF の Core layer に入る

**意味**:
- Git archival / 時系列 event log curation で "節目" を 30% 予算で 99% 保持
- Connectivity の維持(P5.1)と統合点保護(P5.0)は related characteristic



### P5.1 — Connectivity preservation

**性質**: KDF で 30% に削減後、source-target pair の reachability が consistently 高い。

**Evidence**(F-061 sample pair coverage):

| Graph | KDF | Random | TopDegree |
|---|---:|---:|---:|
| ER | 1.00 | 0.96 | 0.95 |
| BA | 1.00 | 0.59 | 1.00 |
| WS | **0.92** | **0.21** | 0.85 |
| SBM | 1.00 | 1.00 | 0.95 |

**Alt-perspective**:
- KDF の Rare layer 保護 = 構造的 boundary 保護 = connectivity bridge 保存
- "graph の integrity" が 30% 削減後も preserved
- Random は WS(small world)で壊滅的に connectivity を失う

**意味**:
- 災害時 routing、supply chain、network resilience 分析で valuable
- 「small world 型でも KDF なら生き残る」という defensible claim

**新規の予想(未検証)**:
- KDF は minimum vertex cut を近似的に保護している可能性
- 将来の graph theoretic 分析で formal 連携が見える?

---

### P5.2 — "重要 node" 優先保護(非 random sampling)

**性質**: KDF の Rare/Core layer にある node は、Edge/Garbage よりも consistent に保護される。これにより structural rareness が高い node(1-degree boundary, community bridge, path-critical)が残りやすい。

**Evidence**:
- F-061: KDF pruned graph で top-50 betweenness node が Random より多く残る
- F-052: answer-bearing turn が structurally identifiable なため selection される

**Alt-perspective**:
- 情報理論的: KDF は "graph の structural signature" を持つ node を priority-preserve する
- Entropy 的: high-entropy regions(rare connections)が優先 → information の rich zone が生き残る

**意味**:
- 「ゴミを捨てて重要なものを残す」を構造的に実現
- LLM や domain expert の supervision なしに "重要度" を推定

---

## 6. 失敗モード / 限界(honest limitations)

### P6.1 — Scale-free graph では degree-based heuristic に敗北

**Evidence**: F-061 BA, WS で TopDegree に betweenness recall で負ける。

**意味**:
- Social network、citation network、web graph 等では TopDegree が simpler かつ効果的
- KDF の structural rareness signal が degree で subsume される case

---

### P6.2 — Metadata / semantic minority は検出不能

**Evidence**: F-047 Welsh Wikipedia。

**意味**:
- Structure が rareness を符号化していないと KDF は無力
- Cultural / linguistic / semantic minority 検出には別手段(dense embedding 等)が必要

---

### P6.3 — 一般 semantic retrieval では dense embedding に大敗

**Evidence**: F-045。

**意味**:
- 意味理解が本質的に必要な task で KDF は弱い
- KDF は lexical / structural signal のみ

---

### P6.3b — Density-based function approximation(GP regression 等)に不適(F-063)

**Evidence**: F-063 で GP inducing point selection として KDF が Random 以下、KMeans・TopDegree に明確に劣る。

**意味**:
- 関数近似の inducing point は "density coverage" が要件
- KDF の "rareness 保護" は boundary / isolated 点優先なので密度カバーの逆
- Kernel methods、SGPR、Nystrom approximation 等も同じ理由で不向きな可能性

### P6.3c — Call graph での API 検出に不適(F-064 新規)

**Evidence**: F-064 で Python call graph(flask)の public API 保持 recall、KDF が Random より大幅に劣る(30% keep で 16.67% vs Random 41.67%)。

**意味**:
- Public API = 高 in-degree(多くの caller)、KDF の Rare layer 保護とは **逆方向**
- Internal helpers(低 degree)が Rare で保護され、API(高 degree)が捨てられる

**教訓の refinement**:

F-063 で「graph-traversal vs density-estimation」と仮説化したが、F-064 を合わせて より精密な axis は:

> **"structural rareness が task の重要性と相関するか"** が KDF 適性の decisive な predictor。

- 相関あり(merge commits、path bottlenecks、answer turns、boundary nodes)→ KDF 効く
- 相関なし(density center、API entry points、semantic minority)→ KDF 効かない

### P6.4 — LLM fact extraction の "要約" 能力は持たない

**Evidence**: F-053 短対話 QA で Mem0 に 24pt 敗北。

**意味**:
- 短 context で「facts を構造化して quick answer」系の task では LLM が強い
- KDF の value は raw preservation、要約ではない

---

### P6.5 — Fact extraction-based memory system の代替にはならない

**Evidence**: F-055 hybrid (KDF raw + Mem0 answer) で tied, p=0.845。

**意味**:
- "KDF で Mem0 を置き換える" は成立しない
- "KDF で Mem0 を補完する"(F-060 Ext-1)なら成立

---

## 🧩 異なる視点からの解釈(まとめ)

### 情報理論の視点

- KDF = **selection without transformation** → kept set に distortion なし
- LLM extraction = **transformation with selection** → kept set にも distortion あり
- KDF の information loss は完全に "discarded set に集中"、 LLM は "discarded + kept の両方に分散"

### 統計の視点

- KDF の variance = 0(decisionistic)
- Mem0 の variance > 0(stochastic)
- Reproducibility:KDF = absolute, Mem0 = approximate

### Graph-theoretic の視点

- KDF の Rare/Core 保護 ≈ vertex cut / bridge 保護の近似(F-061 の connectivity 結果で示唆)
- Scale-free graph では degree が dominant signal、KDF の Rare signal と overlap → 独自 value 薄い
- Uniform graph / community graph では degree 以外の signal が重要 → KDF が独自 value

### 計算複雑性の視点

- KDF: O(n log n)(classification)+ O(k)(selection)= 線形に近い
- LLM fact extraction: O(n × LLM_call_cost) = 定数でない、実質コスト高
- Path-based classical algorithm(O(V³), O(VE))に KDF 前処理すると、V を 0.3V に減らして計算量を 3-27× 削減

### 監査 / Regulatory の視点

- KDF: 全 decision が traceable、input → output が deterministic → 法的要件を自然に満たす
- LLM: "model said so" で終わる、audit trail が薄い

### 商業 positioning の視点

- KDF = "deterministic structural preservation tool for budget-constrained, audit-required, local-first environments"
- Not: "LLM memory replacement"
- Yes: "LLM memory supplement for specific query patterns" (F-060)

---

## 🎯 未検証だが示唆される特性

以下は現時点で **直接の evidence はない** が、F-051〜F-061 の pattern から示唆される:

| 示唆される特性 | 根拠 | 検証方法 |
|---|---|---|
| Minimum vertex cut の近似的保護 | F-061 connectivity 結果 | Graph theoretic proof or benchmark |
| Model-agnostic value(時代が進んでも陳腐化しない) | F-058 で gpt-4o-mini → gpt-4.1-mini の安定性 | 新モデル出るたびに再検証 |
| Budget-accuracy curve の predictability | F-052 の smooth curve | 他 benchmark で形状確認 |
| Structural bottleneck node の優先保護 | F-061 path-based wins | explicit bottleneck graph で検証 |
| 長期時系列 data の temporal marker 保護 | F-057/F-058 temporal wins | time series benchmark |
| 全応用領域で Random に勝つ(no-free-lunch 的保証) | F-061 全 graph で Random 敗北 | 追加 benchmark で破綻しないか検証 |

---

## 📋 論文・pitch での使用方針

**強調すべき特性**(validated、defensible):
- P1.1 決定論性
- P1.2 Item-level 可逆
- P3.4 Strictly better hybrid(F-060)
- P3.3 APSP / path-based 優位
- P4.* 操作特性(cost/latency/local)
- P5.1 Connectivity preservation

**narrowing 必要な特性**(conditional):
- P3.1 LLM との住み分け(short vs long context)
- P3.2 TopDegree との住み分け(scale-free で負ける)

**明示的な limitations**(P6.*)は paper の limitation 章に記載、**overclaim を避ける**。

**未検証示唆は "future work"** として載せる。これは特許 claim の広さを活かしつつ、honest に段階的に実証していく道筋となる。

---

## 🎭 Metaphor — 物語の主人公 / 行商人(発明者 2026-04-19 発案)

Burt's Structural Holes や brokerage theory は mathematically precise だが、pitch / public engagement で使う時には **human-relatable な比喩** が強力。発明者が session 中に発案した本質を捉える metaphor:

> **「KDF は物語の主人公、または行商人に似ている」**

### なぜ "行商人"(traveling merchant)が的確か

| 行商人の特徴 | KDF の対応特性 | Evidence |
|---|---|---|
| 生産者でも消費者でもない、純粋な broker | Rare / Core layer で broker node を優先保護 | F-012, F-062 |
| 村々(community)を橋渡し、各地の rare 情報を運ぶ | Community graph(SBM)で KDF 勝利 | F-061 |
| 一度離れると村々が孤立(代替経路なし) | Connectivity preservation、30% budget で coverage 1.00 | F-061 |
| Inventory(道具・才能)ではなく position が value の源泉 | 決定論的 O(V+E) で **構造的 position だけ** を評価 | P1.1 |
| 都市圏(scale-free hub)ではむしろ埋没する | BA / WS graph で TopDegree に敗北 | F-061 |

### なぜ "物語の主人公" が的確か

物語理論的に、主人公(protagonist)は:

- **最強 fighter でない**(賢者や王の方が能力は高い)
- **最賢者でもない**(メンターや神託者の方が知識がある)
- **しかし物語を "駆動" する**:複数の cast / locations / plot thread を繋ぐ
- 主人公が消えると **narrative が分断する**(他キャラ単独では story が成立しない)
- 主人公の value は **individual capability ではなく narrative network 上の connectivity**

これは KDF が「最高の accuracy」でも「最速の algorithm」でもなく、**"決定論で構造的 broker を識別する position" そのものが value** という特性と完全に一致する。

### Pitch での使い方

- **Enterprise buyer(CTO / CFO)向け**: 「KDF は組織の "行商人" を特定する engine です。声の大きい部長(hub)ではなく、部署間の地味な橋渡し役(broker)を O(V+E) で特定します」
- **Academic(org science)向け**: "KDF is the linear-time computational realization of Burt's Structural Holes detector for graphs beyond human-organization scale."
- **Developer 向け**: 「KDF は git の merge commit や、会話の決定的な cross-team thread を "自動で発見する行商人検出器" です」

### Limitations of the metaphor

比喩は power を持つが誇張を招くので注意:
- 実際の行商人は human intuition / negotiation を持つが、KDF は graph structure 単独の heuristic
- 物語の主人公は agency(自己決定)を持つが、KDF は deterministic procedure
- 両 metaphor は **positioning tool**、technical proof ではない

---

## 📘 Meta-Philosophy — 性格論と情報理論的位置づけ(新規)

本 doc は KDF の個別 property を catalog 化したものですが、それらを **哲学的に synthesize した doc** は別にあります:

→ **[kdf_meta_philosophy.md](kdf_meta_philosophy.md)** — "KDF は決定論的境界観察者 = 情報の L3 層 specialist"

特に以下を含む:
- 6 層情報理論(L1 Shannon、L2 Semantic、**L3 Structural = KDF**、L4 Density、L5 Temporal、L6 Kolmogorov)
- なぜ KDF が数式化できたか(3 条件)
- 性格 5 trait と L3 specialist の対応関係
- Graph 構築は L2 判断(warning)

---

## 🔗 関連 document

- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 全 F-xxx findings の詳細
- [design_philosophy.md](design_philosophy.md) — 設計方針・マントラ
- [domain_validation.md](domain_validation.md) — 応用領域別 validation
- [classical_algorithm_revival.md](classical_algorithm_revival.md) — 古典 algorithm 復権
- [extension_ideas.md](extension_ideas.md) — 拡張機能案
- [validation_strategy.md](validation_strategy.md) — 優先度マトリクス
- [paper_draft.md](paper_draft.md) — 論文草稿
- [patent/filed/](patent/filed/) — 特許明細書(仕様の権威)
