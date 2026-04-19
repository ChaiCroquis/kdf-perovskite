"""
C1+C2: Classical Algorithm Revival — Betweenness Centrality + Floyd-Warshall
via KDF pruning.

Hypothesis: KDF's Rare/Core/Edge/Garbage layer ranking preserves
structurally-important nodes (betweenness bridges, path-critical nodes).
Therefore, running classical O(V^3) or O(VE) algorithms on KDF-pruned
subgraphs should yield top-K results that approximate full-graph results
at dramatically reduced cost.

Design:
  1. Generate/load benchmark graphs (synthetic planted + real SNAP-like)
  2. Run FULL betweenness + full APSP as ground truth (reference)
  3. KDF-prune to 30% and 50% (select top nodes)
  4. Run betweenness + APSP on induced subgraph
  5. Compare:
     - Top-K betweenness node overlap (Jaccard) vs full-graph
     - Spearman rank correlation of overlapping nodes
     - APSP: for sampled source-target pairs, distance error
     - Wall-time speedup

Also compare to baseline pruning strategies:
  - Random pruning (same size)
  - Degree-based pruning (keep high-degree nodes)

Output: JSON + console table.
Cost: $0 (no LLM, no API). Runs in seconds for V=500, minutes for V=2000.
"""
from __future__ import annotations

import json
import subprocess
import sys
import time
from pathlib import Path

import networkx as nx
import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def kdf_select_via_rust(G: nx.Graph, keep_rate: float, tmp_dir: Path) -> set[int]:
    """Export graph to JSON, call kdf_select_generic, parse selected node indices."""
    nodes = list(G.nodes())
    node_to_idx = {n: i for i, n in enumerate(nodes)}
    edges = [(node_to_idx[u], node_to_idx[v], float(d.get("weight", 1.0)))
             for u, v, d in G.edges(data=True)]
    graph_input = {"n": len(nodes), "edges": edges,
                   "node_ids": [str(n) for n in nodes]}
    tmp_dir.mkdir(parents=True, exist_ok=True)
    in_path = tmp_dir / "graph.json"
    out_path = tmp_dir / "selected.json"
    with in_path.open("w", encoding="utf-8") as f:
        json.dump(graph_input, f)
    cmd = [
        "cargo", "run", "--release", "-q",
        "-p", "demo-d8-llm-memory",
        "--bin", "kdf_select_generic", "--",
        "--input", str(in_path),
        "--out", str(out_path),
        "--keep-rate", str(keep_rate),
    ]
    subprocess.run(cmd, check=True, capture_output=True)
    with out_path.open("r", encoding="utf-8") as f:
        result = json.load(f)
    selected_indices = set(result["selected_node_indices"])
    return {nodes[i] for i in selected_indices}


def random_select(G: nx.Graph, keep_rate: float, seed: int = 42) -> set:
    rng = np.random.RandomState(seed)
    n = G.number_of_nodes()
    k = max(1, int(n * keep_rate))
    idxs = rng.choice(n, size=k, replace=False)
    nodes = list(G.nodes())
    return {nodes[i] for i in idxs}


def degree_select(G: nx.Graph, keep_rate: float) -> set:
    n = G.number_of_nodes()
    k = max(1, int(n * keep_rate))
    degrees = dict(G.degree())
    sorted_nodes = sorted(degrees, key=lambda x: -degrees[x])
    return set(sorted_nodes[:k])


def evaluate_betweenness(G_full: nx.Graph, G_subs: dict[str, nx.Graph], top_k: int = 50) -> dict:
    """Run Betweenness Centrality on full graph + each subset; compare."""
    t0 = time.time()
    bc_full = nx.betweenness_centrality(G_full)
    t_full = time.time() - t0

    top_k_full = set(
        n for n, _ in sorted(bc_full.items(), key=lambda x: -x[1])[:top_k]
    )

    results = {"full_graph": {
        "n": G_full.number_of_nodes(),
        "m": G_full.number_of_edges(),
        "time_s": t_full,
        "top_k_nodes": [n for n, _ in sorted(bc_full.items(), key=lambda x: -x[1])[:top_k]],
    }}

    for name, G_sub in G_subs.items():
        if G_sub.number_of_nodes() == 0:
            continue
        t0 = time.time()
        bc_sub = nx.betweenness_centrality(G_sub)
        t_sub = time.time() - t0

        top_k_sub = set(
            n for n, _ in sorted(bc_sub.items(), key=lambda x: -x[1])[:top_k]
        )

        # Overlap with full-graph top-K (among nodes present in sub)
        intersect = top_k_full & top_k_sub
        recall_at_k = len(intersect) / max(len(top_k_full), 1)

        # Rank correlation among common nodes
        common = [n for n in top_k_full if n in bc_sub]
        if len(common) > 2:
            r_full = [bc_full[n] for n in common]
            r_sub = [bc_sub[n] for n in common]
            from scipy.stats import spearmanr
            rho, p_spearman = spearmanr(r_full, r_sub)
        else:
            rho, p_spearman = None, None

        results[name] = {
            "n": G_sub.number_of_nodes(),
            "m": G_sub.number_of_edges(),
            "time_s": t_sub,
            "speedup": t_full / max(t_sub, 1e-9),
            "top_k_recall": recall_at_k,
            "spearman_rho_topK": rho,
            "spearman_p": p_spearman,
        }
    return results


def evaluate_apsp(G_full: nx.Graph, G_subs: dict[str, nx.Graph],
                  n_sample_pairs: int = 500, seed: int = 42) -> dict:
    """Run Floyd-Warshall (all-pairs shortest paths) on full graph vs subsets.
    Compare distances for a sample of node pairs that exist in all."""
    # Full-graph APSP
    t0 = time.time()
    # networkx uses Dijkstra for APSP; for comparability use all_pairs_dijkstra
    apsp_full = dict(nx.all_pairs_dijkstra_path_length(G_full))
    t_full = time.time() - t0

    # Sample source-target pairs
    nodes = list(G_full.nodes())
    rng = np.random.RandomState(seed)
    sample_pairs = []
    while len(sample_pairs) < n_sample_pairs:
        u, v = rng.choice(len(nodes), 2, replace=False)
        s, t = nodes[u], nodes[v]
        if t in apsp_full.get(s, {}) and apsp_full[s][t] < float("inf"):
            sample_pairs.append((s, t))
        if len(sample_pairs) >= len(nodes) * (len(nodes) - 1) // 2:
            break
    sample_pairs = sample_pairs[:n_sample_pairs]

    results = {"full_graph": {"n": G_full.number_of_nodes(), "time_s": t_full, "n_sample_pairs": len(sample_pairs)}}

    for name, G_sub in G_subs.items():
        if G_sub.number_of_nodes() == 0:
            continue
        t0 = time.time()
        apsp_sub = dict(nx.all_pairs_dijkstra_path_length(G_sub))
        t_sub = time.time() - t0

        # For sample pairs that exist in sub, compare distances
        kept = set(G_sub.nodes())
        pairs_in_sub = [(s, t) for s, t in sample_pairs if s in kept and t in kept]
        dist_errors = []
        reachable_sub = 0
        for s, t in pairs_in_sub:
            d_full = apsp_full[s].get(t, float("inf"))
            d_sub = apsp_sub.get(s, {}).get(t, float("inf"))
            if d_full == float("inf"):
                continue
            if d_sub == float("inf"):
                # unreachable in subgraph (connectivity lost)
                continue
            reachable_sub += 1
            # relative error: (d_sub - d_full) / d_full (>= 0 since sub is a subgraph, distance only increases)
            err = (d_sub - d_full) / max(d_full, 1e-9)
            dist_errors.append(err)

        results[name] = {
            "n_sub": G_sub.number_of_nodes(),
            "m_sub": G_sub.number_of_edges(),
            "time_s": t_sub,
            "speedup": t_full / max(t_sub, 1e-9),
            "sample_pairs_in_sub": len(pairs_in_sub),
            "reachable_in_sub": reachable_sub,
            "coverage_rate": reachable_sub / max(len(pairs_in_sub), 1),
            "mean_rel_error": float(np.mean(dist_errors)) if dist_errors else None,
            "median_rel_error": float(np.median(dist_errors)) if dist_errors else None,
            "max_rel_error": float(np.max(dist_errors)) if dist_errors else None,
        }
    return results


def generate_benchmarks() -> list[tuple[str, nx.Graph]]:
    """Return (name, graph) pairs."""
    graphs = []
    rng = np.random.RandomState(42)

    # 1. Erdős–Rényi (uniform random)
    G = nx.erdos_renyi_graph(500, 0.02, seed=42)
    graphs.append(("ER_500_p02", G))

    # 2. Barabási–Albert (scale-free, long-tail degree)
    G = nx.barabasi_albert_graph(1000, 3, seed=42)
    graphs.append(("BA_1000_m3", G))

    # 3. Watts–Strogatz (small world)
    G = nx.watts_strogatz_graph(1000, 6, 0.3, seed=42)
    graphs.append(("WS_1000_k6_p03", G))

    # 4. Stochastic block model (planted communities)
    sizes = [100, 100, 100, 100, 100]
    p = [[0.15, 0.005, 0.005, 0.005, 0.005]] * 5
    for i in range(5):
        p[i][i] = 0.15
    G = nx.stochastic_block_model(sizes, p, seed=42)
    graphs.append(("SBM_500_k5", G))

    return graphs


def run_graph(name: str, G: nx.Graph, tmp_dir: Path) -> dict:
    print(f"\n=== Graph: {name} (n={G.number_of_nodes()}, m={G.number_of_edges()}) ===")
    G = G.to_undirected()
    if not nx.is_connected(G):
        largest_cc = max(nx.connected_components(G), key=len)
        G = G.subgraph(largest_cc).copy()
        print(f"  (using largest connected component: n={G.number_of_nodes()}, m={G.number_of_edges()})")

    out = {"name": name, "n": G.number_of_nodes(), "m": G.number_of_edges()}

    for keep_rate in [0.30, 0.50]:
        keep_label = f"{int(keep_rate*100)}"
        # KDF selection
        kdf_kept = kdf_select_via_rust(G, keep_rate, tmp_dir / f"{name}_{keep_label}")
        # Random
        rand_kept = random_select(G, keep_rate, seed=42)
        # Degree-based
        deg_kept = degree_select(G, keep_rate)

        subs = {
            f"KDF@{keep_label}%": G.subgraph(kdf_kept).copy(),
            f"Random@{keep_label}%": G.subgraph(rand_kept).copy(),
            f"TopDegree@{keep_label}%": G.subgraph(deg_kept).copy(),
        }

        print(f"\n  -- Betweenness @ keep_rate={keep_rate} --")
        bc_res = evaluate_betweenness(G, subs, top_k=50)
        print(f"    full: n={bc_res['full_graph']['n']}, time={bc_res['full_graph']['time_s']:.2f}s")
        for k, v in bc_res.items():
            if k == "full_graph":
                continue
            print(f"    {k:<25} n={v['n']}, time={v['time_s']:.2f}s (speedup {v['speedup']:.1f}×), "
                  f"top50_recall={v['top_k_recall']:.3f}, spearman={v['spearman_rho_topK']}")

        print(f"\n  -- APSP (Dijkstra all-pairs) @ keep_rate={keep_rate} --")
        apsp_res = evaluate_apsp(G, subs, n_sample_pairs=500, seed=42)
        print(f"    full: n={apsp_res['full_graph']['n']}, time={apsp_res['full_graph']['time_s']:.2f}s, "
              f"sample_pairs={apsp_res['full_graph']['n_sample_pairs']}")
        for k, v in apsp_res.items():
            if k == "full_graph":
                continue
            err = f"mean_err={v['mean_rel_error']:.3f}" if v['mean_rel_error'] is not None else "mean_err=N/A"
            print(f"    {k:<25} coverage={v['coverage_rate']:.3f} ({v['reachable_in_sub']}/{v['sample_pairs_in_sub']}), "
                  f"{err}, speedup {v['speedup']:.1f}×")

        out[f"keep_{keep_label}pct"] = {
            "betweenness": {k: {kk: vv for kk, vv in v.items() if kk != "top_k_nodes"} for k, v in bc_res.items()},
            "apsp": apsp_res,
        }
    return out


def main():
    out_dir = Path("benchmarks/classical_revival/out")
    tmp_dir = Path("benchmarks/classical_revival/tmp")
    out_dir.mkdir(parents=True, exist_ok=True)
    tmp_dir.mkdir(parents=True, exist_ok=True)

    benchmarks = generate_benchmarks()
    all_results = []
    for name, G in benchmarks:
        res = run_graph(name, G, tmp_dir)
        all_results.append(res)

    out_path = out_dir / "c1_c2_results.json"
    with out_path.open("w", encoding="utf-8") as f:
        json.dump({"results": all_results}, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {out_path}")

    # Summary
    print("\n" + "=" * 100)
    print("Summary: Betweenness top-50 recall across graphs (KDF vs Random vs TopDegree)")
    print("=" * 100)
    print(f"{'graph':<25}{'keep':>8}{'KDF':>10}{'Random':>10}{'TopDeg':>10}")
    for res in all_results:
        for keep_label in ["30", "50"]:
            key = f"keep_{keep_label}pct"
            if key not in res:
                continue
            bc = res[key]["betweenness"]
            kdf = bc.get(f"KDF@{keep_label}%", {}).get("top_k_recall", 0)
            rand = bc.get(f"Random@{keep_label}%", {}).get("top_k_recall", 0)
            deg = bc.get(f"TopDegree@{keep_label}%", {}).get("top_k_recall", 0)
            print(f"{res['name']:<25}{keep_label+'%':>8}{kdf:>10.3f}{rand:>10.3f}{deg:>10.3f}")

    print("\n(Higher recall = better preservation of top-50 betweenness-central nodes)")


if __name__ == "__main__":
    main()
