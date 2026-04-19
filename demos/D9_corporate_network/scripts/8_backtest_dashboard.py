"""
D9 Step 8: Build backtest dashboard with visualizations.

Generates:
  - backtest_charts/transition_matrix_heatmap.png
  - backtest_charts/base_rate_bars.png
  - backtest_charts/fields_expansion_dist.png
  - backtest_dashboard.html (combining charts + tables + commentary)

Input: out/backtest_results.json
"""
from __future__ import annotations

import html
import json
import sys
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


OUT_DIR = Path("demos/D9_corporate_network/out")
CHARTS_DIR = OUT_DIR / "backtest_charts"
CHARTS_DIR.mkdir(parents=True, exist_ok=True)

LAYER_ORDER = ["Rare", "Core", "Edge", "Garbage"]
LAYER_COLORS = {
    "Rare": "#E74C3C",
    "Core": "#3498DB",
    "Edge": "#95A5A6",
    "Garbage": "#BDC3C7",
    "disappeared": "#34495E",
}


def esc(s):
    return html.escape(str(s) if s is not None else "")


def chart_transition_heatmap(matrix):
    """T1 layer × T2 outcome heatmap (percentages)."""
    data = []
    row_labels = LAYER_ORDER
    col_labels = LAYER_ORDER + ["disappeared"]
    for l1 in row_labels:
        row = matrix.get(l1, {})
        tot = row.get("count", 0)
        if tot == 0:
            data.append([0] * len(col_labels))
            continue
        data.append(
            [row.get(l2, 0) / tot * 100 for l2 in LAYER_ORDER] + [row.get("disappeared", 0) / tot * 100]
        )
    data = np.array(data)

    fig, ax = plt.subplots(figsize=(9, 5.5), dpi=130)
    im = ax.imshow(data, cmap="YlOrRd", aspect="auto")
    ax.set_xticks(range(len(col_labels)))
    ax.set_xticklabels(col_labels)
    ax.set_yticks(range(len(row_labels)))
    ax.set_yticklabels(row_labels)
    ax.set_xlabel("T2 (2020-2024) layer")
    ax.set_ylabel("T1 (2014-2018) layer")
    ax.set_title("KDF layer transition: T1 (2014-2018) → T2 (2020-2024)\nCells = percentage of T1 layer institutions ending up there")
    for i in range(data.shape[0]):
        for j in range(data.shape[1]):
            pct = data[i, j]
            color = "white" if pct > 40 else "black"
            ax.text(j, i, f"{pct:.1f}%", ha="center", va="center", color=color, fontsize=10)
    plt.colorbar(im, ax=ax, label="%")
    plt.tight_layout()
    out = CHARTS_DIR / "transition_matrix_heatmap.png"
    plt.savefig(out, bbox_inches="tight")
    plt.close()
    return out


def chart_base_rate_bars(matrix):
    """Stacked bar chart: for each T1 layer, % transitions to each T2 state."""
    row_labels = LAYER_ORDER
    categories = LAYER_ORDER + ["disappeared"]
    data = np.zeros((len(row_labels), len(categories)))
    for i, l1 in enumerate(row_labels):
        row = matrix.get(l1, {})
        tot = max(row.get("count", 0), 1)
        for j, l2 in enumerate(LAYER_ORDER):
            data[i, j] = row.get(l2, 0) / tot * 100
        data[i, -1] = row.get("disappeared", 0) / tot * 100

    fig, ax = plt.subplots(figsize=(10, 5.5), dpi=130)
    bottoms = np.zeros(len(row_labels))
    for j, cat in enumerate(categories):
        color = LAYER_COLORS.get(cat, "#888")
        ax.bar(row_labels, data[:, j], bottom=bottoms, color=color, label=cat, edgecolor="white", linewidth=0.5)
        # Add text
        for i, val in enumerate(data[:, j]):
            if val > 4:
                ax.text(i, bottoms[i] + val / 2, f"{val:.0f}%", ha="center", va="center",
                        color="white" if val > 10 else "black", fontsize=9)
        bottoms += data[:, j]
    ax.set_ylabel("% of T1 institutions")
    ax.set_xlabel("T1 layer (2014-2018)")
    ax.set_title("Where do T1 institutions go by T2? (base rates per starting layer)")
    ax.set_ylim(0, 100)
    ax.legend(title="T2 outcome", bbox_to_anchor=(1.02, 1), loc="upper left")
    plt.tight_layout()
    out = CHARTS_DIR / "base_rate_bars.png"
    plt.savefig(out, bbox_inches="tight")
    plt.close()
    return out


def chart_fields_expansion(joined):
    """Distribution of field count change (T2 - T1) per T1 layer."""
    per_layer = {l: [] for l in LAYER_ORDER}
    for r in joined:
        if not r.get("in_t2"):
            continue
        delta = r["t2_n_fields"] - r["t1_n_fields"]
        per_layer[r["t1_layer"]].append(delta)

    fig, axes = plt.subplots(1, 4, figsize=(14, 4), dpi=130, sharey=True)
    for ax, layer in zip(axes, LAYER_ORDER):
        data = per_layer[layer]
        if data:
            bins = np.arange(-3.5, 4.5, 1)
            ax.hist(data, bins=bins, color=LAYER_COLORS[layer], edgecolor="white", linewidth=0.5)
            mean_delta = np.mean(data)
            ax.axvline(mean_delta, color="black", linestyle="--", linewidth=1, alpha=0.7)
            ax.text(0.98, 0.95, f"n={len(data)}\nmean={mean_delta:+.2f}",
                    transform=ax.transAxes, ha="right", va="top", fontsize=9,
                    bbox=dict(boxstyle="round", facecolor="white", alpha=0.8))
        ax.set_title(f"T1 = {layer}")
        ax.set_xlabel("Δ n_fields (T2 - T1)")
        ax.axvline(0, color="gray", alpha=0.3)
        if layer == LAYER_ORDER[0]:
            ax.set_ylabel("# institutions")
    plt.suptitle("Field-span evolution: did T1 institutions expand their field coverage by T2?")
    plt.tight_layout()
    out = CHARTS_DIR / "fields_expansion_dist.png"
    plt.savefig(out, bbox_inches="tight")
    plt.close()
    return out


def chart_degree_growth_scatter(joined):
    """Scatter: T1 degree vs T2 degree, colored by T1 layer."""
    fig, ax = plt.subplots(figsize=(8, 6), dpi=130)
    for layer in LAYER_ORDER:
        points = [(r["t1_degree"], r.get("t2_degree", 0)) for r in joined
                  if r["t1_layer"] == layer and r.get("in_t2")]
        if points:
            x, y = zip(*points)
            ax.scatter(x, y, c=LAYER_COLORS[layer], label=f"T1={layer} (n={len(points)})",
                       s=25, alpha=0.6, edgecolors="white", linewidth=0.5)
    # y=x line
    max_v = max(max(r.get("t2_degree", 0) for r in joined if r.get("in_t2")), 1)
    ax.plot([0, max_v], [0, max_v], "k--", alpha=0.3, label="y = x (no change)")
    ax.set_xscale("symlog")
    ax.set_yscale("symlog")
    ax.set_xlabel("T1 degree (weighted co-authorships, 2014-2018)")
    ax.set_ylabel("T2 degree (weighted co-authorships, 2020-2024)")
    ax.set_title("Degree evolution: who expanded collaboration network?")
    ax.legend(loc="upper left")
    plt.tight_layout()
    out = CHARTS_DIR / "degree_growth_scatter.png"
    plt.savefig(out, bbox_inches="tight")
    plt.close()
    return out


def main():
    with open(OUT_DIR / "backtest_results.json", encoding="utf-8") as f:
        data = json.load(f)

    matrix = data["transition_matrix"]
    joined = data["joined"]

    print("Generating charts...")
    p1 = chart_transition_heatmap(matrix)
    print(f"  {p1}")
    p2 = chart_base_rate_bars(matrix)
    print(f"  {p2}")
    p3 = chart_fields_expansion(joined)
    print(f"  {p3}")
    p4 = chart_degree_growth_scatter(joined)
    print(f"  {p4}")

    # Compute narrative stats
    rare_n = matrix["Rare"]["count"]
    rare_disappear_pct = matrix["Rare"]["disappeared"] / max(rare_n, 1) * 100
    rare_stay_active_pct = 100 - rare_disappear_pct
    rare_core_pct = matrix["Rare"]["Core"] / max(rare_n, 1) * 100

    core_n = matrix["Core"]["count"]
    core_stable_pct = matrix["Core"]["Core"] / max(core_n, 1) * 100
    core_disappear_pct = matrix["Core"]["disappeared"] / max(core_n, 1) * 100

    edge_n = matrix["Edge"]["count"]
    edge_to_core_pct = matrix["Edge"]["Core"] / max(edge_n, 1) * 100
    edge_disappear_pct = matrix["Edge"]["disappeared"] / max(edge_n, 1) * 100

    # HTML output
    html_out = f"""<!DOCTYPE html>
<html lang="ja"><head><meta charset="UTF-8">
<title>D9 Backtest — T1→T2 Predictive Power Validation</title>
<style>
body{{font-family:-apple-system,"Hiragino Sans","Yu Gothic",Meiryo,sans-serif;margin:0;padding:20px;background:#f5f5f5;color:#333;}}
h1{{color:#222;border-bottom:3px solid #2C3E50;padding-bottom:10px;}}
h2{{color:#2C3E50;border-left:4px solid #3498DB;padding-left:10px;margin-top:30px;}}
.card{{background:white;padding:20px;margin:20px 0;border-radius:8px;box-shadow:0 1px 3px rgba(0,0,0,0.1);}}
.stat-grid{{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:15px;}}
.stat{{text-align:center;padding:15px;background:#ECF0F1;border-radius:6px;}}
.stat b{{display:block;font-size:2em;color:#2C3E50;}}
.stat span{{color:#666;font-size:0.9em;}}
img{{max-width:100%;height:auto;border-radius:6px;box-shadow:0 1px 3px rgba(0,0,0,0.1);}}
.disclaimer{{background:#FDEDEC;border:2px solid #E74C3C;padding:15px;border-radius:6px;margin:20px 0;}}
table{{width:100%;background:white;border-collapse:collapse;margin:10px 0;}}
th,td{{padding:8px 10px;border:1px solid #ddd;}}
th{{background:#34495E;color:white;}}
.highlight{{background:#FFF9C4;padding:2px 4px;border-radius:3px;}}
code{{background:#ECF0F1;padding:2px 5px;border-radius:3px;font-family:monospace;}}
</style>
</head>
<body>

<h1>D9 Backtest — KDF の "retrospective descriptive" 精度検証</h1>

<div class="card">
<p><b>Question</b>: 過去(T1: 2014-2018)に KDF が "boundary broker" と判定した機関は、
現在(T2: 2020-2024)でどうなっているか?</p>
<p><b>Method</b>: 同じ 4 分野、同じサンプル手法で T1 と T2 のデータを取得。各機関を
T1 の KDF layer で分類し、T2 での outcome(layer / field 数 / degree / still active)を
集計。</p>
<p><b>これで検証できること</b>:
<ul>
  <li>「Rare broker は一時的な position か、それとも Core に昇格するか」</li>
  <li>「Core hub は stable か」</li>
  <li>「Edge から Core に break into する base rate」</li>
</ul>
</p>
</div>

<div class="disclaimer">
<b>⚠️ 重要</b>: これは記述統計であり、予測モデルではありません。
"過去 5 年でこのパターンがあった" を示すだけで、"次の 5 年でこうなる" は保証しません。
参考にどうぞ。
</div>

<h2>Core statistics (summary)</h2>
<div class="stat-grid">
  <div class="stat"><b>{rare_n}</b><span>T1 Rare brokers</span></div>
  <div class="stat"><b>{rare_disappear_pct:.0f}%</b><span>Rare → T2 で消えた</span></div>
  <div class="stat"><b>{rare_core_pct:.0f}%</b><span>Rare → Core 昇格率</span></div>
  <div class="stat"><b>{core_n}</b><span>T1 Core hubs</span></div>
  <div class="stat"><b>{core_stable_pct:.0f}%</b><span>Core → Core 維持率</span></div>
  <div class="stat"><b>{core_disappear_pct:.0f}%</b><span>Core → 消えた</span></div>
  <div class="stat"><b>{edge_n}</b><span>T1 Edge</span></div>
  <div class="stat"><b>{edge_to_core_pct:.1f}%</b><span>Edge → Core 昇格(稀)</span></div>
</div>

<h2>Chart 1 — Transition matrix heatmap</h2>
<div class="card">
<img src="backtest_charts/transition_matrix_heatmap.png">
<p>行 = T1 layer、列 = T2 outcome。"disappeared" = T2 の top-cited 論文 dataset に含まれなくなった機関。</p>
</div>

<h2>Chart 2 — Base rate bars(積み上げ)</h2>
<div class="card">
<img src="backtest_charts/base_rate_bars.png">
<p>同じ matrix を stacked bar として可視化。
各 T1 layer の "運命" 分布が一目で比較できる。</p>
</div>

<h2>Chart 3 — Field-span expansion distribution</h2>
<div class="card">
<img src="backtest_charts/fields_expansion_dist.png">
<p>T2 - T1 の n_fields(分野数)差。正の値 = 分野を広げた、負 = 絞った。
Rare broker は低い n_fields base から start するので、expand 余地が大きい。</p>
</div>

<h2>Chart 4 — Degree growth scatter (log-log)</h2>
<div class="card">
<img src="backtest_charts/degree_growth_scatter.png">
<p>X = T1 degree、Y = T2 degree、log scale。y=x 線より上 = 成長、下 = 縮小。</p>
</div>

<h2>Narrative — What this backtest actually shows</h2>
<div class="card">
<p><b>Finding 1: Core layer is remarkably stable</b></p>
<ul>
  <li>T1 Core hub の <span class="highlight">{core_stable_pct:.0f}%</span> が T2 でも Core layer</li>
  <li>消失率はわずか <span class="highlight">{core_disappear_pct:.0f}%</span></li>
  <li>解釈: 一流研究機関 (Harvard, UCL, Tokyo 等) は 5 年経っても top position を維持する</li>
</ul>

<p><b>Finding 2: Rare brokers are ephemeral</b></p>
<ul>
  <li>T1 Rare broker の <span class="highlight">{rare_disappear_pct:.0f}%</span> が T2 で消失</li>
  <li>Core 昇格はわずか <span class="highlight">{rare_core_pct:.0f}%</span>(本サンプルでは 0%)</li>
  <li>解釈: 低 degree broker は 1-2 回の共著で一時的に表面化するが、持続的な hub には育ちにくい</li>
  <li>示唆: Rare broker の value は "今この瞬間の cross-field access" であり、投資対象としては "現時点でしか捕まえられない"</li>
</ul>

<p><b>Finding 3: Edge → Core breakthrough is rare</b></p>
<ul>
  <li>T1 Edge の中で Core に昇格したのは <span class="highlight">{edge_to_core_pct:.1f}%</span> のみ</li>
  <li>解釈: Research elite は stratified。 一般機関が top tier に break into するのは稀</li>
</ul>

<p><b>Finding 4: "Disappeared" ≠ 組織消滅</b></p>
<ul>
  <li>本 dataset は "4 分野での top-cited 論文" のみを見る narrow window</li>
  <li>"disappeared" は "top-500/field の文脈で visible でなくなった" 意味</li>
  <li>研究方針変更、長期出版 slowdown、分野変遷等の複合要因</li>
  <li>よって "消えた = 終わった" と解釈してはいけない</li>
</ul>
</div>

<h2>Key insight for positioning</h2>
<div class="card">
<p>この backtest は、user の catchphrase <b>"参考にどうぞ"</b> を支える実証の一例です:</p>
<blockquote style="background:#F8F9FA;border-left:4px solid #3498DB;padding:15px;margin:15px 0;">
"過去 10 年のデータで、KDF が Core 判定した機関の <b>{core_stable_pct:.0f}%</b> は 5 年後も Core 層に残っています。
Rare broker 判定の <b>{rare_core_pct:.0f}%</b> のみが Core に昇格し、 <b>{rare_disappear_pct:.0f}%</b> は top-cited から消えました。
参考にどうぞ。判断はご自身で。"
</blockquote>
<p>これは:</p>
<ul>
  <li>❌ 投資助言ではない</li>
  <li>❌ 未来予測ではない</li>
  <li>✅ 過去 10 年の descriptive base rate</li>
  <li>✅ deterministic algorithm で再現可能</li>
  <li>✅ 監査可能な判定 trail を持つ</li>
</ul>
</div>

<h2>Honest limitations</h2>
<div class="card">
<ul>
  <li>Sample 偏り: OpenAlex の被引用上位 500/field のみ。small player 全体の base rate ではない</li>
  <li>Dataset composition shift: T1 と T2 で論文の top 500 構成が違う(5 年で研究流行も変化)</li>
  <li>"Disappeared" は "top-cited 外" であり組織消滅ではない</li>
  <li>4 分野の choice 次第で結果変わる</li>
  <li>KDF layer のみ見ている(fields、degree の細かい変化は複合 metric で別途)</li>
</ul>
</div>

<footer style="margin-top:40px;padding:20px;background:white;border-radius:8px;font-size:0.85em;color:#666;">
<p>Generated: 2026-04-19 | Data: OpenAlex 2014-2018 & 2020-2024 (4 fields × 500 papers each)</p>
<p>Scripts: <code>demos/D9_corporate_network/scripts/</code> (steps 6, 7, 8)</p>
</footer>
</body></html>"""

    out_html = OUT_DIR / "backtest_dashboard.html"
    with out_html.open("w", encoding="utf-8") as f:
        f.write(html_out)
    print(f"\nSaved: {out_html}")


if __name__ == "__main__":
    main()
