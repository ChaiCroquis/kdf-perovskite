"""
Convert LoCoMo (Snap Research) dataset to our LongMemEval-compatible schema,
so existing pipeline scripts (phase_route_a_full_500q.py, w3_rerun_kdf_real.py,
phase_w3_real_kdf_turns.rs) work with minimal changes.

Output schema per question (matching LongMemEval oracle entries):
    {
      "question_id": str,
      "question_type": str (locomo_cat_{1-5} name),
      "question": str,
      "answer": str,
      "haystack_session_ids": [str, ...],
      "haystack_sessions": [[{role, content, has_answer, ...}, ...], ...],
      "answer_session_ids": [str, ...]
    }

Each LoCoMo "sample" has 10+ sessions and ~200 Q; we emit one entry per Q
with the SAME haystack (all sessions from that sample) but per-Q
has_answer/answer_session_ids computed from `evidence` (dia_ids like "D1:3").

Sample by default: 250 non-adversarial (cat 1-4) Q, balanced across categories.
Use --all to emit all 1540 non-adversarial.
"""
from __future__ import annotations

import argparse
import ast
import json
import random
import sys
from collections import defaultdict
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")


CATEGORY_NAMES = {
    1: "locomo_factual",
    2: "locomo_temporal",
    3: "locomo_inferential",
    4: "locomo_narrative",
    5: "locomo_adversarial",
}


def parse_evidence(ev) -> list[str]:
    """Evidence is usually stored as a string repr of a list of dia_ids."""
    if isinstance(ev, list):
        return [str(x) for x in ev]
    if isinstance(ev, str):
        try:
            parsed = ast.literal_eval(ev)
            if isinstance(parsed, list):
                return [str(x) for x in parsed]
        except Exception:
            pass
        # Fallback: single dia_id
        return [ev]
    return []


def convert_sample(sample: dict, sample_idx: int) -> tuple[list[dict], list[dict]]:
    """Return (flat_turns_with_dia_index, per-Q dicts for this sample)."""
    conv = sample["conversation"]
    qas = sample["qa"]
    sample_id = sample.get("sample_id", f"locomo_{sample_idx}")

    # Collect sessions in order (session_1, session_2, ...)
    session_keys = sorted(
        (k for k in conv.keys() if k.startswith("session_") and not k.endswith("_date_time")),
        key=lambda k: int(k.split("_")[1]),
    )

    # Build per-session turns and per-dia_id → (session_key, turn_idx_in_session)
    dia_to_session: dict[str, str] = {}
    haystack_sessions: list[list[dict]] = []
    haystack_session_ids: list[str] = []

    for skey in session_keys:
        turns = conv[skey]
        if not isinstance(turns, list):
            continue
        session_id_pub = f"{sample_id}::{skey}"
        haystack_session_ids.append(session_id_pub)
        converted: list[dict] = []
        for t in turns:
            dia_id = t.get("dia_id", "")
            if dia_id:
                dia_to_session[dia_id] = session_id_pub
            converted.append({
                "role": t.get("speaker", "unknown"),
                "content": t.get("text", ""),
                "dia_id": dia_id,
                "has_answer": False,  # set per-Q below
            })
        haystack_sessions.append(converted)

    # For each QA: build a per-Q entry
    q_entries: list[dict] = []
    for qi, qa in enumerate(qas):
        evidence = parse_evidence(qa.get("evidence", []))
        # Determine answer_session_ids from evidence dia_ids
        ans_sessions = list({dia_to_session[d] for d in evidence if d in dia_to_session})

        # Build per-Q haystack with has_answer flags
        per_q_haystack: list[list[dict]] = []
        for sess in haystack_sessions:
            new_sess: list[dict] = []
            for t in sess:
                copy = dict(t)
                copy["has_answer"] = t.get("dia_id", "") in evidence
                new_sess.append(copy)
            per_q_haystack.append(new_sess)

        cat = qa.get("category", 0)
        ans = qa.get("answer", "")
        if not isinstance(ans, str):
            ans = str(ans)
        q_entries.append({
            "question_id": f"{sample_id}::q{qi}",
            "question_type": CATEGORY_NAMES.get(cat, f"locomo_cat_{cat}"),
            "question": qa.get("question", ""),
            "answer": ans,
            "haystack_session_ids": haystack_session_ids,
            "haystack_sessions": per_q_haystack,
            "answer_session_ids": ans_sessions,
            "_locomo_evidence": evidence,
            "_locomo_category": cat,
        })

    return haystack_sessions, q_entries


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", default="demos/D8_llm_memory/data/locomo/locomo10.json")
    parser.add_argument("--output", default="demos/D8_llm_memory/data/locomo/locomo_oracle_sampled.json")
    parser.add_argument("--n", type=int, default=250, help="Sample size (non-adversarial)")
    parser.add_argument("--include-adversarial", action="store_true",
                        help="Also include category 5 adversarial questions")
    parser.add_argument("--all", action="store_true", help="Emit all (ignores --n)")
    parser.add_argument("--seed", type=int, default=42)
    args = parser.parse_args()

    with open(args.input, encoding="utf-8") as f:
        data = json.load(f)
    print(f"Loaded {len(data)} LoCoMo conversations from {args.input}", file=sys.stderr)

    all_q: list[dict] = []
    for idx, sample in enumerate(data):
        _, q_entries = convert_sample(sample, idx)
        all_q.extend(q_entries)
    print(f"Total Q: {len(all_q)}", file=sys.stderr)

    # Filter by category
    if args.include_adversarial:
        pool = all_q
    else:
        pool = [q for q in all_q if q["_locomo_category"] != 5]
    print(f"After adversarial filter: {len(pool)}", file=sys.stderr)

    # Count per category
    by_cat = defaultdict(list)
    for q in pool:
        by_cat[q["question_type"]].append(q)
    print(f"Category counts: { {k: len(v) for k, v in by_cat.items()} }", file=sys.stderr)

    # Sample balanced
    rng = random.Random(args.seed)
    if args.all:
        sampled = pool[:]
    else:
        per_cat = max(1, args.n // len(by_cat))
        sampled = []
        for cat, qs in by_cat.items():
            rng.shuffle(qs)
            sampled.extend(qs[:per_cat])
        # If we don't reach target, fill randomly from remainder
        if len(sampled) < args.n:
            remainder = [q for q in pool if q not in sampled]
            rng.shuffle(remainder)
            sampled.extend(remainder[: args.n - len(sampled)])
        sampled = sampled[: args.n]
    print(f"Sampled: {len(sampled)}", file=sys.stderr)
    print(f"Per-category sampled: { {k: sum(1 for q in sampled if q['question_type'] == k) for k in by_cat.keys()} }", file=sys.stderr)

    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    with open(args.output, "w", encoding="utf-8") as f:
        json.dump(sampled, f, indent=None, ensure_ascii=False)
    print(f"Wrote: {args.output}", file=sys.stderr)


if __name__ == "__main__":
    main()
