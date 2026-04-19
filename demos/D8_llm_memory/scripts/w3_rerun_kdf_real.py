"""
W3 follow-up: Re-run F-044 KDF answer-generation with REAL KDF retrieval.

Background:
  F-044 Python script (`phase_route_a_full_500q.py::kdf_retrieve`) approximated
  KDF by assuming answer-turn recall = 0.821 (the F-033 claim). But W3 ablation
  revealed F-033's 0.821 was first-100-sample specific; real KDF on full 500Q
  at keep_rate=0.30 gives answer_turn_recall = 0.6646. Thus F-044's reported
  KDF accuracy (0.696) over-approximates real KDF performance.

This script:
  1. Loads F-044 existing results (preserves Mem0 answers/verdicts).
  2. Loads real-KDF turn indices dumped by phase_w3_real_kdf_turns.rs.
  3. For each question, regenerates KDF answer using REAL selected turns.
  4. Re-judges the KDF answer with identical prompt.
  5. Writes updated results to out/route_a_500q_real_kdf_results.json.

Mem0 results are NOT re-run (they are independent of KDF retrieval).

Cost: ~$0.10-0.15 (500Q x gpt-4o-mini, answer + judge only).
Runtime: ~10-20 minutes.

Usage (PowerShell):
    $env:OPENAI_API_KEY = "sk-proj-..."
    python demos/D8_llm_memory/scripts/w3_rerun_kdf_real.py \
        --turns demos/D8_llm_memory/out/w3_real_kdf_turns_KDF_030.json
"""
from __future__ import annotations

import argparse
import json
import os
import signal
import sys
import time
from pathlib import Path

try:
    sys.stdout.reconfigure(encoding="utf-8")
    sys.stderr.reconfigure(encoding="utf-8")
except Exception:
    pass

try:
    from dotenv import load_dotenv
    if Path(".env").exists():
        load_dotenv(".env")
        print("Loaded .env", file=sys.stderr)
except ImportError:
    pass


def extract_flattened_turns(q: dict) -> list[dict]:
    """Flatten haystack_sessions in the same order as the Rust binary
    (session 0 then session 1, etc., preserving turn order)."""
    flat = []
    for s in q["haystack_sessions"]:
        for t in s:
            flat.append(t)
    return flat


def generate_answer(client, model, retrieved_context: str, question: str) -> tuple[str, int, int]:
    prompt = (
        f"You are a memory agent. Using only the facts below, answer the question concisely.\n\n"
        f"Facts:\n{retrieved_context}\n\n"
        f"Question: {question}\n\n"
        f"Answer (1-2 sentences, just the fact):"
    )
    resp = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": prompt}],
        temperature=0,
        max_tokens=150,
    )
    return resp.choices[0].message.content.strip(), resp.usage.prompt_tokens, resp.usage.completion_tokens


def judge_answer(client, model, question: str, ground_truth: str, candidate: str) -> tuple[bool, str, int, int]:
    prompt = (
        f"You are a lenient judge comparing a candidate answer to a ground truth.\n\n"
        f"Question: {question}\n"
        f"Ground truth: {ground_truth}\n"
        f"Candidate: {candidate}\n\n"
        f"Rules for YES:\n"
        f"- Candidate contains the key fact(s) from ground truth (even with extra accurate detail)\n"
        f"- Semantically equivalent phrasing counts as YES\n"
        f"- If candidate adds accurate date/year/context not contradicting ground truth, it's YES\n\n"
        f"Rules for NO:\n"
        f"- Candidate says a different entity / wrong answer\n"
        f"- Candidate contradicts the ground truth fact\n"
        f"- Candidate is evasive (e.g., 'I don't know') when ground truth is specific\n\n"
        f"Reply with exactly 'YES' or 'NO' followed by a 1-sentence reason."
    )
    resp = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": prompt}],
        temperature=0,
        max_tokens=80,
    )
    verdict = resp.choices[0].message.content.strip()
    correct = verdict.upper().startswith("YES")
    return correct, verdict, resp.usage.prompt_tokens, resp.usage.completion_tokens


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--turns", required=True, help="Path to real-KDF turn-indices JSON from phase_w3_real_kdf_turns.rs")
    parser.add_argument("--model", default="gpt-4o-mini")
    parser.add_argument("--original-results", default="demos/D8_llm_memory/out/route_a_500q_results.json")
    parser.add_argument("--oracle", default="demos/D8_llm_memory/data/longmemeval_oracle.json")
    parser.add_argument("--out", default="demos/D8_llm_memory/out/route_a_500q_real_kdf_results.json")
    parser.add_argument("--checkpoint", default="demos/D8_llm_memory/out/w3_rerun_checkpoint.json")
    parser.add_argument("--checkpoint-every", type=int, default=25)
    args = parser.parse_args()

    if not os.environ.get("OPENAI_API_KEY"):
        print("ERROR: OPENAI_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    from openai import OpenAI
    client = OpenAI()

    print(f"Loading: {args.turns}", file=sys.stderr)
    with open(args.turns, encoding="utf-8") as f:
        turns_data = json.load(f)
    turn_selections = {r["question_id"]: r for r in turns_data["results"]}
    print(f"  method={turns_data['method']}, keep_rate={turns_data['keep_rate']}, mean_recall={turns_data['mean_answer_turn_recall']:.4f}", file=sys.stderr)

    print(f"Loading: {args.original_results}", file=sys.stderr)
    with open(args.original_results, encoding="utf-8") as f:
        original = json.load(f)
    orig_by_id = {r["question_id"]: r for r in original["results"]}

    print(f"Loading: {args.oracle}", file=sys.stderr)
    with open(args.oracle, encoding="utf-8") as f:
        oracle = json.load(f)
    oracle_by_id = {q["question_id"]: q for q in oracle}

    # Resume state
    ckpt_path = Path(args.checkpoint)
    if ckpt_path.exists():
        with ckpt_path.open(encoding="utf-8") as f:
            state = json.load(f)
        print(f"Resuming from checkpoint: {len(state['completed_q_ids'])} completed", file=sys.stderr)
    else:
        state = {
            "completed_q_ids": [],
            "results": [],
            "total_tokens_in": 0,
            "total_tokens_out": 0,
            "est_cost_usd": 0.0,
            "method": turns_data["method"],
            "keep_rate": turns_data["keep_rate"],
        }

    done_ids = set(state["completed_q_ids"])

    def save_state():
        ckpt_path.parent.mkdir(parents=True, exist_ok=True)
        with ckpt_path.open("w", encoding="utf-8") as f:
            json.dump(state, f, indent=2, ensure_ascii=False)

    def handle_sigint(sig, frame):
        print("\nSIGINT: saving checkpoint and exiting", file=sys.stderr)
        save_state()
        sys.exit(0)
    signal.signal(signal.SIGINT, handle_sigint)

    # Pricing (gpt-4o-mini as of 2026-04)
    price_in = 0.15 / 1e6  # $ per input token
    price_out = 0.60 / 1e6

    t_start = time.time()
    processed_this_run = 0

    for q_idx, q in enumerate(oracle):
        qid = q["question_id"]
        if qid in done_ids:
            continue
        if qid not in turn_selections:
            print(f"[skip] {qid}: no KDF turns entry", file=sys.stderr)
            continue
        orig = orig_by_id.get(qid, {})

        flat_turns = extract_flattened_turns(q)
        sel_idx = turn_selections[qid]["kept_turn_indices"]
        selected = [flat_turns[i] for i in sel_idx if i < len(flat_turns)]
        context = "\n".join(
            f"[{t.get('role', '')}] {t.get('content', '')[:500]}" for t in selected
        )
        ground_truth = q.get("answer", "")
        if not isinstance(ground_truth, str):
            ground_truth = str(ground_truth)

        # Answer generation
        try:
            ans, tin, tout = generate_answer(client, args.model, context, q["question"])
        except Exception as e:
            print(f"[error q={qid}] generate_answer: {e}", file=sys.stderr)
            time.sleep(5)
            continue

        # Judge
        try:
            correct, verdict, j_in, j_out = judge_answer(
                client, args.model, q["question"], ground_truth, ans
            )
        except Exception as e:
            print(f"[error q={qid}] judge: {e}", file=sys.stderr)
            time.sleep(5)
            continue

        total_in = tin + j_in
        total_out = tout + j_out
        state["total_tokens_in"] += total_in
        state["total_tokens_out"] += total_out
        state["est_cost_usd"] += total_in * price_in + total_out * price_out

        result_row = {
            "q_idx": q_idx,
            "question_id": qid,
            "question_type": q.get("question_type", ""),
            "n_total_turns": len(flat_turns),
            "n_selected_turns": len(sel_idx),
            "n_answer_turns": turn_selections[qid]["n_answer_turns"],
            "real_kdf_answer_turn_recall": turn_selections[qid]["answer_turn_recall"],
            # Preserve original Mem0 answer/correct for reference
            "mem0_answer": orig.get("mem0_answer"),
            "mem0_correct": orig.get("mem0_correct"),
            # Original simulated KDF (for comparison)
            "kdf_sim_answer": orig.get("kdf_answer"),
            "kdf_sim_correct": orig.get("kdf_correct"),
            # NEW real KDF
            "kdf_real_answer": ans,
            "kdf_real_correct": correct,
            "kdf_real_judge_verdict": verdict,
            "tokens_in": total_in,
            "tokens_out": total_out,
        }
        state["results"].append(result_row)
        state["completed_q_ids"].append(qid)
        done_ids.add(qid)

        processed_this_run += 1
        if processed_this_run % args.checkpoint_every == 0:
            save_state()
            elapsed = time.time() - t_start
            rate = processed_this_run / max(elapsed, 0.1)
            remain = len(oracle) - len(done_ids)
            eta = remain / max(rate, 0.01)
            print(
                f"[{len(done_ids)}/{len(oracle)}] real_recall_so_far={sum(1 for r in state['results'] if r['kdf_real_correct'])/len(state['results']):.3f} "
                f"cost=${state['est_cost_usd']:.3f} rate={rate:.1f} q/s eta={eta/60:.0f}min",
                file=sys.stderr,
            )

    save_state()

    # Final results
    n_correct = sum(1 for r in state["results"] if r["kdf_real_correct"])
    n_total = len(state["results"])
    n_mem0_correct = sum(1 for r in state["results"] if r["mem0_correct"])
    n_sim_correct = sum(1 for r in state["results"] if r["kdf_sim_correct"])

    # Paired outcomes vs Mem0
    both_ok = sum(1 for r in state["results"] if r["kdf_real_correct"] and r["mem0_correct"])
    kdf_only = sum(1 for r in state["results"] if r["kdf_real_correct"] and not r["mem0_correct"])
    mem0_only = sum(1 for r in state["results"] if not r["kdf_real_correct"] and r["mem0_correct"])
    both_wrong = sum(1 for r in state["results"] if not r["kdf_real_correct"] and not r["mem0_correct"])

    summary = {
        "method": state["method"],
        "keep_rate": state["keep_rate"],
        "n_questions": n_total,
        "kdf_real_accuracy": n_correct / max(n_total, 1),
        "kdf_sim_accuracy_from_f044": n_sim_correct / max(n_total, 1),
        "mem0_accuracy_from_f044": n_mem0_correct / max(n_total, 1),
        "paired_contingency_real_vs_mem0": {
            "both_correct": both_ok,
            "kdf_real_only": kdf_only,
            "mem0_only": mem0_only,
            "both_wrong": both_wrong,
        },
        "total_tokens_in": state["total_tokens_in"],
        "total_tokens_out": state["total_tokens_out"],
        "cost_usd": state["est_cost_usd"],
        "results": state["results"],
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    print(f"\nWrote: {args.out}", file=sys.stderr)

    print("\n=== Summary ===")
    print(f"Method: {state['method']} @ keep_rate={state['keep_rate']}")
    print(f"Real KDF accuracy:   {summary['kdf_real_accuracy']:.4f}")
    print(f"Sim  KDF accuracy:   {summary['kdf_sim_accuracy_from_f044']:.4f}  (F-044 original)")
    print(f"Mem0 accuracy (ref): {summary['mem0_accuracy_from_f044']:.4f}  (F-044 original)")
    print(f"Real_KDF − Mem0:     {summary['kdf_real_accuracy'] - summary['mem0_accuracy_from_f044']:+.4f}")
    print(f"Paired: both_ok={both_ok}, kdf_real_only={kdf_only}, mem0_only={mem0_only}, both_wrong={both_wrong}")
    print(f"Tokens in={summary['total_tokens_in']}, out={summary['total_tokens_out']}, cost=${summary['cost_usd']:.4f}")


if __name__ == "__main__":
    main()
