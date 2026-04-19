# KDF Design Philosophy — 設計方針メモ

**最終更新**: 2026-04-19(NLP-agnostic framing 追記)
**記録者**: 発明者(Chai)+ Claude Opus 4.7
**目的**: 今後 KDF を実装・拡張・pitch する際の方針指針。Phase W1-W5b+W4+Ext-1 (F-050〜F-060) の検証結果から抽出した設計原則。

---

## 🌐 まず大前提:KDF は NLP 技術ではない

**KDF の本質** = **「項目レベル可逆・集合レベル選別型のグラフ圧縮技術」**

- Input: グラフ(ノード + エッジ)+ budget(保持する割合)
- Output: 決定論的に選ばれたノード集合
- 性質:
  - **選ばれたノードの中身は一切改変されない**(verbatim 保持、項目レベル可逆)
  - **選ばれなかったノードは完全破棄**(集合レベル選別)
  - **同じ入力 → 同じ出力**(完全決定論)
  - **budget/retention 率が事前指定可能**(コスト制御)

**LLM fact extraction との本質的違い**:

| 手法 | 項目レベル | 集合レベル | 決定性 |
|---|:-:|:-:|:-:|
| **KDF** | **可逆**(中身そのまま)| 選別 | **決定論的** |
| **LLM 抽出 (Mem0 等)** | **非可逆**(要約で変形)| 選別+変形 | 確率的 |

F-058 で Mem0 が 日時記憶で degrade した根本原因はこの差。KDF は日付文字列 "7 May 2023" を verbatim 保持、LLM は "around May" に要約してしまう。

### 📦 適用領域は NLP に限らない

今日の検証(LongMemEval, LoCoMo)は **会話を graph node として扱う** 応用 1 つに過ぎない。他の検証済み領域:

| 応用 | 検証 | Graph node = 何か |
|---|:-:|---|
| LLM 会話記憶 (D8) | ✅ F-057/F-058/F-060 | conversation turn |
| NASA HTTP ログ保持 | ✅ F-025 | log entry |
| PKM orphan 検出 (Obsidian) | ✅ F-012/F-017 | note |
| Welsh Wikipedia 記事 | ❌ F-047 で限界 | article |
| ML 再現性 (bias-detector) | ✅ F-030/F-036 | ML experiment |

**未検証だが構造的に適合する領域**(参照: [domain_validation.md](domain_validation.md)):
- git commit 履歴 pruning
- コード依存グラフの選別
- sensor/IoT データ retention
- 金融取引記録
- 医療 event timeline
- 古典アルゴリズムの前処理(参照: [classical_algorithm_revival.md](classical_algorithm_revival.md))

**この認識が重要な理由**: 特許明細書([`docs/patent/filed/`](patent/filed/))は graph-node 抽象で書かれており、NLP に限定されない。pitch で "LLM memory ツール" と narrowing すると特許 claim の広さを活かせない。

---

## 🎯 核となる発想転換

### ❌ 避けるべき発想:「KDF で LLM ベース memory system を置き換える」

- 2026-04-18 時点で、LongMemEval 一般 QA では Mem0 に −23.8 pt で負ける (F-053)
- Hybrid で救済を試みても tied (F-055)
- Budget 倍増でも解決せず (F-054)
- **この土俵で競うと勝てない。戦うな**

### ✅ 採用すべき発想:「LLM ベース system が *必ず失敗する pattern* に KDF が fallback する」

- Mem0 などの LLM fact-extraction は原理的に lossy compression を行う
- 具体的数値 / 日時 / 完全リスト / assistant 発話の verbatim は **systematic に失われる**
- KDF は raw-turn 保持で **これらを決定論的に preserve**
- LoCoMo temporal で実証済: KDF +10.6pt (F-057) → gpt-4.1-mini で +23.4pt (F-058, p=1.6×10⁻¹⁴)

---

## 📐 設計原則

### 原則 1: 決定論アルゴリズムは必ず資産になる

**発明者 (Chai) の設計哲学**:

> 「AI を大きく使おうとするとドツボにハマる。  
>   慎重にやるなら、決定論のアルゴリズムを別でコツコツ作成する。  
>   決定論だと必ず資産になる」

**なぜ真か**:

| 性質 | 決定論アルゴリズム | LLM ベース |
|---|:-:|:-:|
| 結果の再現性 | ✅ 完全一致 | ❌ stochastic |
| エラーカタログ化 | ✅ 有限かつ列挙可能 | ❌ 毎回違う |
| モデル更新の影響 | ✅ 不変 | ❌ silent breakage |
| 監査証跡 | ✅ 決定ルートを辿れる | ❌ LLM blackbox |
| テストスイート構築 | ✅ 可能 | ❌ 困難 |
| コスト scalability | ✅ 定数 (マイクロ秒) | ❌ per-query 課金 |
| オフライン動作 | ✅ 可能 | ❌ API 必須 |
| **時間の経過で陳腐化するか** | **❌ しない** | **✅ する(モデル世代交代ごと)** |

**含意**: 決定論アルゴリズムは「積み上げ」になる。LLM 版は「次世代 LLM で置き換わる」運命。発明者が一人で作るなら、決定論に賭けるのが合理的。

---

### 原則 2: 代替 (replacement) ではなく補完 (complementary) で positioning する

**旧 pitch**:
> "KDF is a memory system that replaces Mem0"
→ 競合アリーナで戦う、accuracy で負ける、売り込みが弱い

**新 pitch**:
> "KDF is a deterministic safety net for LLM-based memory systems.  
>   It catches precision queries (dates, numbers, exact quotes) where LLM fact extraction systematically fails."

**商業的効果**:
- Mem0 と競合しない → Mem0 ユーザーに追加販売可能
- accuracy 比較で負けなくてよい → 「Mem0 が失敗する所」だけ勝てばよい
- pitch 先が広がる(Mem0 既存顧客、Letta ユーザー、MemGPT 採用企業、自社 LLM memory 開発中の会社)

---

### 原則 3: Precision-query routing アーキテクチャ

**実装案**(MVP):

```
[Query]
   ↓
[軽量分類器 (regex + 辞書、LLM 不要)]
   ↓
  ┌── precision query? (日時 / 具体的数値 / 完全一致)
  │     YES → KDF (決定論、無料、即答)
  │     NO  → Mem0 / LLM-based (従来通り)
  │
  └── 両方走らせて比較 (監査モード)
```

**precision query 判定 rules**(regex ベースで開始):
- 数値を含む質問: `\b\d+\b` or `how many|how much`
- 日時クエリ: `when|what date|how long|before|after|\b(Jan|Feb|...)\b|\b\d{4}\b`
- 正確性要求: `exactly|specific|precisely|verbatim`
- リスト queries: `list all|how many different|enumerate`
- Quote 要求: `what did (you|I) say|quote|exact words`

**効果予測**(既存データから):
- LoCoMo temporal (precision) → KDF 0.32 vs Mem0 0.09 = **+23pt**
- 一般 QA (non-precision) → Mem0 が担当
- **全体 pipeline 精度**: Mem0 alone より strictly 高い
- **全体 pipeline cost**: Mem0 alone とほぼ同じ or 微減 (precision query は LLM 不要)

---

### 原則 4: 範囲を狭く、MVP を最小に

**ドツボ回避のための規律**:

| やること(MVP) | やらないこと(過剰設計の芽) |
|---|---|
| regex ベースの precision query 判定 | 完璧な分類器を LLM で作る |
| LoCoMo temporal で勝利を defend | narrative / factual 全 category を同時に狙う |
| Mem0 との複合製品を pitch | Letta, MemGPT, ... 全競合にも勝とうとする |
| decision log を commit に残す | 大規模 experiment tracking system を作る |
| 既存 benchmark で検証 | 新規 benchmark を自作する |
| 2 model で robustness 検証 | 10 model でグリッド検証 |
| 1 precision category に集中 | 5 precision category 同時対応 |

**判定基準**: 「MVP で pitch できるか?」Yes なら進む、No なら scope を削る。

---

### 原則 5: 失敗 pattern のカタログ化が差別化資産

**Mem0 (LLM ベース)は静的に failure mode を enumerate できない**:
- 毎回違う fact 抽出をする
- prompt / model / temperature で挙動が変わる
- 「この pattern で必ず失敗」と claim できない

**KDF (決定論)は static enumeration が可能**:
- 同じ入力 → 同じ selection
- 「この pattern で必ず正解」と claim できる
- 同時に「この pattern で必ず失敗」も enumerate できる

**商業的含意**: 「監査可能な memory system」として regulated 業界(金融 / 医療 / 法務)に訴求できる。これは LLM ベースでは不可能な商品。

**次 research step**:
- F-053 の 160 Q(Mem0 failed, KDF succeeded)の error pattern 分類
- F-058 の 89 Q(Mem0 failed, KDF succeeded on gpt-4.1-mini)の error pattern 分類
- 両者の intersection = "universally Mem0-failure + KDF-success" pattern の identification
- → これが KDF の sellable domain の上限を定義する

---

### 原則 6: 論文 narrative の構造化

**paper の draft で使える一貫した語り**:

1. **Problem**: LLM-based memory systems (Mem0, Letta, MemGPT) have principled weaknesses in preserving exact information (numbers, dates, verbatim content) due to their fact-extraction compression.
2. **Evidence**: On LoCoMo temporal, real KDF outperforms Mem0 by +10.6 pt with gpt-4o-mini (p=0.0014) and +23.4 pt with gpt-4.1-mini (p=1.6×10⁻¹⁴), n=321 each.
3. **Mechanism**: KDF's raw-turn preservation avoids the lossy compression step. Structural rarity ranking (Rare/Core/Edge/Garbage) surfaces answer turns without LLM reasoning.
4. **Complementary design**: KDF is not proposed as replacement for LLM-based systems, but as a deterministic safety net for precision queries that LLMs systematically fail on.
5. **Honest limitations**: KDF loses to Mem0 on short-dialog generic QA (LongMemEval -23.8 pt, F-053). Cost of determinism is loss of semantic reasoning.
6. **Contribution**: (a) first statistical demonstration of principled LLM-memory failure on temporal recall, (b) open-source deterministic complement, (c) hybrid architecture blueprint.

---

## 🪤 ドツボ警告リスト

以下の兆候が出たら scope creep の兆し:

- [ ] 「KDF を LLM-based memory の総合代替にする」と考え始める
- [ ] 分類器の精度を 99% に上げようとする(95% で止める)
- [ ] Mem0 以外の全競合(Letta, MemGPT, LangMem, ...)を同時比較する
- [ ] KDF の prompt engineering を始める(KDF は LLM 不要が売り)
- [ ] 新 benchmark を自作する(既存 peer-reviewed を使う)
- [ ] 5-seed variance を全 finding に適用しようとする(重要 claim だけで十分)
- [ ] "Strict judge" を全 re-run で試す(代表 finding だけ)
- [ ] 発明の originality 追求で 1 週間以上議論する

**対策**: どれか当てはまったら「MVP で pitch できるか?」を自問し、できないなら scope を削る。

---

## 📘 Meta-Philosophy doc への参照

**発明者との 2026-04-19 対話で抽出された KDF の性格論 + 情報理論的位置づけ**:
→ [kdf_meta_philosophy.md](kdf_meta_philosophy.md)

内容:
- Part 1: 5 つの character trait(質素/愚直/節度/正直/境界志向)
- Part 2: なぜ KDF は数式化できたのか(6 層情報理論、L3 specialist としての定式化)
- Part 3: 性格と情報層の統合
- Part 4: 比喩集(行商人 / 灯台守 / 医学者 observer / 俳句詠み)
- Part 5: 場面別 使用指針

---

## 🎭 Metaphor — 発明者発案(2026-04-19)

> **「KDF は物語の主人公、または行商人に似ている」**

- 最強 fighter でも最賢者でもない(highest accuracy / speed ではない)
- しかし **narrative / community 間を繋ぐ unique position が value の源泉**
- 一度離れると世界が分断する(connectivity preservation、F-061 evidence)
- Hub-dominated 都市圏(scale-free graph)では埋没するが、分散した村々(community graph)では決定的

詳細と使用方針: [kdf_characteristics.md §Metaphor](kdf_characteristics.md)

この metaphor は Burt's Structural Holes の直感的 human-relatable 表現として pitch で強力。

---

## 📌 発明者 (Chai) への reminder

> **「AI を大きく使うとドツボ。決定論を別でコツコツ。決定論は必ず資産」**

このマントラは、特に以下の瞬間に思い出す:

1. 新しい feature を KDF に追加する時 → 「決定論か? LLM 依存させていないか?」
2. pitch 先を選ぶ時 → 「補完ポジションか? 代替を主張していないか?」
3. 実験を設計する時 → 「MVP か? 1 つの claim を defend するだけで十分か?」
4. 論文を書く時 → 「limit を honest に記載しているか?」
5. 疲れて speed を上げたくなった時 → 「決定論の積み上げだけやる、LLM の変動追いかけない」

---

## 🔗 関連文書

- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 検証済み knowledge の正典
- [phase_M_market_suitability.md](phase_M_market_suitability.md) — 市場 suitability
- [phase_W_next_verifications.md](phase_W_next_verifications.md) — 追加検証候補
- [paper_draft.md](paper_draft.md) — 論文草稿
- [PUBLIC_SUMMARY.md](PUBLIC_SUMMARY.md) — 公開版要約
- [patent/filed/](patent/filed/) — 特許仕様(frozen)

---

**この doc は user (発明者) の決定哲学の memo であり、今後の設計方針の base となる。更新時は "発明者の設計哲学" セクションの integrity を保つこと。**
