"""Compare real KDF at 30% vs 50% keep_rate and vs Mem0, with per-category stats and recall buckets."""
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


def contingency_vs_mem0(rows, label):
    n = len(rows)
    if n == 0:
        return
    both = sum(1 for r in rows if r["kdf_real_correct"] and r["mem0_correct"])
    k_only = sum(1 for r in rows if r["kdf_real_correct"] and not r["mem0_correct"])
    m_only = sum(1 for r in rows if not r["kdf_real_correct"] and r["mem0_correct"])
    kdf_acc = (both + k_only) / n
    m_acc = (both + m_only) / n
    p = binom2sided(k_only, m_only)
    sig = "★" if p < 0.05 else "-"
    dir_ = " (Mem0)" if p < 0.05 and m_only > k_only else (" (KDF)" if p < 0.05 else "")
    print(
        f"  {label:<28}{n:>4}  real={kdf_acc:.3f}  mem0={m_acc:.3f}  diff={kdf_acc - m_acc:+.3f}  "
        f"b/c={k_only}/{m_only}  p={p:.4f}  {sig}{dir_}"
    )


def main():
    r30 = json.load(open("demos/D8_llm_memory/out/route_a_500q_real_kdf_results.json", encoding="utf-8"))
    r50 = json.load(open("demos/D8_llm_memory/out/route_a_500q_real_kdf_050_results.json", encoding="utf-8"))
    rows30 = r30["results"]
    rows50 = r50["results"]

    print("=" * 100)
    print("W3 real-KDF: 30% vs 50% keep_rate vs Mem0 (all 500Q)")
    print("=" * 100)
    n30 = len(rows30); n50 = len(rows50)
    print(f"\n@ keep_rate=0.30 ({n30}Q): real_acc={sum(1 for r in rows30 if r['kdf_real_correct'])/n30:.4f}")
    print(f"@ keep_rate=0.50 ({n50}Q): real_acc={sum(1 for r in rows50 if r['kdf_real_correct'])/n50:.4f}")
    print(f"Mem0 reference        : mem0_acc={sum(1 for r in rows30 if r['mem0_correct'])/n30:.4f}")
    print()

    # Per-category at each keep_rate
    for rate_label, rows in [("keep_rate=0.30", rows30), ("keep_rate=0.50", rows50)]:
        print(f"\n--- {rate_label} per-category ---")
        contingency_vs_mem0(rows, "overall")
        by_cat = defaultdict(list)
        for r in rows:
            by_cat[r["question_type"]].append(r)
        for cat in sorted(by_cat.keys()):
            contingency_vs_mem0(by_cat[cat], cat)

    # Cross-rate paired — did 50% rescue anything?
    print("\n" + "=" * 100)
    print("Cross-rate: which Q's did 50% rescue that 30% missed? (Mem0 side fixed)")
    print("=" * 100)
    rows30_by_id = {r["question_id"]: r for r in rows30}
    both_ok = rescued = regressed = neither = 0
    for r50 in rows50:
        r30 = rows30_by_id.get(r50["question_id"])
        if r30 is None:
            continue
        c30 = r30["kdf_real_correct"]
        c50 = r50["kdf_real_correct"]
        if c30 and c50:
            both_ok += 1
        elif not c30 and c50:
            rescued += 1  # 50% rescued
        elif c30 and not c50:
            regressed += 1
        else:
            neither += 1
    print(f"  correct at both 30% and 50%: {both_ok}")
    print(f"  rescued by 50% (30% wrong → 50% right): {rescued}  ← benefit of larger budget")
    print(f"  regressed at 50% (30% right → 50% wrong): {regressed}")
    print(f"  wrong at both: {neither}")
    print()
    print(f"Net gain: +{rescued - regressed} correct ({(rescued - regressed) / max(n50, 1) * 100:.1f}pp)")


if __name__ == "__main__":
    main()
