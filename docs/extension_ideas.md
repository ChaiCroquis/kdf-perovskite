# KDF Extension Ideas — 拡張機能・アーキテクチャ案メモ

**最終更新**: 2026-04-18
**由来**: 発明者 (Chai) + Claude Opus 4.7 の Phase W 議論より抽出
**目的**: "代替 (replacement) ではなく補完 (complementary)" パラダイムに基づく具体的な拡張機能案を記録する。実装優先度は MVP から大規模まで段階的に整理。

---

## 🧭 基本思想:「代替」ではなく「補完」

**旧思想(避けるべき)**:
> KDF で Mem0 / Letta / MemGPT を置き換える

→ accuracy で負ける(F-053: −23.8pt on LongMemEval)、pitch 先が狭い、開発コスト膨大

**新思想(採用すべき)**:
> KDF は、**LLM-based memory system が必ず失敗する pattern 専用の deterministic 補完レイヤー**

→ 競合しない、Mem0 ユーザーに追加販売可、「Mem0 + KDF > Mem0 単独」の strictly-better 提案

### 根拠 evidence

| 条件 | KDF vs Mem0 |
|---|:-:|
| 一般 QA(LongMemEval 500Q)| 負け −23.8pt(F-053) |
| 日時 recall (LoCoMo temporal 321Q)| 勝ち +10.6〜+23.4pt(F-057, F-058) |

**含意**: "Mem0 が失敗する pattern(日時・具体数値・verbatim quote 等)" が存在し、その pattern は LLM 更新でも消えない(F-058 で gpt-4.1-mini でも degrade 確認)。

---

## 🧩 拡張機能案 一覧

以下、**実装難易度順**(MVP から大規模順)で整理。

---

### Ext-1: Precision-Query Router(MVP、**2026-04-19 実証完了** F-060)

**概要**: 軽量分類器(regex + 辞書)が "precision query" を判定し、KDF に routing、それ以外は LLM/Mem0 に送る。

**Architecture**:
```
[User Query]
     ↓
[Regex Classifier: is this a precision query?]
     ↓
  ┌── YES → KDF (deterministic, free, <1ms)
  └── NO  → Mem0 / LLM (semantic reasoning)
     ↓
[Merged Response]
```

**Precision query 判定 rules**(regex + 辞書、LLM 不要):
- 数値質問: `\b\d+\b` 含む、`how many|how much|what amount|what number`
- 日時質問: `when|what date|how long|before|after|\b\d{4}\b|\b(Jan|Feb|Mar|...|December)\b`
- 正確性要求: `exactly|specific|precisely|verbatim|word for word`
- リスト question: `list all|enumerate|how many different`
- Quote 要求: `what did (I|you) say|quote|exact words|verbatim`

**MVP 実装範囲**:
- Python wrapper 1 file (~100 行)
- 既存 KDF Rust binary を subprocess で呼ぶ
- Mem0 API を既存のまま使う
- 分類器は regex 50 件程度で 85-90% の precision query 検出を狙う

**実測結果**(F-060, 2026-04-19):
- v2 (precision + long context ≥ 100 turns) で既存 4 cell 検証:
  - LongMemEval 500Q × 2 model: Router = Mem0 alone(害なし、routing 発動せず)
  - LoCoMo temporal 321Q × gpt-4o-mini: **+9.7pt (p=0.003)**
  - LoCoMo temporal 321Q × gpt-4.1-mini: **+22.4pt (p=4×10⁻¹⁴)**
- **strictly better property 実証**: どの cell でも Mem0 alone を下回らず、半数の cell で有意に上回る
- **コスト効果**: LoCoMo 系では 97% の query が KDF に routing → LLM API call 97% 削減

**v1 (precision only) の失敗教訓**:
- LongMemEval も precision query を含む(63.8% を routing)
- 短対話 context では KDF が不利なので router が −11.6 〜 −13.0 pt 悪化
- **教訓**: precision 判定に **conversation length 条件を合わせる** ことが必須

**前提条件**: KDF Rust binary が API 公開していること(`kdf-cli` or library form)

**工数推定**: MVP 完了(F-060 で validation 済み、production 実装は +1-2 日)

---

### Ext-2: Failure-Mode Catalog(研究資産、中期)

**概要**: Mem0 (LLM-based memory) が **systematic に失敗する pattern** を列挙した辞書。precision-query router の元ネタでもあり、商業 pitch の差別化資産にもなる。

**コンセプト**:

| Failure pattern | 具体例 | 頻度(F-053/F-058 から) |
|---|---|---:|
| P1: 具体数値の損失 | `$400K` → Mem0 `$350K` | F-053 で 12/160 Q |
| P2: 日時の粗略化 | `7 May 2023` → Mem0 `around May` | F-058 で 89/321 Q |
| P3: リストの切り詰め | `3 citrus` → Mem0 `lime and orange` | F-053 で 8/66 Q |
| P4: assistant 発話の verbatim 喪失 | 推薦 5 options → Mem0 「options を提案」 | F-053 で 17/56 |
| P5: 時刻演算の放棄 | `30 days passed` → Mem0 `about a month` | F-058 で未集計 |
| P6: ... | ... | ... |

**実装範囲**:
- JSON 形式の pattern DB: `failure_patterns.json`
- 各 pattern に:
  - `pattern_id` / `name` / `description`
  - `query_regex_markers` (どう検出するか)
  - `example_queries` (from F-053/F-058)
  - `mem0_behavior` / `kdf_behavior` (expected)
  - `evidence_findings` (F-053, F-058 等への参照)

**効用**:
- Ext-1 の分類器を手動で作る元ネタになる
- Paper の limitation 章 / strengths 章に直接使える
- 商業 pitch:「KDF は以下 N 個の systematic failure pattern に対応します」と明示できる
- 規制業界向けの監査ドキュメントとしても使える

**工数推定**: 3-7 日(既存 data の分類作業が主、新規 experiment 不要)

---

### Ext-3: Consensus Auditing Mode(規制業界向け、中期)

**概要**: KDF と Mem0 両方を走らせ、**回答の一致 / 不一致** を監査ログに残す。

**Architecture**:
```
[User Query]
     ├→ [KDF Response] (deterministic, reproducible)
     └→ [Mem0 Response] (LLM-based)
         ↓
[Comparison Layer]
 ├── Match → Return answer, log "high confidence"
 └── Mismatch → Return flagged result, log "requires review"
```

**Use case**:
- 金融 compliance: "LLM が出した回答が後で問題になったら、KDF の決定論的回答と照合できる"
- 医療記録: "LLM の要約と原文(raw turn)が乖離していないか自動検出"
- 法務 discovery: "LLM による要約の正確性を KDF 決定論的 retrieval で検証"

**KDF 単体の monetization**: 
- 「Mem0 を monitoring する add-on」として subscription 販売可能
- ターゲット: Mem0 を既に採用している企業で、compliance 要件が厳しい部門

**実装範囲**:
- 両方の retrieve + answer + response diff 計算
- Diff を human-readable format (例: unified diff) で出力
- 監査ログ DB(SQLite で MVP、後に Postgres)

**工数推定**: 1-2 週間(DB 設計含む)

---

### Ext-4: Hierarchical Memory(精度改善、中期)

**概要**: KDF を "前処理フィルタ" として使い、Mem0 / LLM の処理対象を 30% に絞る。  
**注意**: F-044 の simulation 実装はこれに近かったが、real KDF で再実装する必要がある。

**Architecture**:
```
[Raw conversation log: 数千 turn]
     ↓
[KDF で 30% 選別 (deterministic, free)]
     ↓ [選別された 30% の turn のみ]
     ↓
[Mem0 / LLM の fact extraction + retrieval]
     ↓
[Answer generation]
```

**期待効果**:
- LLM input tokens 70% 削減 → **Mem0 のコスト 70% 減**
- KDF で関係ない turn を除外するので LLM の SNR 向上
- 場合によっては accuracy も向上(ノイズ除去効果)

**Risk (F-053 で観察された問題)**:
- KDF recall が 0.665 なので、35% の answer turn が LLM に届かない
- LLM が raw turn 不在で答えられない Q が増える
- **対策**: keep_rate を 50-70% に上げる(コスト削減効果は薄れる)

**実装範囲**:
- 既存 Rust binary (`phase_w3_real_kdf_turns.rs`) と Mem0 script を組み合わせる
- API: `kdf_filter(raw_turns, keep_rate) -> selected_turns` → `mem0_add(selected_turns)`

**工数推定**: 1 週間(scripting + evaluation)

**実際の payoff**: F-044 が失敗したので、**慎重に再検証** すべき。LLM コスト減と accuracy 維持の trade-off が実用レベルで両立するか未確認。

---

### Ext-5: Long-Conversation Temporal Specialist(商用 positioning、短期)

**概要**: "長期会話の日時記憶" だけに特化した standalone 製品。Ext-1 の subset だが、最もシンプルな商用形態。

**Target market**:
- 会議録 archiver / note-taking SaaS(Otter.ai, Fireflies.ai 統合)
- 長期 AI chatbot(Character.ai, Replika)
- Personal journal / diary AI(Rewind.ai, Granola)
- 医療・法務の時系列イベント検索

**MVP 機能**:
- 会話ログを input
- 日時 query に対して raw turn を返す(KDF で retrieval)
- LLM 不要、無料、即答

**製品形態案**:
1. **Rust crate**: 開発者向け、PolyForm Noncommercial 1.0.0 で公開(商用は COMMERCIAL.md 参照)
2. **Python wrapper**: pip でインストール、`kdf-temporal` npm 相当
3. **REST API**: Docker コンテナで self-host、$0 SaaS(infrastructure のみ課金)
4. **Notion / Obsidian plugin**: エンドユーザー向け、日時検索だけ強化

**差別化**:
- Mem0: 有料、LLM 依存、stochastic、日時に弱い(F-058 で実証)
- KDF-temporal: 無料、deterministic、完全オフライン、日時に強い

**工数推定**: MVP (Python wrapper) なら 1 週間。SaaS 形態なら 2-4 週間。

---

### Ext-6: Distillation Signal(研究、長期)

**概要**: KDF の決定論的 selection を "weak supervision label" として使い、**小さな LLM** を訓練する。

**Concept**:
```
[Raw conversations + queries]
     ↓
[KDF で turn selection(ground truth 代わり)]
     ↓
[Small LLM (7B or 13B) を fine-tune:
   input: (query, raw turns) → output: "これらを retrieve すべき" selection]
     ↓
[KDF + semantic reasoning を両立した lightweight selector]
```

**期待効果**:
- KDF の recall 0.665 を semantic LM で 0.80+ に上げる可能性
- LLM-free ではなくなるが、gpt-4o-mini より軽量 7B モデルで動く
- 完全に determinstic ではないが、再訓練すれば再現可能

**実装範囲**:
- F-053 + F-057 + F-058 の raw 結果を教師データに変換
- Llama-3 7B 等の FT パイプライン構築
- Evaluation で KDF vs distilled model の比較

**Risk**:
- KDF の発明 originality から遠ざかる(LLM に寄る)
- 発明者の design philosophy(「決定論 = 資産」)に反する
- ドツボ警告リスト該当

**工数推定**: 2-4 週間 + GPU リソース $500-2000

**推奨**: 他の Ext が商業で traction を得た後のオプション。最優先ではない。

---

## 📊 優先度マトリクス

| Ext | 実装難易度 | 商業 impact | 研究 impact | **優先度** |
|:-:|:-:|:-:|:-:|:-:|
| **Ext-1 Precision Router** | ★ (MVP 2-5 日) | ★★★ | ★★ | **🥇 最優先** |
| **Ext-5 Temporal Specialist** | ★ (1 週間) | ★★★ | ★ | **🥈 次優先** |
| Ext-2 Failure Catalog | ★★ (1 週間) | ★★ | ★★★ | 🥉 並行可 |
| Ext-3 Consensus Auditing | ★★★ (2 週間) | ★★ (niche) | ★ | 中期 |
| Ext-4 Hierarchical Memory | ★★★ (2 週間) | ★★ | ★★ | 要再検証 |
| Ext-6 Distillation | ★★★★ (4 週間) | ★ | ★★★ | 長期 option |

---

## 🎯 推奨される実装 sequence

### Phase 1(今すぐ、1 週間以内)
**Ext-1 Precision-Query Router の MVP**  
→ KDF + Mem0 複合製品の最小形態、Python wrapper + 既存 Rust binary で実現可能。

### Phase 2(Phase 1 成功後、1-2 週間)
**Ext-5 Temporal Specialist を商用形態に**  
→ Ext-1 の router の KDF 側を standalone 製品として独立化。`pip install kdf-temporal` の形。

### Phase 3(並行、2 週間以内)
**Ext-2 Failure Catalog の整理**  
→ 既存 data(F-053, F-058)から error pattern 分類、pitch docs に反映。

### Phase 4(商業 traction 後)
Ext-3 Consensus Auditing または Ext-4 Hierarchical Memory の本格実装。

### Phase 5(将来的)
Ext-6 Distillation は余裕ができてから検討。

---

## 🪤 拡張時のドツボ警告(design_philosophy.md と一貫)

以下の兆候が出たら scope creep:
- Ext-1 の分類器精度を 99% に上げたくなる → 95% で止める
- Ext-2 の failure pattern を 100+ 集めようとする → 10 patterns で十分
- Ext-5 を多言語対応にしようとする → 英語 MVP 後で十分
- Ext-3 の diff algorithm を carefully 設計したくなる → 単純 string compare で MVP

**判定基準**: 「MVP で商用 pitch できるか?」が基準。できないなら scope を削る。

---

## 📝 変更履歴

- **2026-04-18 (初版)**: Phase W1-W5b+W4 (F-050〜F-058) を受けて、発明者との議論から抽出した 6 つの拡張機能案を documented。Ext-1 を優先実装対象として推奨。

## 🎯 拡張機能選定の decisive predictor(F-061〜F-065 確立)

**新規拡張を検討する際、以下の axis で事前に適性判定**する:

> **「提案する応用において、structural rareness が task importance と相関するか?」**

- **Yes → Tier 1/2 candidate**(F-062/F-065 git merge、F-057/F-058 LLM temporal の路線)
- **No or inverse → Tier 4(意味なし)として skip**(F-063 GP、F-064 API、F-047 semantic minority のパターン)

### 検証済みの "Yes" 応用(KDF 優位領域)
- Structurally rare merge commit 保持(小〜中 repo、F-065)
- Path-critical bottleneck 保護(APSP、F-061)
- Long-context date/time literal(raw verbatim、F-057/F-058)
- Orphan note detection(PKM、F-012)

### 検証済みの "No" 応用(KDF 非適領域)
- Density-based function approximation(GP、F-063)
- High in-degree API detection(naive call graph、F-064)
- Scale-free hub centrality(F-061 BA/WS)
- Merge-heavy enterprise repo(pytest-type、F-065)
- Metadata-based minority(F-047)
- General semantic retrieval(F-045)

### 新提案時のセルフチェック

1. Target graph で重要 node は高 degree? → KDF 非適、TopDegree 使え
2. Target が density coverage 要? → KDF 非適、KMeans 使え
3. Target が意味的 metadata 依存? → KDF 非適、dense embedding 使え
4. Target で rare event / boundary / bottleneck が重要? → **KDF 候補**
5. Target が item-level verbatim 保持要? → **KDF 候補**

---

## 🌐 拡張:NLP を超えた応用領域

2026-04-19 に発明者が明確化:**KDF は NLP 技術ではなく、項目可逆・集合選別型の汎用グラフ圧縮**。以下の応用領域は未検証だが構造的に適合。詳細は [domain_validation.md](domain_validation.md) 参照:

### 非 NLP の応用候補(Ext-7 以降)

| # | 応用 | Graph node = | 構造的適合理由 | 優先度 |
|:-:|---|---|---|:-:|
| Ext-7 | **Git commit 履歴 pruning** | commit | ✅ **F-062 + F-065 で validated**(merge rate < 10% の repo で 99% merge recall)、**merge-heavy repo(>20%)では TopDegree が勝つため narrowing 必要** | 🥇 高($0) |
| ~~Ext-8 naive~~ | ~~Naive Python call graph curation~~ | function | ❌ **F-064 で negative: API は高 in-degree、KDF Rare 保護と逆方向** | Tier 4 に降格 |
| Ext-8 proper | Type-aware static analysis call graph | function | **要 engineering**(pycg / rust-analyzer 等)、F-064 caveat あり | 🥈 中、2-3 週間 |
| Ext-9 | **Sensor/IoT データ retention** | sensor reading | 異常値・イベント境界が rare | 🥈 中 |
| Ext-10 | **金融 transaction archival** | transaction | 不正・audit 重要 transaction が rare | 🥈 中(regulated) |
| Ext-11 | **医療 event timeline** | medical event | 診断・投薬境界が rare、日時重要 | 🥈 中(regulated) |
| Ext-12 | **法務 discovery (Enron email)** | document | lawsuit-relevant document が rare | 🥉 低 |
| Ext-13 narrowed | **古典アルゴリズム復権 preprocessing(path-based / traversal 系)** | graph node | **F-061 で partial validated**: APSP / connectivity 保持で KDF 優位、**ただし GP(F-063)/ scale-free betweenness(F-061 BA/WS)では敗北** | 🥈 中(narrowed from novel contribution) |

**Ext-13 詳細(F-061 / F-063 後の narrowing)**: 発明者発案の novel 洞察だが、**適性領域は「structural rareness が task importance と相関する classical algorithm」のみ**:
- ✅ APSP(Floyd-Warshall 等)、routing、connectivity 分析(F-061 で WS/SBM で優位)
- ❌ GP regression、Kernel SVM、density-based 系(F-063 で negative)
- ❌ Scale-free graph の betweenness(F-061 BA/WS で TopDegree 勝利)

詳細: [classical_algorithm_revival.md](classical_algorithm_revival.md) + [validation_strategy.md](validation_strategy.md)

---

## 🔗 関連 document

- [design_philosophy.md](design_philosophy.md) — 設計方針・マントラ
- **[domain_validation.md](domain_validation.md) — 応用領域別 validation 状況**(2026-04-19 追加)
- **[classical_algorithm_revival.md](classical_algorithm_revival.md) — 古典 algorithm 復権候補**(2026-04-19 追加)
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 検証済み knowledge
- [phase_M_market_suitability.md](phase_M_market_suitability.md) — 市場 suitability
- [phase_W_next_verifications.md](phase_W_next_verifications.md) — 追加検証候補
- [paper_draft.md](paper_draft.md) — 論文草稿
