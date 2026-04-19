#!/usr/bin/env python3
"""KDF Showcase — per-demo visualization renderer.

Reads a report.json emitted by any demo's Rust binary and emits:
- bar_comparison.svg   — one bar per (method, metric)
- tradeoff_scatter.svg — 2D metric pair scatter
- kdf_axis_diagram.svg — axis-A / B / C layered view

Usage:
    python render_visualizations.py <path/to/report.json>

Dependencies:
    pip install matplotlib
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

# Force UTF-8 stdout (Windows cp932 default breaks on em-dash etc.)
if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")

import matplotlib
matplotlib.use("Agg")  # non-interactive
import matplotlib.pyplot as plt

# CJK font fallback — try available fonts in priority order
_CJK_FONT_CANDIDATES = [
    "Noto Sans CJK JP", "Noto Sans JP", "Yu Gothic", "Meiryo",
    "MS Gothic", "Hiragino Sans", "TakaoGothic", "IPAGothic", "IPAexGothic",
    "Source Han Sans JP", "WenQuanYi Zen Hei", "DejaVu Sans",
]
matplotlib.rcParams["font.sans-serif"] = _CJK_FONT_CANDIDATES + matplotlib.rcParams["font.sans-serif"]
matplotlib.rcParams["axes.unicode_minus"] = False


def load_report(path: Path) -> dict[str, Any]:
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def render_bar_comparison(report: dict[str, Any], out_dir: Path) -> None:
    """Bar chart per metric, grouped by method."""
    metric_defs = report["metric_definitions"]
    method_results = report["method_results"]

    metrics = [m["name"] for m in metric_defs]
    methods = [r["method"] for r in method_results]

    fig, axes = plt.subplots(1, len(metrics), figsize=(3 * len(metrics), 4), squeeze=False)
    axis_color = {
        "KdfStrength": "#2E7D32",  # green
        "Tie": "#616161",           # gray
        "KdfWeakness": "#C62828",  # red
    }

    for i, m in enumerate(metric_defs):
        ax = axes[0, i]
        vals = [r["metrics"].get(m["name"], float("nan")) for r in method_results]
        colors = [
            axis_color[m["axis"]] if r["method"] == "KDF" else "#90A4AE"
            for r in method_results
        ]
        bars = ax.bar(range(len(methods)), vals, color=colors)
        ax.set_title(
            f"{m['name']}\n({m['axis']}, {'↑' if m['higher_is_better'] else '↓'})",
            fontsize=9,
        )
        ax.set_xticks(range(len(methods)))
        ax.set_xticklabels(methods, rotation=45, ha="right", fontsize=8)
        ax.grid(axis="y", alpha=0.3)
        for b, v in zip(bars, vals):
            if v == v:  # not NaN
                ax.text(
                    b.get_x() + b.get_width() / 2, v,
                    f"{v:.2f}",
                    ha="center", va="bottom", fontsize=7,
                )

    fig.suptitle(
        f"Demo {report['demo_id']}: {report['title']}",
        fontsize=11,
    )
    fig.tight_layout()
    out_path = out_dir / "bar_comparison.svg"
    fig.savefig(out_path, format="svg", bbox_inches="tight")
    plt.close(fig)
    print(f"  → {out_path}")


def render_tradeoff(report: dict[str, Any], out_dir: Path) -> None:
    """Scatter: first KdfStrength metric vs first KdfWeakness metric."""
    metric_defs = report["metric_definitions"]
    strength = next((m for m in metric_defs if m["axis"] == "KdfStrength"), None)
    weakness = next((m for m in metric_defs if m["axis"] == "KdfWeakness"), None)
    if not strength or not weakness:
        return

    xs = [r["metrics"].get(strength["name"], float("nan")) for r in report["method_results"]]
    ys = [r["metrics"].get(weakness["name"], float("nan")) for r in report["method_results"]]
    names = [r["method"] for r in report["method_results"]]

    fig, ax = plt.subplots(figsize=(6, 5))
    for x, y, name in zip(xs, ys, names):
        color = "#2E7D32" if name == "KDF" else "#90A4AE"
        size = 200 if name == "KDF" else 80
        ax.scatter(x, y, c=color, s=size, edgecolors="black", linewidths=0.5)
        ax.annotate(name, (x, y), xytext=(5, 5), textcoords="offset points", fontsize=8)

    ax.set_xlabel(
        f"{strength['name']} ({'↑' if strength['higher_is_better'] else '↓'})",
    )
    ax.set_ylabel(
        f"{weakness['name']} ({'↑' if weakness['higher_is_better'] else '↓'})",
    )
    ax.set_title(f"Trade-off: {report['demo_id']}")
    ax.grid(alpha=0.3)
    fig.tight_layout()
    out_path = out_dir / "tradeoff_scatter.svg"
    fig.savefig(out_path, format="svg", bbox_inches="tight")
    plt.close(fig)
    print(f"  → {out_path}")


def render_axis_diagram(report: dict[str, Any], out_dir: Path) -> None:
    """Horizontal stacked diagram showing which axis each metric sits in."""
    metric_defs = report["metric_definitions"]
    method_results = report["method_results"]

    axis_order = ["KdfStrength", "Tie", "KdfWeakness"]
    axis_label = {
        "KdfStrength": "軸A: KDF 強み",
        "Tie": "軸B: 同等",
        "KdfWeakness": "軸C: KDF 弱み",
    }
    axis_bg = {
        "KdfStrength": "#E8F5E9",
        "Tie": "#F5F5F5",
        "KdfWeakness": "#FFEBEE",
    }

    fig, ax = plt.subplots(figsize=(10, 3 + 0.4 * len(metric_defs)))
    ax.set_xlim(-0.05, 1.1)
    ax.set_ylim(-0.5, len(metric_defs))
    ax.axis("off")

    # background bands per axis
    for ai, axis_name in enumerate(axis_order):
        ms = [m for m in metric_defs if m["axis"] == axis_name]
        if not ms:
            continue
        y_top = len(metric_defs) - metric_defs.index(ms[0])
        y_bottom = len(metric_defs) - metric_defs.index(ms[-1]) - 1
        ax.axhspan(y_bottom, y_top, facecolor=axis_bg[axis_name], alpha=0.5)
        ax.text(
            1.05, (y_top + y_bottom) / 2,
            axis_label[axis_name],
            fontsize=9, va="center", ha="left",
        )

    for mi, m in enumerate(metric_defs):
        y = len(metric_defs) - mi - 1
        vals = [r["metrics"].get(m["name"], float("nan")) for r in method_results]
        vmax = max((v for v in vals if v == v), default=1.0)
        vmin = min((v for v in vals if v == v), default=0.0)
        rng = vmax - vmin if vmax > vmin else 1.0
        ax.text(-0.03, y + 0.1, m["name"], ha="right", va="center", fontsize=8)
        for r, v in zip(method_results, vals):
            if v != v:
                continue
            xnorm = (v - vmin) / rng
            color = "#2E7D32" if r["method"] == "KDF" else "#90A4AE"
            size = 150 if r["method"] == "KDF" else 40
            ax.scatter(xnorm, y + 0.1, c=color, s=size, edgecolors="black", linewidths=0.3, zorder=3)
            if r["method"] == "KDF":
                ax.annotate("KDF", (xnorm, y + 0.1), xytext=(8, 0),
                            textcoords="offset points", fontsize=7, zorder=4)

    fig.suptitle(
        f"Demo {report['demo_id']}: {report['title']} — 3-axis metric view",
        fontsize=10, y=0.98,
    )
    fig.tight_layout()
    out_path = out_dir / "kdf_axis_diagram.svg"
    fig.savefig(out_path, format="svg", bbox_inches="tight")
    plt.close(fig)
    print(f"  → {out_path}")


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: render_visualizations.py <report.json>", file=sys.stderr)
        return 1
    report_path = Path(sys.argv[1])
    if not report_path.exists():
        print(f"not found: {report_path}", file=sys.stderr)
        return 1
    out_dir = report_path.parent
    report = load_report(report_path)
    print(f"Rendering {report['demo_id']} ({report['title']})")
    render_bar_comparison(report, out_dir)
    render_tradeoff(report, out_dir)
    render_axis_diagram(report, out_dir)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
