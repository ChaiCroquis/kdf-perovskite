"""
B4: Financial Fraud Transaction Archival — does KDF pruning preserve
fraud transactions at 30% budget?

Setup:
  - Credit Card Fraud detection dataset (OpenML id=1597, Kaggle classic)
  - 284,807 transactions, 492 fraud (0.17%), PCA-transformed features V1..V28 + Amount + Time
  - Subsample: all 492 fraud + 4,508 random normal = 5,000 transactions
  - Build k-NN similarity graph (k=10) on standardized features
  - Apply 5 selection methods at 30% and 50% keep_rate:
      - KDF (via kdf_select_generic)
      - Random
      - TopDegree (degree-based)
      - IsolationForest (unsupervised anomaly detector — fraud-specific baseline)
      - KMeans (density-center baseline, same as F-063)

Ground truth: fraud label (Class==1).

Question: does KDF's structural-rareness preservation catch fraud
transactions better than generic baselines? Does it match IsolationForest
(domain-specific anomaly detection)?

Cost: $0, ~5-10 minutes.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
from sklearn.cluster import KMeans
from sklearn.datasets import fetch_openml
from sklearn.ensemble import IsolationForest
from sklearn.neighbors import NearestNeighbors
from sklearn.preprocessing import StandardScaler

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def load_fraud_data(n_subsample: int = 5000, seed: int = 42):
    """Fetch CreditCardFraud from OpenML, stratified subsample."""
    print("Fetching CreditCardFraud from OpenML (id=1597)...")
    t0 = time.time()
    ds = fetch_openml(data_id=1597, as_frame=True)
    print(f"  loaded in {time.time()-t0:.1f}s, shape={ds.data.shape}")

    X = ds.data.values
    y = (ds.target.astype(int).values == 1).astype(int)
    n_fraud = y.sum()
    print(f"  fraud: {n_fraud} ({n_fraud/len(y)*100:.3f}%), normal: {len(y)-n_fraud}")

    rng = np.random.RandomState(seed)
    fraud_idx = np.where(y == 1)[0]
    normal_idx = np.where(y == 0)[0]

    n_normal_sample = n_subsample - len(fraud_idx)
    normal_sample = rng.choice(normal_idx, size=n_normal_sample, replace=False)

    all_idx = np.concatenate([fraud_idx, normal_sample])
    rng.shuffle(all_idx)

    X_sub = X[all_idx]
    y_sub = y[all_idx]
    print(f"  subsampled: {len(X_sub)} total, {y_sub.sum()} fraud ({y_sub.sum()/len(y_sub)*100:.2f}%)")
    return X_sub, y_sub


def build_knn_edges(X: np.ndarray, k: int = 10):
    nn = NearestNeighbors(n_neighbors=k + 1).fit(X)
    _, idx = nn.kneighbors(X)
    edges_set = set()
    for i, neighbors in enumerate(idx):
        for j in neighbors[1:]:
            u, v = (int(i), int(j)) if i < j else (int(j), int(i))
            edges_set.add((u, v))
    return [(u, v, 1.0) for (u, v) in edges_set]


def kdf_select(X, edges, keep_rate, tmp_dir: Path):
    tmp_dir.mkdir(parents=True, exist_ok=True)
    graph_input = {"n": X.shape[0], "edges": edges}
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
    return set(result["selected_node_indices"])


def random_select(n, keep_rate, seed=42):
    rng = np.random.RandomState(seed)
    k = max(1, int(n * keep_rate))
    return set(rng.choice(n, size=k, replace=False).tolist())


def top_degree_select(n, edges, keep_rate):
    deg = [0] * n
    for u, v, _ in edges:
        deg[u] += 1
        deg[v] += 1
    k = max(1, int(n * keep_rate))
    sorted_idx = sorted(range(n), key=lambda i: -deg[i])
    return set(sorted_idx[:k])


def isolation_forest_select(X, keep_rate, seed=42):
    """Select points with the most anomalous scores (lowest IsolationForest score)."""
    k = max(1, int(X.shape[0] * keep_rate))
    iso = IsolationForest(contamination="auto", random_state=seed, n_estimators=100)
    iso.fit(X)
    scores = iso.score_samples(X)  # lower = more anomalous
    sorted_idx = np.argsort(scores)  # most anomalous first
    return set(sorted_idx[:k].tolist())


def kmeans_select(X, keep_rate, seed=42):
    """k-means centers → nearest training points."""
    k = max(1, int(X.shape[0] * keep_rate))
    km = KMeans(n_clusters=k, random_state=seed, n_init=5).fit(X)
    nn = NearestNeighbors(n_neighbors=1).fit(X)
    _, nearest = nn.kneighbors(km.cluster_centers_)
    return set(int(i[0]) for i in nearest)


def evaluate(selected: set[int], y: np.ndarray) -> dict:
    y_selected = y[list(selected)]
    n_fraud_total = int(y.sum())
    n_fraud_kept = int(y_selected.sum())
    n_selected = len(selected)
    fraud_recall = n_fraud_kept / max(n_fraud_total, 1)
    # Precision among selected
    fraud_frac_in_selected = n_fraud_kept / max(n_selected, 1)
    return {
        "n_selected": n_selected,
        "n_fraud_kept": n_fraud_kept,
        "fraud_recall": fraud_recall,
        "fraud_fraction_in_selected": fraud_frac_in_selected,
        "baseline_fraud_fraction": float(y.sum() / len(y)),
    }


def main():
    X, y = load_fraud_data(n_subsample=5000)

    print("\nStandardizing features...")
    Xs = StandardScaler().fit_transform(X)

    print("Building k-NN graph (k=10)...")
    t0 = time.time()
    edges = build_knn_edges(Xs, k=10)
    print(f"  {len(edges)} edges built in {time.time()-t0:.1f}s")

    tmp_dir = Path("benchmarks/classical_revival/tmp/b4_fraud")
    out_dir = Path("benchmarks/classical_revival/out")
    out_dir.mkdir(parents=True, exist_ok=True)

    all_results = {}
    for keep_rate in [0.30, 0.50]:
        keep_label = f"{int(keep_rate*100)}"
        print(f"\n=== keep_rate = {keep_rate:.2f} ({int(keep_rate*100)}%) ===")
        t0 = time.time()
        methods = {
            "KDF": kdf_select(Xs, edges, keep_rate, tmp_dir / keep_label),
            "Random": random_select(Xs.shape[0], keep_rate, seed=42),
            "TopDegree": top_degree_select(Xs.shape[0], edges, keep_rate),
            "KMeans": kmeans_select(Xs, keep_rate, seed=42),
            "IsolationForest": isolation_forest_select(Xs, keep_rate, seed=42),
        }
        keep_results = {}
        print(f"  {'method':<20}{'n_sel':>8}{'n_fraud':>10}{'recall':>10}{'precision':>12}")
        for name, sel in methods.items():
            r = evaluate(sel, y)
            keep_results[name] = r
            print(
                f"  {name:<20}{r['n_selected']:>8}{r['n_fraud_kept']:>10}"
                f"{r['fraud_recall']*100:>9.2f}%"
                f"{r['fraud_fraction_in_selected']*100:>11.3f}%"
            )
        print(f"  (baseline fraud rate in pool: {keep_results['Random']['baseline_fraud_fraction']*100:.2f}%)")
        print(f"  elapsed: {time.time()-t0:.1f}s")
        all_results[f"keep_{keep_label}"] = keep_results

    # Save
    out = out_dir / "b4_fraud_results.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "dataset": "CreditCardFraud (OpenML 1597)",
            "n_subsample": int(Xs.shape[0]),
            "n_fraud": int(y.sum()),
            "fraud_rate": float(y.sum() / len(y)),
            "n_edges": len(edges),
            "k_nn": 10,
            "results": all_results,
        }, f, indent=2)
    print(f"\nSaved: {out}")


if __name__ == "__main__":
    main()
