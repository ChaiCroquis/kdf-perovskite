# benchmarks/real_data/data/

公開データセットの置き場所(ファイル自体は**再配布しない**)。

## FB15K-237

```bash
# 取得コマンド(25MB、MIT License)
curl -sL -o /tmp/fb.tar.gz "https://raw.githubusercontent.com/TimDettmers/ConvE/master/FB15k-237.tar.gz"
mkdir -p benchmarks/real_data/data/fb15k-237
tar xzf /tmp/fb.tar.gz -C benchmarks/real_data/data/fb15k-237/
```

展開後のファイル: `train.txt` (21MB) / `valid.txt` (1.3MB) / `test.txt` (1.5MB)。
tab-separated triples `head \t relation \t tail`。

## NASA HTTP log

```bash
# 取得(~200MB、ブラウザで ita.ee.lbl.gov/html/contrib/NASA-HTTP.html から)
mkdir -p benchmarks/real_data/data/nasa-http
# access.log を配置
```

## ogbn-arxiv

OGB Python package 経由で取得後、`edges.csv` と `citation_count.csv` を変換:

```python
from ogb.nodeproppred import DglNodePropPredDataset
# ... (省略)
```
