# Domain Validation Matrix — KDF 汎用性の検証状況

**最終更新**: 2026-04-19
**目的**: KDF を「項目可逆・集合選別型の汎用グラフ圧縮」として捉え直した際、どの応用領域で **本当に使えると実証済か**、どこが **未検証で追加実験が必要か** を整理する。

---

## 🧭 KDF の適用可能条件(再掲)

KDF が有効に働く条件(F-025/F-045/F-046 等の経験則):

1. **Graph structure が情報の rareness を符号化している** (= "structure encodes rarity")
   - 例: 孤立 note、chain の終端、接続が少ない session → Rare layer
   - 反例: 意味的 minority が構造と独立(F-047 Welsh Wikipedia)
2. **Ground truth が手に入りにくい / ラベル無し** 状況
   - Stratified 等の supervised 方法が使えない領域で KDF の価値が出る
3. **Budget 制約(全量保持不可)** が存在する
   - 全部保持できるなら圧縮不要
4. **Item-level verbatim 保持が価値を持つ**
   - 要約で失われる情報(日時・数値・固有名詞)を後で参照したい

---

## 📊 応用領域 × 検証状況マトリクス

### ✅ 検証済(positive evidence)

| # | 応用領域 | Graph node = | Validation | 備考 |
|:-:|---|---|---|---|
| A1 | **LLM 会話記憶**(gpt-4o-mini) | conversation turn | F-057, F-060 | LoCoMo temporal で +11pt, +22pt |
| A2 | **LLM 会話記憶**(gpt-4.1-mini) | conversation turn | F-058 | temporal で +23.4pt (p=1.6×10⁻¹⁴) |
| A3 | **NASA HTTP log retention**(ラベル無し)| log entry | F-025 | Random 比 2.3× |
| A4 | **Obsidian orphan note 検出** | note | F-012, F-017 | F1=0.747, Wilcoxon p=0.006 |
| A5 | **ML 再現性 (bias-detector)** | ML experiment | F-030, F-036 | synthetic benchmarks で 4/5 予測 |
| A6 | **Synthetic D8 (conv history)** | synthetic turn | F-028 | recall 1.000 in hybrid |

### ⚠️ 検証済(negative evidence)

| # | 応用領域 | 失敗理由 | Finding |
|:-:|---|---|:-:|
| A7 | Welsh Wikipedia 文化的 minority 検出 | minority が構造と独立(metadata ベース) | F-047 |
| A8 | OSS GitHub issue 重要度 | 一般化失敗(3 repo 平均 ×1.00) | F-038 |
| A9 | OpenAlex 論文再発見 | ×0.83(Random 以下) | F-039 |
| A10 | General semantic retrieval | dense embedding に大敗 | F-045 |
| A11 | Cross-task applicability predictor | bias-detector は synthetic で 4/5、real cross-task では collapse | F-046 |

### 🔍 未検証(追加検証が必要、"本当に使えるか" 要証明)

以下は **構造的に適合しそうだが未実験** の応用領域。各 candidate について、(a) 何故構造的に適合するか、(b) 必要な追加検証は何か、を記述する。

---

#### B1. Git commit 履歴 pruning ✅ **検証完了 2026-04-19 → F-062**

**結果**: tokio-rs/tokio (4,752 commits) で KDF が 30% keep で **merge commit 99.5% / tag commit 42% を保持**、Random(merge 30%)や TTL_recent(merge 22%)を大きく上回る。TopDegree と同等。

**要約**:
- ✅ **Validated commercial use case**: 中長期 repository archival で構造保持
- ⚠️ **narrowing**: KDF vs TopDegree はほぼ同等 → KDF を押す理由は multi-domain 一貫性(統合 product)

**追加検証候補**:
- 3 repo 比較(Rust / Python / JS)で robustness 確認
- File-overlap edge model で behavior 変化確認

**以下、元の B1 検証計画(参考)**:


**なぜ適合しそうか**:
- Git graph: commits = nodes、parent-child = edges
- 長期リポジトリは線形爆発(Linux kernel 100万 commits 超)
- "重要な" commit(リリース、API 変更、bugfix landmark)は通常 merge point で structurally rare
- 大量の "trivial" commit(typo fix、comment change)は central な接続を持つ

**必要な追加検証**:
- [ ] Dataset: rust-lang/rust, kubernetes/kubernetes, tensorflow/tensorflow 等 public repo の git log
- [ ] Ground truth: 手動でラベル付けされた "重要" commit(タグ、リリース、大きな変更)
- [ ] Task: KDF で 30% 保持時、重要 commit の recall 率
- [ ] Baseline: TTL (recent commits only), Random, betweenness centrality
- **予想コスト**: $0(local git operation のみ)
- **所要時間**: 2-3 日(dataset 準備 + script + eval)
- **expected value**: 高(新 application、LLM 不要で sell できる)

#### B2. コード依存グラフ curation(高優先度、中コスト)

**なぜ適合しそうか**:
- Call graph: function = nodes, call = edges
- 大型 codebase では call graph が巨大(100万 edges+)で全体解析が実質不可能
- "Core" function(Rare/Core layer)だけ保持すれば、多くの静的解析が scale する
- 古典的 inter-procedural analysis を revive できる(classical_algorithm_revival.md 参照)

**必要な追加検証**:
- [ ] Dataset: 中規模 Rust/Python/Java codebase の call graph(tree-sitter or LSP で抽出)
- [ ] Ground truth: manual annotation で "API boundary" "hot path" "entry point"
- [ ] Task: KDF で 30% 保持、important function の recall
- [ ] Baseline: PageRank, betweenness, call frequency
- **予想コスト**: $0
- **所要時間**: 1 週間(extraction pipeline 含む)

#### B3. Sensor / IoT データ retention(高優先度、中コスト)

**なぜ適合しそうか**:
- Sensor log: readings = nodes, temporal/spatial proximity = edges
- 高頻度 sensor data は storage 爆発(1 Hz × 100 sensor × 1 year = 3.15B readings)
- "異常値" "イベント境界" の reading が structurally rare
- 定常状態の reading は highly connected で redundant

**必要な追加検証**:
- [ ] Dataset: 公開 IoT dataset(PhysioBank medical sensor, Intel Lab sensor, etc.)
- [ ] Ground truth: 異常検出ラベル付きデータ
- [ ] Task: KDF で 10% 保持、anomaly recall
- [ ] Baseline: Stratified (label-based), Isolation Forest, random sampling
- **予想コスト**: $0
- **所要時間**: 2-3 日

#### B4. 金融 transaction archival(中優先度、regulated 市場)

**なぜ適合しそうか**:
- Transactions = nodes, related transactions (by account/time/amount) = edges
- Regulatory requirement で "重要取引" 保持必須、"routine" は aggregate 可
- 不正検知や audit trail で rare transaction が重要
- LLM 不要、deterministic、監査可能 = 金融業界の要求に一致

**必要な追加検証**:
- [ ] Dataset: synthetic transaction graph(public fraud dataset: CreditCardFraud, PaySim, etc.)
- [ ] Ground truth: fraud label / audit flag
- [ ] Task: KDF retention で suspicious transaction の保持率
- **予想コスト**: $0
- **所要時間**: 1 週間

#### B5. Citation network pruning(中優先度、academic use)

**なぜ適合しそうか**:
- Papers = nodes, citations = edges
- 学術知識グラフは OpenAlex/Semantic Scholar で 2 億論文超
- 古典的 PageRank/betweenness を full graph で回すのは impractical
- KDF で "seminal paper" を優先保持 → sub-graph PageRank や community detection を revive

**必要な追加検証**:
- [ ] Dataset: OpenAlex subset (e.g., ML/AI 分野のみ)
- [ ] Ground truth: highly-cited papers, award winners
- [ ] Task: KDF + classical algorithm の combined performance
- [ ] Caveat: **F-039 で論文再発見は失敗している**、別 task (citation analysis) で再検証必要
- **予想コスト**: $0
- **所要時間**: 1-2 週間

#### B6. 医療 event timeline(中優先度、regulated 市場)

**なぜ適合しそうか**:
- Medical events(visit, diagnosis, medication) = nodes
- Events linked by patient, time, disease progression = edges
- HIPAA / regulated で長期保持必須、summary 化で詳細損失不可
- 日時記憶の重要性 → LoCoMo temporal で実証された KDF の強み直結

**必要な追加検証**:
- [ ] Dataset: MIMIC-III subset(public, requires access application)
- [ ] Ground truth: clinical outcome-relevant events
- [ ] Task: KDF retention での重要 event 保持率
- **予想コスト**: $0(ただし data access に時間)
- **所要時間**: 1 ヶ月(access 含む)

#### B7. 法務 discovery(低優先度、niche)

**なぜ適合しそうか**:
- Documents in legal case = nodes, topical/reference links = edges
- 膨大な email/document から relevant なものを抽出する task
- KDF の raw 保持性 = "原文のまま" が legal admissibility に適合

**必要な追加検証**:
- [ ] Dataset: Enron email corpus(public)
- [ ] Ground truth: lawsuit-relevant email(公開済 label)
- [ ] Task: KDF pruning の relevant email 保持率
- **予想コスト**: $0
- **所要時間**: 1-2 週間

#### B8. ゲノムシーケンス saliency(低優先度、domain expertise 要)

**なぜ適合しそうか**:
- DNA/protein sequences in graph of similarity
- 膨大な sequence database(NCBI)から "representative" を選別する需要
- 古典的 phylogenetic tree 構築の前処理として

**必要な追加検証**:
- domain expertise が必要、短期では非推奨

---

## 🎯 優先度付き検証 roadmap

### Tier 1(今すぐやるべき、$0、1-2 週間)

1. **B1 Git commit pruning** — rust-lang/rust + tokio + linux subset で 3 repo 再実験
2. **B2 Call graph curation** — tree-sitter で Rust codebase call graph 抽出、KDF で重要関数保持率

**合計コスト**: $0、時間 2-3 週間、価値: 新 application 2 件の独立 validation

### Tier 2(商用インパクト大、$0-10、2-4 週間)

3. **B3 IoT sensor anomaly retention** — 公開 dataset で異常検出
4. **B4 金融 transaction fraud archival** — public fraud dataset で suspicious transaction 保持率
5. **B6 医療 event timeline** — MIMIC-III access 後

### Tier 3(研究向け、低優先度)

6. **B5 Citation network** — F-039 の別 task 再検証
7. **B7 Legal discovery** — Enron email
8. **B8 Genome** — domain expert が必要

---

## 📋 paper_draft との整合

現在の paper_draft.md は既に domain-invariant framing(10 領域横断)で書かれているが:
- **Abstract で "項目内可逆・集合内選別型圧縮" を明記**(2026-04-19 追記済)
- **LLM memory は "1st validated application"** として位置付ける
- **未検証 application は "future work"** として枠を残す(論文投稿時に B1-B8 から選抜)

---

## 🔗 関連 document

- [paper_draft.md](paper_draft.md) — 論文草稿
- [design_philosophy.md](design_philosophy.md) — 設計方針
- [extension_ideas.md](extension_ideas.md) — 拡張機能案
- [classical_algorithm_revival.md](classical_algorithm_revival.md) — 古典アルゴリズム復権候補(本 file と対)
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 検証済み findings
- [SOLVABILITY_VERDICT.md](SOLVABILITY_VERDICT.md) — 既知の限界
