"""
W2: Error analysis on F-044 500Q results.

For each of 4 outcome groups (both-correct, KDF-only, Mem0-only, both-wrong),
analyze:
  1. Question type distribution
  2. Question & answer length statistics
  3. Ground-truth answer "type" (date/entity/list/explanation)
  4. Mem0 retrieval recall distribution
  5. Token usage patterns
  6. Representative examples (q_id, question, ground_truth, both answers)

Goal: identify systematic patterns in where KDF wins vs loses vs Mem0, to:
  a. Narrow the paper's honest claims
  b. Inform W10 (hybrid) design
  c. Inform W11 (adversarial) generation
  d. Surface failure modes worth reporting in discussion

Outputs:
  - Prints summary per group
  - Writes demos/D8_llm_memory/out/w2_error_analysis.json
  - Writes demos/D8_llm_memory/out/w2_error_examples.md (human-readable)

Cost: $0. Runtime: seconds.
"""
from __future__ import annotations

import json
import re
import statistics
import sys
from collections import Counter, defaultdict
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

RESULTS_PATH = Path("demos/D8_llm_memory/out/route_a_500q_results.json")
ORACLE_PATH = Path("demos/D8_llm_memory/data/longmemeval_oracle.json")
OUT_JSON = Path("demos/D8_llm_memory/out/w2_error_analysis.json")
OUT_MD = Path("demos/D8_llm_memory/out/w2_error_examples.md")


def classify_answer_type(gt) -> str:
    """Heuristic classification of ground-truth answer by format."""
    if gt is None:
        return "empty"
    if isinstance(gt, (int, float)):
        return "number"
    if not isinstance(gt, str):
        gt = str(gt)
    gt = gt.strip()
    if not gt:
        return "empty"
    # Date/time
    if re.search(r"\b(19|20)\d{2}\b", gt) or re.search(
        r"\b(January|February|March|April|May|June|July|August|September|October|November|December)\b",
        gt, re.IGNORECASE):
        return "date"
    # Numeric quantity
    if re.match(r"^\s*\d+(\.\d+)?\s*(%|kg|lb|mi|km|mph|dollars?|hours?|days?|weeks?|months?|years?|minutes?)?\s*$",
                gt, re.IGNORECASE):
        return "number"
    # List (comma or newline separated short items)
    if gt.count(",") >= 2 and len(gt) < 200:
        return "list"
    # Short (entity / yes-no / single fact)
    if len(gt) < 80:
        return "short_fact"
    # Long explanation
    return "long_explanation"


def summarize_group(group: list[dict], label: str) -> dict:
    """Compute aggregate stats for one outcome group."""
    if not group:
        return {"label": label, "n": 0}

    n = len(group)
    qtypes = Counter(r["question_type"] for r in group)
    ans_types = Counter(r["_answer_type"] for r in group)

    def _stats(xs: list[float]) -> dict:
        xs = [x for x in xs if x is not None]
        if not xs:
            return {"n": 0}
        return {
            "n": len(xs),
            "mean": round(statistics.mean(xs), 2),
            "median": round(statistics.median(xs), 2),
            "stdev": round(statistics.stdev(xs), 2) if len(xs) > 1 else 0.0,
            "min": round(min(xs), 2),
            "max": round(max(xs), 2),
        }

    q_lens = [len(str(r.get("_question", ""))) for r in group]
    gt_lens = [len(str(r.get("_ground_truth", ""))) for r in group]
    kdf_lens = [len(r.get("kdf_answer", "") or "") for r in group]
    mem0_lens = [len(r.get("mem0_answer", "") or "") for r in group]
    tokens = [r.get("tokens_in", 0) for r in group]
    mem0_recall = [r.get("mem0_retrieval_recall", 0.0) for r in group]
    n_haystack = [len(r.get("_haystack_sessions", [])) for r in group]

    return {
        "label": label,
        "n": n,
        "question_types": dict(qtypes.most_common()),
        "answer_format_types": dict(ans_types.most_common()),
        "question_length_chars": _stats(q_lens),
        "ground_truth_length_chars": _stats(gt_lens),
        "kdf_answer_length_chars": _stats(kdf_lens),
        "mem0_answer_length_chars": _stats(mem0_lens),
        "tokens_in": _stats(tokens),
        "mem0_retrieval_recall": _stats(mem0_recall),
        "n_haystack_sessions": _stats(n_haystack),
    }


def pick_examples(group: list[dict], n: int = 5) -> list[dict]:
    """Pick diverse examples from a group."""
    # Sort by question_type to get variety; take first from each
    by_type = defaultdict(list)
    for r in group:
        by_type[r["question_type"]].append(r)
    examples = []
    # Round-robin across types
    types = sorted(by_type.keys())
    idx = 0
    while len(examples) < n and any(by_type[t] for t in types):
        t = types[idx % len(types)]
        if by_type[t]:
            examples.append(by_type[t].pop(0))
        idx += 1
        if idx > 500:
            break
    return examples[:n]


def truncate(s, n: int = 300) -> str:
    if s is None:
        return ""
    s = str(s)
    if not s:
        return ""
    s = s.replace("\n", " ").replace("\r", " ")
    return s if len(s) <= n else s[:n] + "..."


def main() -> None:
    # Load
    with RESULTS_PATH.open("r", encoding="utf-8") as f:
        results_data = json.load(f)
    with ORACLE_PATH.open("r", encoding="utf-8") as f:
        oracle = json.load(f)

    oracle_by_id = {q["question_id"]: q for q in oracle}
    results = results_data["results"]

    # Enrich each result with oracle data
    for r in results:
        qid = r["question_id"]
        orc = oracle_by_id.get(qid, {})
        r["_question"] = orc.get("question", "")
        r["_ground_truth"] = orc.get("answer", "")
        r["_haystack_sessions"] = orc.get("haystack_sessions", [])
        r["_answer_session_ids"] = orc.get("answer_session_ids", [])
        r["_answer_type"] = classify_answer_type(r["_ground_truth"])

    # Group by outcome
    both_correct = [r for r in results if r["kdf_correct"] and r["mem0_correct"]]
    kdf_only = [r for r in results if r["kdf_correct"] and not r["mem0_correct"]]
    mem0_only = [r for r in results if not r["kdf_correct"] and r["mem0_correct"]]
    both_wrong = [r for r in results if not r["kdf_correct"] and not r["mem0_correct"]]

    groups = [
        ("both_correct", both_correct),
        ("kdf_only", kdf_only),
        ("mem0_only", mem0_only),
        ("both_wrong", both_wrong),
    ]

    # Print summary
    print("=" * 72)
    print("W2: Error analysis on F-044 500Q paired outcomes")
    print("=" * 72)
    print()
    summaries = {}
    for label, g in groups:
        s = summarize_group(g, label)
        summaries[label] = s
        print(f"--- {label}: n={s['n']} ---")
        print(f"  question_types: {s['question_types']}")
        print(f"  answer_format : {s['answer_format_types']}")
        print(f"  Q len         : median={s['question_length_chars']['median']:.0f}, max={s['question_length_chars']['max']:.0f}")
        print(f"  GT len        : median={s['ground_truth_length_chars']['median']:.0f}, max={s['ground_truth_length_chars']['max']:.0f}")
        print(f"  KDF ans len   : median={s['kdf_answer_length_chars']['median']:.0f}, mean={s['kdf_answer_length_chars']['mean']:.0f}")
        print(f"  Mem0 ans len  : median={s['mem0_answer_length_chars']['median']:.0f}, mean={s['mem0_answer_length_chars']['mean']:.0f}")
        print(f"  Mem0 recall   : mean={s['mem0_retrieval_recall']['mean']:.3f}, median={s['mem0_retrieval_recall']['median']:.3f}")
        print(f"  tokens_in     : mean={s['tokens_in']['mean']:.0f}, max={s['tokens_in']['max']:.0f}")
        print(f"  haystack n    : mean={s['n_haystack_sessions']['mean']:.1f}, max={s['n_haystack_sessions']['max']:.0f}")
        print()

    # Per-category within each group
    print("=" * 72)
    print("Per-category breakdown within each outcome group")
    print("=" * 72)
    categories = ["temporal-reasoning", "multi-session", "knowledge-update",
                  "single-session-user", "single-session-assistant",
                  "single-session-preference"]
    print(f"{'category':<30}{'both_c':>8}{'kdf_only':>10}{'mem0_only':>11}{'both_w':>8}")
    per_cat = {}
    for cat in categories:
        row = {
            "both_correct": sum(1 for r in both_correct if r["question_type"] == cat),
            "kdf_only": sum(1 for r in kdf_only if r["question_type"] == cat),
            "mem0_only": sum(1 for r in mem0_only if r["question_type"] == cat),
            "both_wrong": sum(1 for r in both_wrong if r["question_type"] == cat),
        }
        per_cat[cat] = row
        print(f"{cat:<30}{row['both_correct']:>8}{row['kdf_only']:>10}{row['mem0_only']:>11}{row['both_wrong']:>8}")
    print()

    # Answer-format breakdown within each group
    print("=" * 72)
    print("Per-answer-format breakdown (answer type vs outcome)")
    print("=" * 72)
    ans_types = ["short_fact", "long_explanation", "date", "number", "list", "empty"]
    print(f"{'answer_type':<22}{'both_c':>8}{'kdf_only':>10}{'mem0_only':>11}{'both_w':>8}")
    per_ans = {}
    for at in ans_types:
        row = {
            "both_correct": sum(1 for r in both_correct if r["_answer_type"] == at),
            "kdf_only": sum(1 for r in kdf_only if r["_answer_type"] == at),
            "mem0_only": sum(1 for r in mem0_only if r["_answer_type"] == at),
            "both_wrong": sum(1 for r in both_wrong if r["_answer_type"] == at),
        }
        per_ans[at] = row
        print(f"{at:<22}{row['both_correct']:>8}{row['kdf_only']:>10}{row['mem0_only']:>11}{row['both_wrong']:>8}")
    print()

    # Save JSON summary
    output = {
        "source": str(RESULTS_PATH),
        "group_summaries": summaries,
        "per_category_outcome": per_cat,
        "per_answer_type_outcome": per_ans,
    }
    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    with OUT_JSON.open("w", encoding="utf-8") as f:
        json.dump(output, f, indent=2, ensure_ascii=False)
    print(f"Saved JSON summary: {OUT_JSON}")

    # Save human-readable examples
    with OUT_MD.open("w", encoding="utf-8") as f:
        f.write("# W2 Error Analysis — Representative Examples\n\n")
        f.write(f"Source: `{RESULTS_PATH}` (F-044 500Q paired outcomes)\n\n")
        for label, g in [("kdf_only", kdf_only), ("mem0_only", mem0_only), ("both_wrong", both_wrong)]:
            f.write(f"## {label} (n={len(g)})\n\n")
            for ex in pick_examples(g, n=6):
                f.write(f"### {ex['question_id']} ({ex['question_type']}, answer_type={ex['_answer_type']})\n\n")
                f.write(f"- **Question**: {truncate(ex['_question'], 400)}\n")
                f.write(f"- **Ground truth**: `{truncate(ex['_ground_truth'], 400)}`\n")
                f.write(f"- **KDF answer** ({'✅' if ex['kdf_correct'] else '❌'}): {truncate(ex.get('kdf_answer') or '', 400)}\n")
                f.write(f"- **Mem0 answer** ({'✅' if ex['mem0_correct'] else '❌'}): {truncate(ex.get('mem0_answer') or '', 400)}\n")
                f.write(f"- Mem0 retrieval recall: {ex.get('mem0_retrieval_recall', 0):.3f}, haystack sessions: {len(ex['_haystack_sessions'])}\n\n")
    print(f"Saved examples: {OUT_MD}")


if __name__ == "__main__":
    main()
