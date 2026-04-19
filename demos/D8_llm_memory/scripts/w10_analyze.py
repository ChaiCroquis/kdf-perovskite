"""Analyze W10 hybrid vs Mem0: per-category, paired McNemar."""
from __future__ import annotations
import json
import math
import sys
from collections import defaultdict

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


def contingency(rows, label):
    n = len(rows)
    if n == 0:
        return
    both = sum(1 for r in rows if r["hybrid_correct"] and r["mem0_correct"])
    h_only = sum(1 for r in rows if r["hybrid_correct"] and not r["mem0_correct"])
    m_only = sum(1 for r in rows if not r["hybrid_correct"] and r["mem0_correct"])
    hyb = (both + h_only) / n
    m = (both + m_only) / n
    p = binom2sided(h_only, m_only)
    sig = "★" if p < 0.05 else "-"
    dir_ = ""
    if p < 0.05:
        dir_ = " (Mem0)" if m_only > h_only else " (Hyb)"
    print(
        f"  {label:<28}{n:>4}  hyb={hyb:.3f}  mem0={m:.3f}  diff={hyb - m:+.3f}  "
        f"b/c={h_only}/{m_only}  p={p:.4f}  {sig}{dir_}"
    )


def main():
    data = json.load(open("demos/D8_llm_memory/out/route_a_500q_hybrid_results.json", encoding="utf-8"))
    rows = data["results"]
    print(f"W10 Hybrid (Mem0 answer + real KDF raw turns @30%) vs Mem0 alone — n={len(rows)}\n")

    print("--- Overall ---")
    contingency(rows, "overall")
    print()
    print("--- Per-category ---")
    by_cat = defaultdict(list)
    for r in rows:
        by_cat[r["question_type"]].append(r)
    for cat in sorted(by_cat.keys()):
        contingency(by_cat[cat], cat)

    print()
    print("--- Rescue analysis: where does hybrid improve? ---")
    # Compare hybrid vs mem0 across outcomes
    wins = [r for r in rows if r["hybrid_correct"] and not r["mem0_correct"]]
    losses = [r for r in rows if not r["hybrid_correct"] and r["mem0_correct"]]
    print(f"Hybrid wins (Mem0 wrong): {len(wins)}")
    print(f"Hybrid losses (Mem0 right): {len(losses)}")
    print()
    print("--- Per-category rescue/regression ---")
    for cat in sorted(by_cat.keys()):
        cat_wins = [r for r in wins if r["question_type"] == cat]
        cat_losses = [r for r in losses if r["question_type"] == cat]
        print(f"  {cat:<28}  rescues: {len(cat_wins):3d}, regressions: {len(cat_losses):3d}, net: {len(cat_wins)-len(cat_losses):+3d}")


if __name__ == "__main__":
    main()
