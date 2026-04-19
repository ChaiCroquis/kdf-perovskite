"""
D9 Step 3: Build an HTML dashboard showing KDF's boundary-spanner detection.

Input: demos/D9_corporate_network/out/institutions_ranked.json
Output: demos/D9_corporate_network/out/dashboard.html (self-contained, no JS deps)

Panels:
  A. Top KDF-Rare brokers (low-degree bridges, Burt's pure brokers)
  B. Top KDF-Core bridges (medium-degree, multi-field integration)
  C. Japanese institutions ranked (corporate vs academic split)
  D. Corporate-only boundary spanners (type=company, appears in >=2 fields)
  E. Field-by-field TopDegree vs KDF comparison
  F. "Historical reference" disclaimer prominently displayed
"""
from __future__ import annotations

import html
import json
import sys
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


FIELD_COLORS = {
    "AI_ML": "#4A90E2",
    "Materials_SemiCond": "#9C6ADE",
    "Biomed_Pharma": "#50C878",
    "Automotive": "#F39C12",
}


def esc(s):
    if s is None:
        return ""
    return html.escape(str(s))


def field_badge(field_tag):
    color = FIELD_COLORS.get(field_tag, "#888")
    label = field_tag.replace("_", " ")[:12]
    return f'<span style="background:{color};color:white;padding:1px 5px;border-radius:3px;font-size:0.75em;margin-right:3px;">{esc(label)}</span>'


def layer_badge(layer):
    colors = {"Rare": "#E74C3C", "Core": "#3498DB", "Edge": "#95A5A6", "Garbage": "#BDC3C7"}
    color = colors.get(layer, "#999")
    return f'<span style="background:{color};color:white;padding:2px 6px;border-radius:4px;font-size:0.8em;">{esc(layer)}</span>'


def row(r, rank):
    name = r["name"] or r["id"]
    country = r.get("country", "?")
    fields = "".join(field_badge(f) for f in r.get("fields_spanned", []))
    layer = r.get("kdf30_layer", "?")
    return f"""
    <tr>
      <td class="num">{rank}</td>
      <td><b>{esc(name)}</b></td>
      <td class="center">{esc(country)}</td>
      <td class="center">{esc(r.get("type","?"))}</td>
      <td class="center">{r.get("n_fields",0)}</td>
      <td class="fields">{fields}</td>
      <td class="center">{layer_badge(layer)}</td>
      <td class="num">{int(r.get("degree",0))}</td>
      <td class="num">{int(r.get("n_papers",0))}</td>
    </tr>"""


def section(title, subtitle, rows, note=""):
    rows_html = "\n".join(rows)
    note_html = f'<p class="note">{esc(note)}</p>' if note else ""
    return f"""
    <section>
      <h2>{esc(title)}</h2>
      <p class="subtitle">{esc(subtitle)}</p>
      {note_html}
      <table>
        <thead>
          <tr>
            <th>#</th><th>Institution</th><th>Country</th><th>Type</th>
            <th>Fields</th><th>Field tags</th><th>KDF layer</th>
            <th>Degree</th><th>Papers</th>
          </tr>
        </thead>
        <tbody>{rows_html}</tbody>
      </table>
    </section>"""


def main():
    data = json.load(open("demos/D9_corporate_network/out/institutions_ranked.json", encoding="utf-8"))
    insts = data["institutions"]

    # Panel A: KDF-Rare brokers (multi-field + low degree)
    rare_brokers = [r for r in insts if r["kdf30_layer"] == "Rare" and r["n_fields"] >= 2]
    rare_brokers.sort(key=lambda r: (-r["n_fields"], r["degree"]))
    panelA = [row(r, i + 1) for i, r in enumerate(rare_brokers[:20])]

    # Panel B: KDF-Core (most high-volume bridges)
    core_bridges = [r for r in insts if r["kdf30_layer"] == "Core" and r["n_fields"] >= 3]
    core_bridges.sort(key=lambda r: (-r["n_fields"], -r["degree"]))
    panelB = [row(r, i + 1) for i, r in enumerate(core_bridges[:20])]

    # Panel C: Japanese institutions (all)
    jp = [r for r in insts if r.get("country") == "JP"]
    jp.sort(key=lambda r: (-r["n_fields"], r.get("kdf30_layer") != "Rare", r.get("kdf30_layer") != "Core", -r["degree"]))
    panelC = [row(r, i + 1) for i, r in enumerate(jp[:30])]

    # Panel D: Corporate (type=company) boundary spanners
    companies = [r for r in insts if r.get("type") == "company" and r["n_fields"] >= 2]
    companies.sort(key=lambda r: (-r["n_fields"], r.get("kdf30_layer") != "Rare", -r["degree"]))
    panelD = [row(r, i + 1) for i, r in enumerate(companies[:30])]

    # Japanese corporate subset
    jp_companies = [r for r in insts if r.get("type") == "company" and r.get("country") == "JP"]
    jp_companies.sort(key=lambda r: (-r["n_fields"], -r["degree"]))
    panelE = [row(r, i + 1) for i, r in enumerate(jp_companies[:20])]

    n_papers = data["n_papers"]
    n_insts = data["n_institutions"]
    n_edges = data["n_edges"]

    # Stats
    rare_count = sum(1 for r in insts if r["kdf30_layer"] == "Rare")
    core_count = sum(1 for r in insts if r["kdf30_layer"] == "Core")
    edge_count = sum(1 for r in insts if r["kdf30_layer"] == "Edge")
    garbage_count = sum(1 for r in insts if r["kdf30_layer"] == "Garbage")

    html_out = f"""<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<title>KDF Boundary-Spanner Detection — D9 Corporate Network Experiment</title>
<style>
  body {{ font-family: -apple-system, "Hiragino Sans", "Yu Gothic", Meiryo, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; color: #333; }}
  h1 {{ color: #222; border-bottom: 3px solid #2C3E50; padding-bottom: 10px; }}
  h2 {{ color: #2C3E50; margin-top: 40px; border-left: 4px solid #3498DB; padding-left: 10px; }}
  .subtitle {{ color: #666; font-size: 0.9em; margin-top: -10px; }}
  .note {{ background: #FFF3CD; border-left: 4px solid #F1C40F; padding: 10px; margin: 10px 0; font-size: 0.9em; }}
  .disclaimer {{ background: #FDEDEC; border: 2px solid #E74C3C; padding: 15px; margin: 20px 0; border-radius: 6px; }}
  .disclaimer b {{ color: #C0392B; }}
  .stats {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin: 20px 0; }}
  .stats-grid {{ display: grid; grid-template-columns: repeat(4, 1fr); gap: 15px; }}
  .stats-card {{ text-align: center; padding: 10px; background: #ECF0F1; border-radius: 6px; }}
  .stats-card b {{ font-size: 1.5em; color: #2C3E50; display: block; }}
  .stats-card span {{ font-size: 0.85em; color: #666; }}
  table {{ width: 100%; background: white; border-collapse: collapse; margin-top: 10px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
  th {{ background: #34495E; color: white; padding: 10px; text-align: left; font-size: 0.85em; position: sticky; top: 0; }}
  td {{ padding: 8px 10px; border-bottom: 1px solid #eee; font-size: 0.9em; }}
  tr:hover {{ background: #F8F9FA; }}
  .num {{ text-align: right; font-family: monospace; }}
  .center {{ text-align: center; }}
  .fields {{ font-size: 0.85em; }}
  footer {{ margin-top: 50px; padding: 20px; background: white; border-radius: 8px; font-size: 0.85em; color: #666; }}
  section {{ background: white; padding: 20px; margin: 20px 0; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
</style>
</head>
<body>
<h1>KDF による Boundary-Spanner 検出 — D9 遊び検証</h1>
<p>
  <b>Data:</b> OpenAlex より 4 研究分野 × 各 500 論文 (2020-2024、被引用数上位) = {n_papers} 論文<br>
  <b>Graph:</b> {n_insts:,} 研究機関(ノード)、{n_edges:,} 共著関係(エッジ)<br>
  <b>Fields:</b>
  {field_badge("AI_ML")} AI / Machine Learning<br>
  <span style="margin-left:90px;">{field_badge("Materials_SemiCond")} Materials / Semiconductor</span><br>
  <span style="margin-left:90px;">{field_badge("Biomed_Pharma")} Biomedical / Pharmaceutical</span><br>
  <span style="margin-left:90px;">{field_badge("Automotive")} Automotive / Mobility Engineering</span>
</p>

<div class="disclaimer">
<b>⚠️ 重要:これは投資助言ではありません。</b><br>
KDF は「過去の共著 pattern から structural broker を記述的に抽出する」deterministic algorithm です。
未来予測でも、投資推奨でもありません。過去 5 年のデータで、
特定の機関が complex な分野間 bridge 位置に置かれていたことを示すだけです。
判断はご自身で。
</div>

<div class="stats">
  <h3 style="margin-top:0;">KDF Layer 分布</h3>
  <div class="stats-grid">
    <div class="stats-card"><b>{rare_count:,}</b><span>Rare (境界 broker)</span></div>
    <div class="stats-card"><b>{core_count:,}</b><span>Core (多分野 hub)</span></div>
    <div class="stats-card"><b>{edge_count:,}</b><span>Edge (単一クラスタ内)</span></div>
    <div class="stats-card"><b>{garbage_count:,}</b><span>Garbage (低 signal)</span></div>
  </div>
</div>

{section("Panel A — KDF-Rare 層の boundary broker (低 degree × 多分野 bridge)",
         "KDF が最も優先保護する層。少ない接続で分野間を橋渡しする "
         "'Burt の Structural Holes' 本命候補。deg が小さいほど純粋 broker。",
         panelA,
         "deg が 1〜10 程度で fields≥2 な機関は、規模ではなく position で value を出す 'traveling merchant' 型。")}

{section("Panel B — KDF-Core 層の multi-field hub (中〜高 degree × 3+ 分野)",
         "伝統的 elite university や大手研究機関。規模と多様性を両立。boundary_score で上位。",
         panelB,
         "これらは TopDegree heuristic でも上位に出る、平凡な意味で重要な機関。")}

{section("Panel C — Japanese 機関ランキング(全 81 機関)",
         "分野数 + KDF layer 優先順。日本の産学が どこに位置しているか。",
         panelC)}

{section("Panel D — 企業(type=company)のみの boundary spanner",
         "大学ではなく企業研究部門だけに絞った ranking。産業 boundary spanner 候補。",
         panelD)}

{section("Panel E — 日本企業(type=company & country=JP)のみ",
         "日本企業の中で最も多分野橋渡しをしている機関。'投資参考' ではなく '過去 5 年の構造的位置' の記述。",
         panelE,
         "件数が少ない場合、OpenAlex coverage が企業研究をフルに捉えていない可能性あり。")}

<footer>
  <h3>この dashboard の読み方</h3>
  <ul>
    <li><b>Rare 層</b>: deg が小さくても KDF が保護した "純粋 broker"。構造的交渉力が強い position。</li>
    <li><b>Core 層</b>: 大きな hub。重要だが TopDegree と同じ結果で差別化薄い。</li>
    <li><b>n_fields = 4</b>: 全 4 分野で論文がある機関。多分野展開の証拠。</li>
    <li><b>n_fields = 1</b>: 単一分野のみ。specialist で、KDF 的には bridge ではない。</li>
    <li><b>boundary_score</b> = n_fields × layer_score(Rare=3, Core=2, Edge=1, Garbage=0)</li>
  </ul>
  <h3>Algorithm</h3>
  <p>
    KDF (cgb-kdf Rust crate) の <code>NodeClassifier</code> を共著 graph に適用、
    keep_rate=0.30 で 機関を Rare/Core/Edge/Garbage に分類。
    同一 repo の <code>kdf_select_generic</code> binary 経由。
    Script: <code>demos/D9_corporate_network/scripts/</code>
  </p>
  <h3>Honest limitations</h3>
  <ul>
    <li>サンプルは OpenAlex の被引用上位 2000 論文のみ。長 tail の中小企業が欠落する。</li>
    <li>共著関係 ≠ 正式な joint patent。collaboration の proxy。</li>
    <li>OpenAlex は米英中心の coverage が厚い。日本企業の研究発表はやや過小表示される。</li>
    <li>4 分野は arbitrary choice。別分野(energy, quantum, robotics 等)で別 ranking が得られる。</li>
    <li>ある機関が Rare 層に出ても、それが商業的価値を持つかは別問題。過去 5 年の構造的位置の記述のみ。</li>
  </ul>
</footer>

</body>
</html>
"""

    out = Path("demos/D9_corporate_network/out/dashboard.html")
    with out.open("w", encoding="utf-8") as f:
        f.write(html_out)
    print(f"Saved: {out}")
    print(f"Open in browser: file://{out.resolve()}")


if __name__ == "__main__":
    main()
