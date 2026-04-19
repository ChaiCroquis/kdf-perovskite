#!/usr/bin/env python3
"""
Benchmark: Python vs Rust Backend Performance Comparison

Compares the performance of pure Python and Rust (PyO3) backends
for the KDF-ICH Perovskite Explorer.
"""

import time
import sys
from typing import List, Tuple
import numpy as np

# Add src to path
sys.path.insert(0, ".")

from src.composition import Composition, generate_composition_space
from src.efficiency_model import EfficiencyModel
from src.kdf_explorer import KDFPerovskiteExplorer
from src.hybrid_explorer import HybridKDFExplorer
from src.rust_backend import is_rust_available, get_backend_info


def generate_test_compositions(n: int, seed: int = 42) -> List[Composition]:
    """Generate diverse test compositions."""
    rng = np.random.RandomState(seed)

    a_sites = ["MA", "FA", "Cs", "K", "Rb"]
    b_sites = ["Pb", "Sn", "Ge", "Bi", "Ti"]
    x_sites = ["I", "Br", "Cl"]

    compositions = []
    for i in range(n):
        # Random A-site (1-2 components)
        n_a = rng.choice([1, 2], p=[0.6, 0.4])
        a_ions = rng.choice(a_sites, size=n_a, replace=False)
        a_fracs = rng.dirichlet([1] * n_a)
        a_site = {ion: frac for ion, frac in zip(a_ions, a_fracs)}

        # Random B-site (1-2 components)
        n_b = rng.choice([1, 2], p=[0.7, 0.3])
        b_ions = rng.choice(b_sites, size=n_b, replace=False)
        b_fracs = rng.dirichlet([1] * n_b)
        b_site = {ion: frac for ion, frac in zip(b_ions, b_fracs)}

        # Random X-site (1-2 components)
        n_x = rng.choice([1, 2], p=[0.5, 0.5])
        x_ions = rng.choice(x_sites, size=n_x, replace=False)
        x_fracs = rng.dirichlet([1] * n_x)
        x_site = {ion: frac for ion, frac in zip(x_ions, x_fracs)}

        comp = Composition(
            id=f"comp_{i:05d}",
            A_site=a_site,
            B_site=b_site,
            X_site=x_site,
        )
        compositions.append(comp)

    return compositions


def benchmark_exploration(
    explorer,
    compositions: List[Composition],
    model: EfficiencyModel,
    n_iterations: int,
    name: str,
) -> Tuple[float, int]:
    """
    Benchmark exploration performance.

    Returns:
        Tuple of (total_time, discoveries_found)
    """
    # Initialize
    start = time.perf_counter()
    explorer.initialize(compositions, efficiency_model=model)
    init_time = time.perf_counter() - start

    # Run exploration
    explore_start = time.perf_counter()
    discoveries = 0

    for i in range(n_iterations):
        candidates = explorer.propose_candidates(5)
        if not candidates:
            break

        for comp_id in candidates:
            comp = explorer.compositions[comp_id]
            result = model.evaluate(comp)
            explorer.update(comp_id, result.efficiency)

            if result.efficiency >= 0.15:
                discoveries += 1

        # Run simulation step periodically
        if i % 10 == 0:
            explorer.simulation_step()

    explore_time = time.perf_counter() - explore_start
    total_time = init_time + explore_time

    return total_time, discoveries


def run_benchmarks():
    """Run all benchmarks."""
    print("=" * 60)
    print("KDF-ICH Perovskite Explorer Benchmark")
    print("=" * 60)
    print()

    # Backend info
    info = get_backend_info()
    print(f"Backend Info: {info}")
    print(f"Rust Available: {is_rust_available()}")
    print()

    # Test configurations
    configs = [
        {"n_comps": 100, "n_iter": 50, "name": "Small (100 compositions)"},
        {"n_comps": 500, "n_iter": 100, "name": "Medium (500 compositions)"},
        {"n_comps": 1000, "n_iter": 200, "name": "Large (1000 compositions)"},
    ]

    results = []

    for config in configs:
        print("-" * 60)
        print(f"Configuration: {config['name']}")
        print("-" * 60)

        # Generate compositions
        compositions = generate_test_compositions(config["n_comps"])
        model = EfficiencyModel(seed=42)

        # Add some hidden gems
        gem_ids = [f"comp_{i:05d}" for i in range(5, 15)]
        for gem_id in gem_ids:
            model.register_hidden_gem(gem_id, 0.25)

        # Benchmark Pure Python
        print("\n[Pure Python Backend]")
        python_explorer = KDFPerovskiteExplorer(seed=42)
        python_time, python_discoveries = benchmark_exploration(
            python_explorer,
            compositions.copy(),
            model,
            config["n_iter"],
            "Python",
        )
        print(f"  Time: {python_time:.3f}s")
        print(f"  Discoveries: {python_discoveries}")
        print(f"  Evaluated: {python_explorer.get_statistics().total_evaluations}")

        # Benchmark Hybrid (Python + Rust if available)
        print("\n[Hybrid Backend]")
        hybrid_explorer = HybridKDFExplorer(seed=42, prefer_rust=True)
        print(f"  Using: {hybrid_explorer.backend}")
        hybrid_time, hybrid_discoveries = benchmark_exploration(
            hybrid_explorer,
            compositions.copy(),
            model,
            config["n_iter"],
            "Hybrid",
        )
        print(f"  Time: {hybrid_time:.3f}s")
        print(f"  Discoveries: {hybrid_discoveries}")
        print(f"  Evaluated: {hybrid_explorer.get_statistics().total_evaluations}")

        # Compare
        if hybrid_explorer.backend == "rust":
            speedup = python_time / hybrid_time if hybrid_time > 0 else 0
            print(f"\n  Speedup: {speedup:.2f}x")
        else:
            print("\n  (Rust not available - using Python fallback)")

        results.append({
            "config": config["name"],
            "python_time": python_time,
            "hybrid_time": hybrid_time,
            "backend": hybrid_explorer.backend,
        })

    # Summary
    print("\n" + "=" * 60)
    print("Summary")
    print("=" * 60)

    for r in results:
        speedup = r["python_time"] / r["hybrid_time"] if r["hybrid_time"] > 0 else 0
        backend_label = "Rust" if r["backend"] == "rust" else "Python"
        print(f"{r['config']:30s} | Python: {r['python_time']:.3f}s | Hybrid ({backend_label}): {r['hybrid_time']:.3f}s | Speedup: {speedup:.2f}x")


if __name__ == "__main__":
    run_benchmarks()
