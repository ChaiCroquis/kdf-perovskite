# Demo D2: HTTP アクセスログ圧縮 — 稀なエラー応答の自動保持

**Dataset:** NASA-HTTP (synthetic, Zipf) (n=20000)

**Patent section:** 明細書 §0002 (ログ管理) / Claim 1, 18 (保護属性), 33 (孤立度指標)

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `rare_recall` ↑: 高い方が良い
- `label_free` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | rare_recall | label_free | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|---:|
| Random | No | 0.104 | 1.000 | 0.900 | 0.155 | 0.15 |
| Reservoir | No | 0.104 | 1.000 | 0.900 | 0.121 | 0.12 |
| Head | No | 0.115 | 1.000 | 0.900 | 0.020 | 0.02 |
| TailBasedLabeled | Yes | 1.000 | 0.000 | 0.900 | 0.315 | 0.32 |
| StratifiedLabeled | Yes | 1.000 | 0.000 | 0.900 | 0.146 | 0.15 |
| **KDF** | No | 0.078 | 1.000 | 0.900 | 1.685 | 1.69 |
| KDF+RelDensity | No | 0.307 | 1.000 | 0.900 | 1.451 | 1.45 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- ラベル(status code など)が得られない / 到着遅れのログストリーム
- 長期保存で高圧縮率を狙いつつ rare error を残したい観測基盤

### ⚠️ KDF を避けるべきシナリオ

- リアルタイム sampling(KDF は graph 構築コストあり)
- status code ラベルが常に利用可能で完全に信頼できる環境(Stratified が最適)

### 📋 正直な制限事項

- 本実行は **合成ログ (n=20000, Zipf分布, planted error rate ~5%)** を使用。実 NASA log は `demos/D2_nasa_log/data/access.log` に配置すると自動で使用されます。
- 選択比率を 10% に固定した単一ポイント評価(sweep は Phase 9 候補)
- Bipartite graph 化(IP×resource)は NASA log の自然な構造、他ログで検証要

## 再現

各 demo の README.md を参照してください。
