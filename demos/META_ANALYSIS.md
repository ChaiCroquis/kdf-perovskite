# KDF Showcase メタ分析 — Stage 1 横断考察

**生成日:** 2026-04-17
**対象:** D1 (Obsidian) / D2 (NASA log) / D5 (FB15K-237) の 3 demos

---

## 1. 横断結果まとめ

| 側面 | D1 Obsidian | D2 NASA log | D5 FB15K-237 |
|---|---|---|---|
| データ構造 | 単方向 link graph (wiki-link) | Bipartite (IP × resource) | Multi-relation graph |
| Rare の操作的定義 | indegree ∈ [1,2] | 4xx/5xx status code | 稀 relation を触れる entity |
| KDF が勝つか | ✅ **圧勝** (F1=0.747) | ❌ **baseline 劣勢** / ✅ 拡張版で中位 | ◐ **僅差** (+8.6%) |
| 必要な拡張 | なし(baseline のまま) | Phase 7 S2 RelDensity 必須 | Phase 7 S3 fingerprint 必須 |
| ラベル有り手法との関係 | N/A(ラベル不在) | 完敗(Stratified = 1.000 vs KDF = 0.307) | 中立(TransE 近似より微優) |

---

## 2. 発見したパターン:「構造が rareness をエンコードしているか」

3 demo を横断すると、**KDF の勝敗は一つの軸で説明できる**:

```
Q: データの「グラフ構造」が、ground truth の rareness と "同型" か?
```

### 2.1 同型が強い場合 → KDF 圧勝

**D1 Obsidian**: rare = indegree ≤ 2 = **構造的に定義された孤立度**
- Node classifier の deg-based rule が直接 ground truth を表す
- Phase 7 拡張不要、baseline KDF が最強
- F1=0.747, 全指標トップ

### 2.2 同型が弱い場合 → 拡張が必要

**D2 NASA log**: rare = 4xx/5xx status code = **グラフ構造と独立**
- 実世界では「error-prone resource」は人気 endpoint のこともある(次数中-高)
- `deg == 1` rule では検出不可 → baseline は Random 以下
- Phase 7 S2 RelDensity(局所相対次数)で部分救済
- それでも**ラベル有り手法には完敗**

### 2.3 同型が無い場合 → KDF は marginal

**D5 FB15K-237**: rare = 稀 relation の端点 = **relation 情報が必要**
- KDF は relation type を見ない(単純 graph のみ)
- rare entity は構造的に他と区別つかない
- Phase 7 S3 fingerprint でも僅か +8.6%

---

## 3. この発見が示唆すること

### 3.1 KDF の **適用可能性判定ルール**

新規ドメインで KDF を採用するかを判断する **事前スクリーニング**:

```
Step 1: rare の operational definition を特定
Step 2: "deg, clustering coefficient, neighbor diversity" といった
        グラフ構造量で rare が定義できるか?
  → Yes: baseline KDF で勝てる (D1 型)
Step 3: 定義は構造的だが、絶対閾値 (deg==1) では駄目?
  → Phase 7 S2 RelDensity 拡張で対応可能 (D2 型)
Step 4: rare が graph 構造と独立?
  → KDF は marginal. 他手法を検討 (D5 型)
```

### 3.2 特許 Claim 33 の解釈余地

請求項 33 は「孤立度指標は、強度、頻度、接続量、**またはこれらの時間的推移** の少なくとも一つに基づく」と規定。
これは Phase 7 S1 (PersistMem — 時間推移) / S2 (RelDensity — 相対接続量) を **Claim 範囲内の実施形態** と解釈できる。

つまり、特許請求の範囲自体が上記の適用シナリオを想定した柔軟性を持っている。

### 3.3 「ラベル」の存在感

D2 は **ラベルが使える環境では Stratified が完勝**する現実を示す。KDF の価値は純粋に:

> 「**ラベルが取れない** 環境での label-free rare preservation」

に限定される。これを隠さず README に書くことで、信頼性が高まる。

---

## 4. Stage 2 以降の含意

### 4.1 D3 (ML 長尾データ選択) の予想
- rare = 少数クラスのラベル = **完全にラベルベース**
- 構造(例: feature 空間近傍グラフ)と rareness が弱く相関
- **予想:** 中間 — KDF+Analogy で +5~15% 程度、ただし Stratified に負ける

### 4.2 D4 (MovieLens 推薦) の予想
- long-tail item = **degree** で定義(user-item bipartite)
- これは D1 型(構造同型)に近い
- **予想:** 強い — KDF baseline で勝てる可能性

### 4.3 D6 (forum dedup) の予想
- minority 意見 = **reply パターン** に現れる可能性 → 中度の構造同型
- **予想:** 中間 — KDF 拡張で MinHash/SimHash を補完

### 4.4 D7 (GitHub issue archive) の予想
- "忘れられた重要 issue" = **label + 引用 + reply pattern** の合成
- Claim 46 analogy discovery の活躍領域
- **予想:** D1 以来の "KDF 強い" 候補

---

## 5. ユーザ視点への示唆(セールス pitch ガイド)

Stage 1 から推定すると、KDF を pitch する相手ごとの訴求ポイント:

| 相手 | 使う数値 | 注意点 |
|---|---|---|
| 知識管理ツールベンダー | D1 F1=0.747 | 「個人 vault 運用で有効」と限定 |
| Observability ベンダー | D2 KDF+RelDensity 3x Random | 「ラベル無し環境で」と強調、ラベル有りなら Stratified |
| Graph DB ベンダー | D5 +8.6% | 弱い、追加検証必要と正直に |
| **研究者** | 3軸フレーム全体 | 「trade-off を定量化する手法」という位置付け |

---

## 6. ユーザの思考パターン(発明者本人の要求特性)

本 Stage 1 セッションで観察された要求パターン:

1. **検証の連鎖**: 「やった」→「検証して」→「乖離は無いか」— 一次ソース確認を常に要求
2. **失敗の透明性**: 「コンパイルだけチェックしてないか」「成功だけ見て誤魔化してないか」— 数字だけでなく意味を検証
3. **目的の再定義**: 作業が進むと「で、どこに向かっている?」を問う — 忠実に進めた後で方向転換を許す
4. **段階承認**: 「1-4 OK」式の圧縮した応答 — 詳細を全部書かせるより構造で決める
5. **コスト意識**: 「実行時間は使いたくない」「既存リソースで」— 合成データ / キャッシュ優先
6. **公開志向**: repo は公開前提、blog / 学術発表を最終目標に

**これを Stage 2 でどう反映するか:**
- 各 demo の仕様決定時に「hypothesis → test → honest result」で進める
- 実行時間最小化: 合成データ or 既存キャッシュ優先
- 各 demo は README + SVG で「5分で読めるピッチ」になること
- "KDF が勝った" 結果だけでなく、明らかな負けも記録

---

## 7. Stage 2 設計への反映

上記分析を踏まえ、Stage 2 の実装では:

1. **各 demo の冒頭に "hypothesis" セクション追加**(この demo で KDF が勝つ/負けるかの事前予想を明示)
2. **実測が予想を外したら、外した旨を explicitly 書く**(外した失敗も学びの材料)
3. **ラベル有り vs 無しの位置付けを D2 型に準じて併記**(Stratified / Supervised 手法があれば必ず横に置く)
4. **"同型判定" セクション**(データの構造と rareness の同型性を 1 行で記述)

---

## 8. Stage 1 からの移行に際してのリスク

- Stage 2 で使う合成データが **trivial に解ける**設計になると KDF が虚栄勝利する危険
- 対策: D2/D5 のように rare を構造と独立に定義(relation / label 経由)して meaningful comparison にする
- 合成データは必ず seed 固定 + 生成パラメータ公開
