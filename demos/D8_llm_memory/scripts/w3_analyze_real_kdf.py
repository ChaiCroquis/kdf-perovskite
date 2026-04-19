"""Analyze real-KDF F-044 rerun vs Mem0: overall McNemar, per-category, single-session-assistant deep dive."""
from __future__ import annotations
import json
import math
import sys
from collections import defaultdict
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def binom2sided(b: int, c: int) -> float:
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    log_half_n = n * math.log(0.5)
    cdf = 0.0
    for i in range(k + 1):
        log_term = math.lgamma(n + 1) - math.lgamma(i + 1) - math.lgamma(n - i + 1) + log_half_n
        cdf += math.exp(log_term)
    return min(2.0 * cdf, 1.0)


def analyze(rows: list[dict], label: str) -> None:
    n = len(rows)
    if n == 0:
        return
    both = sum(1 for r in rows if r["kdf_real_correct"] and r["mem0_correct"])
    kdf_only = sum(1 for r in rows if r["kdf_real_correct"] and not r["mem0_correct"])
    mem0_only = sum(1 for r in rows if not r["kdf_real_correct"] and r["mem0_correct"])
    both_w = sum(1 for r in rows if not r["kdf_real_correct"] and not r["mem0_correct"])
    kdf_acc = (both + kdf_only) / n
    mem0_acc = (both + mem0_only) / n
    sim_acc = sum(1 for r in rows if r["kdf_sim_correct"]) / n
    diff = kdf_acc - mem0_acc
    p = binom2sided(kdf_only, mem0_only)
    sig = "★" if p < 0.05 else "-"
    sig_dir = ""
    if p < 0.05:
        sig_dir = " (KDF wins)" if kdf_only > mem0_only else " (Mem0 wins)"
    print(
        f"{label:<30}{n:>4}  real={kdf_acc:.3f}  sim={sim_acc:.3f}  mem0={mem0_acc:.3f}  "
        f"diff={diff:+.3f}  b/c={kdf_only}/{mem0_only}  p={p:.4f}  {sig}{sig_dir}"
    )


def main() -> None:
    src = Path("demos/D8_llm_memory/out/route_a_500q_real_kdf_results.json")
    with src.open(encoding="utf-8") as f:
        data = json.load(f)
    rows = data["results"]
    print(f"Loaded {len(rows)} results from {src}\n")

    print("=" * 100)
    print("Real-KDF F-044 rerun: paired outcomes vs Mem0")
    print("=" * 100)
    print()
    print(f"{'group':<30}{'n':>4}  {'real':>7}  {'sim':>7}  {'mem0':>7}  {'diff':>8}  {'b/c':>9}  {'p':>8}  sig")
    print("-" * 100)
    analyze(rows, "overall")
    print()
    print("--- by question_type ---")
    by_cat = defaultdict(list)
    for r in rows:
        by_cat[r["question_type"]].append(r)
    for cat in sorted(by_cat.keys()):
        analyze(by_cat[cat], cat)
    print()

    # Compare sim vs real
    print("=" * 100)
    print("Simulation vs reality: how many q's did sim get right but real get wrong (and vice versa)?")
    print("=" * 100)
    sim_only = [r for r in rows if r["kdf_sim_correct"] and not r["kdf_real_correct"]]
    real_only = [r for r in rows if not r["kdf_sim_correct"] and r["kdf_real_correct"]]
    both_k = [r for r in rows if r["kdf_sim_correct"] and r["kdf_real_correct"]]
    neither_k = [r for r in rows if not r["kdf_sim_correct"] and not r["kdf_real_correct"]]
    print(f"  sim correct & real correct : {len(both_k):3d}")
    print(f"  sim correct & real WRONG   : {len(sim_only):3d}  ← questions F-044 got right by over-approximating recall")
    print(f"  sim WRONG   & real correct : {len(real_only):3d}")
    print(f"  sim wrong   & real wrong   : {len(neither_k):3d}")
    print()

    # Retrieval recall distribution
    recalls = [r["real_kdf_answer_turn_recall"] for r in rows]
    import statistics as st
    print(f"Real retrieval answer_turn_recall: mean={st.mean(recalls):.3f}, median={st.median(recalls):.3f}")
    # Success rate vs recall
    print()
    print("--- Success rate by real retrieval answer_turn_recall bucket ---")
    buckets = [(0.0, 0.0001), (0.0001, 0.25), (0.25, 0.5), (0.5, 0.75), (0.75, 1.0001)]
    for lo, hi in buckets:
        sub = [r for r in rows if lo <= r["real_kdf_answer_turn_recall"] < hi]
        if not sub:
            continue
        acc = sum(1 for r in sub if r["kdf_real_correct"]) / len(sub)
        m_acc = sum(1 for r in sub if r["mem0_correct"]) / len(sub)
        print(f"  recall [{lo:.2f}, {hi:.2f}): n={len(sub):3d}, real_acc={acc:.3f}, mem0_acc={m_acc:.3f}")


if __name__ == "__main__":
    main()
