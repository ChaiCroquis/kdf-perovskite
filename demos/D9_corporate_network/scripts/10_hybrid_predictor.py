"""
D9 Step 10: Hybrid predictor — KDF + PageRank + Betweenness + features.

Empirically test: does adding classical graph metrics (PageRank,
Betweenness) to KDF layer improve Edge→Core breakthrough prediction?

Compared models (all predicting y = "T1-Edge became T2-Core"):
  M0. Random (base rate) baseline
  M1. KDF layer alone (one-hot: Rare/Core/Edge/Garbage)
  M2. Degree alone (from step 9 we know this is strongest single feature)
  M3. PageRank alone
  M4. Betweenness alone
  M5. KDF + degree (simple hybrid)
  M6. KDF + degree + PageRank (medium hybrid)
  M7. KDF + all features (full hybrid, GBT)

Metrics:
  - 5-fold cross-validated AUC
  - Precision @ top-5%, 10%, 20% (breakthrough rates in top-K)
  - Lift vs base rate

Cost: $0. Runtime: 3-5 min (betweenness computation dominates).
"""
from __future__ import annotations

import json
import sys
import time
from collections import defaultdict
from itertools import combinations
from pathlib import Path

import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

try:
    import networkx as nx
except ImportError:
    print("ERROR: networkx not installed")
    sys.exit(1)

from sklearn.ensemble import GradientBoostingClassifier
from sklearn.linear_model import LogisticRegression
from sklearn.model_selection import StratifiedKFold, cross_val_predict
from sklearn.preprocessing import StandardScaler
from sklearn.metrics import roc_auc_score

OUT_DIR = Path("demos/D9_corporate_network/out")


def build_t1_graph():
    """Rebuild T1 (2014-2018) graph as networkx.Graph."""
    print("Loading T1 papers and rebuilding graph...", file=sys.stderr)
    with open(OUT_DIR / "papers_t1_2014_2018.json", encoding="utf-8") as f:
        papers = json.load(f)

    edge_weights = defaultdict(float)
    all_insts = set()
    for p in papers:
        paper_insts = set()
        for a in p.get("authorships", []):
            for inst in a.get("institutions", []):
                iid = inst.get("id")
                if iid:
                    paper_insts.add(iid)
                    all_insts.add(iid)
        for u, v in combinations(sorted(paper_insts), 2):
            edge_weights[(u, v)] += 1

    G = nx.Graph()
    G.add_nodes_from(all_insts)
    for (u, v), w in edge_weights.items():
        G.add_edge(u, v, weight=w)

    print(f"  T1 graph: {G.number_of_nodes()} nodes, {G.number_of_edges()} edges", file=sys.stderr)
    return G


def compute_graph_features(G):
    """Return dict: inst_id → {pagerank, betweenness}."""
    print("Computing PageRank...", file=sys.stderr)
    t0 = time.time()
    pagerank = nx.pagerank(G, weight="weight", max_iter=200)
    print(f"  PageRank done in {time.time()-t0:.1f}s", file=sys.stderr)

    print("Computing Betweenness (sampled, k=500)...", file=sys.stderr)
    t0 = time.time()
    # Use k-sample approximation for speed (full is O(VE))
    betweenness = nx.betweenness_centrality(G, k=500, weight="weight", seed=42)
    print(f"  Betweenness done in {time.time()-t0:.1f}s", file=sys.stderr)

    return {iid: {"pagerank": pagerank.get(iid, 0.0),
                  "betweenness": betweenness.get(iid, 0.0)}
            for iid in G.nodes()}


def load_backtest():
    with open(OUT_DIR / "backtest_results.json", encoding="utf-8") as f:
        return json.load(f)


def build_feature_matrix(joined, graph_features):
    """For T1-Edge institutions present in backtest, build X, y."""
    t1_edges = [r for r in joined if r["t1_layer"] == "Edge"]
    print(f"T1 Edge institutions: {len(t1_edges)}", file=sys.stderr)

    rows = []
    for r in t1_edges:
        iid = r["id"]
        gf = graph_features.get(iid, {"pagerank": 0.0, "betweenness": 0.0})
        if not r.get("in_t2"):
            # Faders: we still include them as y=0 (didn't become Core)
            y = 0
        else:
            y = 1 if r["t2_layer"] == "Core" else 0
        rows.append({
            "id": iid,
            "name": r.get("name"),
            "kdf_is_rare": 1.0 if r["t1_layer"] == "Rare" else 0.0,
            "kdf_is_core": 1.0 if r["t1_layer"] == "Core" else 0.0,
            "kdf_is_edge": 1.0 if r["t1_layer"] == "Edge" else 0.0,
            "kdf_is_garbage": 1.0 if r["t1_layer"] == "Garbage" else 0.0,
            "degree": r["t1_degree"],
            "n_fields": r["t1_n_fields"],
            "n_papers": r["t1_n_papers"],
            "pagerank": gf["pagerank"],
            "betweenness": gf["betweenness"],
            "y": y,
        })
    return rows


def eval_model(X, y, model, name, k_percentiles=[5, 10, 20]):
    """Cross-validated AUC + precision@k%."""
    skf = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
    probs = cross_val_predict(model, X, y, cv=skf, method="predict_proba")[:, 1]
    auc = roc_auc_score(y, probs)

    # Precision at top K% (breakthrough rate in top-ranked)
    precisions = {}
    for k_pct in k_percentiles:
        k = max(1, int(len(y) * k_pct / 100))
        top_k_idx = np.argsort(probs)[::-1][:k]
        precisions[k_pct] = np.mean(y[top_k_idx])

    base_rate = float(np.mean(y))
    return {
        "name": name,
        "auc": auc,
        "base_rate": base_rate,
        "precision_at_5pct": precisions[5],
        "precision_at_10pct": precisions[10],
        "precision_at_20pct": precisions[20],
        "lift_at_5pct": precisions[5] / max(base_rate, 1e-9),
        "lift_at_10pct": precisions[10] / max(base_rate, 1e-9),
    }


def main():
    # Graph features for T1
    G = build_t1_graph()
    graph_features = compute_graph_features(G)

    # Load backtest results
    bt = load_backtest()
    joined = bt["joined"]

    # Build feature matrix
    rows = build_feature_matrix(joined, graph_features)
    print(f"\nFeature matrix: {len(rows)} rows")
    n_pos = sum(1 for r in rows if r["y"] == 1)
    n_neg = len(rows) - n_pos
    base_rate = n_pos / len(rows)
    print(f"  Positive (Edge→Core): {n_pos}")
    print(f"  Negative: {n_neg}")
    print(f"  Base rate: {base_rate*100:.2f}%")

    # Feature arrays
    def select_cols(rows, cols):
        return np.array([[r[c] for c in cols] for r in rows])

    y = np.array([r["y"] for r in rows])

    # Define models
    def lr():
        return LogisticRegression(max_iter=1000, random_state=42, C=1.0)

    def gbt():
        return GradientBoostingClassifier(random_state=42, n_estimators=100, max_depth=3)

    results = []

    # M0: base rate
    results.append({
        "name": "M0. Random / base rate",
        "auc": 0.500,
        "base_rate": base_rate,
        "precision_at_5pct": base_rate,
        "precision_at_10pct": base_rate,
        "precision_at_20pct": base_rate,
        "lift_at_5pct": 1.0,
        "lift_at_10pct": 1.0,
    })

    # M1: KDF layer alone (one-hot)
    X1 = select_cols(rows, ["kdf_is_rare", "kdf_is_core", "kdf_is_edge", "kdf_is_garbage"])
    # Note: all rows are Edge so kdf_is_edge=1 always → uninformative
    # Simulate "if I only knew KDF layer" by using degree (closest single KDF output)
    # Actually KDF layer IS all "Edge" here. So M1 tests "does the fact that T1 was Edge help?" → no.
    # Instead, use a layer-based approach: rare neighbors? Not available here.
    # Let's skip M1 pure-layer (all = Edge) and use degree as closest KDF-derived proxy.

    # M2: Degree alone
    X2 = select_cols(rows, ["degree"])
    X2s = StandardScaler().fit_transform(X2)
    results.append(eval_model(X2s, y, lr(), "M2. Degree alone (LogReg)"))

    # M3: PageRank alone
    X3 = select_cols(rows, ["pagerank"])
    X3s = StandardScaler().fit_transform(X3)
    results.append(eval_model(X3s, y, lr(), "M3. PageRank alone (LogReg)"))

    # M4: Betweenness alone
    X4 = select_cols(rows, ["betweenness"])
    X4s = StandardScaler().fit_transform(X4)
    results.append(eval_model(X4s, y, lr(), "M4. Betweenness alone (LogReg)"))

    # M5: Degree + n_fields (KDF-inspired simple rule proxy)
    X5 = select_cols(rows, ["degree", "n_fields"])
    X5s = StandardScaler().fit_transform(X5)
    results.append(eval_model(X5s, y, lr(), "M5. Degree + n_fields (LogReg)"))

    # M6: Degree + PageRank (two centrality signals)
    X6 = select_cols(rows, ["degree", "pagerank"])
    X6s = StandardScaler().fit_transform(X6)
    results.append(eval_model(X6s, y, lr(), "M6. Degree + PageRank (LogReg)"))

    # M7: All features + LogReg
    all_cols = ["degree", "n_fields", "n_papers", "pagerank", "betweenness"]
    X7 = select_cols(rows, all_cols)
    X7s = StandardScaler().fit_transform(X7)
    results.append(eval_model(X7s, y, lr(), "M7. All features (LogReg)"))

    # M8: All features + Gradient Boosting
    results.append(eval_model(X7, y, gbt(), "M8. All features (GBT)"))

    # M9: All + KDF layer one-hot + GBT
    all_cols_full = ["kdf_is_rare", "kdf_is_core", "kdf_is_edge", "kdf_is_garbage"] + all_cols
    X9 = select_cols(rows, all_cols_full)
    results.append(eval_model(X9, y, gbt(), "M9. All + KDF layer (GBT)"))

    # Print table
    print("\n" + "=" * 110)
    print(f"Model comparison — predict T1 Edge → T2 Core (base rate {base_rate*100:.2f}%)")
    print("=" * 110)
    print(f"{'Model':<42}{'AUC':>8}{'P@5%':>10}{'Lift@5%':>10}{'P@10%':>10}{'Lift@10%':>11}{'P@20%':>10}")
    for r in results:
        p5 = r["precision_at_5pct"] * 100
        p10 = r["precision_at_10pct"] * 100
        p20 = r["precision_at_20pct"] * 100
        print(f"{r['name']:<42}{r['auc']:>8.3f}"
              f"{p5:>9.2f}%"
              f"{r['lift_at_5pct']:>9.2f}x"
              f"{p10:>9.2f}%"
              f"{r['lift_at_10pct']:>10.2f}x"
              f"{p20:>9.2f}%")

    # Save
    out = OUT_DIR / "hybrid_predictor_results.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "n_samples": len(rows),
            "n_positives": n_pos,
            "base_rate": base_rate,
            "models": results,
        }, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {out}")

    # Best-performing model summary
    best_by_auc = max(results, key=lambda r: r["auc"])
    best_by_p5 = max(results, key=lambda r: r["precision_at_5pct"])
    print("\n--- Best performers ---")
    print(f"  Best AUC:        {best_by_auc['name']}  (AUC={best_by_auc['auc']:.3f})")
    print(f"  Best P@5%:       {best_by_p5['name']}  (P@5%={best_by_p5['precision_at_5pct']*100:.2f}%, lift={best_by_p5['lift_at_5pct']:.2f}x)")

    # Compare with KDF-single-rule baseline from step 9
    # Step 9: top 5% degree → 22.0%, lift 2.1x
    print("\n--- vs Step 9 (KDF degree top 5% rule) ---")
    print(f"  Step 9 simple rule: 22.0% precision (lift 2.1x)")
    print(f"  Best ML model:      {best_by_p5['precision_at_5pct']*100:.2f}% precision (lift {best_by_p5['lift_at_5pct']:.2f}x)")
    improvement = (best_by_p5["precision_at_5pct"] - 0.22) * 100
    print(f"  Improvement:        {improvement:+.2f}pt")


if __name__ == "__main__":
    main()
