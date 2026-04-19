"""
B1: Git commit pruning validation on tokio-rs/tokio.

Task: given a commit graph, prune to 30% / 50% keep_rate with 4 methods
(KDF, Random, TTL_recent, TopDegree), and measure recall of "important"
commits defined as:
  - Tagged commits (release points, maintainer-curated importance)
  - Merge commits referencing PR (feature integration points)

Dataset: tokio-rs/tokio (4467 commits, 383 tags, 183 merges), cloned bare
with --filter=blob:none at /tmp/b1_repos/tokio.git

Graph structure (parent-child):
  - node = commit SHA
  - edge (u,v) = u is parent of v

Cost: $0 (no LLM, no API). Runtime: seconds.
"""
from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

import numpy as np

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

import os, argparse as _argparse
# /tmp in Git Bash maps to TEMP on Windows; resolve properly
_tmp_base = os.environ.get("TEMP", os.environ.get("TMP", "/tmp"))
_ap = _argparse.ArgumentParser(add_help=False)
_ap.add_argument("--repo", default="tokio.git")
_ap.add_argument("--label", default=None)
_args, _rest = _ap.parse_known_args()
REPO_DIR = str(Path(_tmp_base) / "b1_repos" / _args.repo)
REPO_LABEL = _args.label or _args.repo.replace(".git", "")


def git(*args: str) -> str:
    """Run git command in the bare repo."""
    result = subprocess.run(
        ["git", f"--git-dir={REPO_DIR}"] + list(args),
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        raise RuntimeError(f"git {args} failed: {result.stderr[:500]}")
    return result.stdout


def build_commit_graph() -> dict:
    """Extract commits, parent edges, tags, and subjects."""
    # All commits from the main branch descendants
    # Format: hash, parents (space-separated), subject
    log_out = git("log", "--all", "--format=%H\x01%P\x01%s")
    commits = []
    commit_set = set()
    subjects = {}
    parents_map = {}
    for line in log_out.strip().splitlines():
        parts = line.split("\x01")
        if len(parts) < 3:
            continue
        h, ps, s = parts[0], parts[1], parts[2]
        commits.append(h)
        commit_set.add(h)
        subjects[h] = s
        parents_map[h] = ps.split() if ps else []

    # Edges: parent → child
    edges = []
    for c, ps in parents_map.items():
        for p in ps:
            if p in commit_set:
                edges.append((p, c))

    # Tags
    tag_out = git("tag", "-l")
    tag_commits = set()
    for tag in tag_out.strip().splitlines():
        if not tag:
            continue
        try:
            sha = git("rev-list", "-n", "1", tag).strip()
            if sha in commit_set:
                tag_commits.add(sha)
        except RuntimeError:
            continue

    # Merge commits: hash with 2+ parents
    merge_commits = {c for c, ps in parents_map.items() if len(ps) >= 2}

    # PR-merge commits (contain "Merge pull request" or "(#NNN)" in subject)
    pr_merge_commits = set()
    for c, s in subjects.items():
        if "Merge pull request" in s or "Merge PR" in s:
            pr_merge_commits.add(c)
        elif re.search(r"\(#\d+\)", s):
            pr_merge_commits.add(c)

    return {
        "commits": commits,
        "commit_set": commit_set,
        "subjects": subjects,
        "parents": parents_map,
        "edges": edges,
        "tag_commits": tag_commits,
        "merge_commits": merge_commits,
        "pr_merge_commits": pr_merge_commits,
    }


def kdf_select_commits(graph: dict, keep_rate: float, tmp_dir: Path) -> set[str]:
    """Call kdf_select_generic on the commit graph."""
    commits = graph["commits"]
    commit_to_idx = {c: i for i, c in enumerate(commits)}
    edges_u = [(commit_to_idx[u], commit_to_idx[v], 1.0) for u, v in graph["edges"]]

    graph_input = {
        "n": len(commits),
        "edges": edges_u,
        "node_ids": commits,
    }
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
    return set(result.get("selected_node_ids", []))


def random_select_commits(graph: dict, keep_rate: float, seed: int = 42) -> set[str]:
    rng = np.random.RandomState(seed)
    n = len(graph["commits"])
    k = max(1, int(n * keep_rate))
    idxs = rng.choice(n, size=k, replace=False)
    return {graph["commits"][i] for i in idxs}


def ttl_recent_select_commits(graph: dict, keep_rate: float) -> set[str]:
    """Keep the last k commits (ordered by git log --all order = rev-list order, newest first)."""
    n = len(graph["commits"])
    k = max(1, int(n * keep_rate))
    # git log --all output is newest first by default (topological/date)
    return set(graph["commits"][:k])


def top_degree_select_commits(graph: dict, keep_rate: float) -> set[str]:
    """Keep top-degree nodes (parents count + children count)."""
    commits = graph["commits"]
    degree = {c: 0 for c in commits}
    # parents = incoming, children = outgoing (both count)
    for u, v in graph["edges"]:
        degree[u] = degree.get(u, 0) + 1
        degree[v] = degree.get(v, 0) + 1
    n = len(commits)
    k = max(1, int(n * keep_rate))
    sorted_commits = sorted(commits, key=lambda c: -degree[c])
    return set(sorted_commits[:k])


def evaluate(selected: set[str], graph: dict) -> dict:
    n = len(graph["commits"])
    tag_recall = (
        len(selected & graph["tag_commits"]) / max(len(graph["tag_commits"]), 1)
    )
    merge_recall = (
        len(selected & graph["merge_commits"]) / max(len(graph["merge_commits"]), 1)
    )
    pr_recall = (
        len(selected & graph["pr_merge_commits"]) / max(len(graph["pr_merge_commits"]), 1)
    )
    # Combined "important" = tag OR merge OR pr-merge
    important = graph["tag_commits"] | graph["merge_commits"] | graph["pr_merge_commits"]
    imp_recall = len(selected & important) / max(len(important), 1)
    return {
        "n_selected": len(selected),
        "keep_rate_actual": len(selected) / n,
        "tag_recall": tag_recall,
        "merge_recall": merge_recall,
        "pr_merge_recall": pr_recall,
        "combined_importance_recall": imp_recall,
    }


def main():
    print(f"Loading tokio commit graph from {REPO_DIR} ...")
    graph = build_commit_graph()
    n = len(graph["commits"])
    print(f"  commits: {n}")
    print(f"  edges (parent-child): {len(graph['edges'])}")
    print(f"  tag commits: {len(graph['tag_commits'])}")
    print(f"  merge commits (2+ parents): {len(graph['merge_commits'])}")
    print(f"  PR-merge commits: {len(graph['pr_merge_commits'])}")
    total_important = len(graph["tag_commits"] | graph["merge_commits"] | graph["pr_merge_commits"])
    print(f"  combined important: {total_important} ({total_important / n * 100:.2f}%)")

    tmp_dir = Path(f"benchmarks/classical_revival/tmp/b1_{REPO_LABEL}")

    all_results = {}
    for keep_rate in [0.30, 0.50]:
        print(f"\n=== keep_rate = {keep_rate:.2f} (expected {int(keep_rate*100)}%) ===")
        methods = {
            "KDF": kdf_select_commits(graph, keep_rate, tmp_dir / f"{int(keep_rate*100)}"),
            "Random": random_select_commits(graph, keep_rate, seed=42),
            "TTL_recent": ttl_recent_select_commits(graph, keep_rate),
            "TopDegree": top_degree_select_commits(graph, keep_rate),
        }
        print(f"  {'method':<14}{'n':>7}{'tag%':>8}{'merge%':>10}{'pr%':>8}{'combined':>12}")
        keep_results = {}
        for name, sel in methods.items():
            r = evaluate(sel, graph)
            keep_results[name] = r
            print(
                f"  {name:<14}{r['n_selected']:>7}"
                f"{r['tag_recall']*100:>7.2f}%"
                f"{r['merge_recall']*100:>9.2f}%"
                f"{r['pr_merge_recall']*100:>7.2f}%"
                f"{r['combined_importance_recall']*100:>11.2f}%"
            )
        all_results[f"keep_{int(keep_rate*100)}"] = keep_results

    # Save
    out = Path(f"benchmarks/classical_revival/out/b1_{REPO_LABEL}_results.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8") as f:
        json.dump(
            {
                "repo": REPO_LABEL,
                "n_commits": n,
                "n_edges": len(graph["edges"]),
                "n_tag_commits": len(graph["tag_commits"]),
                "n_merge_commits": len(graph["merge_commits"]),
                "n_pr_merge_commits": len(graph["pr_merge_commits"]),
                "n_combined_important": total_important,
                "results_by_keep_rate": all_results,
            },
            f,
            indent=2,
        )
    print(f"\nSaved: {out}")


if __name__ == "__main__":
    main()
