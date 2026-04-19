"""
B2: Call Graph Curation validation on Flask (Python).

Task: extract the call graph from a Python codebase, prune to 30%/50%
keep_rate, and measure recall of "public API" functions (those exported
in __init__.py). Graph node = function/method; edge = "A calls B".

Dataset: pallets/flask (18 Python files, medium-sized web framework).

Ground truth "API":
  - Level 1: functions imported in src/flask/__init__.py (root public API)
  - Level 2: all public (non-_) top-level defs in public modules

Methods compared:
  - KDF (via kdf_select_generic Rust binary)
  - Random
  - TopDegree (degree-based pruning)
  - TopIncoming (in-degree = "many callers", simple API heuristic)

Cost: $0. Runtime: seconds to minutes.
"""
from __future__ import annotations

import ast
import json
import os
import subprocess
import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


_tmp_base = os.environ.get("TEMP", os.environ.get("TMP", "/tmp"))
REPO_SRC = Path(_tmp_base) / "b1_repos" / "flask" / "src" / "flask"


def fqn(module: str, cls: str | None, name: str) -> str:
    """Fully-qualified name."""
    if cls:
        return f"{module}::{cls}.{name}"
    return f"{module}::{name}"


def parse_file(py_path: Path, module_name: str):
    """Return (definitions, calls_by_function) for a Python file.

    definitions: dict of {fqn: metadata_dict}
    calls_by_function: dict of {caller_fqn: list of simple call names}
    """
    try:
        with py_path.open(encoding="utf-8") as f:
            tree = ast.parse(f.read(), filename=str(py_path))
    except Exception as e:
        print(f"[skip] {py_path}: parse error {e}")
        return {}, {}

    definitions = {}
    calls_by_func = {}

    class Visitor(ast.NodeVisitor):
        def __init__(self):
            self.cls_stack = []
            self.func_stack = []  # stack of fqn

        def visit_ClassDef(self, node):
            self.cls_stack.append(node.name)
            self.generic_visit(node)
            self.cls_stack.pop()

        def _register(self, node, kind: str):
            cls = ".".join(self.cls_stack) if self.cls_stack else None
            name = node.name
            q = fqn(module_name, cls, name)
            definitions[q] = {
                "module": module_name,
                "class": cls,
                "name": name,
                "is_public": not name.startswith("_"),
                "is_method": cls is not None,
                "kind": kind,
                "line": node.lineno,
            }
            self.func_stack.append(q)
            calls_by_func[q] = []
            self.generic_visit(node)
            self.func_stack.pop()

        def visit_FunctionDef(self, node):
            self._register(node, "function")

        def visit_AsyncFunctionDef(self, node):
            self._register(node, "async_function")

        def visit_Call(self, node):
            # Record the call simple name(s) — resolve later
            if not self.func_stack:
                # module-level call, skip
                self.generic_visit(node)
                return
            caller = self.func_stack[-1]
            target_name = None
            if isinstance(node.func, ast.Name):
                target_name = node.func.id
            elif isinstance(node.func, ast.Attribute):
                # a.b.c() → use rightmost attribute
                target_name = node.func.attr
            if target_name:
                calls_by_func[caller].append(target_name)
            self.generic_visit(node)

    Visitor().visit(tree)
    return definitions, calls_by_func


def build_call_graph(src_root: Path):
    all_defs = {}  # fqn → meta
    all_calls = {}  # fqn → list of called simple names
    py_files = sorted(src_root.rglob("*.py"))
    print(f"Parsing {len(py_files)} Python files...")
    for pf in py_files:
        rel = pf.relative_to(src_root)
        module_parts = list(rel.with_suffix("").parts)
        if module_parts[-1] == "__init__":
            module_parts = module_parts[:-1]
        module_name = ".".join(module_parts) or "__root__"
        defs, calls = parse_file(pf, module_name)
        all_defs.update(defs)
        all_calls.update(calls)

    # Build reverse name index for resolution
    name_to_fqns = defaultdict(list)
    for q, meta in all_defs.items():
        name_to_fqns[meta["name"]].append(q)

    # Resolve calls by simple name → possible fqns (may be 1 or multiple)
    edges_set = set()
    for caller, targets in all_calls.items():
        for t in targets:
            candidates = name_to_fqns.get(t, [])
            for cand in candidates:
                if cand != caller:
                    edges_set.add((caller, cand))

    print(f"  defs: {len(all_defs)}, edges: {len(edges_set)}")
    return all_defs, list(edges_set)


def get_api_ground_truth(src_root: Path, definitions: dict) -> dict:
    """Extract API ground truth from __init__.py exports."""
    init_path = src_root / "__init__.py"
    level1 = set()
    if init_path.exists():
        with init_path.open(encoding="utf-8") as f:
            tree = ast.parse(f.read())
        for node in ast.walk(tree):
            if isinstance(node, ast.ImportFrom):
                for alias in node.names:
                    imported = alias.asname or alias.name
                    level1.add(imported)

    # Match imports to fqn definitions by name
    level1_fqns = set()
    name_to_fqns = defaultdict(list)
    for q, meta in definitions.items():
        name_to_fqns[meta["name"]].append(q)
    for imported in level1:
        for q in name_to_fqns.get(imported, []):
            level1_fqns.add(q)

    # Level 2 = all public (non-_) top-level defs in public modules
    level2_fqns = set()
    for q, meta in definitions.items():
        if meta["is_public"] and not meta["is_method"]:
            mod = meta["module"]
            if not any(part.startswith("_") for part in mod.split(".")):
                level2_fqns.add(q)

    return {
        "level1_api_names": sorted(level1),
        "level1_api_fqns": level1_fqns,
        "level2_public_fqns": level2_fqns,
    }


def kdf_select_fqns(fqns_ordered, edges, keep_rate, tmp_dir: Path) -> set[str]:
    fqn_to_idx = {q: i for i, q in enumerate(fqns_ordered)}
    edges_u = [(fqn_to_idx[u], fqn_to_idx[v], 1.0) for u, v in edges]
    graph_input = {
        "n": len(fqns_ordered),
        "edges": edges_u,
        "node_ids": fqns_ordered,
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
    return set(result["selected_node_ids"])


def random_select_fqns(fqns, keep_rate, seed=42):
    rng = np.random.RandomState(seed)
    n = len(fqns)
    k = max(1, int(n * keep_rate))
    idxs = rng.choice(n, size=k, replace=False)
    return {fqns[i] for i in idxs}


def top_degree_select_fqns(fqns, edges, keep_rate):
    degree = defaultdict(int)
    for u, v in edges:
        degree[u] += 1
        degree[v] += 1
    n = len(fqns)
    k = max(1, int(n * keep_rate))
    sorted_fqns = sorted(fqns, key=lambda q: -degree[q])
    return set(sorted_fqns[:k])


def top_incoming_select_fqns(fqns, edges, keep_rate):
    """Select by in-degree (many callers). Good heuristic for "used" functions."""
    in_degree = defaultdict(int)
    for u, v in edges:
        in_degree[v] += 1
    n = len(fqns)
    k = max(1, int(n * keep_rate))
    sorted_fqns = sorted(fqns, key=lambda q: -in_degree[q])
    return set(sorted_fqns[:k])


def evaluate(selected: set[str], gt: dict, all_defs: dict) -> dict:
    l1 = gt["level1_api_fqns"]
    l2 = gt["level2_public_fqns"]
    n = len(all_defs)
    return {
        "n_selected": len(selected),
        "keep_rate_actual": len(selected) / n,
        "level1_api_recall": len(selected & l1) / max(len(l1), 1),
        "level2_public_recall": len(selected & l2) / max(len(l2), 1),
        "level1_count": len(l1),
        "level2_count": len(l2),
    }


def main():
    print(f"REPO_SRC = {REPO_SRC}")
    assert REPO_SRC.exists(), f"Missing repo source: {REPO_SRC}"

    defs, edges = build_call_graph(REPO_SRC)
    gt = get_api_ground_truth(REPO_SRC, defs)
    fqns = sorted(defs.keys())

    print(f"\nGround truth:")
    print(f"  Level 1 API (names in __init__.py): {len(gt['level1_api_names'])} names → {len(gt['level1_api_fqns'])} resolved fqns")
    print(f"  Level 2 public (non-_ top-level in public module): {len(gt['level2_public_fqns'])}")
    print(f"  Total defs in graph: {len(fqns)}")

    tmp_dir = Path("benchmarks/classical_revival/tmp/b2_flask")
    results_all = {}

    for keep_rate in [0.30, 0.50]:
        print(f"\n=== keep_rate = {keep_rate:.2f} ===")
        methods = {
            "KDF": kdf_select_fqns(fqns, edges, keep_rate, tmp_dir / f"{int(keep_rate*100)}"),
            "Random": random_select_fqns(fqns, keep_rate, seed=42),
            "TopDegree": top_degree_select_fqns(fqns, edges, keep_rate),
            "TopIncoming": top_incoming_select_fqns(fqns, edges, keep_rate),
        }
        keep_key = f"keep_{int(keep_rate*100)}pct"
        keep_results = {}
        print(f"  {'method':<15}{'n_sel':>8}{'L1 recall':>14}{'L2 recall':>14}")
        for name, sel in methods.items():
            r = evaluate(sel, gt, defs)
            keep_results[name] = r
            print(
                f"  {name:<15}{r['n_selected']:>8}"
                f"{r['level1_api_recall']*100:>12.2f}%"
                f"{r['level2_public_recall']*100:>12.2f}%"
            )
        results_all[keep_key] = keep_results

    out = Path("benchmarks/classical_revival/out/b2_flask_results.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "repo": "pallets/flask",
            "n_defs": len(defs),
            "n_edges": len(edges),
            "ground_truth": {
                "level1_api_names": gt["level1_api_names"],
                "level1_count": len(gt["level1_api_fqns"]),
                "level2_count": len(gt["level2_public_fqns"]),
            },
            "results": results_all,
        }, f, indent=2)
    print(f"\nSaved: {out}")


if __name__ == "__main__":
    main()
