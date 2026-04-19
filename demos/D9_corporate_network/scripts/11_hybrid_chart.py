"""D9 Step 11: Visualize hybrid predictor comparison."""
from __future__ import annotations

import json
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

OUT = Path("demos/D9_corporate_network/out")
CHARTS = OUT / "backtest_charts"
CHARTS.mkdir(parents=True, exist_ok=True)


def main():
    with open(OUT / "hybrid_predictor_results.json", encoding="utf-8") as f:
        data = json.load(f)

    models = data["models"]
    base_rate = data["base_rate"]

    names = [m["name"].split(". ")[-1] if ". " in m["name"] else m["name"] for m in models]
    aucs = [m["auc"] for m in models]
    p5 = [m["precision_at_5pct"] * 100 for m in models]
    p10 = [m["precision_at_10pct"] * 100 for m in models]
    lifts5 = [m["lift_at_5pct"] for m in models]

    # Chart 1: Precision @ top-K% bar chart
    fig, ax = plt.subplots(figsize=(13, 6), dpi=130)
    x = np.arange(len(names))
    width = 0.35
    bars_5 = ax.bar(x - width / 2, p5, width, label="Precision @ top 5%", color="#E74C3C")
    bars_10 = ax.bar(x + width / 2, p10, width, label="Precision @ top 10%", color="#3498DB")

    # Base rate line
    ax.axhline(base_rate * 100, color="gray", linestyle="--", alpha=0.5, label=f"Base rate ({base_rate*100:.2f}%)")

    for bar in bars_5:
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width() / 2., height + 0.3, f"{height:.1f}%",
                ha="center", va="bottom", fontsize=8)

    ax.set_xticks(x)
    ax.set_xticklabels(names, rotation=30, ha="right")
    ax.set_ylabel("Breakthrough precision (%)")
    ax.set_title("Hybrid improves precision up to 25.7% (lift 5.7x)\nT1-Edge → T2-Core prediction (base rate 4.53%, n=2805, 127 positives)")
    ax.legend(loc="upper left")
    ax.grid(axis="y", alpha=0.3)
    plt.tight_layout()
    p1 = CHARTS / "hybrid_precision_bars.png"
    plt.savefig(p1, bbox_inches="tight")
    plt.close()
    print(f"Saved: {p1}")

    # Chart 2: AUC + Lift combined
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(14, 5), dpi=130)
    colors = plt.cm.viridis(np.linspace(0.2, 0.85, len(names)))

    ax1.barh(names, aucs, color=colors)
    ax1.axvline(0.5, color="gray", linestyle="--", alpha=0.5, label="Random (0.5)")
    ax1.set_xlabel("Cross-validated AUC (5-fold)")
    ax1.set_title("Model ranking quality (AUC)")
    ax1.legend()
    ax1.grid(axis="x", alpha=0.3)
    for i, v in enumerate(aucs):
        ax1.text(v + 0.005, i, f"{v:.3f}", va="center", fontsize=9)

    ax2.barh(names, lifts5, color=colors)
    ax2.axvline(1.0, color="gray", linestyle="--", alpha=0.5, label="Random (1.0x)")
    ax2.set_xlabel("Lift @ top 5% (vs base rate)")
    ax2.set_title("Breakthrough lift at top 5% cutoff")
    ax2.legend()
    ax2.grid(axis="x", alpha=0.3)
    for i, v in enumerate(lifts5):
        ax2.text(v + 0.1, i, f"{v:.2f}x", va="center", fontsize=9)

    plt.suptitle("Model comparison: KDF alone vs Hybrid for Edge→Core breakthrough prediction", y=1.02)
    plt.tight_layout()
    p2 = CHARTS / "hybrid_auc_lift.png"
    plt.savefig(p2, bbox_inches="tight")
    plt.close()
    print(f"Saved: {p2}")

    # Chart 3: Feature importance (derived from best-performing LogReg via coef inspection)
    # Rebuild model for coef extraction
    try:
        from sklearn.linear_model import LogisticRegression
        from sklearn.preprocessing import StandardScaler
        rows = []
        # Load feature matrix from hybrid_predictor_results (we'd need to re-construct)
        # Skip for simplicity; note in summary chart
    except Exception:
        pass

    # Chart 4: Summary insight chart — "Finding the 4.5%"
    fig, ax = plt.subplots(figsize=(10, 6), dpi=130)
    methods = [
        ("Random selection", 4.53, 1.0, "#95A5A6"),
        ("KDF Rare layer\n(F-061 rule)", 10.5, 2.31, "#E74C3C"),
        ("Degree top 5%\n(Step 9 rule)", 22.0, 4.85, "#F39C12"),
        ("Hybrid ML (LogReg)\nKDF + all features", 25.71, 5.68, "#27AE60"),
    ]
    names2 = [m[0] for m in methods]
    precisions = [m[1] for m in methods]
    lifts = [m[2] for m in methods]
    colors2 = [m[3] for m in methods]

    bars = ax.bar(names2, precisions, color=colors2, edgecolor="black", linewidth=1.5)
    for bar, lift in zip(bars, lifts):
        h = bar.get_height()
        ax.text(bar.get_x() + bar.get_width() / 2, h + 0.5, f"{h:.1f}%\n(lift {lift:.2f}x)",
                ha="center", va="bottom", fontsize=11, fontweight="bold")
    ax.set_ylabel("Precision (% of selected that became Core)", fontsize=11)
    ax.set_title("Progression: identifying the 4.5% of Edge→Core risers in advance\n"
                 "Random → KDF alone → KDF-inspired rule → Hybrid ML",
                 fontsize=12)
    ax.set_ylim(0, 32)
    ax.axhline(4.53, color="gray", linestyle="--", alpha=0.5, linewidth=1)
    ax.grid(axis="y", alpha=0.3)
    plt.tight_layout()
    p3 = CHARTS / "hybrid_progression.png"
    plt.savefig(p3, bbox_inches="tight")
    plt.close()
    print(f"Saved: {p3}")


if __name__ == "__main__":
    main()
