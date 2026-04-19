"""
D9 Step 9: Can KDF identify the "risers" in advance?

User's question:
  60% of top-tier stays, but 40% gets replaced. Can KDF identify
  the 4.5% Edge → Core breakthrough *from T1 signals alone*?

If YES: KDF's T1 features (layer, degree, n_fields) are leading indicators.
If NO: KDF only describes current state, doesn't predict break-ins.

Method:
  Cohort A: T1-Edge institutions that became Core at T2 (risers, n=127)
  Cohort B: T1-Edge institutions that stayed Edge at T2 (stayers, n=1050)
  Cohort C: T1-Edge institutions that disappeared at T2 (faders, n=1598)

  Compare T1 features (degree, n_fields, n_papers) between cohorts.
  Statistical test: Mann-Whitney U for feature distributions.

Report:
  - Feature distributions per cohort
  - Effect sizes
  - Predictive rules (e.g. "T1 Edge with degree > X predicts Core breakthrough at rate Y%")
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from statistics import mean, median, stdev

try:
    from scipy.stats import mannwhitneyu
    HAS_SCIPY = True
except ImportError:
    HAS_SCIPY = False

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def cohen_d(x, y):
    """Standardized mean difference."""
    if not x or not y:
        return None
    nx, ny = len(x), len(y)
    mx, my = mean(x), mean(y)
    if nx > 1 and ny > 1:
        sx, sy = stdev(x), stdev(y)
        pooled = (((nx - 1) * sx ** 2 + (ny - 1) * sy ** 2) / (nx + ny - 2)) ** 0.5
        return (mx - my) / pooled if pooled > 0 else None
    return None


def percentile_breakthrough_rate(edges_all, feature, threshold_pcts=[50, 70, 85, 95]):
    """For T1-Edge institutions, compute breakthrough rate above each percentile."""
    if not edges_all:
        return []
    import numpy as np
    values = np.array([e[feature] for e in edges_all])
    results = []
    for pct in threshold_pcts:
        thresh = np.percentile(values, pct)
        above = [e for e in edges_all if e[feature] >= thresh]
        if not above:
            continue
        risers = [e for e in above if e.get("in_t2") and e["t2_layer"] == "Core"]
        rate = len(risers) / len(above) * 100
        results.append((pct, thresh, len(above), len(risers), rate))
    return results


def main():
    data = json.load(open("demos/D9_corporate_network/out/backtest_results.json", encoding="utf-8"))
    joined = data["joined"]

    # T1 Edge cohorts
    t1_edges = [r for r in joined if r["t1_layer"] == "Edge"]
    risers = [r for r in t1_edges if r.get("in_t2") and r["t2_layer"] == "Core"]
    stayers = [r for r in t1_edges if r.get("in_t2") and r["t2_layer"] == "Edge"]
    faders = [r for r in t1_edges if not r.get("in_t2")]

    print(f"=" * 80)
    print("T1 Edge の運命分解")
    print(f"=" * 80)
    total_edge = len(t1_edges)
    print(f"T1 Edge total: {total_edge}")
    print(f"  ★ Risers (→Core):  {len(risers)} ({len(risers)/total_edge*100:.1f}%)")
    print(f"    Stayers (→Edge): {len(stayers)} ({len(stayers)/total_edge*100:.1f}%)")
    print(f"    Faders(消失):    {len(faders)} ({len(faders)/total_edge*100:.1f}%)")

    print("\n" + "=" * 80)
    print("T1 特徴量の分布比較(risers vs stayers vs faders)")
    print(f"=" * 80)
    features = ["t1_degree", "t1_n_fields", "t1_n_papers"]
    for feature in features:
        r_vals = [r[feature] for r in risers]
        s_vals = [r[feature] for r in stayers]
        f_vals = [r[feature] for r in faders]
        print(f"\n--- {feature} ---")
        print(f"  Risers  (n={len(r_vals):4d}): mean={mean(r_vals):8.2f}, median={median(r_vals):6.2f}, max={max(r_vals):6.0f}")
        print(f"  Stayers (n={len(s_vals):4d}): mean={mean(s_vals):8.2f}, median={median(s_vals):6.2f}, max={max(s_vals):6.0f}")
        print(f"  Faders  (n={len(f_vals):4d}): mean={mean(f_vals):8.2f}, median={median(f_vals):6.2f}, max={max(f_vals):6.0f}")
        d_rs = cohen_d(r_vals, s_vals)
        if d_rs is not None:
            print(f"  Cohen's d (risers vs stayers): {d_rs:+.3f}  {'★ effect' if abs(d_rs) > 0.5 else ''}")
        if HAS_SCIPY and r_vals and s_vals:
            u, p = mannwhitneyu(r_vals, s_vals, alternative="two-sided")
            print(f"  Mann-Whitney U p-value: {p:.4g}  {'★ significant' if p < 0.05 else ''}")

    print("\n" + "=" * 80)
    print("予測規則:T1 feature の高 percentile でいるほど、Core 昇格率は?")
    print(f"=" * 80)
    # Base rate = overall Edge → Core rate
    active_edges = [e for e in t1_edges if e.get("in_t2")]
    base_rate = len([e for e in active_edges if e["t2_layer"] == "Core"]) / len(active_edges) * 100
    print(f"Base rate (T2 で active な T1 Edge の Core 昇格率): {base_rate:.1f}%")

    for feature in features:
        print(f"\n--- T1 {feature} 上位 percentile での Core 昇格率 ---")
        results = percentile_breakthrough_rate(t1_edges, feature)
        print(f"  {'percentile':<12}{'threshold':>12}{'n_above':>10}{'n_risers':>12}{'breakthrough %':>18}{'vs base':>10}")
        for pct, thresh, n_above, n_risers, rate in results:
            vs_base = rate - base_rate
            marker = " ★" if abs(vs_base) > 3 else ""
            print(f"  top {100-pct:>3}%     {thresh:>10.2f}{n_above:>10}{n_risers:>12}{rate:>16.2f}%{vs_base:>+10.2f}pt{marker}")

    # Combined rule test: (degree > X) AND (n_fields >= Y)
    print("\n" + "=" * 80)
    print("複合 rule: T1 degree & n_fields 両方高い場合")
    print(f"=" * 80)
    import numpy as np
    deg_p85 = np.percentile([e["t1_degree"] for e in t1_edges], 85)
    print(f"\nRule: T1 degree >= {deg_p85:.1f}(top 15%)AND n_fields >= 2")
    selected = [e for e in t1_edges if e["t1_degree"] >= deg_p85 and e["t1_n_fields"] >= 2]
    sel_risers = [e for e in selected if e.get("in_t2") and e["t2_layer"] == "Core"]
    print(f"  該当: {len(selected)} 件 / T1 Edge 全 {len(t1_edges)} 件 = {len(selected)/len(t1_edges)*100:.1f}%")
    print(f"  うち Core 昇格: {len(sel_risers)} 件 = {len(sel_risers)/max(len(selected),1)*100:.1f}%")
    print(f"  base rate との差: {len(sel_risers)/max(len(selected),1)*100 - base_rate:+.1f}pt")

    # Show the actual riser names and their T1 characteristics
    print("\n" + "=" * 80)
    print(f"実際の riser 一覧(T1 Edge → T2 Core、計 {len(risers)} 件から上位 20)")
    print(f"=" * 80)
    risers_sorted = sorted(risers, key=lambda r: -r["t1_degree"])
    print(f"  {'#':<4}{'name':<55}{'country':<6}{'type':<12}{'T1 deg':>8}{'T1 nf':>6}{'T2 deg':>8}")
    for i, r in enumerate(risers_sorted[:20], 1):
        name = (r.get("name") or "")[:53]
        country = r.get("country") or "?"
        ty = (r.get("type") or "?")[:10]
        t1_deg = int(r["t1_degree"])
        t1_nf = r["t1_n_fields"]
        t2_deg = int(r.get("t2_degree", 0))
        print(f"  {i:<4}{name:<55}{country:<6}{ty:<12}{t1_deg:>8}{t1_nf:>6}{t2_deg:>8}")

    # Save
    out = Path("demos/D9_corporate_network/out/breakthrough_analysis.json")
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "n_t1_edge": total_edge,
            "n_risers": len(risers),
            "n_stayers": len(stayers),
            "n_faders": len(faders),
            "base_rate_pct": base_rate,
            "risers_list": risers_sorted,
        }, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {out}")


if __name__ == "__main__":
    main()
