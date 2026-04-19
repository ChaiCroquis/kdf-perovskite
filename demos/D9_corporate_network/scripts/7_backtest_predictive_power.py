"""
D9 Step 7: Time-split backtest — T1 KDF layer predicts T2 outcomes?

The key validation the demo was missing per user's insight:
  "今のデータは、今現在の状態を表現しているだけ"

This script fills that gap:
  T1 = 2014-2018 (build graph, run KDF → assign layer per institution)
  T2 = 2020-2024 (same institutions, measure outcomes)

For each T1 institution, compute T2 outcomes:
  - Still active? (appears in T2 data)
  - n_fields T2 vs T1 (expanded field coverage?)
  - degree T2 vs T1 (more collaborators?)
  - paper count T2 vs T1 (output growth?)
  - Moved to Core layer at T2? (promotion signal)

Base rates are computed per T1 KDF layer:
  "Of T1 Rare-layer institutions, X% became Core at T2"
  "Of T1 Edge-layer institutions, only Y% ever reached Core at T2"

These base rates are the concrete "参考にどうぞ" statistics —
descriptive, auditable, not predictive.

Output:
  out/backtest_results.json: per-institution T1/T2 comparison
  out/backtest_dashboard.html: base-rate tables + simple charts
"""
from __future__ import annotations

import json
import subprocess
import sys
from collections import defaultdict
from itertools import combinations
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def load_papers(path):
    with open(path, encoding="utf-8") as f:
        return json.load(f)


def build_institution_metrics(papers):
    """Return dict: inst_id → metrics(name, country, type, n_papers, n_fields, degree)."""
    inst_meta = {}
    inst_fields = defaultdict(set)
    inst_papers_count = defaultdict(int)
    edge_weights = defaultdict(float)

    for p in papers:
        paper_insts = set()
        for a in p.get("authorships", []):
            for inst in a.get("institutions", []):
                iid = inst.get("id")
                if not iid:
                    continue
                paper_insts.add(iid)
                if iid not in inst_meta:
                    inst_meta[iid] = {
                        "id": iid,
                        "name": inst.get("name"),
                        "country": inst.get("country"),
                        "type": inst.get("type"),
                    }
                inst_fields[iid].add(p["field_tag"])
        for iid in paper_insts:
            inst_papers_count[iid] += 1
        for u, v in combinations(sorted(paper_insts), 2):
            edge_weights[(u, v)] += 1

    for iid, m in inst_meta.items():
        m["fields_spanned"] = sorted(inst_fields[iid])
        m["n_fields"] = len(inst_fields[iid])
        m["n_papers"] = inst_papers_count[iid]

    # Compute degree from edges
    degree = defaultdict(float)
    for (u, v), w in edge_weights.items():
        degree[u] += w
        degree[v] += w
    for iid, m in inst_meta.items():
        m["degree"] = degree.get(iid, 0)

    edges = [(u, v, w) for (u, v), w in edge_weights.items()]
    return inst_meta, edges


def run_kdf_layer_assignment(inst_ids, edges, keep_rate, tmp_dir: Path):
    id_to_idx = {iid: i for i, iid in enumerate(inst_ids)}
    edge_idx = [(id_to_idx[u], id_to_idx[v], float(w)) for (u, v, w) in edges if u in id_to_idx and v in id_to_idx]
    graph_input = {"n": len(inst_ids), "edges": edge_idx, "node_ids": inst_ids}
    tmp_dir.mkdir(parents=True, exist_ok=True)
    in_path = tmp_dir / "graph.json"
    out_path = tmp_dir / "selected.json"
    with in_path.open("w", encoding="utf-8") as f:
        json.dump(graph_input, f)
    subprocess.run(
        ["cargo", "run", "--release", "-q",
         "-p", "demo-d8-llm-memory",
         "--bin", "kdf_select_generic", "--",
         "--input", str(in_path),
         "--out", str(out_path),
         "--keep-rate", str(keep_rate)],
        check=True, capture_output=True,
    )
    with out_path.open("r", encoding="utf-8") as f:
        result = json.load(f)
    layer_map = {}
    for layer_name, idxs in result.get("layers", {}).items():
        for i in idxs:
            if i < len(inst_ids):
                layer_map[inst_ids[i]] = layer_name
    return layer_map


def main():
    out_dir = Path("demos/D9_corporate_network/out")
    tmp_dir = out_dir / "tmp_backtest_kdf"

    print("Loading T1 (2014-2018) and T2 (2020-2024) papers...")
    t1_papers = load_papers(out_dir / "papers_t1_2014_2018.json")
    t2_papers = load_papers(out_dir / "papers_raw.json")
    print(f"  T1: {len(t1_papers)}, T2: {len(t2_papers)}")

    print("Building T1 institution metrics + KDF layers...")
    t1_inst, t1_edges = build_institution_metrics(t1_papers)
    t1_ids = sorted(t1_inst.keys())
    t1_layers = run_kdf_layer_assignment(t1_ids, t1_edges, 0.30, tmp_dir / "t1")
    print(f"  T1: {len(t1_inst)} institutions, {len(t1_edges)} edges")

    print("Building T2 institution metrics + KDF layers...")
    t2_inst, t2_edges = build_institution_metrics(t2_papers)
    t2_ids = sorted(t2_inst.keys())
    t2_layers = run_kdf_layer_assignment(t2_ids, t2_edges, 0.30, tmp_dir / "t2")
    print(f"  T2: {len(t2_inst)} institutions, {len(t2_edges)} edges")

    # Join: T1 institutions → what is their T2 state?
    print("\nJoining T1 → T2 and computing outcomes...")
    joined = []
    for iid, m1 in t1_inst.items():
        m2 = t2_inst.get(iid)
        record = {
            "id": iid,
            "name": m1.get("name"),
            "country": m1.get("country"),
            "type": m1.get("type"),
            "t1_layer": t1_layers.get(iid, "Garbage"),
            "t1_n_fields": m1["n_fields"],
            "t1_n_papers": m1["n_papers"],
            "t1_degree": m1["degree"],
        }
        if m2:
            record.update({
                "in_t2": True,
                "t2_layer": t2_layers.get(iid, "Garbage"),
                "t2_n_fields": m2["n_fields"],
                "t2_n_papers": m2["n_papers"],
                "t2_degree": m2["degree"],
                "fields_expanded": m2["n_fields"] > m1["n_fields"],
                "fields_same": m2["n_fields"] == m1["n_fields"],
                "fields_shrunk": m2["n_fields"] < m1["n_fields"],
                "degree_grew": m2["degree"] > m1["degree"],
                "papers_grew": m2["n_papers"] > m1["n_papers"],
                "promoted_to_core": (t1_layers.get(iid) != "Core") and (t2_layers.get(iid) == "Core"),
                "stayed_rare": (t1_layers.get(iid) == "Rare") and (t2_layers.get(iid) == "Rare"),
                "demoted": _is_demoted(t1_layers.get(iid), t2_layers.get(iid)),
            })
        else:
            record.update({
                "in_t2": False,
                "t2_layer": None,
                "disappeared": True,
            })
        joined.append(record)

    # Compute transition matrix
    print("\nTransition matrix (T1 layer → T2 layer):")
    layers = ["Rare", "Core", "Edge", "Garbage"]
    matrix = {l: {"count": 0, **{l2: 0 for l2 in layers}, "disappeared": 0} for l in layers}
    for r in joined:
        t1l = r["t1_layer"]
        matrix[t1l]["count"] += 1
        if r.get("in_t2"):
            matrix[t1l][r["t2_layer"]] += 1
        else:
            matrix[t1l]["disappeared"] += 1
    print(f"  {'T1\\T2':<10}" + "".join(f"{l:>10}" for l in layers) + f"{'disappeared':>13}  {'total':>7}")
    for l in layers:
        row = matrix[l]
        print(f"  {l:<10}" + "".join(f"{row[l2]:>10}" for l2 in layers) + f"{row['disappeared']:>13}  {row['count']:>7}")
    print("\n(rows = T1 layer; cells = count of institutions in that T1 layer that ended up in T2 layer)")

    # Base rates (% of each T1 layer that made each transition)
    print("\nBase rates (% of each T1 layer):")
    print(f"  {'T1 layer':<10}" + "".join(f"{l:>10}" for l in layers) + f"{'disapp.':>10}")
    for l in layers:
        row = matrix[l]
        tot = row["count"]
        if tot == 0:
            continue
        pcts = [row[l2] / tot * 100 for l2 in layers] + [row["disappeared"] / tot * 100]
        print(f"  {l:<10}" + "".join(f"{p:>9.1f}%" for p in pcts))

    # Key question: "Did T1-Rare brokers become Core at T2?"
    t1_rare = [r for r in joined if r["t1_layer"] == "Rare"]
    t1_rare_present_t2 = [r for r in t1_rare if r.get("in_t2")]
    t1_rare_promoted = [r for r in t1_rare if r.get("promoted_to_core")]
    t1_rare_expanded = [r for r in t1_rare if r.get("fields_expanded")]

    t1_core = [r for r in joined if r["t1_layer"] == "Core"]
    t1_core_stable = [r for r in t1_core if r.get("in_t2") and r["t2_layer"] == "Core"]

    t1_edge = [r for r in joined if r["t1_layer"] == "Edge"]
    t1_edge_became_core = [r for r in t1_edge if r.get("in_t2") and r["t2_layer"] == "Core"]

    print("\n" + "=" * 80)
    print("主要な base rate(KDF の retrospective descriptive 精度)")
    print("=" * 80)
    print(f"\n1. T1-Rare broker(n={len(t1_rare)})の T2 での outcome:")
    print(f"   - T2 でも active: {len(t1_rare_present_t2)} ({len(t1_rare_present_t2)/max(len(t1_rare),1)*100:.1f}%)")
    if t1_rare_present_t2:
        print(f"   - T2 で Core 昇格: {len(t1_rare_promoted)} ({len(t1_rare_promoted)/len(t1_rare_present_t2)*100:.1f}%) ★")
        print(f"   - 分野拡張(T2 で n_fields 増): {len(t1_rare_expanded)} ({len(t1_rare_expanded)/len(t1_rare_present_t2)*100:.1f}%)")

    print(f"\n2. T1-Core hub(n={len(t1_core)})の T2 での outcome:")
    if t1_core:
        t1_core_present = [r for r in t1_core if r.get("in_t2")]
        print(f"   - T2 でも active: {len(t1_core_present)} ({len(t1_core_present)/max(len(t1_core),1)*100:.1f}%)")
        if t1_core_present:
            print(f"   - T2 で Core 維持: {len(t1_core_stable)} ({len(t1_core_stable)/len(t1_core_present)*100:.1f}%)")

    print(f"\n3. T1-Edge(n={len(t1_edge)})の T2 での outcome:")
    if t1_edge:
        t1_edge_present = [r for r in t1_edge if r.get("in_t2")]
        print(f"   - T2 でも active: {len(t1_edge_present)} ({len(t1_edge_present)/max(len(t1_edge),1)*100:.1f}%)")
        if t1_edge_present:
            print(f"   - T2 で Core 昇格(rare event): {len(t1_edge_became_core)} ({len(t1_edge_became_core)/len(t1_edge_present)*100:.1f}%)")

    # Save
    backtest_out = out_dir / "backtest_results.json"
    with backtest_out.open("w", encoding="utf-8") as f:
        json.dump({
            "t1_period": "2014-2018",
            "t2_period": "2020-2024",
            "t1_n_institutions": len(t1_inst),
            "t2_n_institutions": len(t2_inst),
            "transition_matrix": matrix,
            "joined": joined,
        }, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {backtest_out}")


def _is_demoted(t1l, t2l):
    order = {"Rare": 3, "Core": 2, "Edge": 1, "Garbage": 0}
    return order.get(t1l, -1) > order.get(t2l, -1)


if __name__ == "__main__":
    main()
