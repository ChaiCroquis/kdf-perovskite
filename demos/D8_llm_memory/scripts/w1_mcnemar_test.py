"""
W1: McNemar's paired significance test on F-044 500Q results.

Tests whether KDF's +2.4 pt overall advantage over Mem0 on LongMemEval 500Q
is statistically significant, or could be explained by sample variance.

Null hypothesis (H0): KDF and Mem0 accuracies are equal.
Test: McNemar's chi-squared (with continuity correction) + exact binomial.

Inputs:
    demos/D8_llm_memory/out/route_a_500q_results.json  (F-044 raw per-question)

Outputs:
    Prints contingency table, chi-squared, p-values, effect size (odds ratio,
    risk difference), per-category breakdown.
    Writes demos/D8_llm_memory/out/w1_mcnemar_results.json.

Runtime: ~1 second, $0.
"""
from __future__ import annotations

import json
import math
import sys
from collections import defaultdict
from pathlib import Path

# Force UTF-8 output on Windows
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

try:
    from scipy import stats
    HAS_SCIPY = True
except ImportError:
    HAS_SCIPY = False


RESULTS_PATH = Path("demos/D8_llm_memory/out/route_a_500q_results.json")
OUTPUT_PATH = Path("demos/D8_llm_memory/out/w1_mcnemar_results.json")


def binomial_two_sided_exact(b: int, c: int) -> float:
    """Exact two-sided binomial p-value for McNemar's test.

    Under H0, if X ~ Binom(n=b+c, p=0.5), the two-sided p-value is
    2 * P(X <= min(b, c)), capped at 1.0.
    """
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    # P(X <= k) = sum_{i=0}^k C(n,i) * 0.5^n
    log_half_n = n * math.log(0.5)
    cdf = 0.0
    for i in range(k + 1):
        log_term = math.lgamma(n + 1) - math.lgamma(i + 1) - math.lgamma(n - i + 1) + log_half_n
        cdf += math.exp(log_term)
    return min(2.0 * cdf, 1.0)


def mcnemar_chi2(b: int, c: int, continuity: bool = True) -> tuple[float, float]:
    """McNemar's chi-squared statistic and asymptotic p-value.

    With continuity correction (Edwards): chi2 = (|b-c| - 1)^2 / (b+c).
    Without: chi2 = (b-c)^2 / (b+c).
    """
    n = b + c
    if n == 0:
        return 0.0, 1.0
    if continuity:
        chi2 = (abs(b - c) - 1) ** 2 / n if n > 0 else 0.0
    else:
        chi2 = (b - c) ** 2 / n

    # p-value from chi-squared with 1 df
    if HAS_SCIPY:
        p = 1.0 - stats.chi2.cdf(chi2, df=1)
    else:
        # Approximation via erf for df=1: P(chi2 > x) = erfc(sqrt(x/2))
        p = math.erfc(math.sqrt(chi2 / 2.0))
    return chi2, p


def analyze(results: list[dict], label: str = "overall") -> dict:
    """Compute contingency table and McNemar's test for the given results."""
    # Contingency table
    # a: both correct, b: KDF correct / Mem0 wrong, c: KDF wrong / Mem0 correct, d: both wrong
    a = b = c = d = 0
    for r in results:
        kdf = r.get("kdf_correct", False)
        mem0 = r.get("mem0_correct", False)
        if kdf and mem0:
            a += 1
        elif kdf and not mem0:
            b += 1
        elif not kdf and mem0:
            c += 1
        else:
            d += 1

    n = a + b + c + d
    if n == 0:
        return {"label": label, "n": 0, "error": "empty"}

    # Accuracies
    kdf_acc = (a + b) / n
    mem0_acc = (a + c) / n
    diff = kdf_acc - mem0_acc

    # McNemar's tests
    chi2_cc, p_chi2_cc = mcnemar_chi2(b, c, continuity=True)
    chi2_raw, p_chi2_raw = mcnemar_chi2(b, c, continuity=False)
    p_exact = binomial_two_sided_exact(b, c)

    # Odds ratio of discordant pairs (b/c)
    if c > 0:
        odds_ratio_disc = b / c
    else:
        odds_ratio_disc = float("inf") if b > 0 else 1.0

    # 95% CI for the difference of paired proportions (Newcombe method, approximate)
    # Using asymptotic SE: se = sqrt((b+c) - (b-c)^2/n) / n
    discordant = b + c
    if discordant > 0:
        se_diff = math.sqrt(discordant - (b - c) ** 2 / n) / n
    else:
        se_diff = 0.0
    ci_low = diff - 1.96 * se_diff
    ci_high = diff + 1.96 * se_diff

    return {
        "label": label,
        "n": n,
        "contingency": {
            "both_correct (a)": a,
            "kdf_only (b)": b,
            "mem0_only (c)": c,
            "both_wrong (d)": d,
        },
        "kdf_accuracy": round(kdf_acc, 4),
        "mem0_accuracy": round(mem0_acc, 4),
        "diff_kdf_minus_mem0": round(diff, 4),
        "diff_95ci": [round(ci_low, 4), round(ci_high, 4)],
        "discordant_pairs": discordant,
        "mcnemar_chi2_cc": round(chi2_cc, 4),
        "mcnemar_p_chi2_cc": round(p_chi2_cc, 6),
        "mcnemar_chi2_raw": round(chi2_raw, 4),
        "mcnemar_p_chi2_raw": round(p_chi2_raw, 6),
        "mcnemar_p_exact_binomial": round(p_exact, 6),
        "odds_ratio_discordant": round(odds_ratio_disc, 4) if math.isfinite(odds_ratio_disc) else "inf",
        "significant_at_0.05": p_exact < 0.05,
    }


def main() -> None:
    if not RESULTS_PATH.exists():
        print(f"ERROR: results file not found at {RESULTS_PATH}", file=sys.stderr)
        sys.exit(1)

    with RESULTS_PATH.open("r", encoding="utf-8") as f:
        data = json.load(f)

    results = data.get("results", [])
    if not results:
        print("ERROR: no results in file", file=sys.stderr)
        sys.exit(1)

    print(f"Loaded {len(results)} results from {RESULTS_PATH}\n")
    print("=" * 70)
    print("W1: McNemar's paired significance test")
    print("    (F-044: KDF 0.696 vs Mem0 0.672 on 500Q LongMemEval)")
    print("=" * 70)
    print()

    # Overall
    overall = analyze(results, label="overall")

    print("--- Overall (all 500 questions) ---")
    print(f"Contingency table:")
    print(f"                  Mem0 correct  Mem0 wrong")
    c = overall["contingency"]
    print(f"  KDF correct  :       {c['both_correct (a)']:3d}         {c['kdf_only (b)']:3d}   (= KDF_acc * n)")
    print(f"  KDF wrong    :       {c['mem0_only (c)']:3d}         {c['both_wrong (d)']:3d}")
    print()
    print(f"  KDF accuracy  : {overall['kdf_accuracy']:.4f}")
    print(f"  Mem0 accuracy : {overall['mem0_accuracy']:.4f}")
    print(f"  Difference    : {overall['diff_kdf_minus_mem0']:+.4f} (95% CI: [{overall['diff_95ci'][0]:+.4f}, {overall['diff_95ci'][1]:+.4f}])")
    print()
    print(f"  Discordant pairs     : b+c = {overall['discordant_pairs']}")
    print(f"  McNemar's chi^2 (cc) : {overall['mcnemar_chi2_cc']}  → p = {overall['mcnemar_p_chi2_cc']:.4f}")
    print(f"  McNemar's chi^2 (raw): {overall['mcnemar_chi2_raw']}  → p = {overall['mcnemar_p_chi2_raw']:.4f}")
    print(f"  Exact binomial p     : {overall['mcnemar_p_exact_binomial']:.4f}  ← most reliable for small b+c")
    print(f"  Odds ratio (b/c)     : {overall['odds_ratio_discordant']}")
    print()
    sig = "YES" if overall["significant_at_0.05"] else "NO"
    print(f"  *** Statistically significant at alpha=0.05? {sig} ***")
    print()

    # Per-category
    by_cat: dict[str, list[dict]] = defaultdict(list)
    for r in results:
        qt = r.get("question_type", "unknown")
        by_cat[qt].append(r)

    print("--- Per-category ---")
    print(f"{'category':<32}{'n':>5}{'KDF':>8}{'Mem0':>8}{'diff':>9}{'b/c':>9}{'p_exact':>10}{'sig?':>6}")
    cat_results = {}
    for cat in sorted(by_cat.keys()):
        cat_rs = by_cat[cat]
        res = analyze(cat_rs, label=cat)
        cat_results[cat] = res
        c = res["contingency"]
        bc = f"{c['kdf_only (b)']}/{c['mem0_only (c)']}"
        sig = "*" if res["significant_at_0.05"] else ""
        print(
            f"{cat:<32}{res['n']:>5}{res['kdf_accuracy']:>8.3f}{res['mem0_accuracy']:>8.3f}"
            f"{res['diff_kdf_minus_mem0']:>+9.3f}{bc:>9}{res['mcnemar_p_exact_binomial']:>10.4f}{sig:>6}"
        )
    print()
    print("(* = significant at alpha=0.05)")
    print()

    # Interpretation
    print("=" * 70)
    print("Interpretation")
    print("=" * 70)
    p_over = overall["mcnemar_p_exact_binomial"]
    diff_over = overall["diff_kdf_minus_mem0"]
    b_over = overall["contingency"]["kdf_only (b)"]
    c_over = overall["contingency"]["mem0_only (c)"]
    if overall["significant_at_0.05"]:
        print(f"  Overall: KDF wins by {diff_over:+.3f} pt (p={p_over:.4f}). SIGNIFICANT.")
        print(f"  → The +2.4 pt KDF advantage is unlikely due to chance at n=500.")
    else:
        print(f"  Overall: KDF wins by {diff_over:+.3f} pt (p={p_over:.4f}). NOT significant at alpha=0.05.")
        print(f"  → The +2.4 pt overall advantage CANNOT be confidently distinguished")
        print(f"    from sampling variance at n=500 (b={b_over}, c={c_over}).")
        print(f"  → Paper claim should narrow to: 'overall parity + decisive")
        print(f"    single-session-assistant dominance'.")
        # Power calculation: what n would we need?
        # Effect size (discordant proportion difference): assumes current b/(b+c)
        if b_over + c_over > 0:
            p_b = b_over / (b_over + c_over)
            # For two-sided test at alpha=0.05 with power 0.8, need about
            # (z_alpha/2 + z_beta)^2 / (2*p_b - 1)^2 * ... simplified:
            # n_discordant ≈ (1.96 + 0.84)^2 / (2*p_b - 1)^2
            if abs(2 * p_b - 1) > 0.001:
                n_disc_needed = (1.96 + 0.84) ** 2 / (2 * p_b - 1) ** 2
                # Scale to total n assuming same discordant rate
                disc_rate = (b_over + c_over) / overall["n"]
                n_total_needed = n_disc_needed / disc_rate
                print(f"  → For power=0.8 at alpha=0.05, need approximately")
                print(f"    n ≈ {int(n_total_needed)} questions (at current effect size).")
    print()

    # Decisive category check
    sig_cats = [c for c, r in cat_results.items() if r["significant_at_0.05"]]
    if sig_cats:
        print(f"  Significant per-category wins (p < 0.05):")
        for cat in sig_cats:
            r = cat_results[cat]
            print(f"    - {cat}: {r['diff_kdf_minus_mem0']:+.3f} pt (p={r['mcnemar_p_exact_binomial']:.4f})")
    else:
        print(f"  No per-category wins at p < 0.05.")
    print()

    # Save
    output = {
        "test": "W1 McNemar's paired significance test",
        "source": str(RESULTS_PATH),
        "overall": overall,
        "by_category": cat_results,
    }
    OUTPUT_PATH.parent.mkdir(parents=True, exist_ok=True)
    with OUTPUT_PATH.open("w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)
    print(f"Saved: {OUTPUT_PATH}")


if __name__ == "__main__":
    main()
