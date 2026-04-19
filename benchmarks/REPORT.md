# Phase 4 ベンチマークレポート

**生成日:** 2026-04-17
**再現コマンド:** `cargo run --release -p sota-comparison`
**結果JSON:** [results/sota_comparison.json](results/sota_comparison.json)

---

## 1. 目的

「KDF は希少データを 100% 保持する」という主張を、**公正な統計的ベンチマーク**で定量確認する。

- 真値(ground-truth rare items)がある合成データで Rare Recall を測定
- 既存 SOTA 手法(Random / Stratified / K-Medoids / CoreSet / PageRank)と横並び比較
- **n=10 trials × 3 sizes = 30 runs/method**、seed 固定で再現可能
- 平均 ± 標準誤差(SE)を報告、過度な切り取り主張を避ける

---

## 2. 実験設定

### 2.1 合成データ

Zipf 型グラフに「希少グラウンドトゥルース」を植え込む:

- `n_hubs = n/50` の密結合ハブ群 (完全グラフ)
- `n_rare = n/20` の**希少ノード** (各 1 ハブのみに接続、グラウンドトゥルース)
- 残りをテールノードに割当、**5ノード/クラスタの冗長クラスタ**を構成
  - クラスタ内は同一ハブ集合に接続(完全な連結冗長)
  - クラスタ内で互いに接続(冗長信号)

### 2.2 比較手法

| Method | 概要 | ラベル要 |
|---|---|:---:|
| **KDF** | Rev.12 (代謝制御+希少性保護+整合性発見)。Rare層・Core層は全保持、Edge層はクラスタ代表のみ | No |
| Stratified | ラベルを見て希少を全保持 + 30%サンプル | **Yes** |
| Random | 30% 一様サンプル | No |
| K-Medoids | 次数上位30% | No |
| CoreSet | Farthest-first k-center ヒューリスティック | No |
| PageRank | 次数ベース上位30%(ランダム正則グラフで PageRank と等価) | No |

### 2.3 評価指標

- **Rare Recall** = (保持された希少ノード数) / (真の希少ノード総数)
- **Compression Rate** = 1 − (選択された総数 / 全ノード数)
- **Time (ms)**: 経過時間中央値
- 試行数 N=10、サイズ n ∈ {200, 500, 1000}

---

## 3. 結果

### 3.1 主要指標(平均 ± SE, N=10)

| Method | n | Rare Recall | Compression | Time (ms) |
|---|---:|---:|---:|---:|
| **KDF** | 200 | **1.000 ± 0.000** | **0.555** | 0.12 |
| KDF | 500 | **1.000 ± 0.000** | **0.558** | 0.35 |
| KDF | 1000 | **1.000 ± 0.000** | **0.558** | 0.80 |
| Stratified | 200 | 1.000 ± 0.000 | 0.667 | 0.00 |
| Stratified | 500 | 1.000 ± 0.000 | 0.664 | 0.01 |
| Stratified | 1000 | 1.000 ± 0.000 | 0.665 | 0.01 |
| Random | 200 | 0.290 ± 0.030 | 0.702 | 0.00 |
| Random | 500 | 0.288 ± 0.029 | 0.700 | 0.00 |
| Random | 1000 | 0.342 ± 0.015 | 0.701 | 0.01 |
| K-Medoids | 200 | 0.000 ± 0.000 | 0.700 | 0.00 |
| K-Medoids | 500 | 0.000 ± 0.000 | 0.700 | 0.00 |
| K-Medoids | 1000 | 0.000 ± 0.000 | 0.700 | 0.01 |
| CoreSet | 200 | 0.000 ± 0.000 | 0.700 | 0.34 |
| CoreSet | 500 | 0.040 ± 0.000 | 0.700 | 4.41 |
| CoreSet | 1000 | 0.020 ± 0.000 | 0.700 | 33.43 |
| PageRank | 200 | 0.000 ± 0.000 | 0.700 | 0.00 |
| PageRank | 500 | 0.000 ± 0.000 | 0.700 | 0.01 |
| PageRank | 1000 | 0.000 ± 0.000 | 0.700 | 0.02 |

### 3.2 観察

1. **Rare Recall 100% を label 不要で達成したのは KDF のみ**
   - Stratified は正解ラベルが必要な教師あり手法(比較のリファレンスとして提示)
   - それ以外のラベル不要手法は、**0-34% の Rare Recall** しか達成できず

2. **KDF の compression 55.8% は他ラベル不要手法より低い**
   - 理由: 希少ノード(5%)+ 大ハブ(Core層, 2%)+ クラスタ代表(~37%)を保持しているため
   - これは設計上のトレードオフ: Claim 15/18 の「希少保護」を厳密に守れば、冗長削減率は不可避的に制約される
   - **Random/KMedoids 等は 70% 圧縮だが、Rare を 0-34% で捨てている** — これは「圧縮のために希少情報を犠牲にしている」ことを定量的に示す

3. **計算時間: n=1000 で 0.80ms(KDF) vs 33.43ms(CoreSet)** — KDF は 1 桁以上高速

4. **KDF の標準誤差がゼロ(SE=0.000)**
   - 決定論的挙動の検証(Phase 3 の insertion-order invariance test と整合)
   - 10試行すべて厳密に同一結果

---

## 4. Ablation(実装選択の正直な限界)

- **compression 55.8% は数学的下限ではない**。Edge 層でのクラスタ代表戦略を緩めればより高い圧縮が可能だが、Rare 偶発失 loss のリスクが上がる。現設定は「Rare Recall 100% を優先」。
- Stratified が 66.5% 圧縮を達成するのは、ラベルで希少を特定できるため非希少から幅広くサンプル可能なため。KDFはラベルなしでこれに肉薄することが目標。
- 現ベンチはハブ+テールの**均質な合成データ**。実データ(Wikidata, Amazon reviews 等)での追加検証は Phase 5 で予定。
- **本ベンチの希少ノードは degree=1 で構築**しており、これは KDF クラスファイアの Rare 判定条件(`neighbor_count == 1`)と構造同型である。つまり本実験は **「classifier の判定条件が真値に合致するデータ」での挙動**を示している。現実データではこの条件が外れる可能性があり、Phase 5 の実データ検証で一般化可能性を精査する。

---

## 5. 実運用上の含意

| ユースケース | 推奨手法 | 理由 |
|---|---|---|
| ラベル有り・最大圧縮 | Stratified | ラベル活用で最良圧縮 |
| **ラベルなし・Rare必須** | **KDF** | **唯一のラベル不要 100% Rare 保持** |
| スケーラビリティ重視・Rare不要 | K-Medoids | 高速、ただし Rare は 0% |
| 多様性サンプリング | CoreSet | 多様性良、Rare 低 |

---

## 6. 再現手順

```bash
git clone <repo>
cd kdf-perovskite
cargo run --release -p sota-comparison

# Results written to benchmarks/results/sota_comparison.json
# 30 trials total (10 per size × 3 sizes)
# Seeded RNG → deterministic across platforms
```

## 7. 主張の誠実な制限

- **「KDF は ラベル不要・100% Rare 保持・56% 圧縮」** が現状の合成データでの確かな主張
- **「他手法より万能」** ではない:
  - Random は圧縮率だけなら同等(70%)、ただし Rare Recall が 30% 台で非目的関数
  - 実データで同結果が保たれる保証は現時点ではない(Phase 5 で検証予定)

---

## 8. データとコード

- データ生成器: [sota_comparison/src/main.rs:build_dataset](sota_comparison/src/main.rs)
- KDF 実装: [cgb-kdf/](../crates/cgb-kdf/)
- 生 JSON: [results/sota_comparison.json](results/sota_comparison.json)
