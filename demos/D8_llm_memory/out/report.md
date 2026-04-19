# Demo D8: LLM エージェント持続的メモリ curation

**Dataset:** synth_llm_memory (n=250)

**Patent section:** 明細書 §0002 (広義ナレッジ/エージェントメモリ) / Claim 1, 25, 46

## 測定指標の3軸フレーム

### 軸A: KDF の強み(想定)

- `rare_fact_recall` ↑: 高い方が良い

### 軸B: 他手法と同等(想定)

- `compression` ↑: 高い方が良い

### 軸C: KDF の弱み / トレードオフ(想定)

- `wall_ms` ↓: 低い方が良い

## 結果

| Method | ラベル要 | rare_fact_recall | compression | wall_ms | wall(ms) |
|---|:---:|---:|---:|---:|---:|
| TTL_oldest | No | 0.000 | 0.800 | 0.001 | 0.00 |
| RecentTop | No | 0.000 | 0.800 | 0.001 | 0.00 |
| FreqSummary | No | 0.000 | 0.800 | 0.370 | 0.37 |
| **KDF** | No | 0.195 | 0.800 | 0.265 | 0.27 |
| KDF+TextSim | No | 1.000 | 0.800 | 0.633 | 0.63 |

## 結論(正直)

### ✅ KDF が選ばれるべきシナリオ

- LLM エージェント会話履歴の long-term memory curation
- 構造(session, reply chain, shared vocabulary)が残っている環境
- LLM API コスト無しでの決定論的 memory 選別

### ⚠️ KDF を避けるべきシナリオ

- 意味解釈が必須の memory 運用 → LLM summary 系 (Mem0, MemGPT) 併用要
- 構造がほぼ無い純粹発話リスト

### 📋 正直な制限事項

- 合成 conversation(5 sessions × 50 utterances × 10 rare planted)
- 実 LLM 会話 log (Anthropic/OpenAI 等の memory bench) での検証は未実施
- セマンティック類似は shingle proxy のみ(embedding 未使用)

## 再現

各 demo の README.md を参照してください。
