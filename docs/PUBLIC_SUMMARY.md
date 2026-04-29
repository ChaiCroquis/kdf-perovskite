# KDF: どの世界問題に、どこまで貢献できるか — 公開サマリ

**特許:** 2026-027032(日本、2026-02-24 出願)
**参照実装:** Rust (PolyForm Noncommercial 1.0.0)、[`crates/cgb-kdf`](../crates/cgb-kdf/)
**検証レポート:** [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md)(90 件超の検証済み知見、F-073〜F-090 で scope narrowing が確定)
**Phase 2 振り返り:** [PHASE_2_RETROSPECTIVE.md](PHASE_2_RETROSPECTIVE.md)(2026-04-29、現在の正味 position の単一文書要約)
**生成日:** 2026-04-17(本文)/ 2026-04-29 P7 撤回マーカー追加

---

## 1 行で言うと

> **KDF は「大半が冗長だが一部がユニークに重要で、どれが重要か事前に分からない関係網」から、ラベル不要で希少情報を構造的に保護するフレームワーク。3 つの実世界問題で業界標準を上回る実証がある一方、ほとんどの大問題(気候・貧困・戦争等)とは無関係であることも誠実に明示する。**

---

## エビデンス強度マトリクス

| # | 問題 | 層 | 検証状態 | KDF の数値 | 次のステップ |
|:-:|---|:-:|:-:|---|---|
| **P1** | [LLM エージェント持続的記憶喪失](world_problems/P1_llm_agent_memory.md) | 1 | ✅ **検証済** | TTL の **7.7 倍** Recall | 実装統合 |
| **P2** | [個人知識ベース(Obsidian)管理](world_problems/P2_personal_knowledge_base.md) | 1 | ✅ **検証済** | F1=0.747, p=0.006 | Obsidian plugin |
| **P3** | [大規模ログ観測コスト](world_problems/P3_log_observability.md) | 1 | ✅ **検証済(ニッチ)** | Random の **2.3 倍** | ラベル無し環境向け |
| **P4** | [AI 公平性 / 少数言語保全](world_problems/P4_ai_fairness_minority_language.md) | 2 | ❓ **未検証** | 類似実験で marginal 予測 | Wikipedia 多言語実験 |
| **P5** | [論文再発見 / 科学再現性](world_problems/P5_research_paper_rediscovery.md) | 2 | ❌ **検証失敗** | OpenAlex で KDF/Random = **×0.83**(D5 型予測的中)| 別 graph 表現で再試行は可能 |
| **P6** | [OSS メンテナンス](world_problems/P6_oss_maintenance.md) | 2 | ❌ **一般化失敗** | 3 repo 平均 **×1.00**(rust-lang のみ +15%、golang は -15%)| 一般主張を撤回 |
| **P7** | [ML 再現性危機(メタ)](world_problems/P7_ml_reproducibility.md) | 3 | ❌ **2026-04-29 撤回**(F-090) | N=21 systematic test で 45.5% < 70% threshold | `bias-detector` crate は code として残るが商材化 path は閉じた |
| **P8** | 記憶・忘却の形式化(学術) | 3 | ❓ **論文案** | Lyapunov 100k 実証 | arxiv preprint |

**適用外(out-of-scope として誠実に記録):**

- ❌ 気候変動: 物理・政策問題
- ❌ 貧困・不平等: 経済・社会制度
- ❌ 戦争・紛争: 政治・人間性
- ❌ 医療診断全般: 臨床専門知識 + データ
- ❌ 教育格差: P1/P2 経由で間接的のみ

---

## ✅ 検証済みの貢献(3 件)

### P1. LLM エージェント持続的記憶喪失

> **LongMemEval** (ICLR 2025 ベンチ, 500 questions)で、**KDF は LLM memory 業界 default の TTL (time-to-live) を 7.7 倍上回る** answer-turn recall を達成。

| Method | Recall |
|---|---:|
| TTL_recent (業界慣例) | 0.107 |
| Random | 0.294 |
| **KDF** | **0.821** (2.8× Random, 7.7× TTL) |

**インパクト:** 数億人のユーザを持つ AI エージェントの記憶挙動を業界標準より遥かに改善する余地。Anthropic memory tool / Mem0 / Letta への統合候補。

詳細: [P1_llm_agent_memory.md](world_problems/P1_llm_agent_memory.md)

---

### P2. 個人知識ベース自動キュレーション

> 発明者自身の **2,182 ノート Obsidian Vault**(PII masked)で、忘れていた重要ノートの **F1=0.747** 発見。Random / OrphanOnly / TextSim いずれも Wilcoxon p=0.006 で有意に上回る。

**インパクト:** Obsidian / Logseq / Roam の数千万 PKM ユーザの生産性。Obsidian plugin として配布可能。

詳細: [P2_personal_knowledge_base.md](world_problems/P2_personal_knowledge_base.md)

---

### P3. 観測ログの rare-error 保持

> **実 NASA HTTP log(50k records)**で、ラベル不要の KDF が Random の **2.3 倍** で 4xx/5xx エラーを保持。

**誠実な注記:** ラベル(status code)が取れる環境では Tail-based / Stratified sampling の方が 4 倍強い。**ニッチ利用限定**。

詳細: [P3_log_observability.md](world_problems/P3_log_observability.md)

---

### P7. ML 再現性危機 — bias-detector(副産物)— **❌ 2026-04-29 撤回**

> ~~KDF プロジェクトから副産物として生まれた `bias-detector` crate。**どんな graph benchmark が "synthetic data で手法に偏って有利" か**を**事前に検知**する zero-dependency tool。~~

> ~~**テスト結果:** 5 dataset 中 4 件で完全予測一致、1 件は別経路で一致(B_isolated)。~~

> ~~**インパクト:** KDF とは独立して ML 研究全般で利用可能。~~

**2026-04-29 撤回**(F-090): N=21 systematic test で certain prediction accuracy = **5/11 = 45.5%** ≪ 70% threshold。87.5% は初期 5 synthetic + 3 simple cases の sampling bias artifact。`bias-detector` crate は code として残るが、商材化 path は閉じた。詳細は [VERIFIED_FINDINGS.md F-090](VERIFIED_FINDINGS.md) と [PHASE_2_RETROSPECTIVE.md §5](PHASE_2_RETROSPECTIVE.md) 参照。

別 framework(F-086 γ domain-fit predictor、hub-peripheral / hub-biased / peer-network 判別)は独立に有効、撤回の影響を受けない。

---

## ⚠️ 部分検証・未検証(4 件)

| 問題 | 状態 | 何があれば検証可能か |
|---|---|---|
| P4 AI 公平性 | ❓ 未検証(pilot 成功、feasible)| Welsh + Wikidata QID pilot 成功(32% minority 特定)、追加 1-2 時間で full 実験 |
| P5 論文再発見 | ❌ 検証失敗(×0.83, D5 型) | 別 graph 表現(著者協力/参考文献時系列)での再試行は残余 |
| P6 OSS メンテナンス | ❌ 一般化失敗 | 3 repo 平均 ×1.00。rust-lang のみの局所 signal と判明 |
| P8 記憶形式化 | ❓ 学術論文案 | 1-2 ヶ月の執筆 + arxiv submission |

---

## 誇張を避けるための明確な宣言

**KDF では解決できない:**
- 気候変動 / 貧困 / 戦争 / 医療診断 / 教育格差
- 大規模な社会変革 / 倫理問題
- 創造性 / 感情理解 / 意識

**KDF で対応できるのは、ただ以下の構造:**
- データが**グラフ**として表現できる
- 大半が**冗長**で一部が**ユニーク**
- **ラベル取得が困難** or **不可能**
- **どれが重要か事前に分からない**
- **query は retention 時点で未知**(後続の検索で決まる)
- **one-off / 稀少言及の保護** が目的

**KDF は以下には効かない**(実証済):
- ❌ **Query-document matching 型の semantic retrieval**(F-045, SciFact で recall@10 = 0.000 = Random 以下)
- ❌ **独立文書 corpus の検索**(KDF の graph 構造が活かせない)
- ❌ 一般的な embedding-based retrieval の代替(dense embedding は圧倒的に強い)
- ❌ **Cultural/semantic minority の検出**(F-047, Welsh Wikipedia で KDF ×0.61 = Random 以下)
- ❌ **Cross-task 適用可否の bias-detector 自動判定**(F-046, LongMemEval で予測 MISS)

**市場 suitability 最終整理**(2026-04-18 F-053 retraction + F-056 LoCoMo nuance 後):
- 🚨 **F-044/F-049 の「KDF beats Mem0 on LongMemEval」は simulation artifact として撤回**(F-052/F-053)。LongMemEval 500Q では real KDF は Mem0 に overall −23.8 pt で敗北。
- ✨ **2 benchmark × 2 model matrix 完成 (2026-04-19) — benchmark-dependent な住み分けが model-agnostic に robust**:

| benchmark × model | Mem0 | KDF | gap | p | 勝者 |
|---|---:|---:|---:|---:|:-:|
| LongMemEval 500Q × gpt-4o-mini (F-053) | 0.672 | 0.434 | −0.238 | <10⁻¹⁶ | **Mem0** |
| LongMemEval 500Q × gpt-4.1-mini (F-059) | 0.722 | 0.452 | **−0.270** | 3×10⁻²³ | **Mem0** |
| LoCoMo temporal 321Q × gpt-4o-mini (F-057) | 0.206 | 0.312 | +0.106 | 1.4×10⁻³ | **KDF** |
| LoCoMo temporal 321Q × gpt-4.1-mini (F-058) | 0.090 | 0.324 | **+0.234** | 1.6×10⁻¹⁴ | **KDF** |

  → **長期会話(300-700 turns)の date/time recall は 2 model で KDF が robust 勝利**、短対話一般 QA は 2 model で Mem0 勝利。model 更新で傾向は強化される。
- ✨ **F-060 (2026-04-19): Ext-1 Precision-Query Router で "Mem0 + KDF > Mem0 alone" を実証**:
  - Routing logic: 「precision query + 長会話 (≥100 turns) なら KDF、それ以外 Mem0」
  - 短対話では router 発動せず(Mem0 alone と同一、害なし)
  - 長会話で precision query の場合、**最大 +22.4pt accuracy 改善、97% LLM API 削減**
  - Python 100 行の wrapper で既存 Mem0 deployment に統合可能、追加 API コスト 0
- ✅ **Tier 1 新(nuanced)**: KDF の defensible 用途:
  - **長期会話の temporal recall**(F-056 +22pt 実証、LoCoMo 型 benchmark):日付/時間情報が raw turn 内に散在
  - **cost/latency/privacy/determinism 必須領域**(LongMemEval 型の QA accuracy 不要 or 妥協可):local-first chatbot, air-gapped agent, real-time memory gating(<1ms), budget-constrained, deterministic
  - **Retrieval pre-filter**(TTL_recent より 3.7× 賢い retention、F-052/F-056 real 実証)
- **Tier 2 narrow**: Obsidian orphan note 検出(他 PKM usecase は F-047 で反証)
- **Tier 3 niche**: Log retention(ラベル無し条件限定、F-025 で 1 件実証)
- **Tier 4 pitch 弱**: bias-detector は ML 再現性 tool としては有用、KDF applicability predictor としては未 validated
- 詳細: [phase_M_market_suitability.md](phase_M_market_suitability.md)

### F-053 Real-KDF 500Q 直接対戦(2026-04-18、F-044 retraction 後)

**F-044 の simulation を real KDF に差し替えた再実行結果**:

| Method | accuracy | correct/500 | cost | latency |
|---|---:|---:|---:|---:|
| Random baseline | 0.344 | 172 | $0 | <1ms |
| **Real KDF (full 500Q, 30% keep)** | **0.434** | **217** | **$0** | **<1ms** |
| **Mem0 (GPT-4o-mini)** | **0.672** | **336** | $0.38 | ~30s/Q |
| ~~F-044 sim KDF~~ | ~~0.696~~ | ~~348~~ | — | — |(simulation artifact、retracted)

**Overall**: Mem0 が **+23.8 pt で real KDF を勝利**(p<10⁻¹⁶、McNemar's exact, b/c=41/160)。

**category 別(real KDF vs Mem0、すべて Mem0 勝利)**:

| category | n | real KDF | Mem0 | gap | sig |
|---|---:|---:|---:|---:|:-:|
| temporal-reasoning | 133 | 0.361 | 0.466 | **−0.105** | ★ (p=0.034) |
| multi-session | 133 | 0.451 | 0.677 | **−0.226** | ★ (p=0.0001) |
| knowledge-update | 78 | 0.423 | 0.731 | **−0.308** | ★ (p=0.0002) |
| single-session-user | 70 | 0.557 | 0.957 | **−0.400** | ★ (p<10⁻⁴) |
| single-session-assistant | 56 | 0.339 | 0.679 | **−0.339** | ★ (p=0.0009) |
| single-session-preference | 30 | 0.600 | 0.733 | −0.133 | (p=0.29) |

→ **全 6 category のうち 5 で Mem0 が有意に勝利**。single-session-preference のみ sample 小(n=30)で有意差検出されず。

**なぜ結論が F-044 から逆転したか**:
- F-044 Python script の `kdf_retrieve()` は F-033 の "KDF recall=0.821" を定数として assume、実際には 500Q での real KDF recall=0.665(F-052 で測定)
- Real KDF は answer turn を全 miss する bucket が 110/500 存在(recall=0)、この bucket で Mem0 は 83.6% 正解 → Mem0 の LLM fact-extraction は raw turn が無くても answer を抽出できるため、KDF の retrieval quality と end-to-end accuracy の直結性は我々の初期想定より強い

**依然として成立する KDF 優位**:
- Cost: KDF $0 vs Mem0 $0.38(retrieval quality と独立)
- Latency: KDF <1ms vs Mem0 ~30s(独立)
- vs TTL_recent: real KDF recall 0.665 vs TTL 0.180 (×3.7、retrieval layer only)
- vs Random: real KDF accuracy 0.434 vs Random 0.344 (+9pt, p≈0.02)

この制約下で、KDF は上記 P1-P7 のような**具体的問題**(conversational memory, PKM, log retention 等)に貢献する。それ以上の主張はしない。

---

## 公開資産

### コード
- [`crates/cgb-kdf/`](../crates/cgb-kdf/) — KDF 参照実装(**Claim 1-50 すべてに直接テスト `test_claimN_*`**、計 56 tests、workspace 449 tests pass)
- [`crates/bias-detector/`](../crates/bias-detector/) — ML 再現性ツール(stand-alone)
- [`crates/cgb-kdf/src/framework/classifier_fast.rs`](../crates/cgb-kdf/src/framework/classifier_fast.rs) — O(n) scaling(F-029)
- [`crates/cgb-kdf/src/framework/multimodal.rs`](../crates/cgb-kdf/src/framework/multimodal.rs) — graph + text 合成(F-032)

### 検証エビデンス
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 50 件 F-xxx 検証済み知見(F-050 まで)
- [SOLVABILITY_VERDICT.md](SOLVABILITY_VERDICT.md) — 5 件の正直な限界に対する solvability verdict
- [benchmarks/](../benchmarks/) — 全 phase のデータ・検証 binary
- [demos/](../demos/) — 8 ショーケース実施例

### ドキュメント
- [世界問題別詳細](world_problems/) — P1-P7 各問題の詳細
- [patent/filed/](patent/filed/) — 特許出願書類(frozen)
- [math/decay_analysis.md](math/decay_analysis.md) — 数理解析

---

## ライセンスと特許

- **コード**: PolyForm Noncommercial 1.0.0(研究・教育は無償、商用は別途ライセンス — COMMERCIAL.md 参照)
- **特許権**: 特願 2026-027032(独立管理、コード利用 ≠ 特許実施許諾)
- **データ**: 各 dataset の元ライセンスを継承、再配布しない

特許が成立した場合、商用実装のライセンスについては別途お問い合わせください。研究・教育目的での利用は広く歓迎。

---

## コミュニティ・連絡

**GitHub:** [ChaiCroquis/kdf-perovskite](https://github.com/ChaiCroquis/kdf-perovskite)
**発明者:** Chai(黒木康博)
**連絡:** Issues 経由

---

## 最終メッセージ

KDF は**万能ではない**。しかし、**「忘却と希少性保護」という普遍的な問題の一側面**に、ラベル不要・構造ベースで取り組める数少ない道具であることは、実データで実証できた。

**最大の impact 候補は LLM agent memory**(TTL 7.7×)。この 1 点だけでも、現実の数億人のユーザ体験を改善する可能性がある。

残りの問題は、**何ができる/できないを honest に言える道具**として、オープンに検証可能な形で提供する。発明者も発明を絶対視せず、実証を重んじる立場を取る。

---

*Last updated: 2026-04-17*
*Independent verification agents invoked: 12 times across project lifecycle*
*Total verified findings: 34 (F-001 ~ F-034)*
