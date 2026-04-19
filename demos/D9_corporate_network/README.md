# D9 — Corporate Network Boundary-Spanner Detection(遊び検証)

Burt's Structural Holes theory(paper_draft.md Theoretical Foundation 節 参照)の
実証デモ。企業/大学の共著 graph で **分野間を橋渡しする "boundary spanner"** を
KDF で抽出。"遊びだがそれなりに動く" を目指した proof-of-concept。

## Pipeline

```
[Step 1] OpenAlex API → 4 分野 × 500 論文 = 2000 papers
[Step 2] 共著 graph 構築 + KDF で 30%/50% ranking
[Step 3] HTML dashboard 生成(browser で閲覧)
[Step 4] SQLite + Obsidian 風 tag 構造に export
[Step 5] Tag filter query + 部分 graph の KDF 再実行 demo
```

## Run

```bash
# 全 pipeline(data fetch ~1-2 min、以降は数秒ずつ)
python demos/D9_corporate_network/scripts/1_fetch_papers.py           # OpenAlex から 2000 論文
python demos/D9_corporate_network/scripts/2_build_graph_and_rank.py   # graph + KDF
python demos/D9_corporate_network/scripts/3_dashboard.py              # HTML dashboard 生成
python demos/D9_corporate_network/scripts/4_export_to_sqlite.py       # SQLite + Obsidian tag
python demos/D9_corporate_network/scripts/5_tag_query_demo.py         # Tag 別 filter + KDF 再実行
```

**Cost**: $0(OpenAlex free tier、auth 不要)
**所要**: ~2-5 分 total

## 生成物

| ファイル | 用途 | Git 管理 |
|---|---|:-:|
| `out/papers_raw.json`(~5.6 MB) | OpenAlex raw 論文 data | ❌ gitignored |
| `out/institutions_ranked.json`(~1.7 MB) | 機関ランキング | ❌ gitignored |
| `out/corporate_graph.db`(~18 MB) | SQLite DB(再生成可) | ❌ gitignored |
| `out/dashboard.html`(~90 KB) | ブラウザで閲覧 | ✅ committed |
| `out/boundary_spanners.json`(~140 KB) | top boundary spanners | ✅ committed |

## 得られる Insights の例(2026-04-19 実行時)

### Rare broker(pure Burt-type、deg=1-3 で 3 分野橋渡し)
- Brunel University of London(UK)
- University of Rochester(US)

### 日本企業で注目
- **Samsung (Japan)**: 車載 × 半導体(2 分野)で Rare layer
- **The University of Tokyo**: 4 分野全てで Core hub
- Nagoya University、Kyoto University: multi-field Edge layer

### 多分野企業(type=company、3+ 分野)
- Microsoft Research UK(Rare、deg=3、AI/Auto/Mat)
- Novartis (Switzerland)(Rare、biomed cross-industry)
- AstraZeneca、Janssen、Pharma 系が Core hub

## 重要 disclaimer

これは **記述的統計** であり、投資助言ではありません。

- 未来予測ではなく、過去の共著 pattern の structural 整理
- OpenAlex 被引用上位 2000 論文のみなので long-tail 中小企業は欠落
- 共著関係 ≠ 正式な joint patent(proxy)
- US / CN 中心 coverage、日本企業の研究発表はやや過小表示

## この demo で実証したこと

1. **OpenAlex の free API で、日経 225 級の機関 coverage が数分で取れる**
2. **KDF の Rare/Core layer が、Burt's Structural Holes の broker と mathematically match する**
3. **SQLite + Obsidian-style tag で、任意の subset に filter して KDF を即座に再実行できる**(例: 「車載 × 半導体 × 企業」→ 37 機関の subgraph を 2.6ms で抽出、KDF 130ms で rank)

## 発展アイディア

[docs/extension_ideas.md の Ext-7〜Ext-13 参照]

この demo が動くなら、同じ pipeline で:
- Git commit graph(Ext-7)
- Call graph(Ext-8)
- IoT sensor retention(Ext-9)
- 金融 transaction(Ext-10)
- 医療 event(Ext-11)

等 全て同じ 5-step 構造で流用可能。`kdf_select_generic` Rust binary + SQLite + tag schema の組み合わせが **universal pipeline**。
