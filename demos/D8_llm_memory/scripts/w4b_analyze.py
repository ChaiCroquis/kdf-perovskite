"""W4b analysis: LongMemEval 500Q × gpt-4.1-mini per-category vs F-053 (gpt-4o-mini)."""
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
    cdf = sum(math.exp(math.lgamma(n+1)-math.lgamma(i+1)-math.lgamma(n-i+1)+log_half_n) for i in range(k+1))
    return min(2.0*cdf, 1.0)


def paired(rows, label):
    n = len(rows)
    if n == 0:
        return
    both = sum(1 for r in rows if r["kdf_correct"] and r["mem0_correct"])
    k_only = sum(1 for r in rows if r["kdf_correct"] and not r["mem0_correct"])
    m_only = sum(1 for r in rows if not r["kdf_correct"] and r["mem0_correct"])
    k_acc = (both + k_only) / n
    m_acc = (both + m_only) / n
    p = binom2sided(k_only, m_only)
    sig = "★" if p < 0.05 else "-"
    d = ""
    if p < 0.05:
        d = " (Mem0)" if m_only > k_only else " (KDF)"
    print(f"  {label:<30}{n:>5}  mem0={m_acc:.3f}  kdf={k_acc:.3f}  diff={k_acc-m_acc:+.3f}  b/c={k_only}/{m_only}  p={p:.2e}  {sig}{d}")


def main():
    data = json.load(open("demos/D8_llm_memory/out/w4b_longmemeval_41mini_results.json", encoding="utf-8"))
    rows = data["results"]
    print(f"W4b LongMemEval × gpt-4.1-mini: {len(rows)} Q\n")
    print("--- Overall ---")
    paired(rows, "overall")
    print("\n--- Per-category ---")
    by_cat = defaultdict(list)
    for r in rows:
        by_cat[r["question_type"]].append(r)
    for cat in sorted(by_cat.keys()):
        paired(by_cat[cat], cat)
    print()

    # Cross-finding comparison
    print("\n--- F-053 (gpt-4o-mini) vs W4b (gpt-4.1-mini) overall ---")
    print(f"| benchmark | model | Mem0 | KDF | gap | p |")
    print(f"|---|---|---|---|---|---|")
    print(f"| LongMemEval 500 | gpt-4o-mini (F-053) | 0.672 | 0.434 | -0.238 | <1e-16 |")
    print(f"| LongMemEval 500 | gpt-4.1-mini (W4b)  | 0.722 | 0.452 | -0.270 | 3e-23 |")
    print(f"| LoCoMo temporal 321 | gpt-4o-mini (F-057) | 0.206 | 0.312 | +0.106 | 1.4e-3 |")
    print(f"| LoCoMo temporal 321 | gpt-4.1-mini (F-058) | 0.090 | 0.324 | +0.234 | 1.6e-14 |")


if __name__ == "__main__":
    main()
