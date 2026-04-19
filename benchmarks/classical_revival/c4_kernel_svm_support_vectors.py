"""
C4: Kernel SVM Training Point Selection via KDF.

Hypothesis(from classical_algorithm_revival.md): for RBF-kernel SVM on
n-sample training data, using KDF-selected ~30% subset as training set
could yield comparable classifier accuracy to training on all n (with
O(n^2) → O(m^2) speedup where m = 0.3n).

Expected result (per F-063 GP pattern): KDF likely underperforms because
SVM support vectors lie at class decision boundaries (density-dependent),
which aligns with density coverage, not structural rareness.

This is an honest confirmation of the applicability predictor from
F-063/F-064/F-066.

Design:
  - UCI classification datasets (Breast Cancer, Digits subset)
  - k-NN graph on standardized features (k=7)
  - Select 30%/50% training subset via 4 methods: KDF, Random, KMeans, TopDegree
  - Train RBF SVM on each subset, measure test accuracy
  - Baseline: full-data SVM (reference)

Cost: $0, runtime: minutes.
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
from sklearn.datasets import load_breast_cancer, load_digits
from sklearn.model_selection import train_test_split
from sklearn.neighbors import NearestNeighbors
from sklearn.preprocessing import StandardScaler
from sklearn.svm import SVC

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def build_knn_edges(X, k=7):
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
    return sorted(result["selected_node_indices"])


def random_select(n, keep_rate, seed=42):
    rng = np.random.RandomState(seed)
    k = max(1, int(n * keep_rate))
    return sorted(rng.choice(n, size=k, replace=False).tolist())


def top_degree_select(n, edges, keep_rate):
    deg = [0] * n
    for u, v, _ in edges:
        deg[u] += 1
        deg[v] += 1
    k = max(1, int(n * keep_rate))
    return sorted(sorted(range(n), key=lambda i: -deg[i])[:k])


def kmeans_select(X, keep_rate, seed=42):
    k = max(1, int(X.shape[0] * keep_rate))
    km = KMeans(n_clusters=k, random_state=seed, n_init=5).fit(X)
    nn = NearestNeighbors(n_neighbors=1).fit(X)
    _, nearest = nn.kneighbors(km.cluster_centers_)
    return sorted(set(int(i[0]) for i in nearest))


def train_svm(X_train, y_train, X_test, y_test):
    t0 = time.time()
    clf = SVC(kernel="rbf", C=1.0, gamma="scale")
    clf.fit(X_train, y_train)
    train_time = time.time() - t0
    acc = clf.score(X_test, y_test)
    return {
        "accuracy": float(acc),
        "train_time_s": train_time,
        "n_used": int(X_train.shape[0]),
        "n_support_vectors": int(sum(clf.n_support_)),
    }


def run_benchmark(name, X, y, tmp_dir):
    print(f"\n=== {name}: N={X.shape[0]}, d={X.shape[1]}, classes={len(set(y))} ===")
    Xs = StandardScaler().fit_transform(X)
    X_train, X_test, y_train, y_test = train_test_split(
        Xs, y, test_size=0.25, random_state=42, stratify=y
    )
    print(f"  train={X_train.shape[0]}, test={X_test.shape[0]}")

    print("  Building k-NN graph (k=7)...")
    edges = build_knn_edges(X_train, k=7)

    print(f"  [ref] Full SVM on {X_train.shape[0]} samples...")
    ref = train_svm(X_train, y_train, X_test, y_test)
    print(f"    acc={ref['accuracy']:.4f}, time={ref['train_time_s']:.2f}s, SVs={ref['n_support_vectors']}")

    results = {"dataset": name, "n_train": X_train.shape[0], "n_test": X_test.shape[0], "full": ref}
    for keep_rate in [0.30, 0.50]:
        print(f"\n  -- keep_rate={keep_rate} --")
        methods = {
            "KDF": kdf_select(X_train, edges, keep_rate, tmp_dir / f"{name}_{int(keep_rate*100)}"),
            "Random": random_select(X_train.shape[0], keep_rate, seed=42),
            "KMeans": kmeans_select(X_train, keep_rate, seed=42),
            "TopDegree": top_degree_select(X_train.shape[0], edges, keep_rate),
        }
        keep_results = {}
        print(f"    {'method':<12}{'n':>6}{'acc':>10}{'SVs':>8}{'time(s)':>10}{'Δ vs ref':>12}")
        for m_name, idxs in methods.items():
            Xs_sub = X_train[idxs]
            ys_sub = y_train[idxs]
            if len(set(ys_sub)) < 2:
                print(f"    {m_name:<12} skip (only 1 class in subset)")
                continue
            r = train_svm(Xs_sub, ys_sub, X_test, y_test)
            r["delta_acc"] = r["accuracy"] - ref["accuracy"]
            keep_results[m_name] = r
            print(
                f"    {m_name:<12}{r['n_used']:>6}"
                f"{r['accuracy']:>10.4f}"
                f"{r['n_support_vectors']:>8}"
                f"{r['train_time_s']:>10.3f}"
                f"{r['delta_acc']:>+12.4f}"
            )
        results[f"keep_{int(keep_rate*100)}pct"] = keep_results
    return results


def main():
    tmp_dir = Path("benchmarks/classical_revival/tmp/c4_svm")
    out_dir = Path("benchmarks/classical_revival/out")
    out_dir.mkdir(parents=True, exist_ok=True)

    datasets = []
    bc = load_breast_cancer()
    datasets.append(("BreastCancer", bc.data, bc.target))
    dg = load_digits()
    datasets.append(("Digits", dg.data, dg.target))

    all_results = []
    for name, X, y in datasets:
        res = run_benchmark(name, X, y, tmp_dir)
        all_results.append(res)

    out = out_dir / "c4_kernel_svm_results.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump({"results": all_results}, f, indent=2)
    print(f"\nSaved: {out}")

    print("\n" + "=" * 100)
    print("Summary: Kernel SVM test accuracy on subset training")
    print("=" * 100)
    print(f"{'dataset':<18}{'keep':>6}{'full':>10}{'KDF':>10}{'Random':>10}{'KMeans':>10}{'TopDeg':>10}")
    for r in all_results:
        full_acc = r["full"]["accuracy"]
        for keep in ["30", "50"]:
            key = f"keep_{keep}pct"
            if key not in r:
                continue
            row = r[key]
            def g(name, k="accuracy"):
                return f"{row[name][k]:.4f}" if name in row else "    -"
            print(f"{r['dataset']:<18}{keep+'%':>6}{full_acc:>10.4f}"
                  f"{g('KDF'):>10}{g('Random'):>10}{g('KMeans'):>10}{g('TopDegree'):>10}")


if __name__ == "__main__":
    main()
