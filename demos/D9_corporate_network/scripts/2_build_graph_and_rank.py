"""
D9 Step 2: Build institution co-authorship graph + run KDF + compare to baselines.

From OpenAlex papers (2000 papers, 4 fields), build:
  - Node = institution (id + metadata: name, country, type, fields_spanned)
  - Edge (u, v) = u and v co-authored a paper; weight = count of shared papers
  - Per-institution: n_papers, n_fields_spanned (1..4)

Then:
  - Run KDF at 30% / 50% keep_rate via kdf_select_generic
  - Compare to TopDegree, Random, fields_spanned-based heuristic
  - Identify "boundary spanner" = institution appearing in >=3 of 4 fields AND
    bridges different industry clusters

Output:
  - institutions.json (full graph + ranked lists)
  - boundary_spanners.json (top 50 candidates)
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


def load_papers():
    with open("demos/D9_corporate_network/out/papers_raw.json", encoding="utf-8") as f:
        return json.load(f)


def build_graph(papers: list[dict]):
    """Return (institutions_dict, edges_list).

    institutions_dict: id -> {name, country, type, fields_spanned, papers_count}
    edges_list: list of (inst_u, inst_v, weight)
    """
    inst_meta = {}  # id -> meta
    inst_fields = defaultdict(set)  # id -> set of field_tags
    inst_papers_count = defaultdict(int)  # id -> n papers
    edge_weights = defaultdict(int)  # (u, v) sorted -> weight

    for paper in papers:
        paper_insts = set()
        for a in paper.get("authorships", []):
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
                inst_fields[iid].add(paper["field_tag"])
        # Each institution counted once per paper
        for iid in paper_insts:
            inst_papers_count[iid] += 1
        # Edges between all pairs
        for u, v in combinations(sorted(paper_insts), 2):
            edge_weights[(u, v)] += 1

    # Finalize
    for iid, meta in inst_meta.items():
        meta["fields_spanned"] = sorted(inst_fields[iid])
        meta["n_fields"] = len(inst_fields[iid])
        meta["n_papers"] = inst_papers_count[iid]
    edges = [(u, v, w) for (u, v), w in edge_weights.items()]
    return inst_meta, edges


def kdf_rank_institutions(inst_ids: list[str], edges: list[tuple], keep_rate: float, tmp_dir: Path):
    """Map institution ids to indices and call kdf_select_generic."""
    id_to_idx = {iid: i for i, iid in enumerate(inst_ids)}
    edge_idx = [(id_to_idx[u], id_to_idx[v], float(w)) for (u, v, w) in edges if u in id_to_idx and v in id_to_idx]
    graph_input = {
        "n": len(inst_ids),
        "edges": edge_idx,
        "node_ids": inst_ids,
    }
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
    # Map layers back to institution ids
    layer_map = {}
    for layer_name, idxs in result.get("layers", {}).items():
        for i in idxs:
            if i < len(inst_ids):
                layer_map[inst_ids[i]] = layer_name
    selected_ids = set(result.get("selected_node_ids", []))
    return selected_ids, layer_map


def top_degree_ids(inst_ids, edges, keep_rate):
    deg = defaultdict(int)
    for u, v, w in edges:
        deg[u] += w
        deg[v] += w
    k = max(1, int(len(inst_ids) * keep_rate))
    return set(sorted(inst_ids, key=lambda i: -deg.get(i, 0))[:k]), deg


def main():
    print("Loading papers...")
    papers = load_papers()
    print(f"  {len(papers)} papers")

    print("Building institution graph...")
    inst_meta, edges = build_graph(papers)
    print(f"  {len(inst_meta)} institutions, {len(edges)} edges")

    # Field span distribution
    from collections import Counter
    span_dist = Counter(m["n_fields"] for m in inst_meta.values())
    print(f"  Field-span distribution: {dict(sorted(span_dist.items()))}")
    multi_field = [m for m in inst_meta.values() if m["n_fields"] >= 3]
    print(f"  Institutions in >=3 fields: {len(multi_field)}")

    inst_ids = sorted(inst_meta.keys())
    tmp_dir = Path("demos/D9_corporate_network/out/tmp_kdf")

    print("\nRunning KDF at 30%...")
    kdf30_ids, kdf30_layers = kdf_rank_institutions(inst_ids, edges, 0.30, tmp_dir / "30")
    print(f"  selected {len(kdf30_ids)}")

    print("Running KDF at 50%...")
    kdf50_ids, kdf50_layers = kdf_rank_institutions(inst_ids, edges, 0.50, tmp_dir / "50")
    print(f"  selected {len(kdf50_ids)}")

    print("Computing TopDegree...")
    td30_ids, degree_by_id = top_degree_ids(inst_ids, edges, 0.30)

    # Build ranked output
    # Score each institution:
    # - kdf_layer: 3=Rare, 2=Core, 1=Edge, 0=Garbage
    # - in_kdf30: bool
    # - n_fields: 1..4
    # - degree: sum of edge weights
    # - "boundary_score" = n_fields × kdf_layer_score
    layer_score = {"Rare": 3, "Core": 2, "Edge": 1, "Garbage": 0}

    ranked = []
    for iid, meta in inst_meta.items():
        k30_layer = kdf30_layers.get(iid, "Garbage")
        record = {
            **meta,
            "degree": degree_by_id.get(iid, 0),
            "kdf30_layer": k30_layer,
            "kdf30_layer_score": layer_score.get(k30_layer, 0),
            "in_kdf30": iid in kdf30_ids,
            "in_kdf50": iid in kdf50_ids,
            "in_topdegree30": iid in td30_ids,
            "boundary_score": meta["n_fields"] * layer_score.get(k30_layer, 0),
        }
        ranked.append(record)

    # Save all institutions ranked
    ranked.sort(key=lambda r: (-r["boundary_score"], -r["n_fields"], -r["degree"]))
    out = Path("demos/D9_corporate_network/out/institutions_ranked.json")
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "n_institutions": len(ranked),
            "n_edges": len(edges),
            "n_papers": len(papers),
            "institutions": ranked,
        }, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {out}")

    # Extract boundary spanners: fields >=3 AND KDF30 selected
    boundary_spanners = [r for r in ranked if r["n_fields"] >= 3 and r["in_kdf30"]]
    print(f"\nBoundary spanners (≥3 fields + KDF30 selected): {len(boundary_spanners)}")

    boundary_out = Path("demos/D9_corporate_network/out/boundary_spanners.json")
    with boundary_out.open("w", encoding="utf-8") as f:
        json.dump(boundary_spanners, f, indent=2, ensure_ascii=False)
    print(f"Saved: {boundary_out}")

    # Print top 30 human-readable
    print("\n" + "=" * 100)
    print("Top 30 boundary-spanner candidates (by boundary_score)")
    print("=" * 100)
    print(f"{'rank':<5}{'name':<50}{'country':<4}{'type':<12}{'n_f':>4}{'KDF':>6}{'deg':>6}")
    for i, r in enumerate(ranked[:30], 1):
        name = (r["name"] or "")[:48]
        country = r["country"] or "?"
        ty = (r["type"] or "?")[:10]
        fields = r["n_fields"]
        kdf = r["kdf30_layer"][:4]
        deg = int(r["degree"])
        print(f"{i:<5}{name:<50}{country:<4}{ty:<12}{fields:>4}{kdf:>6}{deg:>6}")

    # Japanese boundary spanners specifically
    jp_ranked = [r for r in ranked if r["country"] == "JP"]
    print(f"\nJapanese institutions in sample: {len(jp_ranked)}")
    print("\nTop 20 Japanese boundary-spanner candidates:")
    print(f"{'rank':<5}{'name':<50}{'type':<12}{'n_f':>4}{'fields':<40}{'KDF':>6}")
    for i, r in enumerate(jp_ranked[:20], 1):
        name = (r["name"] or "")[:48]
        ty = (r["type"] or "?")[:10]
        fields = r["n_fields"]
        flist = ",".join(f.replace("_", "")[:5] for f in r["fields_spanned"])[:38]
        kdf = r["kdf30_layer"][:4]
        print(f"{i:<5}{name:<50}{ty:<12}{fields:>4}{flist:<40}{kdf:>6}")


if __name__ == "__main__":
    main()
