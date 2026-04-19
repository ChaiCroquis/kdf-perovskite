"""
C5: Gaussian Process Regression with KDF-selected inducing points.

Hypothesis: KDF's structural rarity selection, applied to a k-NN
similarity graph built from feature vectors, produces inducing points
that preserve the data distribution's "structure-critical" regions.
These should work as well or better than the standard k-means-centers
inducing point selection.

Design:
  1. Load UCI regression dataset (small enough for full-GP reference)
  2. Build k-NN similarity graph on training features (k=5)
  3. Select m inducing points (m = 30% of n_train) via 4 methods:
     - KDF (call the Rust kdf_select_generic binary)
     - Random
     - k-means (standard baseline)
     - TopDegree (simple baseline)
  4. Train sparse GP: use inducing points + kernel ridge on subset
  5. Evaluate test RMSE, NLL (negative log-likelihood of test under
     predictive posterior), training time

Datasets: Boston (N=506) and synthetic Friedman (configurable).

Cost: $0. Runtime: minutes.
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
from sklearn.datasets import make_friedman1
from sklearn.gaussian_process import GaussianProcessRegressor
from sklearn.gaussian_process.kernels import ConstantKernel, RBF, WhiteKernel
from sklearn.model_selection import train_test_split
from sklearn.neighbors import NearestNeighbors
from sklearn.preprocessing import StandardScaler

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


def build_knn_graph(X: np.ndarray, k: int = 5):
    """Build k-NN similarity graph from feature matrix X. Returns edges as
    list of (u, v, w=1.0) tuples (undirected, dedup)."""
    nn = NearestNeighbors(n_neighbors=k + 1).fit(X)  # +1 to exclude self
    _, indices = nn.kneighbors(X)
    edges_set = set()
    for i, neighbors in enumerate(indices):
        for j in neighbors[1:]:  # skip self
            u, v = (int(i), int(j)) if i < j else (int(j), int(i))
            edges_set.add((u, v))
    edges = [(u, v, 1.0) for (u, v) in edges_set]
    return edges


def kdf_select_points(X: np.ndarray, keep_rate: float, tmp_dir: Path, k_nn: int = 5) -> list[int]:
    """Build k-NN graph and call Rust KDF selector."""
    tmp_dir.mkdir(parents=True, exist_ok=True)
    edges = build_knn_graph(X, k=k_nn)
    graph_input = {
        "n": X.shape[0],
        "edges": edges,
    }
    in_path = tmp_dir / "graph.json"
    out_path = tmp_dir / "selected.json"
    with in_path.open("w", encoding="utf-8") as f:
        json.dump(graph_input, f)
    subprocess.run(
        [
            "cargo", "run", "--release", "-q",
            "-p", "demo-d8-llm-memory",
            "--bin", "kdf_select_generic", "--",
            "--input", str(in_path),
            "--out", str(out_path),
            "--keep-rate", str(keep_rate),
        ],
        check=True, capture_output=True,
    )
    with out_path.open("r", encoding="utf-8") as f:
        result = json.load(f)
    return sorted(result["selected_node_indices"])


def random_select_points(n: int, keep_rate: float, seed: int = 42) -> list[int]:
    rng = np.random.RandomState(seed)
    k = max(1, int(n * keep_rate))
    return sorted(rng.choice(n, size=k, replace=False).tolist())


def kmeans_inducing_points(X: np.ndarray, keep_rate: float, seed: int = 42):
    """Return centers from k-means. Note: these are NOT training data points, but
    for a fair comparison we pick the nearest training point to each center."""
    k = max(1, int(X.shape[0] * keep_rate))
    km = KMeans(n_clusters=k, random_state=seed, n_init=10).fit(X)
    nn = NearestNeighbors(n_neighbors=1).fit(X)
    _, nearest = nn.kneighbors(km.cluster_centers_)
    indices = sorted(set(int(i[0]) for i in nearest))
    return indices


def top_degree_select_points(X: np.ndarray, keep_rate: float, k_nn: int = 5) -> list[int]:
    edges = build_knn_graph(X, k=k_nn)
    degree = [0] * X.shape[0]
    for u, v, _ in edges:
        degree[u] += 1
        degree[v] += 1
    k = max(1, int(X.shape[0] * keep_rate))
    sorted_idx = sorted(range(X.shape[0]), key=lambda i: -degree[i])
    return sorted(sorted_idx[:k])


def train_gp_on_subset(X_sub, y_sub, X_test, y_test):
    """Train GP on a subset; return test RMSE, NLL, train time."""
    kernel = (
        ConstantKernel(1.0, (1e-3, 1e3))
        * RBF(length_scale=1.0, length_scale_bounds=(1e-2, 1e2))
        + WhiteKernel(noise_level=1e-3, noise_level_bounds=(1e-6, 1e+1))
    )
    gp = GaussianProcessRegressor(kernel=kernel, n_restarts_optimizer=2, random_state=42)
    t0 = time.time()
    gp.fit(X_sub, y_sub)
    train_time = time.time() - t0

    y_pred, y_std = gp.predict(X_test, return_std=True)
    rmse = float(np.sqrt(np.mean((y_pred - y_test) ** 2)))
    # NLL (negative log-likelihood under Gaussian predictive)
    var = y_std ** 2 + 1e-9
    nll = float(
        0.5 * np.mean(
            np.log(2 * np.pi * var)
            + (y_test - y_pred) ** 2 / var
        )
    )
    return {"rmse": rmse, "nll": nll, "train_time_s": train_time, "n_used": X_sub.shape[0]}


def run_benchmark(name: str, X: np.ndarray, y: np.ndarray, tmp_dir: Path) -> dict:
    print(f"\n=== Benchmark: {name} (N={X.shape[0]}, d={X.shape[1]}) ===")
    # Standardize
    scaler_X = StandardScaler().fit(X)
    scaler_y = StandardScaler().fit(y.reshape(-1, 1))
    Xs = scaler_X.transform(X)
    ys = scaler_y.transform(y.reshape(-1, 1)).ravel()

    X_train, X_test, y_train, y_test = train_test_split(
        Xs, ys, test_size=0.2, random_state=42
    )
    print(f"  train: {X_train.shape[0]}, test: {X_test.shape[0]}")

    # Reference: full GP
    print(f"\n  [ref] Training full GP on {X_train.shape[0]} points...")
    ref = train_gp_on_subset(X_train, y_train, X_test, y_test)
    print(f"    RMSE={ref['rmse']:.4f}, NLL={ref['nll']:.4f}, time={ref['train_time_s']:.2f}s")

    results = {"dataset": name, "n_train": X_train.shape[0], "n_test": X_test.shape[0], "full": ref}

    for keep_rate in [0.30, 0.50]:
        keep_label = f"{int(keep_rate*100)}"
        print(f"\n  -- keep_rate={keep_rate} (~{int(X_train.shape[0] * keep_rate)} inducing points) --")
        methods_idxs = {
            "KDF": kdf_select_points(X_train, keep_rate, tmp_dir / f"{name}_{keep_label}"),
            "Random": random_select_points(X_train.shape[0], keep_rate, seed=42),
            "KMeans": kmeans_inducing_points(X_train, keep_rate, seed=42),
            "TopDegree": top_degree_select_points(X_train, keep_rate, k_nn=5),
        }
        keep_results = {}
        print(f"    {'method':<12}{'n':>6}{'RMSE':>10}{'NLL':>10}{'time(s)':>10}{'Δ vs ref':>12}")
        for method_name, idxs in methods_idxs.items():
            X_sub = X_train[idxs]
            y_sub = y_train[idxs]
            res = train_gp_on_subset(X_sub, y_sub, X_test, y_test)
            res["delta_rmse"] = res["rmse"] - ref["rmse"]
            keep_results[method_name] = res
            print(
                f"    {method_name:<12}{res['n_used']:>6}"
                f"{res['rmse']:>10.4f}{res['nll']:>10.4f}"
                f"{res['train_time_s']:>10.2f}"
                f"{res['delta_rmse']:>+12.4f}"
            )
        results[f"keep_{keep_label}pct"] = keep_results

    return results


def load_boston_fallback():
    """Load Boston-like housing dataset (Boston was deprecated in sklearn 1.2+).
    Use California as a larger alternative."""
    from sklearn.datasets import fetch_california_housing
    ds = fetch_california_housing()
    X = ds.data
    y = ds.target
    # Subsample for GP tractability (full = 20,640 too big for exact GP)
    rng = np.random.RandomState(42)
    idx = rng.choice(X.shape[0], size=800, replace=False)
    return X[idx], y[idx]


def load_friedman1(n_samples: int = 500, random_state: int = 42):
    X, y = make_friedman1(n_samples=n_samples, random_state=random_state, noise=0.5)
    return X, y


def main():
    tmp_dir = Path("benchmarks/classical_revival/tmp/c5_gp")
    out_dir = Path("benchmarks/classical_revival/out")
    out_dir.mkdir(parents=True, exist_ok=True)

    benchmarks = []
    # California housing sub-sample
    try:
        X, y = load_boston_fallback()
        benchmarks.append(("CaliforniaHousing_800", X, y))
    except Exception as e:
        print(f"[skip] CaliforniaHousing: {e}")
    # Friedman 1 synthetic
    X, y = load_friedman1(n_samples=500)
    benchmarks.append(("Friedman1_500", X, y))

    all_results = []
    for name, X, y in benchmarks:
        res = run_benchmark(name, X, y, tmp_dir)
        all_results.append(res)

    # Save
    out_path = out_dir / "c5_gp_results.json"
    with out_path.open("w", encoding="utf-8") as f:
        json.dump({"results": all_results}, f, indent=2)
    print(f"\nSaved: {out_path}")

    # Summary
    print("\n" + "=" * 100)
    print("Summary: GP with subset inducing points — test RMSE (lower = better)")
    print("=" * 100)
    print(f"{'dataset':<25}{'keep':>6}{'full':>10}{'KDF':>10}{'Random':>10}{'KMeans':>10}{'TopDeg':>10}")
    for r in all_results:
        full_rmse = r["full"]["rmse"]
        for keep_label in ["30", "50"]:
            key = f"keep_{keep_label}pct"
            if key not in r:
                continue
            print(
                f"{r['dataset']:<25}{keep_label+'%':>6}"
                f"{full_rmse:>10.4f}"
                f"{r[key]['KDF']['rmse']:>10.4f}"
                f"{r[key]['Random']['rmse']:>10.4f}"
                f"{r[key]['KMeans']['rmse']:>10.4f}"
                f"{r[key]['TopDegree']['rmse']:>10.4f}"
            )


if __name__ == "__main__":
    main()
