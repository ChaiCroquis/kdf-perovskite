# KDF 権威仕様 (Authoritative Specification)

**ステータス:** FROZEN / 改変禁止
**固定日:** 2026-04-17
**整合性:** [HASHES.sha256](HASHES.sha256) で SHA-256 管理

> **公開リポジトリでの注意**: `filed/` 配下の特許出願書類 5 点(特許願 / 特許請求の範囲 / 明細書 / 要約書 / 図面)は、日本特許庁による自動公開(2027-08-24 頃)まで本公開リポジトリに含めていません。詳細は [`filed/README.md`](filed/README.md) 参照。本文書内の `filed/*.pdf` への参照リンクはその時点で有効化されます。発明の技術的中身は [プレプリント論文](../arxiv_submission/paper.pdf) を参照してください。

---

## 1. この文書の位置付け

**`docs/patent/filed/` の5書類 = 特許庁提出書類 = 唯一のマスター仕様**である。
他はすべて「参考資料」であり、`filed/` と矛盾した場合は `filed/` を正とする。

```
┌──────────────────────────────────────────────────────────────┐
│  MASTER (絶対・唯一)                                         │
│  ─────────────────────────────                               │
│  filed/特許願.pdf          ← 出願書                          │
│  filed/特許請求の範囲.pdf  ← 法的権利範囲(請求項50)          │
│  filed/明細書.pdf          ← 発明の定義・数式・実施形態      │
│  filed/要約書.pdf          ← 要約                            │
│  filed/図面.pdf            ← 図面                            │
│                                                              │
│  → この5書類が KDF そのものである。                         │
│  → 他のいかなる文書・コード・発言とも、矛盾時は filed/ が正 │
└──────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────┐
│  REFERENCE (参考・補助)  ※ filed/ との矛盾時は従属          │
│  ─────────────────────────────                               │
│  technical/発明提案書_マスターファイル.md                    │
│  technical/完全版_あなたの3つの発明.md                       │
│  technical/01_統合技術説明書.md                              │
│  technical/KDF_用語集_完全版.md                              │
│  technical/kdf_rev12_complete_jp.md                          │
│  technical/kdf_integrated_*.{md,csv}                         │
│  technical/補足１.md / 補足２*.md                             │
│  revisions/KDF Rev.10/11/12 原典                             │
│                                                              │
│  → 出願前の検討資料。filed/ の解釈を助けるために参照する。  │
│  → ただし filed/ に記載されていない拡張は「発明の範囲外」。 │
└──────────────────────────────────────────────────────────────┘
┌──────────────────────────────────────────────────────────────┐
│  NON-AUTHORITATIVE (仕様根拠にしない)                        │
│  ─────────────────────────────                               │
│  docs/master/*        過去のリファクタ議論                   │
│  docs/KDF_*.md        解説文書                               │
│  README.md            概要・広告                             │
│  kdf-lib/ 以下の実装  仕様への準拠が未確認                   │
└──────────────────────────────────────────────────────────────┘
```

### 解釈ルール

1. KDF に関する質問・実装判断は **必ず `filed/` を最初に参照する**。
2. `filed/` に明示されている事項は、他の文書が何と言おうと `filed/` が正。
3. `filed/` に書かれていない事項について `technical/`, `revisions/` を参照してよい。
4. `technical/`, `revisions/` どうしで矛盾する場合は、より後に作成されたもの・より最終稿に近いものを優先(例: `発明提案書_マスターファイル.md` は「最終確定版」)。
5. 実装 (`kdf-lib/` 等) は仕様根拠にならない。実装と `filed/` が食い違えば実装がバグ。

## 2. 変更禁止ルール

- 本フォルダ (`docs/patent/`) 配下のファイルは **いかなる理由でも改変しない**。誤字・表記揺れがあってもそのまま保持する。
- 新しい版が出た場合は **上書きせず** `docs/patent/v2/`, `docs/patent/v3/` のように並列追加し、本 `SPEC.md` で優先順を宣言する。
- 整合性は `HASHES.sha256` で確認できる。CI でハッシュ検証することを推奨:

```bash
cd docs/patent && sha256sum -c HASHES.sha256
```

## 3. 実装に対する拘束

本仕様と実装 (`kdf-lib/`, `crates/`, `kdf-python/`, `kdf-wasm/`, `kdf-cli/`) の間に齟齬がある場合:

1. **仕様が正**、実装は **バグ** と見なす。
2. 「実装が動いているから仕様を合わせる」という方向の変更は禁止。
3. 実装を変更する前に、本フォルダ内のどの条文・数式・請求項に準拠するかを明記する。
4. README/docs/master の記述が本仕様と矛盾する場合、README/docs/master を書き換える。

### 3.1 既知の実装と仕様の乖離

**方針(ADR-0001 採択済, 2026-04-17):**
- **`crates/cgb-kdf/` を参照実装(reference implementation)とし、Claim 1-50 の準拠目標はこれに集約する**
- `kdf-lib/` は **Rev.10 Basic サブセット実装** と位置付け、Claim 1 準拠は主張しない([ADR-0001](../adr/0001-cgb-kdf-is-reference-impl.md))
- したがって、`kdf-lib` と本仕様との差分は **意図的 scope 外** であり、修正対象ではない

#### cgb-kdf の準拠状況(2026-04-18, per-claim 直接テスト整備完了)

**cgb-kdf は Claim 1-50 すべてに直接テスト付き**:`test_claim1_*` から `test_claim50_*` まで 56 tests(複数テストを持つ claim 含む)。workspace 全体で **449 tests all pass**。独立検証エージェントが 2 回の re-audit(2026-04-18)で全テストを STRONG/ADEQUATE 判定済([COMPLIANCE.md](COMPLIANCE.md))。

後段で直接テストを追加した Claim:

| Claim | 要件(明細書) | 状態 | 実装箇所 | 根拠テスト |
|---|---|---|---|---|
| 5 | 時間評価成分(§0013) | ✓ | [`decay.rs:compute_time_component` / `compute_evaluation_value`](../../crates/cgb-kdf/src/framework/decay.rs) | `test_claim5_*` |
| 10 | α=2 (§0018) | ✓ | [`decay.rs:alpha_core=2.0`](../../crates/cgb-kdf/src/framework/decay.rs) | `test_claim10_alpha_equals_two` |
| 17 | 分散実行(§0025) | ✓ | [`decay.rs:apply_edge_decay_local`](../../crates/cgb-kdf/src/framework/decay.rs) | `test_claim17_local_decay_matches_global` |
| 33 | 孤立度指標多成分(§0041) | ✓ | [`classifier.rs:加重次数`](../../crates/cgb-kdf/src/framework/classifier.rs) | `test_claim33_isolation_metric_uses_strength_and_connection_count` |

COMPLIANCE.md を正とする。

#### 廃止された旧リスト(参考)

ADR-0001 採択前に kdf-lib に対して列挙されていた違反(ノード次数減衰、線形減衰、Rare 相対閾値、Meta 層未定義、昇格/抑制/二段階審査/摩擦関数 $W_{eff}$/メタ認知制御未実装、構造フィンガープリント未実装)は、現在では **kdf-lib の subset scope 設計内** として解釈する。cgb-kdf では既にすべて Phase 1 で実装済([TRACEABILITY.md](TRACEABILITY.md) Claim 14, 20-32, 46, 48 参照)。

## 4. AI エージェント向け指示 (Claude/Copilot 等)

このリポジトリで作業する AI エージェントは以下を遵守すること:

1. **KDF の数式・挙動・用語に関する質問**には、まず `docs/patent/filed/明細書.pdf` と `docs/patent/technical/` を参照する。`README.md` や `docs/KDF_*.md` を根拠にしてはならない。
2. **実装に関する提案・修正**をする前に、本 SPEC.md §3 に従い、どの条文に準拠するか明示する。
3. **本フォルダ配下のファイルを編集・削除・リネームしない**。ユーザーが明示的に許可した場合のみ例外。
4. 「仕様と実装が違う」と気づいた場合、**実装側を仕様に合わせる** 方向のみで提案する。

## 5. 内容インデックス

### 5.1 `filed/` — **マスター仕様 (特許庁提出書類)**

これが KDF の定義そのもの。他のすべてに優先する。

| ファイル | 内容 |
|---|---|
| [特許願.pdf](filed/特許願.pdf) | 出願書 |
| [特許請求の範囲.pdf](filed/特許請求の範囲.pdf) | **請求項50** — 発明の法的定義(権利範囲) |
| [明細書.pdf](filed/明細書.pdf) | 発明の詳細説明・実施形態・数式 |
| [要約書.pdf](filed/要約書.pdf) | 要約 |
| [図面.pdf](filed/図面.pdf) | 図面 |

### 5.2 `technical/` — 参考資料(発明者による説明)

| ファイル | 内容 |
|---|---|
| [発明提案書_マスターファイル.md](technical/発明提案書_マスターファイル.md) | 最終確定版の発明提案書 |
| [完全版_あなたの3つの発明.md](technical/完全版_あなたの3つの発明.md) | 3つの独立発明の整理 |
| [01_統合技術説明書.md](technical/01_統合技術説明書.md) | 弁理士提出用統合説明 |
| [KDF_用語集_完全版.md](technical/KDF_用語集_完全版.md) | **用語定義の基準** |
| [kdf_rev12_complete_jp.md](technical/kdf_rev12_complete_jp.md) | Rev.12 完全版 |
| [kdf_integrated_critical_significance.md](technical/kdf_integrated_critical_significance.md) | 臨界的意義 |
| [kdf_integrated_parameters.csv](technical/kdf_integrated_parameters.csv) | **標準パラメータ表** |
| [補足１.md](technical/補足１.md) | 補足説明1 |
| [補足２_特許申請候補技術リスト（概要と既存技術対比）.md](technical/補足２_特許申請候補技術リスト（概要と既存技術対比）.md) | 既存技術比較 |

### 5.3 `revisions/` — 参考資料(Rev.10〜12 原典)

| ファイル | 内容 |
|---|---|
| [KDF Rev.10](revisions/KDF%20Rev.10%20%E2%80%94%20適応的シナプススケーリングによる知識代謝の恒常性維持（自己組織化臨界の数理）.md) | 基本仕様(必須) |
| [KDF Rev.11](revisions/KDF_Rev11_Mathematical_Specification_Validated%201.md) | Dual-α 拡張 |
| [KDF Rev.12](revisions/KDF_Rev12_Analogy_Discovery.md) | Rare層・アナロジー発見 |

---

## 6. 改訂履歴

| 日付 | 行為 | 備考 |
|---|---|---|
| 2026-04-17 | 初版固定 | Obsidian Vault より `申請内容/` + `KDF特許相談資料/` + `kdf_rev12_complete_jp/` + Rev.10/11/12 を複製。SHA-256 記録。 |
