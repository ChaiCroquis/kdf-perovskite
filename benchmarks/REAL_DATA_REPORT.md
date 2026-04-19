# Phase 6 実データ検証レポート

**生成日:** 2026-04-17
**目的:** Phase 4 合成データの限界を越え、実データと対抗的条件で KDF の一般化可能性を検証する
**再現:**
```bash
cargo run --release -p adversarial-bench            # 対抗的合成
cargo run --release -p real-data-bench -- small     # Obsidian Vault
```
**結果JSON:**
- [results/adversarial.json](results/adversarial.json)
- [results/real_data.json](results/real_data.json)

---

## 概要

Phase 4 の合成データでは **KDF が有利な構造(rare=deg-1 が classifier と同型)**で評価していた。Phase 6 では:

1. **対抗的合成データ** 6 種で KDF の**失敗条件**を地図化
2. **Obsidian Vault** 実データ(2,182 ノート, 557 エッジ)で一般化可能性を確認
3. **公開データ(FB15K-237/ogbn-arxiv/NASA log)** はローダ実装済、データ配布は別途

## TL;DR

- **実データ (Obsidian Vault)**: KDF は F1=0.747, Precision=0.659, Compression=0.868 で全手法中トップ。Wilcoxon p=0.006 で Random より有意 (n=10)。
- **対抗的成功**: A(deg=1), A(deg=10), B, C, D の 7/8 条件で有意優位 (p<0.01)
- **対抗的失敗(正直な報告)**:
  - **E 時間発展 t1-t4: KDF 0% vs Random 30%**(Random の方が有意に良い、p=0.006)
  - **A 高次数 rare (deg=3): KDF 28% vs KMedoids 71%**(KDF が deg=3 を見逃す)

KDF は特定の構造(rare=deg-1 付近)で強いが、**rare が中次数で埋め込まれる / 時間進化で rare が再分類される**場合は弱い。

---

## 1. 対抗的合成データ結果

### 1.1 Rare Recall サマリ

| 条件 | KDF | Random | KMedoids | Stratified(ラベル要) | KDF 優位? |
|---|---:|---:|---:|---:|:---:|
| A) 高次数 rare deg=1 | **1.000** | 0.310 | 0.000 | 1.000 | ✓ p<0.001 |
| A) 高次数 rare deg=3 | 0.280 | 0.300 | **0.712** | 1.000 | ✗ 不利 |
| A) 高次数 rare deg=10 | 0.968 | 0.300 | **1.000** | 1.000 | KMedoids 同等 |
| B) 構造的孤立 deg=2 | **0.988** | 0.300 | 0.824 | 1.000 | ✓ p=0.006 |
| B) 構造的孤立 deg=5 | **1.000** | 0.300 | 1.000 | 1.000 | ✓ p=0.006 |
| C) 冗長度0 | **0.982** | 0.285 | 0.000 | 1.000 | ✓ p=0.006 |
| D) ノイズ 10% | **1.000** | 0.300 | 0.024 | 1.000 | ✓ p=0.006 |
| D) ノイズ 30% | **1.000** | 0.300 | 0.120 | 1.000 | ✓ p=0.006 |
| E) 時間 t=0 | **1.000** | 0.300 | 0.000 | 1.000 | ✓ p=0.006 |
| **E) 時間 t=1** | **0.000** | **0.300** | 0.000 | 1.000 | **✗ Random 有意に良い** |
| **E) 時間 t=2** | **0.000** | **0.300** | 0.000 | 1.000 | **✗ Random 有意に良い** |
| **E) 時間 t=3** | **0.000** | **0.300** | 0.000 | 1.000 | **✗ Random 有意に良い** |
| **E) 時間 t=4** | **0.000** | **0.300** | 0.000 | 1.000 | **✗ Random 有意に良い** |

### 1.2 失敗モード分析

**失敗 1: A) deg=3 の中次数 rare**
- classifier の Rare 判定条件 `neighbor_count == 1` を意図的に破ったデータ
- KDF は deg=3 を Edge 層と見なすため、Rare として保護されない
- KMedoids は「次数上位」で拾えるため、逆に相対優位
- **含意**: 「rare = isolated とは限らない」ドメインでは classifier の事前定義を再考が必要

**失敗 2: E) 時間発展 t1-t4**
- 時刻 t=0 では rare が deg=1 → KDF が発見 (Recall=1.0)
- 時刻 t=1 以降、グラフ進化(新ノード追加+古エッジ除去)で rare 周囲が希薄化
- 結果として rare 周辺の構造が KDF の判定境界を外れ、Recall=0 に転落
- **Random は常に 30% サンプルなので、確率的に 30% の rare を拾い続ける**
- **含意**: KDF はインクリメンタル運用でも**定期的な再分類**が必須。静的 snapshot 評価だけでは欠陥を見落とす。

### 1.3 Precision@Rare の重要性

Rare Recall だけでは不十分:

```
KMedoids (deg=10): Recall=1.000, Precision=0.167, F1=0.286
KDF     (deg=10): Recall=0.968, Precision=0.125, F1=0.221
Stratified       : Recall=1.000, Precision=0.149, F1=0.259
```

この条件では KMedoids が Recall=1.0 を達成しているが、これは「上位次数 30% を全部拾っているだけ」。高次数 rare 条件では偶然ヒット率が上がるだけで、rare を本当に識別してはいない。**F1 では KMedoids と KDF の差は小さい**。

---

## 2. 実データ(Obsidian Vault)結果

### 2.1 データ概要

- ソース: ローカル Obsidian Vault(発明者自身の運用ログ)
- ノード数: 2,182 (全 .md ノート)
- エッジ数: 557 (wiki-link `[[target]]`)
- グラウンドトゥルース rare: 219 件(indegree ∈ [1, 2])
- **PII マスキング済**: email / phone / credit card / 32+char hex を `<EMAIL>/<PHONE>/<CARD>/<HEX>` で置換してから解析
- ノードラベルは FNV-1a 8hex でハッシュ化 (ノート題名の外部漏洩防止)

### 2.2 結果

| Method | Rare Recall | Precision@Rare | **F1@Rare** | Compression |
|---|---:|---:|---:|---:|
| **KDF** | 0.863 ± 0.000 | **0.659** | **0.747** | **0.868** |
| Stratified (ラベル要) | 1.000 | 0.273 | 0.429 | 0.632 |
| KMedoids | 1.000 | 0.334 | 0.501 | 0.700 |
| PageRank | 1.000 | 0.334 | 0.501 | 0.700 |
| Random | 0.302 ± 0.012 | 0.102 | 0.153 | 0.703 |
| CoreSet | 0.010 | 0.003 | 0.005 | 0.700 |

**Wilcoxon signed-rank (KDF vs Random, Rare Recall)**: n=10, 中央差分 +0.571, z=2.75, p=**0.006**, 有意 (α=0.01)

### 2.3 観察

1. **KDF は Recall は最高ではない**(0.863)が、**F1 と Precision と Compression で全手法トップ**
2. Stratified/KMedoids/PageRank が Recall=1.0 なのは、選択比率 30% で 2,182 × 0.3 ≈ 654 ノードを取ると 219 の rare が偶然全部含まれるため
3. KDF は 2,182 → 287 ノード(13.2%)に絞りつつ、rare の 86.3% を残す
4. Precision 0.659 は「KDF が rare と判定したものの 2/3 が真に rare」という現実的に有用な水準
5. 分析時間 0.42ms は CoreSet(359ms)の 1000倍速い

### 2.4 実用含意

**Obsidian Vault のような知識ネットワーク運用では、KDF は「rare として指定された全ノートの 86% を保持しつつ、全体 87% を圧縮する」** — 長期運用で古いノートを半自動アーカイブする場合、KDF は実用的な選択になり得る。ただし「残った 14% の rare(30 件相当)は失う」リスクも定量化されている。

### 2.5 PII マスキングの挙動検証

`PiiMasker` の unit test(既存)で動作保証:

```
pii_masker_emails:  ✓
pii_masker_phone_jp: ✓
pii_masker_card:    ✓
pii_masker_hex_blob: ✓
```

実ヴォールト content に対して逐次適用され、**ディスクには一切書き戻さない** (in-memory のみ)。ノード ID は FNV-1a 8hex ハッシュで匿名化。

---

## 3. 公開データセット

`benchmarks/real_data/src/public_datasets.rs` に loader 実装済み:

- **FB15K-237**: `data/fb15k-237/{train,valid,test}.txt` が必要(未配布)
- **ogbn-arxiv**: `data/ogbn-arxiv/{edges,citation_count}.csv` が必要
- **NASA HTTP log**: `data/nasa-http/access.log` が必要

データ配布は別途、ダウンロード手順は `public_datasets::download_instructions()` で案内。

---

## 4. 全体まとめ

| 観点 | 結論 |
|---|---|
| **KDF は合成データ外で有効か?** | **実データ(Obsidian Vault)で有効**。F1/Precision/Compression 全てで首位。 |
| **KDF の失敗モードは?** | (1) rare が中次数 (deg≈3), (2) 時間発展で rare 再分類が必要な snapshot |
| **Rare Recall 100% は常に成立するか?** | **否**。実データでは 86%、対抗データでは最悪 0%。Phase 4 結果は構造同型条件に依存 |
| **他手法より優れるか?** | Recall だけなら KMedoids と拮抗。**F1/Precision/Compression の同時最適**では KDF 優位が示された |

**設計へのフィードバック**: Phase 1 実装済みの `TransitionController`(Claim 23-26)と `MetaController`(Claim 27-32)を実際にループで回すと時間発展耐性が改善される可能性が高い。Phase 6 の bench は pure static classifier のみを評価しており、**動的制御系全体の評価は Phase 7 候補**。

---

## 5. 次フェーズ候補(Phase 7)

1. 動的制御(TransitionController + MetaController)を組み込んだ時間発展耐性の再評価
2. 公開データセット(FB15K-237, ogbn-arxiv, NASA log)の実測
3. 大規模(n ≥ 100,000)での O(n log n) 主張の実測
4. 失敗条件(deg=3, temporal drift)に対する classifier 拡張
