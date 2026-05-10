# KDF Reproducibility Guide

**Purpose**: paper v2.7 (Zenodo / arXiv 公開) 内で言及される F-xxx 検証結果を、第三者が手元で再現できるようにする single index document。

**Audience**: paper を読んで「F-097 BGL +33.33pt は本当か?」「自分の log でも試したい」と思った reader。Rust toolchain と一般的な data download tool だけで動かせる前提。

**Last review**: 2026-05-10 (paper v2.7 / 7-pattern arc 反映)

---

## §0 セットアップ (一度だけ)

### §0.1 必須環境

| 要件 | 確認コマンド | 期待出力 |
|---|---|---|
| Rust toolchain (1.75+) | `cargo --version` | `cargo 1.75.0` 以上 |
| git | `git --version` | `git version 2.x` |
| Python 3.10+ (一部 W/g 系で必要) | `python3 --version` | `Python 3.10` 以上 |
| disk space | `df -h .` | 30 GB+ 空き(data + build) |

オプション(一部 finding のみ必要):
- Docker (LaTeX paper rebuild 用、検証目的では不要)
- CUDA / GPU (なくても全 finding 動く、ただし local LLM 系は wall-clock 大幅増)
- `gh` CLI (GitHub PR / issue 確認用、検証目的では不要)

### §0.2 リポジトリ取得

```bash
git clone https://github.com/ChaiCroquis/kdf-perovskite.git
cd kdf-perovskite
cargo build --release --workspace --exclude kdf-python --exclude kdf-wasm
```

**verification**:
```bash
cargo build --release --workspace --exclude kdf-python --exclude kdf-wasm 2>&1 | tail -1
# 期待: "Finished `release` profile [optimized] target(s) in <時間>s"
```

`kdf-python` / `kdf-wasm` は native dep (pyo3 / wasm-bindgen) が CI 環境で揃わないことがあるため除外推奨。本 reproducibility guide の F-xxx 検証には不要。

### §0.3 data 取得 (finding 別、必要な分だけ)

| dataset | 取得元 | 配置先 | 必要な finding |
|---|---|---|---|
| NASA HTTP log (Jul95) | `wget https://ita.ee.lbl.gov/traces/NASA_access_log_Jul95.gz` → gunzip | `benchmarks/real_data/data/nasa-http/access.log` | F-072, F-091, F-093 |
| LogHub Apache.log | https://github.com/logpai/loghub/tree/master/Apache → `Apache.log` | `experiments/streaming_phase_2_5/data/Apache.log` | F-087, F-091, F-094 |
| LogHub BGL | https://zenodo.org/records/8196385 → BGL.tar.gz → BGL.log | `experiments/g9_bgl_recurring/data/BGL.log` | F-097 |
| LogHub HDFS_v1 | https://zenodo.org/records/8196385 → HDFS_v1.tar.gz | `experiments/g11_hdfs_recurring/data/HDFS.log` | F-100 |
| LongMemEval oracle | (synthetic generation in-tree) | 自動生成 | F-053, F-060, F-099 |
| LoCoMo | https://github.com/snap-stanford/locomo | `demos/D8_llm_memory/data/locomo/` | F-056〜F-059, F-069, F-070, F-071 |
| MovieLens 100K | `wget https://files.grouplens.org/datasets/movielens/ml-100k.zip` | `experiments/movielens_multi_genre/data/` | F-082, F-085 |

**verification (data 配置確認 + 完全性 check)**:

サイズ + sha256 で 2 段確認。**期待値は本 guide 著者が実際に使った version のもの** であり、upstream が更新された場合は size/hash が変わる。reader の checksum がここと違っても下記 期待出力が再現されれば finding として valid(checksum は debug 用途)。

```bash
# 著者環境での実測値 (2026-05-10):

# NASA HTTP log (sample subset、benchmarks/real_data の demo 用)
sha256sum benchmarks/real_data/data/nasa-http/access.log
# 期待: 577e600d6877fa786d115f325cbbd7d3f3ce543be11de4474a9965b3b0e37b7e
# size: 5,507,444 bytes (約 5.5 MB、demo 用 sample)
#
# 完全 dataset (NASA archive 直接 download) は ~205 MB、phase_x4_nasa_streaming.rs は 50k records 上限で動作するため
# sample / full どちらでも F-072 の +3.06pt は再現する想定

# Apache.log (LogHub 提供版)
sha256sum experiments/streaming_phase_2_5/data/Apache.log
# 期待: 23e616a1ed529bab9b20b9bc345d37035061eee971d246102bcc4937e51a039b
# size: 5,135,876 bytes (約 5.1 MB)

# BGL.log (LogHub Zenodo 提供版、配置先 path は finding 別に異なる)
sha256sum experiments/bgl_phase2/data/BGL.log  # F-074 で使用
# 期待: 666130b15ef44eb32fd02bd053e6c6e007c37696b5e7e8b9d8e45b729876a5d2
# size: 743,185,031 bytes (約 743 MB)
# 注: F-097 g9 用の BGL.log path は phase_g9_bgl_recurring.rs 内 path 参照を確認、
#     現状 experiments/bgl_phase2/data/BGL.log を symlink / copy で再利用するか
#     experiments/g9_bgl_recurring/data/BGL.log に追加配置する
```

data download が面倒な finding は §1 表で「synthetic」と明記してあり、その項目は data 取得不要で動く。**checksum 公開していない datasets** (LongMemEval / LoCoMo / MovieLens / HDFS 等) は §6 推奨 reproducer 動かして 期待 finding 数値が再現すれば valid とみなす(strict checksum match は要求しない、upstream variability 許容)。

---

## §1 Tier 1: Paper-cited findings (再現必須)

paper v2.7 の Abstract / §5 / §7 / Addendum で直接 cited される finding。各行は実行可能 (data deps を §0.3 で揃えた前提)。

### §1.0a Tier 1 reproducer code 実在 verification (一括 check)

§1 全表を実行する前に、code path がすべて存在することを 1 commands で機械確認:

```bash
cd /c/work/kdf-perovskite

# Rust binary list (cargo bin name で確認)
RUST_BINS="phase_x1_time_decay_locomo phase_x2_sandwich_twait_locomo phase_x3_dynamic_control_locomo phase_x4_nasa_streaming \
           phase_route_a_baselines phase_route_a_q2_dense phase_w3_real_kdf_turns kdf_select_generic \
           phase_2_5_apache_streaming f090_bias_detector_aggregate \
           phase_g2_alpha_sweep phase_g3_lyapunov phase_g4_rare_subset phase_g5_apache_recurring \
           phase_g9_bgl_recurring phase_g11_hdfs_recurring"
for bin in $RUST_BINS; do
    if [[ -f "demos/D8_llm_memory/src/bin/${bin}.rs" ]]; then
        echo "OK   Rust  $bin"
    else
        echo "MISS Rust  $bin (expected at demos/D8_llm_memory/src/bin/${bin}.rs)"
    fi
done

# cgb-kdf claim tests (embedded #[test] fn in src/ modules、not separate files)
N_CLAIM_TESTS=$(grep -rE "fn test_claim" crates/cgb-kdf/src/ crates/cgb-kdf/tests/ --include="*.rs" 2>/dev/null | wc -l)
N_UNIQUE_CLAIMS=$(grep -rEo "test_claim_?[0-9]+" crates/cgb-kdf/src/ crates/cgb-kdf/tests/ --include="*.rs" 2>/dev/null | sed -E 's/.*test_claim_?([0-9]+).*/\1/' | sort -un | wc -l)
[[ "$N_CLAIM_TESTS" -ge 50 ]] && [[ "$N_UNIQUE_CLAIMS" -ge 50 ]] && \
    echo "OK   cgb-kdf  $N_CLAIM_TESTS test_claim_* fns covering $N_UNIQUE_CLAIMS unique Claims (1-50 expected)" || \
    echo "MISS cgb-kdf  test_claim coverage: $N_CLAIM_TESTS fns / $N_UNIQUE_CLAIMS unique Claims"

# F-068 example
[[ -f "crates/cgb-kdf/examples/f068_analogy_benchmark.rs" ]] && echo "OK   example  f068_analogy_benchmark" || \
    echo "MISS example  f068_analogy_benchmark"

# Python scripts
PY_SCRIPTS="ext1_precision_router.py phase_g7_local_mem0.py phase_g8_local_qwen3b.py \
            phase_g10_v1_router_full.py w1_mcnemar_test.py w5_locomo_mem0_vs_kdf.py"
for script in $PY_SCRIPTS; do
    if [[ -f "demos/D8_llm_memory/scripts/$script" ]]; then
        echo "OK   Python $script"
    else
        echo "MISS Python $script (expected at demos/D8_llm_memory/scripts/$script)"
    fi
done

# 期待: 全行 "OK ..."
# "MISS" → 対応 finding は §1 表記載と乖離、§4 Gaps 節の不在 finding と同列扱い
```

reader が `MISS` を検出した場合、その finding は §4 Gaps の "code path 不明確" 扱いとし、再現は §4 と同様 reader independent 構築が必要。

### §1.0 期待出力の機械的検証 pattern

下記表の "expected" 列は paper の値そのもの。実行結果が一致するかは grep 系で機械判定できる。汎用 pattern:

```bash
# 汎用 verification template (任意の Tier 1 finding 用)
EXPECTED_PATTERN='<expected 列に書かれた key 数値、正規表現でも literal でも可>'
RUN_CMD='<§1.x 該当行の run command>'

OUTPUT=$(eval "$RUN_CMD" 2>&1)
if echo "$OUTPUT" | grep -qE "$EXPECTED_PATTERN"; then
    echo "OK   finding reproduced"
else
    echo "FAIL expected pattern not found in output"
    echo "$OUTPUT" | tail -20
fi
```

具体例(最も paper-critical な 3 件、F-072 / F-087 / F-097)は §6 Quick Start に full executable form で配置。残り Tier 1 findings はこの template に expected 列の数値を埋めて利用。

### §1.1 Patent Claim 機構実装 (F-001〜F-040)

| F-xxx | 結果 | code path | run command | expected |
|---|---|---|---|---|
| F-001 | Claim 1-50 中 44/50 (88%) 完全準拠 | `crates/cgb-kdf/src/**/*.rs`(embedded `#[test] fn test_claim_NN_*` 56 件、Claims 1-50 全カバー)+ `tests/properties.rs` | `cargo test -p cgb-kdf` | 392 lib tests + integration tests pass、test_claim* で 50 Claim カバー |
| F-002 | Claim 14 指数減衰、rel_err < 1e-10 | `crates/cgb-kdf/tests/math_properties.rs` | `cargo test -p cgb-kdf test_exp_decay_analytic_solution` | 1 test pass, rel_err < 1e-10 |
| F-005 | bit-exact 決定論 | `crates/cgb-kdf/tests/math_properties.rs` | `cargo test -p cgb-kdf decay_determinism_bitwise` | 1 test pass, 1,000-step bitwise match |
| F-007 | proptest 1,792 cases | `crates/cgb-kdf/tests/properties.rs` | `cargo test -p cgb-kdf properties::` | 7 properties × 256 = 1,792 cases pass |
| F-008 | 実測 O(n^1.20) (特許主張より悪い) | `benchmarks/PHASE7_REPORT.md` (ref) + `benchmarks/scaling_phase7/` | `cargo run --release -p benchmark-scaling` | n=500/5k/50k で ns/(n·log₂n)=77/62/111 |
| F-029 | FastNodeClassifier 真の線形 O(n) | `crates/cgb-kdf/src/framework/classifier.rs` | `cargo test -p cgb-kdf test_classifier_linear` | 1 test pass |
| F-040 | Claim 1-50 全 50 項 per-claim test 整備 | `crates/cgb-kdf/src/**/*.rs`(`fn test_claim_NN_*` を src 各 module 内に embedded、56 fn / 50 unique Claim)| `cargo test -p cgb-kdf test_claim` | test_claim* 関数 56 件 pass(`grep -rE "fn test_claim" crates/cgb-kdf/src/` で確認可)|

### §1.2 Retrieval & LLM memory (F-042〜F-060, F-099)

| F-xxx | 結果 | code path | run command | expected |
|---|---|---|---|---|
| F-042 | Route A: KDF > BM25/TF-IDF | `demos/D8_llm_memory/src/bin/phase_route_a_baselines.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_route_a_baselines` | KDF recall > BM25 baseline |
| F-043 | Q2: KDF > sentence-transformers dense | `demos/D8_llm_memory/src/bin/phase_route_a_q2_dense.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_route_a_q2_dense` | KDF > dense embed on 500 Q |
| **F-053** | real KDF vs Mem0 LongMemEval、Mem0 +23.8pt勝ち | `demos/D8_llm_memory/src/bin/phase_w3_real_kdf_turns.rs` + `scripts/w1_mcnemar_test.py` | `cargo run --release -p demo-d8-llm-memory --bin phase_w3_real_kdf_turns` then `python demos/D8_llm_memory/scripts/w1_mcnemar_test.py` | KDF 0.434 vs Mem0 0.672, p < 1e-16 |
| F-056〜F-058 | LoCoMo temporal +10〜+23 pt KDF 勝ち | `demos/D8_llm_memory/src/bin/phase_x1_time_decay_locomo.rs` + `scripts/w5_locomo_mem0_vs_kdf.py` | `cargo run --release -p demo-d8-llm-memory --bin phase_x1_time_decay_locomo` then `python demos/D8_llm_memory/scripts/w5_locomo_mem0_vs_kdf.py` | gpt-4o-mini +10.6pt p=0.0014, gpt-4.1-mini +23.4pt p=1.6e-14 |
| **F-060** | Ext-1 Precision Router strictly better than Mem0 | `demos/D8_llm_memory/scripts/ext1_precision_router.py` (post-hoc replay) | `python demos/D8_llm_memory/scripts/ext1_precision_router.py` | LongMemEval 0pt safe, LoCoMo +9.7/+22.4pt p<0.003 |
| F-099 | v1 router (precision-only) PASS_negative on paid | `demos/D8_llm_memory/scripts/phase_g10_v1_router_full.py` | `python demos/D8_llm_memory/scripts/phase_g10_v1_router_full.py` | F-053 cell Δ_v1=-11.60pt p=1e-7, v1 ≠ product |

### §1.3 Phase X realistic benchmarks (F-068〜F-072)

| F-xxx | 結果 | code path | run command | expected |
|---|---|---|---|---|
| F-068 | Analogy discovery 90% recall + negative reject | `crates/cgb-kdf/examples/f068_analogy_benchmark.rs` | `cargo run --release --example f068_analogy_benchmark -p cgb-kdf` | 90% on Gentner classics, 0% false positive |
| F-069 | Claim 5/14/17 LoCoMo: C17 bit-exact PASS、C5/14 static で劣る | `demos/D8_llm_memory/src/bin/phase_x1_time_decay_locomo.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_x1_time_decay_locomo` | KDF_static 0.5286, time-aware ≤ KDF_static |
| **F-070** | (θ_L,θ_U)=(0.70,0.80) refute、4-benchmark | `demos/D8_llm_memory/src/bin/phase_x2_sandwich_twait_locomo.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_x2_sandwich_twait_locomo` | F1=0.000 vs F1((0.70,1.00))=1.000 |
| F-071 | Claim 20-32 mechanism PASS / no selection benefit | `demos/D8_llm_memory/src/bin/phase_x3_dynamic_control_locomo.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_x3_dynamic_control_locomo` | 5:3:1 integer tick exact, MetaController α-bound clamp |
| **F-072** | NASA streaming Δ=+3.06pt **anchor** | `demos/D8_llm_memory/src/bin/phase_x4_nasa_streaming.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_x4_nasa_streaming` | C0 static 0.4592, C1 decay 0.4898, **Δ=+3.06pt** |

### §1.4 Cross-domain & narrowing arc (F-082〜F-100)

7-pattern arc(narrow 5 + positive 2)を構成する findings:

| F-xxx | direction | 結果 | pre-reg | code path | run command | expected |
|---|---|---|---|---|---|---|
| F-082, F-085 | strengthening | MovieLens niche genre γ-check 100% | (Phase 2 plan) | `experiments/movielens_multi_genre/verify_movielens_multi_genre.py` | `python experiments/movielens_multi_genre/verify_movielens_multi_genre.py` | 6 genres γ=1.00 (Film-Noir/IMAX/Western/Musical/War/Documentary) |
| F-086 γ | strengthening | hub-peripheral / hub-biased / peer-network 判別、N=5 で 3 PASS / 2 REJECT | (Phase 2 discovery) | `crates/bias-detector/src/lib.rs` + `demos/D8_llm_memory/src/bin/f090_bias_detector_aggregate.rs` | `cargo run --release -p demo-d8-llm-memory --bin f090_bias_detector_aggregate` | γ-check correlation rate per meta-family |
| **F-087** | narrow 1 (streaming → recurring rare) | Apache replication Δ=-13.04pt sign reversal | `docs/exploration/phase_2_5_pre_reg_addendum.md` | `demos/D8_llm_memory/src/bin/phase_2_5_apache_streaming.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_2_5_apache_streaming` | C0 0.3934 → C4 0.2631, **Δ=-13.04pt** |
| **F-090** | retraction | bias-detector 5/11=45.5% < 70% threshold で撤回 | `docs/exploration/phase_2_5_pre_reg_addendum.md` | `demos/D8_llm_memory/src/bin/f090_bias_detector_aggregate.rs` | `cargo run --release -p demo-d8-llm-memory --bin f090_bias_detector_aggregate` | accuracy 5/11=45.5%, certain-prediction count 11 |
| **F-091** | narrow 2 (α=2 → NASA-specific) | NASA α=2.0 optimal、Apache α=4.0 optimal、α=2.0 off -17.39pt | `docs/exploration/g2_alpha_sweep_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g2_alpha_sweep.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g2_alpha_sweep` | NASA: α=2.0 diff 0.00pt; Apache: α=4.0 optimal, α=2.0 -17.39pt |
| **F-092** | narrow 3 (Claim 31 → non-adversarial) | controller robust、functional rare FAIL recall=0.000 vs 0.4592 | `docs/exploration/g3_lyapunov_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g3_lyapunov.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g3_lyapunov` | boundedness PASS α∈[1.0,2.5], recovery PASS, functional FAIL |
| F-093 | anchor sharpening | F-072 V1/V2/V4 = +3.06pt (404-pattern driven) | `docs/exploration/g4_nasa_rare_subset_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g4_rare_subset.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g4_rare_subset` | V1+5xx, V2+4xx, V4+404 全て +3.06pt; V3/V5 trivial (n_rare=0) |
| **F-094** | positive 1 (recurring rare durability N=2) | Apache recurring +3.67pt PASS | `docs/exploration/g5_apache_recurring_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g5_apache_recurring.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g5_apache_recurring` | V_recurring **+3.67pt > +1.0pt**; sanity V_one-shot -13.04±0.0pt (F-087 reproduce) |
| F-095, F-096 | infra honest record | local 8B infeasible / 3B sub-noise floor | `docs/exploration/g7_mem0_latest_local_pre_reg.md`, `g8_local_qwen3b_pre_reg.md` | `demos/D8_llm_memory/scripts/phase_g7_local_mem0.py` (g7), `phase_g8_local_qwen3b.py` (g8) | `python demos/D8_llm_memory/scripts/phase_g8_local_qwen3b.py` (g8 推奨、g7 は infeasible 確認用) | g8: LongMemEval n=479 + LoCoMo n=321 完走、Δ_router=+0.62pt p=0.5 |
| **F-097** | positive 2 (cross-family N=3) | BGL HW kernel +33.33pt PASS | `docs/exploration/g9_bgl_recurring_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g9_bgl_recurring.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g9_bgl_recurring` | V_recurring **+33.33pt** (C0=0.0 → C2/C4=0.333); sanity V_one-shot ±0pt (small alphabet caveat) |
| **F-100** | narrow 5 (HDFS scope-out) | HDFS H_R+_literal -23.08pt FAIL + H_anomaly -4.30pt FAIL | `docs/exploration/g11_hdfs_recurring_pre_reg.md` | `demos/D8_llm_memory/src/bin/phase_g11_hdfs_recurring.rs` | `cargo run --release -p demo-d8-llm-memory --bin phase_g11_hdfs_recurring` | H_R+_literal **-23.08pt FAIL**; H_anomaly **-4.30pt FAIL**; Sanity ±0pt (small alphabet) |

(**bold** = paper Abstract / §5 P11 caveat / §7 で direct cited、最優先 reproduce 対象)

---

## §2 Tier 2: Context findings (best-effort、paper credibility に直接影響しない)

paper 本文 §6 / §7 / Conclusion で言及されるが、Tier 1 ほど core でない finding(decisive predictor establishment / domain validation 補強)。下記 code path / run command は **best-effort** で、agent 探索ベースのため reader が試して FAIL でも paper の主張根拠は揺らがない。確実な reproducer は Tier 1 (§1) と Quick Start (§6) に集約。

**実在 verification (best-effort)**:
```bash
# Tier 2 entry が指す path / script の実在を 一括 check (best-effort)
TIER2_PATHS="experiments/cross_domain_validation experiments/git_commit_pruning experiments/gp_inducing \
             experiments/python_callgraph experiments/git_archival_n9 experiments/f1_benchmark experiments/biological_ppi"
for p in $TIER2_PATHS; do
    [[ -d "$p" ]] && echo "OK   dir  $p" || echo "MISS dir  $p (best-effort: reader 独立構築 or skip)"
done
# 期待: "OK" / "MISS" 混在、MISS は best-effort scope 外と認識して skip 可
```

| F-xxx | 結果 | code path (best-effort) | run command (best-effort) |
|---|---|---|---|
| F-061 | decisive predictor establishment (path-based wins, density-based fails) | `experiments/cross_domain_validation/` (4 synthetic graphs) | `python experiments/cross_domain_validation/run_4graphs.py` |
| F-062 | Git commit pruning: tokio merge recall 99.5% | `experiments/git_commit_pruning/` | `python experiments/git_commit_pruning/run_tokio.py` |
| F-063 | GP inducing points: KDF < Random < KMeans | `experiments/gp_inducing/` | `python experiments/gp_inducing/run.py` |
| F-064 | Python call graph API preservation: KDF 16% vs Random 41% | `experiments/python_callgraph/` | `python experiments/python_callgraph/run_flask.py` |
| F-065 | git pruning 3-repo replication (tokio/pytest/lodash) | `experiments/git_commit_pruning/` | `python experiments/git_commit_pruning/run_3repos.py` |
| F-076, F-077 | Git archival cross-repo expansion N=9 | `experiments/git_archival_n9/` | `python experiments/git_archival_n9/run_all.py` |
| F-083 | KDF F1 honest measurement (Rare layer ≈ degree-rank) | `experiments/f1_benchmark/` | `python experiments/f1_benchmark/run.py` |
| F-084 | Biological PPI + OncoKB γ=74% < 95% threshold | `experiments/biological_ppi/` | `python experiments/biological_ppi/run_oncokb.py` |

---

## §3 Tier 3: 撤回 / 補助 findings

paper では historical context として参照、reproduce は誠実性 audit 目的のみ。

| F-xxx | 状態 | reproduce 用途 |
|---|---|---|
| F-030 | retracted by F-090 | bias-detector の retract 経緯確認 (F-086 γ replaces) |
| F-044 | retracted by F-053 | "simulation artifact" 教訓記録、F-053 で real KDF が逆向き判明 |
| F-049, F-050 | retracted by F-053 | 同上 (W series early simulations) |

---

## §4 Gaps (現時点で reproducer 不在 / deferred)

paper / retrospective に F-xxx 番号としては記録されているが、再現 code が現リポジトリで明示パスにない / deferred の finding。本 guide は誠実性のため隠さず明示する。各 gap には不在の **verification command** (= 「本当に code が無い」ことを reader 自身が確認する手順) を併記。

### §4.1 F-045 BEIR SciFact 完全敗北 (code MISSING)

paper §7 で direct cited、KDF が general semantic retrieval に不適である主張の root finding。BEIR dataset と sentence-transformers 環境を reader が独自構築する必要あり。

**不在 verification**:
```bash
ls demos/D8_llm_memory/src/bin/ scripts/ experiments/ 2>/dev/null | grep -iE "beir|scifact"
# 期待: 空出力 (binary / script 不在を確認)

grep -r "BEIR\|scifact" --include="*.rs" --include="*.py" demos/ experiments/ scripts/ 2>/dev/null | head -3
# 期待: 空出力または README 内 mention のみ (実装 code 不在を確認)
```

reader 側で再現したい場合: `pip install beir sentence-transformers` → BEIR SciFact 取得 → KDF select 後 retrieval、recall@10 ~0.000 vs Random ~0.05 となるはず。canonical reproducer は未提供。

### §4.2 F-073, F-074, F-075 Phase 2 Top 3 LOSS (code path 不明確)

Wikipedia orphan / BGL anomaly templates / Citation interdisciplinary bridge の 3 candidates、すべて Random 以下の LOSS。supporting artifacts は存在する可能性あるが canonical entry-point 未確認。

**不在 verification**:
```bash
ls experiments/wikipedia_phase2/ experiments/bgl_phase2/ experiments/citation_phase2/ 2>/dev/null
# 期待: 一部 dir 存在、ただし入口 binary なし or 不完全

ls demos/D8_llm_memory/src/bin/ scripts/ | grep -iE "wikipedia|orphan|citation|interdiscipl"
# 期待: 空出力 (canonical entry-point 不在を確認)
```

注: F-074 (BGL anomaly templates) と F-097 (BGL recurring rare) は同 BGL dataset を使うが finding は別軸 (anomaly preservation vs recurring rare benefit)。`experiments/bgl_phase2/data/BGL.log` を `experiments/g9_bgl_recurring/data/BGL.log` に symlink / copy で reuse 可。

### §4.3 F-088, F-089 HPC/Linux streaming (DEFERRED)

`docs/exploration/phase_2_5_pre_reg_addendum.md §2.3` で意図的に future sprint へ deferred 明記済。code は未着手。

**deferred 状態 verification**:
```bash
grep -A3 "F-088\|F-089\|HPC.log\|Linux.log" docs/exploration/phase_2_5_pre_reg_addendum.md | head -20
# 期待: "deferred" / "future sprint" / "separate pre-reg needed" 等の語が出現

ls demos/D8_llm_memory/src/bin/ | grep -iE "hpc|linux"
# 期待: 空出力 (binary 不在 = deferred 状態)
```

| F-xxx | 状態 | 不在の理由 |
|---|---|---|
| F-045 | code MISSING | reproducer 構築 deferred、reader independent 再現可能 |
| F-073, F-074, F-075 | code path 不明確 | Phase 2 Top 3 negative findings、canonical entry-point 整備 backlog |
| F-088, F-089 | DEFERRED | pre-reg で意図的 deferred (separate sprint 必要) |

---

## §5 Naming convention summary

理解の助けに:

| prefix | 領域 | 例 |
|---|---|---|
| `phase_x{N}` | Phase X = Claim 5/14/17/20-32 realistic benchmark | `phase_x4_nasa_streaming.rs` (F-072) |
| `phase_w*` / `w{N}_*` | W series = LongMemEval / Mem0 比較 | `phase_w3_real_kdf_turns.rs` (F-053) |
| `phase_2_5_*` | Phase 2.5 streaming replication | `phase_2_5_apache_streaming.rs` (F-087) |
| `phase_g{N}` | Phase 2.5+ cross-domain expansion (g2〜g11) | `phase_g9_bgl_recurring.rs` (F-097) |
| `f{NNN}_*` | F-xxx 専用 binary (rare) | `f090_bias_detector_aggregate.rs` (F-090) |

pre-reg 文書は同じ番号で `docs/exploration/g{N}_*_pre_reg.md` 配置。例: F-097 → `g9` 番号 → `docs/exploration/g9_bgl_recurring_pre_reg.md` ↔ `demos/D8_llm_memory/src/bin/phase_g9_bgl_recurring.rs`。

---

## §6 「気楽に検証」 quick start (推奨 3 reproducer)

何から手をつけるか迷う場合、以下 3 つで paper の epistemic stance が掴める:

### §6.1 F-072 streaming benefit anchor (positive、+3.06pt)

```bash
# (1) NASA HTTP log の取得 (~205 MB)
mkdir -p benchmarks/real_data/data/nasa-http
wget -O benchmarks/real_data/data/nasa-http/access.log.gz \
    https://ita.ee.lbl.gov/traces/NASA_access_log_Jul95.gz
gunzip benchmarks/real_data/data/nasa-http/access.log.gz

# (2) reproduce + 機械判定
OUTPUT=$(cargo run --release -p demo-d8-llm-memory --bin phase_x4_nasa_streaming 2>&1)
echo "$OUTPUT" | tail -10  # 出力末尾を表示
if echo "$OUTPUT" | grep -qE "Δ\s*=\s*\+?3\.0[0-9]\s*pt|0\.4898.*0\.4592|\+3\.06"; then
    echo "OK   F-072 reproduced (+3.06pt streaming benefit)"
else
    echo "FAIL F-072 pattern not found — check output above"
fi
# 期待 (約 1-2 分): "OK   F-072 reproduced ..."
```

### §6.2 F-087 narrowing finding (negative、-13.04pt sign reversal)

```bash
# (1) Apache.log 取得 (~5 MB)
mkdir -p experiments/streaming_phase_2_5/data
wget -O experiments/streaming_phase_2_5/data/Apache.log \
    https://raw.githubusercontent.com/logpai/loghub/master/Apache/Apache.log

# (2) reproduce + 機械判定
OUTPUT=$(cargo run --release -p demo-d8-llm-memory --bin phase_2_5_apache_streaming 2>&1)
echo "$OUTPUT" | tail -10
if echo "$OUTPUT" | grep -qE "Δ\s*=\s*-?13\.0[0-9]\s*pt|-13\.04|0\.2631.*0\.3934"; then
    echo "OK   F-087 reproduced (-13.04pt sign reversal, narrowing valid)"
else
    echo "FAIL F-087 pattern not found"
fi
# 期待 (約 30 秒): "OK   F-087 reproduced ..."
```

### §6.3 F-097 positive cross-family (positive、+33.33pt)

```bash
# (1) BGL.log 取得 (~700 MB compressed)
mkdir -p experiments/g9_bgl_recurring/data
# Loghub Zenodo から BGL.tar.gz をダウンロード → 展開
# https://zenodo.org/records/8196385

# (2) reproduce + 機械判定
OUTPUT=$(cargo run --release -p demo-d8-llm-memory --bin phase_g9_bgl_recurring 2>&1)
echo "$OUTPUT" | tail -10
if echo "$OUTPUT" | grep -qE "Δ\s*=\s*\+?33\.3[0-9]\s*pt|\+33\.33|0\.333.*0\.0+"; then
    echo "OK   F-097 reproduced (+33.33pt cross-family N=3 positive)"
else
    echo "FAIL F-097 pattern not found"
fi
# 期待 (約 5-10 分): "OK   F-097 reproduced ..."
```

これら 3 つを動かせば paper の中核(narrow + positive 両方向の epistemic arc)が体感できる。

---

## §7 困った時 / バグ報告

- 期待値と実測値が乖離した: GitHub Issue (https://github.com/ChaiCroquis/kdf-perovskite/issues) に finding 番号 + 環境情報 (Rust version / OS / dataset checksum) を添えて報告
- data 取得 link 切れ: 同上
- pre-reg と code の不一致: pre-reg を canonical とし、code 側の bug 報告として扱う

---

## §8 関連文書

- [paper.md](arxiv_submission/paper.md) — paper v2.7 source
- [VERIFIED_FINDINGS.md](VERIFIED_FINDINGS.md) — F-001〜F-100 全件 verbose 記録
- [PHASE_2_RETROSPECTIVE.md](PHASE_2_RETROSPECTIVE.md) — Phase 2 narrowing arc 単一文書要約
- [exploration/](exploration/) — 各 g{N} pre-registration 文書
- [patent/COMPLIANCE.md](patent/COMPLIANCE.md) — Claim 1-50 → test mapping (F-001 source)

---

最終更新: 2026-05-10 (paper v2.7 / 7-pattern arc 反映)
