# Classical Algorithm Revival via KDF — データ量問題で忘却された古典 algorithm の復権候補

**最終更新**: 2026-04-19
**記録者**: 発明者(Chai)発案 + Claude Opus 4.7 調査
**目的**: 「KDF で前処理 → データ量削減 → 古典 algorithm が scale できるようになる」という novel な利用方向を体系化する。古典的に elegant だが、現代データ規模で impractical になった algorithm を、KDF の 70% 削減で revive できるケースを enumerate する。

---

## 🎯 基本戦略

```
[大規模データ(n = 10⁶〜10⁹)]
         ↓
[KDF で 30% に削減(n → 0.3n)]
         ↓
[古典 algorithm(多項式オーダー)が practical に]
         ↓
[結果 = 近似解だが structural に valid]
```

KDF の決定論性が保証するもの:
- 同じ入力 → 同じ削減結果 → **再現可能な近似**
- Item-level verbatim 保持 → 残った node の分析は original と同等
- Structural rareness 保護 → **important node が落ちにくい**

---

## 📊 復権候補 algorithm 一覧

### 🥇 Tier 1: 計算量で死んだ古典グラフ algorithm

#### C1. Floyd-Warshall 全対最短路(O(V³))

**原 algorithm**:
- 全ノード対の最短距離を求める古典的 DP
- O(V³) 時間、O(V²) 空間
- V=1,000 で 10⁹ operations(数秒)、V=10,000 で 10¹² operations(数時間)、V=100,000 以上は実質無理

**なぜ使われなくなったか**:
- 現代グラフは V=10⁶+ が普通(social network, web graph, knowledge graph)
- メモリも V² なので V=10⁵ で 100 GB、V=10⁶ で 10 TB
- → Dijkstra 等の single-source 版に置き換えられた

**KDF 復権の idea**:
- V=100,000 の graph → KDF で V=30,000 に削減 → Floyd-Warshall O(2.7 × 10¹³) が O(2.7 × 10¹⁰) に → 3 桁高速化
- 30% 削減でメモリも 100 GB → 9 GB(単一サーバー可能)
- **近似だが、Rare/Core layer を保持しているので重要な距離 query に答えられる**

**use case**:
- 社内 Slack network の "all-pairs collaboration distance" 分析
- Research paper citation network での "knowledge distance" 全対計算
- 道路網の災害時 alternative route 全対計算(道路重要度で KDF 削減)

**検証 experiment(未実施)**:
- Dataset: Stanford SNAP の中規模 graph(Epinions, Slashdot 等 V=50k-100k)
- Baseline: 全量 Floyd-Warshall(reference)vs KDF + Floyd-Warshall
- Metric: top-K 最短距離の recall、top-K 誤差率
- 予想: KDF で 30% 削減時、重要 pair の距離誤差 < 5%

**価値**: high(新 application + 古典復権 + 3 桁高速化)

---

#### C2. Betweenness Centrality(Brandes O(VE))

**原 algorithm**:
- ノードが最短路上にどれだけ乗るかで重要度を測る
- Brandes 版でも O(VE)、V=10⁶ E=10⁷ なら 10¹³ ops(数日)
- 並列化してもスケールしない

**なぜ使われなくなったか**:
- 大規模 network で全 node の betweenness 計算は不可能
- 近似 algorithm(sampling-based)に置き換えられたが精度が問題

**KDF 復権の idea**:
- KDF の Rare/Core layer は構造的ボトルネック node を保護する傾向
- V=10⁶ → V=3×10⁵ に削減後 → O(V×E) が 9% に
- **Core layer node の betweenness が full graph の top-K と近似するか検証すべき**

**use case**:
- 完全 social network で influencer 検出
- Citation network で seminal paper 検出
- Supply chain graph で critical node 検出

**検証 experiment**:
- Dataset: Stanford SNAP ego-Twitter、ego-Facebook(V=50k-80k)
- KDF pruning + Brandes vs 全量 Brandes
- Metric: top-50 betweenness node の Jaccard similarity、Spearman rank correlation

**価値**: high(決定的かつ実用)

---

#### C3. Community Detection(Girvan-Newman O(m²n))

**原 algorithm**:
- Edge betweenness で最も "bridge" な edge を繰り返し除去 → community 分離
- O(m²n)、m=10⁵ で n=10⁴ だと 10¹³ ops — 実質不可能

**なぜ使われなくなったか**:
- Louvain, Leiden 等の greedy algorithm が台頭(O(n log n))
- しかし Louvain は modularity trap や小 community 無視 problem あり
- Girvan-Newman の方が elegant だが scale しない

**KDF 復権の idea**:
- Rare/Core layer 保持で community 境界を preserve
- V を 30% に削減、m を 40-50% に削減 → Girvan-Newman が O(n³) オーダーで動く

**検証 experiment**:
- Dataset: Zachary karate, Dolphins, Football(small benchmarks), scaled up に LFR benchmark
- Metric: NMI(normalized mutual information)vs ground-truth community

**価値**: medium(Louvain が実用的に十分強いので niche)

---

### 🥈 Tier 2: カーネル法 / 行列分解の復権

#### C4. カーネル SVM(O(n²) training)

**原 algorithm**:
- RBF / polynomial kernel SVM は古典的 ML standard
- n² の kernel matrix computation、n=100,000 で impractical

**なぜ使われなくなったか**:
- Deep learning 台頭で大規模 labeled data に SGD-based NN が主流
- Linear SVM や approximation(random features)に置き換え

**KDF 復権の idea**:
- Training data を graph 化(similarity edge)、KDF で 30% 削減
- n=10⁵ → n=3×10⁴ → kernel matrix が 10¹⁰ → 10⁹ に
- **selected support vector 候補は structurally rare な点 = KDF 保護 target と一致する仮説**

**use case**:
- 小〜中規模 labeled data での解釈可能 classifier(医療、法務等 regulated)
- Few-shot learning で NN より data efficient な classical ML 復活

**検証 experiment**:
- Dataset: UCI adult, MNIST subset
- Baseline: 全量 SVM(RBF kernel)vs KDF + SVM
- Metric: test accuracy, training time

**価値**: medium(domain 次第、regulated 市場で value)

---

#### C5. Gaussian Process Regression(O(n³))

**原 algorithm**:
- ベイズ的 nonparametric regression
- n³ の Cholesky decomposition、n=10,000 で既に impractical

**なぜ使われなくなったか**:
- Inducing point approximation (SGPR、O(nm²) for m inducing points) で回避
- しかし inducing point の選択は heuristic

**KDF 復権の idea**:
- **KDF を inducing point selector として使う**
- Rare/Core layer = "情報量の多い点" → inducing points として valid な仮説
- 既存の SGPR の inducing point heuristic(k-means, DPP)と同等以上の性能?

**検証 experiment**:
- Dataset: UCI regression datasets、sensor network temporal data
- Baseline: inducing point via k-means vs KDF-selected
- Metric: test NLL、predictive variance accuracy

**価値**: high(学術的 novel、GP 研究 community の関心高い)

---

#### C6. SVD / Latent Semantic Indexing(O(mn²))

**原 algorithm**:
- 1990s に text IR の dominant 方法(LSI / LSA)
- SVD の計算量で impractical(m=10⁵ 文書、n=10⁴ terms で 10¹³ ops)

**なぜ使われなくなったか**:
- Dense embedding (word2vec 等)に置き換え
- しかし SVD は決定論的で interpretable、embedding は stochastic
- **regulated industry で SVD の revive 需要あり**

**KDF 復権の idea**:
- Document-term graph で KDF pruning
- 代表 document / 代表 term のみで SVD → 計算量 2-3 桁削減
- 結果: compact、決定論的、interpretable な latent space

**検証 experiment**:
- Dataset: Reuters-21578, 20 Newsgroups
- Baseline: 全量 LSI vs KDF + LSI
- Metric: document retrieval accuracy(with vs without)

**価値**: medium-high(interpretable IR の復権)

---

### 🥉 Tier 3: 計算理論・最適化の古典

#### C7. Bootstrap / Permutation test(O(Bn))

**原 algorithm**:
- Non-parametric 統計検定の古典
- B=10,000 resample × n=10⁶ data で 10¹⁰ ops

**なぜ使われなくなったか**:
- 大規模 data で non-parametric は cost 爆発
- Asymptotic test に置き換え(CLT 仮定)

**KDF 復権の idea**:
- KDF で n → 0.3n、bootstrap が 3 倍高速化
- **Structural rareness を保持しているので tail behavior 保持**
- Central-limit な中央部は捨てても OK

**検証 experiment**:
- Dataset: UCI regression + permutation test baseline
- Metric: p-value 近似精度

**価値**: niche(統計 community 向け)

---

#### C8. Graph Laplacian Eigen-decomposition(O(V³))

**原 algorithm**:
- Spectral clustering、Normalized cut 等の基盤
- V³ で V=10⁴ 以上は impractical

**なぜ使われなくなったか**:
- Random projection や Nyström approximation で回避
- しかし精度 trade-off あり

**KDF 復権の idea**:
- V=10⁵ → V=3×10⁴ → V³ が 3.6 × 10¹³ ops(数週間)→ 2.7 × 10¹³ の 1/37 = 数時間
- **ただし eigen-decomposition は pruning で結果が大きく変わるリスク**

**検証 experiment**:
- Dataset: 中規模 Protein-Protein Interaction network
- Metric: spectral gap、first eigenvalue correlation

**価値**: low-medium(dangerous で慎重に)

---

### 🎭 Tier 4: Surprise / niche candidate

#### C9. Multiple Sequence Alignment (O(n^k) for k sequences)

**原 algorithm**:
- 生物学でホモロジー解析の基盤
- k 系列の alignment は O(n^k)、k=10 で既に impossible

**なぜ使われなくなったか**:
- Progressive alignment (Clustal) 等の heuristic
- しかし phylogenetic 正確性で compromise

**KDF 復権の idea**:
- Sequence similarity graph を KDF で pruning
- "代表" sequences のみで exact MSA を可能化
- 小規模だが exact な結果

**価値**: niche(bioinformatics specialist が必要)

---

#### C10. Exact Graph Coloring / Vertex Cover (NP-hard)

**原 algorithm**:
- 小 instance では exact algorithm が可能
- 大 instance では approximation に頼る

**KDF 復権の idea**:
- V=100 程度なら exact solver が動く
- KDF で大 graph → 近傍保持型 sub-graph → exact coloring
- **構造的鍵点だけの coloring で全体の approximation**

**価値**: niche research、theoretical interest

---

## 🎯 検証優先度(recommended roadmap)

### 高優先度(validation で新 paper 1 本分の価値)

1. **C2 Betweenness centrality via KDF pruning** — SNAP benchmark で robust validation 可能、大 graph で実用的 value
2. **C5 Gaussian Process regression via KDF inducing points** — ML 学術 community にアピール、SGPR 比較が出せる
3. **C4 Kernel SVM via KDF point selection** — regulated ML 市場への橋渡し

### 中優先度

4. **C1 Floyd-Warshall revival** — 特定用途(collaboration distance 等)では instant impact
5. **C6 LSI / SVD revival** — interpretable IR として regulated 市場

### 実験設計 template(全 candidate 共通)

1. Dataset: 古典的 benchmark(UCI, SNAP, Reuters 等)から medium size(n=10⁴-10⁵)を選ぶ
2. **全量 classical algorithm** を reference として実行
3. **KDF で 30% 削減** → classical algorithm を実行
4. Metric: 
   - 近似精度(top-K recall、rank correlation、NLL 等)
   - 計算時間比(speed-up)
   - メモリ使用量比
5. Crossing over point: 何 n から KDF pruning が full-run より有利か

---

## 💡 発明者 (Chai) の novel 洞察について

**発明者提案**:
> 「古典アルゴリズムでデータ量が多くて使われなくなったものを、KDF でデータ整理することで復権させる」

この洞察は以下の 3 つの意味で novel:

1. **KDF を "圧縮 layer" として positioning** — 従来の retention / memory curation から一段抽象化
2. **"古典の復権" というアカデミック narrative** — ML/CS research で rarely 扱われる angle
3. **Deterministic pre-processing → classical post-processing** という pipeline 設計 — chain rule として一貫

この framing で paper を書くと、KDF が **general-purpose graph compression framework** として位置付けられ、LLM memory は **1 つの validation application** に過ぎなくなる。特許 claim の広さを最大限活用できる angle。

---

## 🔗 関連 document

- [domain_validation.md](domain_validation.md) — 応用領域別 validation 状況
- [design_philosophy.md](design_philosophy.md) — 設計方針(NLP-agnostic framing)
- [extension_ideas.md](extension_ideas.md) — 拡張機能案
- [paper_draft.md](paper_draft.md) — 論文草稿
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — 検証済 findings
