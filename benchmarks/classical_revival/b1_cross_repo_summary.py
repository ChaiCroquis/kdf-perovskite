"""Consolidate B1 replication results across tokio / pytest / lodash."""
from __future__ import annotations
import json
import sys
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def main():
    repos = ["tokio", "pytest", "lodash"]
    out_dir = Path("benchmarks/classical_revival/out")

    print("=" * 110)
    print("B1 Cross-repo replication summary (3 repos × 4 methods × 2 budgets)")
    print("=" * 110)
    # Show basic stats
    stats = {}
    for r in repos:
        p = out_dir / f"b1_{r}_results.json"
        if not p.exists():
            continue
        with p.open(encoding="utf-8") as f:
            d = json.load(f)
        stats[r] = d
        print(f"\n{r}: {d['n_commits']} commits, {d['n_tag_commits']} tags "
              f"({d['n_tag_commits']/d['n_commits']*100:.1f}%), "
              f"{d['n_merge_commits']} merges ({d['n_merge_commits']/d['n_commits']*100:.1f}%), "
              f"{d['n_pr_merge_commits']} PR-merges ({d['n_pr_merge_commits']/d['n_commits']*100:.1f}%)")

    # Summary tables
    for metric_key, metric_label in [
        ("tag_recall", "Tag recall (release commits)"),
        ("merge_recall", "Merge recall (2+ parents)"),
    ]:
        print(f"\n### {metric_label}")
        print(f"{'repo':<10}{'keep':>6}{'KDF':>10}{'Random':>10}{'TTL':>10}{'TopDeg':>10}")
        for r in repos:
            if r not in stats:
                continue
            for keep in ["30", "50"]:
                key = f"keep_{keep}"
                if key not in stats[r]["results_by_keep_rate"]:
                    continue
                results = stats[r]["results_by_keep_rate"][key]
                print(f"{r:<10}{keep+'%':>6}"
                      f"{results['KDF'][metric_key]*100:>9.2f}%"
                      f"{results['Random'][metric_key]*100:>9.2f}%"
                      f"{results['TTL_recent'][metric_key]*100:>9.2f}%"
                      f"{results['TopDegree'][metric_key]*100:>9.2f}%")

    # KDF vs Random gain (positive = KDF wins)
    print("\n### KDF - Random gain (positive = KDF better than Random)")
    print(f"{'repo':<10}{'keep':>6}{'tag_gain':>12}{'merge_gain':>14}")
    for r in repos:
        if r not in stats:
            continue
        for keep in ["30", "50"]:
            key = f"keep_{keep}"
            if key not in stats[r]["results_by_keep_rate"]:
                continue
            results = stats[r]["results_by_keep_rate"][key]
            tag_gain = results["KDF"]["tag_recall"] - results["Random"]["tag_recall"]
            merge_gain = results["KDF"]["merge_recall"] - results["Random"]["merge_recall"]
            print(f"{r:<10}{keep+'%':>6}"
                  f"{tag_gain*100:>+11.2f}%"
                  f"{merge_gain*100:>+13.2f}%")


if __name__ == "__main__":
    main()
