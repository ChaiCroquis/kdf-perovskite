"""
Ext-1: Precision-Query Router MVP

Implements the "complementary (not replacement)" architecture proposed in
docs/design_philosophy.md and docs/extension_ideas.md.

Hypothesis: By routing precision queries (dates, numbers, exact quotes,
lists) to KDF's deterministic raw-turn retrieval, and routing generic
queries to Mem0's LLM-based fact extraction, we can achieve strictly
better accuracy than Mem0 alone, at near-zero additional cost.

Evaluation: Uses existing results (F-053, F-057, F-058, F-059) — no
additional API calls needed. Joins results with oracle to get question
text, applies regex-based precision classifier, and computes routed
accuracy for each of the 4 matrix cells.

Also reports:
  - Classification stats (how many routed to KDF vs Mem0)
  - vs Mem0 alone, vs KDF alone
  - McNemar significance
  - Per-category breakdown

Cost: $0. Runtime: seconds.

Usage:
    python demos/D8_llm_memory/scripts/ext1_precision_router.py
"""
from __future__ import annotations

import json
import math
import re
import sys
from pathlib import Path
from collections import defaultdict

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


# ---------------------------------------------------------------------
# Precision query classifier (regex-based, LLM-free)
# ---------------------------------------------------------------------
PRECISION_PATTERNS = [
    # Temporal markers
    (r"\bwhen\b", "temporal:when"),
    (r"\bwhat (date|day|time|year|month)\b", "temporal:what_X"),
    (r"\b(before|after) (the|my|our|when|what)\b", "temporal:before_after"),
    (r"\bhow long\b", "temporal:duration"),
    (r"\bhow old\b", "temporal:age"),
    (r"\b(19|20)\d{2}\b", "temporal:year"),
    (r"\b(january|february|march|april|may|june|july|august|september|october|november|december)\b", "temporal:month"),
    (r"\b(yesterday|today|tomorrow|last (week|month|year)|next (week|month|year))\b", "temporal:relative"),
    (r"\bmost recent\b", "temporal:recent"),
    (r"\b(first|last|latest|earliest)\b", "temporal:ordinal"),

    # Numeric / quantity
    (r"\bhow (many|much)\b", "numeric:how_many_much"),
    (r"\bwhat (amount|number|size|quantity)\b", "numeric:what_amount"),
    (r"\b\d+\b", "numeric:any_number"),
    (r"\$\d", "numeric:dollar_amount"),

    # Exact quote / verbatim
    (r"\bexact(ly)?\b", "exact:keyword"),
    (r"\bspecific(ally)?\b", "exact:specific"),
    (r"\bprecise(ly)?\b", "exact:precise"),
    (r"\bverbatim\b", "exact:verbatim"),
    (r"\b(exact|exact) (quote|words|phrase)\b", "exact:exact_quote"),
    (r"\bwhat did (i|you|the (user|assistant|ai)) say\b", "exact:what_did_say"),

    # List query
    (r"\blist (all|every|each)\b", "list:all"),
    (r"\benumerate\b", "list:enumerate"),
    (r"\bhow many (different|types|kinds)\b", "list:how_many_kinds"),
    (r"\ball (the|of the)\b", "list:all_the"),
]


def classify_precision(question: str) -> tuple[bool, list[str]]:
    """Return (is_precision, matched_patterns)."""
    if not question:
        return False, []
    q_lower = question.lower()
    matched = []
    for pattern, label in PRECISION_PATTERNS:
        if re.search(pattern, q_lower):
            matched.append(label)
    return bool(matched), matched


# ---------------------------------------------------------------------
# McNemar exact binomial
# ---------------------------------------------------------------------
def binom2sided(b: int, c: int) -> float:
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    log_half_n = n * math.log(0.5)
    cdf = sum(
        math.exp(math.lgamma(n + 1) - math.lgamma(i + 1) - math.lgamma(n - i + 1) + log_half_n)
        for i in range(k + 1)
    )
    return min(2.0 * cdf, 1.0)


# ---------------------------------------------------------------------
# Router evaluation
# ---------------------------------------------------------------------
def count_turns(oracle_q: dict) -> int:
    """Count total turns in haystack_sessions."""
    total = 0
    for sess in oracle_q.get("haystack_sessions", []):
        total += len(sess)
    return total


def route_decision(
    question: str,
    n_turns: int,
    length_threshold: int = 0,
    require_precision: bool = True,
) -> tuple[bool, str]:
    """Decide whether to route to KDF.

    Args:
        question: the user question
        n_turns: total conversation turns in this question's haystack
        length_threshold: if > 0, only route to KDF when n_turns >= threshold
        require_precision: if True, require precision-query match to route to KDF

    Returns:
        (route_to_kdf: bool, reason: str)
    """
    is_prec, matched = classify_precision(question)
    if length_threshold > 0 and n_turns < length_threshold:
        return False, f"short_context(n_turns={n_turns}<{length_threshold})"
    if require_precision and not is_prec:
        return False, "not_precision_query"
    return True, "precision_and_long_context" if length_threshold > 0 else "precision_query"


def evaluate_routing(
    results_path: str,
    oracle_path: str,
    mem0_correct_key: str = "mem0_correct",
    kdf_correct_key: str = "kdf_correct",
    mem0_answer_key: str = "mem0_answer",
    kdf_answer_key: str = "kdf_answer",
    label: str = "",
    length_threshold: int = 0,
) -> dict:
    """Apply router to an existing results file, compute comparative metrics."""
    with open(results_path, encoding="utf-8") as f:
        data = json.load(f)
    rows = data.get("results", data if isinstance(data, list) else [])

    with open(oracle_path, encoding="utf-8") as f:
        oracle = json.load(f)
    oracle_by_id = {q["question_id"]: q for q in oracle}

    n = 0
    n_routed_kdf = 0
    # Outcomes
    router_correct = 0
    mem0_alone_correct = 0
    kdf_alone_correct = 0
    # Paired outcomes (router vs Mem0 alone)
    r_only = 0  # router correct, Mem0 alone wrong
    m_only = 0  # Mem0 alone correct, router wrong
    both_ok = 0
    both_w = 0
    # Per-category
    per_cat: dict = defaultdict(lambda: {"n": 0, "router": 0, "mem0": 0, "kdf": 0, "n_to_kdf": 0})

    for row in rows:
        qid = row["question_id"]
        orc = oracle_by_id.get(qid)
        if orc is None:
            continue
        q = orc.get("question", "")
        n_turns_q = count_turns(orc)
        to_kdf, _reason = route_decision(q, n_turns_q, length_threshold=length_threshold)

        m_corr = bool(row.get(mem0_correct_key, False))
        k_corr = bool(row.get(kdf_correct_key, False))

        # Routing: use KDF result for precision+length, Mem0 result for others
        routed_corr = k_corr if to_kdf else m_corr
        is_prec = to_kdf  # backward-compat for per_cat tracking
        cat = row.get("question_type", "unknown")

        n += 1
        if is_prec:
            n_routed_kdf += 1
        per_cat[cat]["n"] += 1
        if is_prec:
            per_cat[cat]["n_to_kdf"] += 1
        if routed_corr:
            router_correct += 1
            per_cat[cat]["router"] += 1
        if m_corr:
            mem0_alone_correct += 1
            per_cat[cat]["mem0"] += 1
        if k_corr:
            kdf_alone_correct += 1
            per_cat[cat]["kdf"] += 1

        # Paired: router vs mem0-alone
        if routed_corr and m_corr:
            both_ok += 1
        elif routed_corr and not m_corr:
            r_only += 1
        elif not routed_corr and m_corr:
            m_only += 1
        else:
            both_w += 1

    router_acc = router_correct / max(n, 1)
    m_acc = mem0_alone_correct / max(n, 1)
    k_acc = kdf_alone_correct / max(n, 1)
    p = binom2sided(r_only, m_only)

    return {
        "label": label,
        "n": n,
        "n_routed_to_kdf": n_routed_kdf,
        "n_routed_to_mem0": n - n_routed_kdf,
        "pct_to_kdf": n_routed_kdf / max(n, 1),
        "router_accuracy": router_acc,
        "mem0_alone_accuracy": m_acc,
        "kdf_alone_accuracy": k_acc,
        "router_minus_mem0": router_acc - m_acc,
        "contingency_router_vs_mem0": {
            "both_correct": both_ok,
            "router_only": r_only,
            "mem0_only": m_only,
            "both_wrong": both_w,
        },
        "mcnemar_exact_p": p,
        "per_category": dict(per_cat),
    }


def _evaluate_all(configs, length_threshold):
    all_results = []
    for cfg in configs:
        if not Path(cfg["results"]).exists():
            continue
        if not Path(cfg["oracle"]).exists():
            continue
        res = evaluate_routing(
            cfg["results"], cfg["oracle"],
            mem0_correct_key=cfg["mem0_correct_key"],
            kdf_correct_key=cfg["kdf_correct_key"],
            label=cfg["label"],
            length_threshold=length_threshold,
        )
        all_results.append(res)
    return all_results


def _print_summary(all_results):
    print(f"{'cell':<45}{'Mem0 alone':>12}{'Router':>10}{'diff':>9}{'%→KDF':>9}{'p':>12}")
    for res in all_results:
        sig = "★" if res["mcnemar_exact_p"] < 0.05 else "-"
        sign = ""
        if res["mcnemar_exact_p"] < 0.05:
            sign = " (R)" if res["contingency_router_vs_mem0"]["router_only"] > res["contingency_router_vs_mem0"]["mem0_only"] else " (M)"
        print(
            f"{res['label']:<45}{res['mem0_alone_accuracy']:>12.4f}"
            f"{res['router_accuracy']:>10.4f}{res['router_minus_mem0']:>+9.4f}"
            f"{res['pct_to_kdf']*100:>8.1f}%{res['mcnemar_exact_p']:>10.4g}  {sig}{sign}"
        )


def main():
    # 4 matrix cells
    configs = [
        {
            "label": "F-053 LongMemEval × gpt-4o-mini",
            "results": "demos/D8_llm_memory/out/route_a_500q_real_kdf_results.json",
            "oracle": "demos/D8_llm_memory/data/longmemeval_oracle.json",
            "mem0_correct_key": "mem0_correct",
            "kdf_correct_key": "kdf_real_correct",  # different schema
        },
        {
            "label": "F-059 LongMemEval × gpt-4.1-mini",
            "results": "demos/D8_llm_memory/out/w4b_longmemeval_41mini_results.json",
            "oracle": "demos/D8_llm_memory/data/longmemeval_oracle.json",
            "mem0_correct_key": "mem0_correct",
            "kdf_correct_key": "kdf_correct",
        },
        {
            "label": "F-057 LoCoMo temporal × gpt-4o-mini",
            "results": "demos/D8_llm_memory/out/w5b_locomo_temporal_results.json",
            "oracle": "demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json",
            "mem0_correct_key": "mem0_correct",
            "kdf_correct_key": "kdf_correct",
        },
        {
            "label": "F-058 LoCoMo temporal × gpt-4.1-mini",
            "results": "demos/D8_llm_memory/out/w4_locomo_temporal_41mini_results.json",
            "oracle": "demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json",
            "mem0_correct_key": "mem0_correct",
            "kdf_correct_key": "kdf_correct",
        },
    ]

    # Evaluate 3 router variants
    variants = [
        {"name": "v1_precision_only", "length_threshold": 0,
         "desc": "Route to KDF if query is precision (no length filter)"},
        {"name": "v2_precision_and_long_context", "length_threshold": 100,
         "desc": "Route to KDF only if precision AND conversation >= 100 turns"},
        {"name": "v3_long_context_only", "length_threshold": 100,
         "desc": "Route to KDF only if conversation >= 100 turns (precision not required)"},
    ]

    print("=" * 100)
    print("Ext-1 Precision-Query Router MVP — Evaluation on 4 matrix cells × 3 variants")
    print("=" * 100)
    print()

    for variant in variants:
        print(f"\n### Variant: {variant['name']}  —  {variant['desc']}\n")
        all_results = _evaluate_all(configs, variant["length_threshold"])
        _print_summary(all_results)

    # Save v2 (most promising) detailed per-cell breakdown
    print("\n" + "=" * 100)
    print("v2 (precision + long context >= 100 turns) detailed per-cell breakdown")
    print("=" * 100)
    v2_results = _evaluate_all(configs, length_threshold=100)
    for res in v2_results:
        print(f"\n--- {res['label']} ---")
        print(f"  n={res['n']} total, routed to KDF: {res['n_routed_to_kdf']} ({res['pct_to_kdf']*100:.1f}%)")
        print(f"  Mem0 alone:   {res['mem0_alone_accuracy']:.4f}")
        print(f"  KDF alone:    {res['kdf_alone_accuracy']:.4f}")
        print(f"  ★ Router:     {res['router_accuracy']:.4f}  "
              f"(vs Mem0: {res['router_minus_mem0']:+.4f}, p={res['mcnemar_exact_p']:.4g})")
        c = res["contingency_router_vs_mem0"]
        print(f"    Paired: both_ok={c['both_correct']}, router_only={c['router_only']}, "
              f"mem0_only={c['mem0_only']}, both_wrong={c['both_wrong']}")

    # Save JSON
    all_variant_results = {}
    for variant in variants:
        all_variant_results[variant["name"]] = [
            {k: (v if not isinstance(v, dict) else v) for k, v in r.items() if k != "per_category"}
            for r in _evaluate_all(configs, variant["length_threshold"])
        ]
    out = Path("demos/D8_llm_memory/out/ext1_router_results.json")
    with out.open("w", encoding="utf-8") as f:
        json.dump({
            "configs": [{k: v for k, v in c.items() if k != "raw"} for c in configs],
            "variants": variants,
            "results_by_variant": all_variant_results,
        }, f, indent=2, ensure_ascii=False)
    print(f"\nSaved: {out}")


if __name__ == "__main__":
    main()
