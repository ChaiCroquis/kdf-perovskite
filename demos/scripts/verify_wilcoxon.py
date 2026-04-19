#!/usr/bin/env python3
"""Phase A — compute Wilcoxon signed-rank test for every demo's raw_trials.

Reads each `demos/D*/out/report.json`, extracts per-method raw trial vectors,
and runs KDF-variant vs baseline Wilcoxon tests. Emits
`demos/verification/wilcoxon_summary.md`.

Pure stdlib (no scipy) — we reimplement the same normal-approximation Wilcoxon
as used inside the Rust `real_data_bench::wilcoxon` crate, to cross-validate.
"""
from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import Sequence

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")


def erf(x: float) -> float:
    """Abramowitz & Stegun 7.1.26 — matches our Rust impl."""
    sign = -1.0 if x < 0 else 1.0
    x = abs(x)
    a1, a2, a3, a4, a5 = 0.254829592, -0.284496736, 1.421413741, -1.453152027, 1.061405429
    p = 0.3275911
    t = 1.0 / (1.0 + p * x)
    y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * math.exp(-x * x)
    return sign * y


def standard_normal_cdf(x: float) -> float:
    return 0.5 * (1.0 + erf(x / math.sqrt(2)))


def wilcoxon_signed_rank(x: Sequence[float], y: Sequence[float]) -> dict | None:
    assert len(x) == len(y)
    if len(x) == 0:
        return None
    diffs = [(abs(a - b), 1 if a - b > 0 else -1 if a - b < 0 else 0) for a, b in zip(x, y)]
    diffs = [d for d in diffs if d[0] > 1e-12]
    n = len(diffs)
    if n == 0:
        return None
    diffs.sort(key=lambda d: d[0])
    # Average-rank for ties
    ranks = [0.0] * n
    i = 0
    while i < n:
        j = i
        while j + 1 < n and abs(diffs[j + 1][0] - diffs[i][0]) < 1e-12:
            j += 1
        avg = (i + 1 + j + 1) / 2
        for k in range(i, j + 1):
            ranks[k] = avg
        i = j + 1
    w_plus = sum(r for r, d in zip(ranks, diffs) if d[1] > 0)
    w_minus = sum(r for r, d in zip(ranks, diffs) if d[1] < 0)
    n_f = float(n)
    mean_w = n_f * (n_f + 1) / 4
    var_w = n_f * (n_f + 1) * (2 * n_f + 1) / 24
    if var_w == 0:
        return None
    signed = 1.0 if (w_plus - mean_w) > 0 else -1.0 if (w_plus - mean_w) < 0 else 0.0
    z = (w_plus - mean_w - 0.5 * signed) / math.sqrt(var_w)
    p = 2.0 * (1.0 - standard_normal_cdf(abs(z)))
    all_diffs = [a - b for a, b in zip(x, y)]
    all_diffs.sort()
    median_diff = all_diffs[len(all_diffs) // 2]
    return {
        "n_effective": n,
        "z": z,
        "p_value": p,
        "significant_at_01": p < 0.01,
        "median_diff": median_diff,
    }


def main() -> int:
    demos_dir = Path(__file__).resolve().parent.parent
    out_dir = demos_dir / "verification"
    out_dir.mkdir(exist_ok=True)
    out_md = out_dir / "wilcoxon_summary.md"

    reports = sorted(demos_dir.glob("D*_*/out/report.json"))
    if not reports:
        print("No report.json files found")
        return 1

    md_lines = []
    md_lines.append("# Phase A — Wilcoxon signed-rank 統計検定(全 demos 横断)")
    md_lines.append("")
    md_lines.append("各 demo で、KDF variants vs 各 baseline の対応サンプル(trial seed 揃え)を用いた")
    md_lines.append("Wilcoxon signed-rank 検定。実装は Rust の `real_data_bench::wilcoxon` と同一")
    md_lines.append("(A&S 7.1.26 erf、正規近似、連続性補正)をクロスチェック目的で Python 側に再実装。")
    md_lines.append("")

    for rp in reports:
        with open(rp, "r", encoding="utf-8") as f:
            report = json.load(f)
        demo_id = report["demo_id"]
        title = report["title"]
        raw = report["raw_trials"]  # {"method/metric": [vals...]}
        md_lines.append(f"## {demo_id}: {title}")
        md_lines.append("")
        md_lines.append(f"Dataset: {report['dataset_name']} (n={report['n_items']})")
        md_lines.append("")

        # Group by metric name, extract methods
        methods = sorted({k.split("/")[0] for k in raw})
        metrics = sorted({k.split("/")[1] for k in raw})
        kdf_methods = [m for m in methods if m.startswith("KDF")]
        other_methods = [m for m in methods if not m.startswith("KDF")]

        for metric in metrics:
            md_lines.append(f"### Metric: `{metric}`")
            md_lines.append("")
            md_lines.append("| KDF variant | vs baseline | n | median diff | z | p | sig@0.01 |")
            md_lines.append("|---|---|---:|---:|---:|---:|:---:|")
            any_row = False
            for kdf in kdf_methods:
                k_vals = raw.get(f"{kdf}/{metric}", [])
                for base in other_methods:
                    b_vals = raw.get(f"{base}/{metric}", [])
                    if not k_vals or not b_vals or len(k_vals) != len(b_vals):
                        continue
                    w = wilcoxon_signed_rank(k_vals, b_vals)
                    if w is None:
                        md_lines.append(f"| {kdf} | {base} | — | — | — | — | — |")
                    else:
                        sig = "**YES**" if w["significant_at_01"] else "no"
                        md_lines.append(
                            f"| {kdf} | {base} | {w['n_effective']} | "
                            f"{w['median_diff']:+.3f} | {w['z']:+.2f} | "
                            f"{w['p_value']:.3f} | {sig} |"
                        )
                        any_row = True
            if not any_row:
                md_lines.append("| _(no paired data)_ | | | | | | |")
            md_lines.append("")

    # Summary section
    md_lines.append("## 総合サマリ")
    md_lines.append("")
    md_lines.append("統計的有意性 (α=0.01) の判定を各 (demo, metric, KDF vs baseline) タプルで実施。")
    md_lines.append("")
    md_lines.append("`report.json` の raw_trials 配列は seed 一対一対応なので、paired Wilcoxon が")
    md_lines.append("最適。α=0.01 に設定して多重検定を部分的に吸収(補正なし、複数確認用途)。")
    md_lines.append("")

    with open(out_md, "w", encoding="utf-8") as f:
        f.write("\n".join(md_lines))

    print(f"✅ Wrote {out_md}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
