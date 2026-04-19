"""Analyze W5 LoCoMo results: per-category, McNemar, comparison to LongMemEval."""
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


def paired(rows, label):
    n = len(rows)
    if n == 0:
        return
    both = sum(1 for r in rows if r["kdf_correct"] and r["mem0_correct"])
    k_only = sum(1 for r in rows if r["kdf_correct"] and not r["mem0_correct"])
    m_only = sum(1 for r in rows if not r["kdf_correct"] and r["mem0_correct"])
    both_w = n - both - k_only - m_only
    k_acc = (both + k_only) / n
    m_acc = (both + m_only) / n
    p = binom2sided(k_only, m_only)
    sig = "★" if p < 0.05 else "-"
    d = " (Mem0)" if p < 0.05 and m_only > k_only else (" (KDF)" if p < 0.05 else "")
    print(
        f"  {label:<24}{n:>4}  mem0={m_acc:.3f}  kdf={k_acc:.3f}  diff={k_acc-m_acc:+.3f}  "
        f"b/c={k_only}/{m_only}  p={p:.4f}  {sig}{d}"
    )
    return {"n": n, "mem0_acc": m_acc, "kdf_acc": k_acc, "diff": k_acc-m_acc,
            "k_only": k_only, "m_only": m_only, "p": p, "both": both, "both_w": both_w}


def main():
    data = json.load(open("demos/D8_llm_memory/out/w5_locomo_results.json", encoding="utf-8"))
    rows = data["results"]
    print(f"W5 LoCoMo: {len(rows)} Q, model={data['model']}")
    print()
    print("--- Overall ---")
    paired(rows, "overall")
    print()
    print("--- Per-category ---")
    by_cat = defaultdict(list)
    for r in rows:
        by_cat[r["question_type"]].append(r)
    for cat in sorted(by_cat.keys()):
        paired(by_cat[cat], cat)
    print()

    # Recall bucket analysis
    print("--- Success rate by KDF real retrieval recall bucket ---")
    buckets = [(0.0, 0.0001), (0.0001, 0.25), (0.25, 0.5), (0.5, 0.75), (0.75, 1.0001)]
    for lo, hi in buckets:
        sub = [r for r in rows if lo <= r["kdf_real_turn_recall"] < hi]
        if not sub:
            continue
        k_acc = sum(1 for r in sub if r["kdf_correct"]) / len(sub)
        m_acc = sum(1 for r in sub if r["mem0_correct"]) / len(sub)
        print(f"  recall [{lo:.2f}, {hi:.2f}): n={len(sub):3d}, kdf={k_acc:.3f}, mem0={m_acc:.3f}")
    print()

    # Compare to LongMemEval real KDF
    print("--- Comparison to LongMemEval (F-053 real KDF, n=500) ---")
    print(f"  Benchmark         n     Mem0    KDF    gap (KDF-Mem0)")
    print(f"  LongMemEval     500   0.672  0.434  -0.238  (p<1e-16)")
    print(f"  LoCoMo          {len(rows)}   {sum(1 for r in rows if r['mem0_correct'])/len(rows):.3f}  {sum(1 for r in rows if r['kdf_correct'])/len(rows):.3f}  {sum(1 for r in rows if r['kdf_correct'])/len(rows) - sum(1 for r in rows if r['mem0_correct'])/len(rows):+.3f}")
    print()

    # Haystack sizes
    print("--- Haystack size (n_total_turns) distribution ---")
    ns = [r["n_total_turns"] for r in rows]
    import statistics as st
    print(f"  n_turns: mean={st.mean(ns):.1f}, median={st.median(ns):.0f}, min={min(ns)}, max={max(ns)}")


if __name__ == "__main__":
    main()
