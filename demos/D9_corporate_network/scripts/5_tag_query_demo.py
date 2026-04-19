"""
D9 Step 5: Tag-filtered sub-graph extraction + KDF re-run demo.

Demonstrates the real win of the SQLite + Obsidian-tag schema:
  - JSON 処理は全 3791 機関 × 111k edges を毎回 load
  - SQLite tag query は index で必要な sub-graph だけを extract (10-100x smaller)

Three example tag queries + KDF re-run on each sub-graph:
  Q1: Japanese AI research (country/JP AND field/AI_ML)
  Q2: Multi-field companies (type/company AND span/multi_3plus)
  Q3: Multi-field rare brokers (kdf/rare AND span/multi_3plus)

For each query: extract sub-graph, re-run KDF locally, show new ranking.
"""
from __future__ import annotations

import json
import sqlite3
import subprocess
import sys
import time
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

DB_PATH = Path("demos/D9_corporate_network/out/corporate_graph.db")


def connect():
    conn = sqlite3.connect(DB_PATH)
    conn.row_factory = sqlite3.Row
    return conn


def query_subgraph(conn, tag_filters: list[str], edge_type: str = "coauthored"):
    """Extract notes matching ALL tag_filters + induced edges.

    Returns (nodes: list[Row], edges: list[Row]).
    Much smaller than the full graph — this is the data-volume win.
    """
    t0 = time.time()
    placeholders = ",".join("?" for _ in tag_filters)
    # Find nodes having ALL tags
    sql_nodes = f"""
        SELECT n.id, n.name, n.note_type, n.body_json
        FROM notes n
        WHERE n.id IN (
            SELECT note_id FROM tags
            WHERE tag IN ({placeholders})
            GROUP BY note_id
            HAVING COUNT(DISTINCT tag) = ?
        )
    """
    nodes = conn.execute(sql_nodes, (*tag_filters, len(tag_filters))).fetchall()
    t_nodes = time.time() - t0

    node_ids = set(n["id"] for n in nodes)
    if not node_ids:
        return list(nodes), [], t_nodes, 0.0

    # Induced edges: both endpoints in node_ids
    placeholders_n = ",".join("?" for _ in node_ids)
    node_ids_list = list(node_ids)
    t0 = time.time()
    sql_edges = f"""
        SELECT e.src_id, e.tgt_id, e.weight
        FROM edges e
        WHERE e.edge_type = ?
          AND e.src_id IN ({placeholders_n})
          AND e.tgt_id IN ({placeholders_n})
    """
    edges = conn.execute(sql_edges, (edge_type, *node_ids_list, *node_ids_list)).fetchall()
    t_edges = time.time() - t0

    return list(nodes), list(edges), t_nodes, t_edges


def kdf_rank(nodes: list, edges: list, keep_rate: float, tmp_dir: Path):
    """Call kdf_select_generic on extracted subgraph."""
    id_to_idx = {n["id"]: i for i, n in enumerate(nodes)}
    edge_idx = [(id_to_idx[e["src_id"]], id_to_idx[e["tgt_id"]], float(e["weight"]))
                for e in edges if e["src_id"] in id_to_idx and e["tgt_id"] in id_to_idx]
    if not nodes or not edge_idx:
        return set(), {}
    graph_input = {
        "n": len(nodes),
        "edges": edge_idx,
        "node_ids": [n["id"] for n in nodes],
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
    selected_ids = set(result.get("selected_node_ids", []))
    layer_map = {}
    for layer_name, idxs in result.get("layers", {}).items():
        for i in idxs:
            if i < len(nodes):
                layer_map[nodes[i]["id"]] = layer_name
    return selected_ids, layer_map


def display_query(conn, query_name: str, tags: list[str], tmp_dir: Path):
    print(f"\n{'='*90}\n{query_name}\nTags = {tags}\n{'='*90}")
    t_start = time.time()
    nodes, edges, t_nodes, t_edges = query_subgraph(conn, tags)
    t_extract = time.time() - t_start
    print(f"Extracted: {len(nodes)} nodes ({t_nodes*1000:.1f}ms), {len(edges)} edges ({t_edges*1000:.1f}ms), total {t_extract*1000:.1f}ms")

    if not nodes:
        print("  (no matching nodes)")
        return
    if not edges:
        print("  (no induced edges — nodes are isolated within this tag slice)")
        # Just rank by degree-equivalent (body.degree)
        sorted_nodes = sorted(nodes, key=lambda n: -json.loads(n["body_json"]).get("degree", 0))[:20]
        for i, n in enumerate(sorted_nodes, 1):
            body = json.loads(n["body_json"])
            print(f"  {i:>3}. {n['name']:<55}  n_fields={body['n_fields']}  deg={int(body['degree'])}")
        return

    t0 = time.time()
    selected, layers = kdf_rank(nodes, edges, 0.30, tmp_dir)
    t_kdf = time.time() - t0
    print(f"KDF on subgraph: {t_kdf*1000:.1f}ms, {len(selected)} selected")

    # Display top 15 Rare + top 15 Core
    rare_nodes = []
    core_nodes = []
    for n in nodes:
        body = json.loads(n["body_json"])
        layer = layers.get(n["id"], "Garbage")
        entry = (n["id"], n["name"], body, layer)
        if layer == "Rare":
            rare_nodes.append(entry)
        elif layer == "Core":
            core_nodes.append(entry)
    rare_nodes.sort(key=lambda x: (-x[2].get("n_fields", 0), x[2].get("degree", 0)))
    core_nodes.sort(key=lambda x: (-x[2].get("n_fields", 0), -x[2].get("degree", 0)))

    if rare_nodes:
        print(f"\n  Rare layer (broker position, top 10):")
        for i, (iid, name, body, _) in enumerate(rare_nodes[:10], 1):
            fields = ",".join(f.replace("_", "")[:5] for f in body.get("fields_spanned", []))[:30]
            print(f"    {i:>3}. {(name or iid)[:55]:<55}  n_f={body.get('n_fields',0)}  deg={int(body.get('degree',0)):>4}  [{fields}]")
    if core_nodes:
        print(f"\n  Core layer (multi-field hubs, top 10):")
        for i, (iid, name, body, _) in enumerate(core_nodes[:10], 1):
            fields = ",".join(f.replace("_", "")[:5] for f in body.get("fields_spanned", []))[:30]
            print(f"    {i:>3}. {(name or iid)[:55]:<55}  n_f={body.get('n_fields',0)}  deg={int(body.get('degree',0)):>4}  [{fields}]")


def main():
    conn = connect()
    tmp_dir = Path("demos/D9_corporate_network/out/tmp_query_kdf")

    # Metadata
    total_notes = conn.execute("SELECT COUNT(*) FROM notes").fetchone()[0]
    total_edges = conn.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    print(f"Full graph: {total_notes} notes, {total_edges} edges")

    # Show tag distribution (top 20)
    print("\n-- Tag distribution (top 20) --")
    rows = conn.execute("SELECT tag, COUNT(*) AS c FROM tags GROUP BY tag ORDER BY c DESC LIMIT 20").fetchall()
    for r in rows:
        print(f"  {r['tag']:<30}  {r['c']}")

    # Q1: Japanese AI research
    display_query(
        conn,
        "Q1: Japanese AI research(country/JP × field/AI_ML)",
        ["country/JP", "field/AI_ML"],
        tmp_dir / "q1",
    )

    # Q2: Multi-field companies (corporate boundary spanners)
    display_query(
        conn,
        "Q2: 多分野展開している企業(type/company × span/multi_3plus)",
        ["type/company", "span/multi_3plus"],
        tmp_dir / "q2",
    )

    # Q3: Rare brokers with multi-field spanning
    display_query(
        conn,
        "Q3: KDF-Rare × 多分野 brokers(最も純粋な Burt-type brokers)",
        ["kdf/rare", "span/multi_3plus"],
        tmp_dir / "q3",
    )

    # Q4: Automotive × Semiconductor 境界(車載半導体 broker)
    display_query(
        conn,
        "Q4: 車載 × 半導体(field/Automotive × field/Materials_SemiCond × type/company)",
        ["field/Automotive", "field/Materials_SemiCond", "type/company"],
        tmp_dir / "q4",
    )

    conn.close()


if __name__ == "__main__":
    main()
