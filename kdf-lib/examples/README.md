# KDF サンプルガイド

KDFライブラリの使い方を学ぶためのサンプル集です。

## KDFとは

```
KDF = 情報の整理整頓フレームワーク
    + 「ゴミに見えても確信がないから捨てない」ポリシー

KDFは発見アルゴリズムではない。
KDFは早まった廃棄を防ぐ保守的戦略。
```

**4層の意味:**
- **CORE**: 知識が飽和している領域（追加情報の価値が低い）
- **EDGE**: まだ知識に余地がある領域
- **RARE**: 判断材料が少なすぎる領域（ゴミか宝か不明）
- **GARBAGE**: 十分な証拠でゴミと確定した領域

---

## クイックスタート

```bash
# 基本的な使い方を確認
cargo run --release --example basic_usage

# KDFの効果を確認
cargo run --release --example effectiveness_test
```

---

## 学習パス

### 🎯 初めてKDFを使う人

```
1. kdf_core_philosophy  → KDFの本質を理解
2. basic_usage          → 基本的なAPI
3. kdf_auto_threshold   → 自動閾値選択 (NEW)
4. custom_similarity    → カスタム類似度関数
5. kdf_features         → 主要機能の確認
```

### 📊 KDFの効果を検証したい人

```
1. effectiveness_test → 希少データ保持の検証
2. kdf_vs_modern      → 現代手法との比較
3. kdf_knn_hybrid     → k-NNとの組み合わせ効果
```

### 🔬 実用アプリケーションに興味がある人

```
1. kdf_genetic           → 遺伝的アルゴリズム + KDF
2. kdf_experience_replay → 強化学習 + KDF
3. kdf_active_learning   → 能動学習 + KDF
4. kdf_hidden_values     → 異常検知、公平性など
5. kdf_test_selection    → テスト選択・優先順位 (NEW)
6. kdf_user_segmentation → 顧客セグメンテーション (NEW)
7. kdf_code_review       → コードレビュー優先順位 (NEW)
8. kdf_keyframe          → 動画キーフレーム抽出 (NEW)
```

### ⚡ 高速化が必要な人

```
1. kdf_fast_benchmark         → process_fast() 100-1000x高速化 (NEW)
2. kdf_fast_approximation     → 高速近似手法
3. kdf_optimization_strategies → 最適化戦略
4. kdf_complexity_verification → 計算量検証
5. kdf_complexity_reduction   → O(n²)削減
6. kdf_kernel_pca             → O(n³)削減
```

### 💡 応用機能を探求したい人

```
1. kdf_hidden_values    → 隠れた価値の発見
2. kdf_bridge_proposal  → 少数派と多数派の橋渡し
3. explain_test         → 説明可能性
4. info_theory_test     → 情報理論的分析
```

### 🔬 実用性検証（重要）

```
1. kdf_quantitative_validation → 定量的検証（他手法比較、統計的再現性）
2. kdf_practical_usecases      → 実用ユースケース（ログ分析、テキスト重複排除）
3. kdf_public_datasets         → 公開データセット（Credit Card Fraud, Iris, 20 Newsgroups）
4. kdf_real_validation         → 冗長整理、判断保留、一貫性
```

### 🔮 先進的応用

```
1. kdf_diff                  → 差分KDF (2時点間の変化分析) (NEW)
2. kdf_graph                 → グラフKDF (ノード/エッジ分析) (NEW)
3. kdf_multimodal            → マルチモーダルKDF (複合データ) (NEW)
4. kdf_recommend             → KDFレコメンド (多様性推薦) (NEW)
5. kdf_adversarial           → 敵対的サンプル検出 (NEW)
6. kdf_causal                → 因果KDF (処置効果分析) (NEW)
7. kdf_differential_privacy  → 差分プライバシーKDF (NEW)
8. kdf_self_supervised       → 自己教師あり学習統合 (NEW)
9. kdf_concept_drift         → コンセプトドリフト検出
10. kdf_data_valuation       → データ価値評価
11. kdf_continual_learning   → 継続学習 (破滅的忘却軽減)
12. kdf_privacy              → プライバシー敏感データ検出
13. kdf_model_debug          → モデルデバッグ・失敗分析
14. kdf_negative_sampling    → Contrastive Learning用サンプリング
15. kdf_federated            → 連合学習 (局所希少パターン保持)
```

---

## サンプル一覧

### 本質理解

| サンプル | 説明 | 実行コマンド |
|---------|------|-------------|
| `kdf_core_philosophy` | KDFの3つの核心を示す | `cargo run --example kdf_core_philosophy` |

### 基本

| サンプル | 説明 | 実行コマンド |
|---------|------|-------------|
| `basic_usage` | KDFの基本的な使い方 | `cargo run --example basic_usage` |
| `kdf_auto_threshold` | **自動閾値選択** (NEW) | `cargo run --example kdf_auto_threshold` |
| `custom_similarity` | カスタム類似度関数の定義 | `cargo run --example custom_similarity` |
| `incremental` | インクリメンタル処理 | `cargo run --example incremental` |
| `benchmark` | パフォーマンス測定 | `cargo run --release --example benchmark` |

### 機能テスト

| サンプル | 説明 | 検証内容 |
|---------|------|---------|
| `kdf_features` | 主要機能のデモ | 層分類、重み計算 |
| `effectiveness_test` | 有効性検証 | 希少データ保持率 |
| `temporal_test` | 時間的特性 | 時系列での効果 |
| `explain_test` | 説明可能性 | 判断理由の解説 |
| `info_theory_test` | 情報理論分析 | エントロピー、相互情報量 |

### 比較検証

| サンプル | 比較対象 | 結果サマリ |
|---------|---------|-----------|
| `kdf_vs_modern` | Random, Stratified, Core-set | KDF: 希少100%保持 |
| `kdf_knn_hybrid` | Standard k-NN | 希少クラス精度向上 |

### ハイブリッドアルゴリズム

| サンプル | 組み合わせ | 効果 |
|---------|-----------|------|
| `kdf_genetic` | 遺伝的アルゴリズム | 多様性維持、適応度2x |
| `kdf_experience_replay` | 強化学習バッファ | 希少経験77.5%保持 |
| `kdf_active_learning` | 能動学習 | 希少クラス発見84% |

### 高速化

| サンプル | 手法 | 効果 |
|---------|------|------|
| `kdf_fast_benchmark` | **process_fast() (NEW)** | **100-1000x高速化** |
| `kdf_fast_approximation` | Mini-batch, Grid, Hierarchical | 最大36.5x高速化 |
| `kdf_optimization_strategies` | 枝刈り, LSH | 希少100%維持で67%削減 |
| `kdf_complexity_verification` | O(n²)検証 | 標準KDFの計算量確認 |
| `kdf_complexity_reduction` | O(n²)削減 | 47.7x高速化 |
| `kdf_kernel_pca` | O(n³)削減 | 8.6-30x高速化 |

### 応用

| サンプル | 応用分野 | ユースケース |
|---------|---------|-------------|
| `kdf_hidden_values` | 異常検知、公平性 | 教師なし異常検出100% |
| `kdf_bridge_proposal` | 意見調整、市場参入 | 少数派→多数派接続 |

### 実用性検証（重要）

| サンプル | 検証内容 | 実行コマンド |
|---------|---------|-------------|
| `kdf_quantitative_validation` | 定量的検証（他手法比較、統計的再現性） | `cargo run --release --example kdf_quantitative_validation` |
| `kdf_practical_usecases` | 実用ユースケース（ログ分析、テキスト重複排除） | `cargo run --example kdf_practical_usecases` |
| `kdf_public_datasets` | 公開データセット検証（Credit Card Fraud, Iris, 20 Newsgroups） | `cargo run --release --example kdf_public_datasets` |
| `kdf_real_validation` | 冗長整理、判断保留、一貫性の検証 | `cargo run --example kdf_real_validation` |

### 先進的応用

| サンプル | 応用分野 | ユースケース |
|---------|---------|-------------|
| `kdf_diff` | **時系列分析** (NEW) | 2時点間の層変化、分布ドリフト検出 |
| `kdf_graph` | **グラフ分析** (NEW) | ノード/エッジの希少度、GNN前処理 |
| `kdf_multimodal` | **マルチモーダル** (NEW) | テキスト+数値、画像+メタデータ統合 |
| `kdf_recommend` | **推薦システム** (NEW) | 多様性を考慮した推薦、説明付き |
| `kdf_adversarial` | **セキュリティ** (NEW) | 敵対的サンプル検出、異常入力検知 |
| `kdf_causal` | **因果推論** (NEW) | 処置効果分析、層別ATE推定 |
| `kdf_differential_privacy` | **プライバシー** (NEW) | 層適応ノイズ、DP保証 |
| `kdf_self_supervised` | **表現学習** (NEW) | ハードネガティブ、カリキュラム学習 |
| `kdf_test_selection` | **テスト選択** (NEW) | テストケース優先順位、カバレッジ最適化 |
| `kdf_user_segmentation` | **顧客分析** (NEW) | ユーザーセグメント、マーケティング戦略 |
| `kdf_code_review` | **コードレビュー** (NEW) | 変更優先順位、レビュー時間見積もり |
| `kdf_keyframe` | **動画分析** (NEW) | キーフレーム抽出、シーン検出 |
| `kdf_concept_drift` | 分布変化検出 | 段階的/突発的ドリフト、季節パターン |
| `kdf_data_valuation` | データ価値評価 | Rare=高価値、マーケット価格設定 |
| `kdf_continual_learning` | 継続学習 | Rare優先リプレイで破滅的忘却軽減 |
| `kdf_privacy` | プライバシー | Rare層=高リスク、k-匿名性との関連 |
| `kdf_model_debug` | モデル診断 | Rare層での失敗分析、テストケース生成 |
| `kdf_negative_sampling` | 対照学習 | Edge層からハードネガティブ選択 |
| `kdf_federated` | 連合学習 | Rare優先集約で局所パターン保持 |

---

## 検証結果サマリ

### KDFの優位性

```
┌────────────────────────────────────────────────────┐
│  比較実験結果                                      │
├────────────────────────────────────────────────────┤
│                                                    │
│  希少データ保持率:                                 │
│    Random Sampling:  10-20%                        │
│    Stratified:       ラベル必要                    │
│    Core-set:         0% (多様性重視で希少を排除)   │
│    KDF:              96-100%                       │
│                                                    │
│  Experience Replay (RL):                           │
│    FIFO:             10%                           │
│    PER:              10%                           │
│    KDF:              77.5%                         │
│                                                    │
│  Active Learning:                                  │
│    Random:           0% 希少発見                   │
│    Uncertainty:      0% 希少発見                   │
│    KDF:              84% 希少発見                  │
│                                                    │
└────────────────────────────────────────────────────┘
```

### 高速化効果

```
┌────────────────────────────────────────────────────┐
│  最適化手法の比較                                  │
├────────────────────────────────────────────────────┤
│                                                    │
│  手法           高速化    希少保持                 │
│  ─────────────────────────────────────             │
│  Standard       1.0x      100%                     │
│  process_fast   100-1000x 0% (⚠️冗長削減専用)     │
│  Mini-batch     36.5x     52%                      │
│  Grid           4.8x      53%                      │
│  枝刈り         2-3x      100%                     │
│  LSH            3-4x      100%                     │
│                                                    │
│  推奨:                                             │
│    品質重視 → 枝刈り/LSH                           │
│    速度重視 (Rare検出不要) → process_fast()        │
│    速度重視 (Rare検出必要) → Mini-batch            │
│                                                    │
└────────────────────────────────────────────────────┘
```

### KDFの本質（重要）

```
┌────────────────────────────────────────────────────┐
│  よくある誤解と正しい理解                          │
├────────────────────────────────────────────────────┤
│                                                    │
│  × KDFは宝を探しに行くアルゴリズム                │
│  × KDFはRAREを保護して育てるシステム              │
│  × KDFは効率的な発見手法                          │
│                                                    │
│  ○ KDFは冗長な情報を減衰させる整理術              │
│  ○ KDFは早まった廃棄を防ぐ保守的ポリシー          │
│  ○ KDFは「念のため残しておく」という判断基準      │
│                                                    │
├────────────────────────────────────────────────────┤
│  RAREの本質                                        │
├────────────────────────────────────────────────────┤
│                                                    │
│  RARE ≠ 「宝がある場所」                          │
│  RARE = 「判断できないから捨てない場所」          │
│                                                    │
│  RAREに宝があるかどうかは分からない。              │
│  分からないから捨てない。それだけ。                │
│                                                    │
└────────────────────────────────────────────────────┘
```

---

## よくある質問

### Q: どのサンプルから始めるべき？

**A:** `basic_usage` → `effectiveness_test` の順がおすすめです。

### Q: 自分のデータで試すには？

**A:** `custom_similarity` を参考に、データに適した類似度関数を定義してください。

### Q: 大規模データで使うには？

**A:** `kdf_optimization_strategies` で枝刈りまたはLSHを適用してください。

### Q: KDFはどんな時に有効？

**A:** 以下の場合に有効です：
- 冗長な情報を整理・圧縮したい
- 判断材料が不足しているデータを早まって廃棄したくない
- 情報の「重複度」に基づいて整理したい

**注意:** KDFは「宝を発見する」ツールではありません。「捨てて後悔するリスク」を回避する保守的戦略です。

---

## 実行方法

```bash
# 通常実行
cargo run --example <サンプル名>

# リリースビルド (ベンチマーク用)
cargo run --release --example <サンプル名>

# 全サンプルをリスト
cargo run --example 2>&1 | grep "Available"
```

---

*Last Updated: 2025-12-31*
