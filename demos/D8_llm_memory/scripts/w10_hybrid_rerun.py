"""
W10: KDF + Mem0 hybrid answer generation (post-F-053 pivot).

Given W3/F-053 showed real KDF loses to Mem0 by -23.8 pt overall, the
question is whether combining BOTH sources (Mem0's fact-extracted answer +
KDF's raw retrieved turns) can exceed either alone.

Design — H1 (cheapest valid hybrid):
  The LLM sees both:
    1. Mem0's final answer (structured summary, already generated)
    2. KDF's raw retrieved turns (lossless evidence)
  and is prompted to produce the best answer, using raw turns to override
  summary when a specific detail appears contradicted.

This tests: "Does giving the LLM access to raw evidence alongside a summary
improve over the summary alone?"

Cost: ~$0.08 (500Q x gpt-4o-mini, 1 hybrid answer + 1 judge each).
Runtime: ~15 minutes.

Usage:
    python demos/D8_llm_memory/scripts/w10_hybrid_rerun.py \
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


def extract_flat(q):
    out = []
    for s in q["haystack_sessions"]:
        for t in s:
            out.append(t)
    return out


def hybrid_answer(client, model, mem0_ans, kdf_ctx, question):
    prompt = (
        "You are a memory agent. You have TWO complementary sources of memory:\n\n"
        "SOURCE A — Structured summary from a fact-extraction memory system (may be abbreviated,\n"
        "may lose specific numbers/quantities/names but good for high-level facts):\n"
        f"{mem0_ans}\n\n"
        "SOURCE B — Raw retrieved turns from a lossless memory system (preserves exact numbers,\n"
        "lists, and assistant phrasing, but may be noisy or miss structure):\n"
        f"{kdf_ctx}\n\n"
        "Principles:\n"
        "- If Source B contains a specific number / date / name / list that appears more reliable\n"
        "  than Source A's summary, prefer Source B.\n"
        "- If Source B is silent or noisy on the question, fall back to Source A.\n"
        "- If both agree, answer confidently; if both silent, say you do not know.\n\n"
        f"Question: {question}\n\n"
        "Answer (1-2 sentences, just the fact):"
    )
    resp = client.chat.completions.create(
        model=model,
        messages=[{"role": "user", "content": prompt}],
        temperature=0,
        max_tokens=150,
    )
    return resp.choices[0].message.content.strip(), resp.usage.prompt_tokens, resp.usage.completion_tokens


def judge_answer(client, model, question, ground_truth, candidate):
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


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--turns", required=True)
    parser.add_argument("--model", default="gpt-4o-mini")
    parser.add_argument("--original-results", default="demos/D8_llm_memory/out/route_a_500q_results.json")
    parser.add_argument("--oracle", default="demos/D8_llm_memory/data/longmemeval_oracle.json")
    parser.add_argument("--out", default="demos/D8_llm_memory/out/route_a_500q_hybrid_results.json")
    parser.add_argument("--checkpoint", default="demos/D8_llm_memory/out/w10_hybrid_checkpoint.json")
    parser.add_argument("--checkpoint-every", type=int, default=25)
    args = parser.parse_args()

    if not os.environ.get("OPENAI_API_KEY"):
        print("ERROR: OPENAI_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    from openai import OpenAI
    client = OpenAI()

    with open(args.turns, encoding="utf-8") as f:
        turns_data = json.load(f)
    turn_sel = {r["question_id"]: r for r in turns_data["results"]}

    with open(args.original_results, encoding="utf-8") as f:
        orig = json.load(f)
    orig_by_id = {r["question_id"]: r for r in orig["results"]}

    with open(args.oracle, encoding="utf-8") as f:
        oracle = json.load(f)

    ckpt_path = Path(args.checkpoint)
    if ckpt_path.exists():
        with ckpt_path.open(encoding="utf-8") as f:
            state = json.load(f)
        print(f"Resuming: {len(state['completed_q_ids'])} done", file=sys.stderr)
    else:
        state = {
            "completed_q_ids": [],
            "results": [],
            "total_tokens_in": 0,
            "total_tokens_out": 0,
            "est_cost_usd": 0.0,
            "method": "hybrid(Mem0+realKDF)",
            "kdf_keep_rate": turns_data["keep_rate"],
        }
    done = set(state["completed_q_ids"])

    def save():
        ckpt_path.parent.mkdir(parents=True, exist_ok=True)
        with ckpt_path.open("w", encoding="utf-8") as f:
            json.dump(state, f, indent=2, ensure_ascii=False)

    def handle_sigint(sig, frame):
        save()
        sys.exit(0)
    signal.signal(signal.SIGINT, handle_sigint)

    price_in = 0.15 / 1e6
    price_out = 0.60 / 1e6

    t0 = time.time()
    n = 0
    for q_idx, q in enumerate(oracle):
        qid = q["question_id"]
        if qid in done:
            continue
        if qid not in turn_sel or qid not in orig_by_id:
            continue
        flat = extract_flat(q)
        sel_idx = turn_sel[qid]["kept_turn_indices"]
        kdf_ctx = "\n".join(f"[{t.get('role', '')}] {t.get('content', '')[:500]}"
                            for t in (flat[i] for i in sel_idx if i < len(flat)))
        mem0_ans = orig_by_id[qid].get("mem0_answer", "")
        gt = q.get("answer", "")
        if not isinstance(gt, str):
            gt = str(gt)

        try:
            ans, tin, tout = hybrid_answer(client, args.model, mem0_ans, kdf_ctx, q["question"])
        except Exception as e:
            print(f"[err q={qid}] gen: {e}", file=sys.stderr)
            time.sleep(5)
            continue
        try:
            correct, verdict, jin, jout = judge_answer(client, args.model, q["question"], gt, ans)
        except Exception as e:
            print(f"[err q={qid}] judge: {e}", file=sys.stderr)
            time.sleep(5)
            continue

        total_in = tin + jin
        total_out = tout + jout
        state["total_tokens_in"] += total_in
        state["total_tokens_out"] += total_out
        state["est_cost_usd"] += total_in * price_in + total_out * price_out

        state["results"].append({
            "q_idx": q_idx,
            "question_id": qid,
            "question_type": q.get("question_type", ""),
            "mem0_answer": mem0_ans,
            "mem0_correct": orig_by_id[qid].get("mem0_correct"),
            "hybrid_answer": ans,
            "hybrid_correct": correct,
            "hybrid_judge_verdict": verdict,
            "tokens_in": total_in,
            "tokens_out": total_out,
        })
        state["completed_q_ids"].append(qid)
        done.add(qid)
        n += 1
        if n % args.checkpoint_every == 0:
            save()
            elapsed = time.time() - t0
            rate = n / max(elapsed, 0.1)
            eta = (len(oracle) - len(done)) / max(rate, 0.01)
            acc = sum(1 for r in state["results"] if r["hybrid_correct"]) / len(state["results"])
            print(f"[{len(done)}/{len(oracle)}] hybrid_acc={acc:.3f} cost=${state['est_cost_usd']:.3f} eta={eta/60:.0f}m",
                  file=sys.stderr)
    save()

    n_correct = sum(1 for r in state["results"] if r["hybrid_correct"])
    n_total = len(state["results"])
    n_mem0 = sum(1 for r in state["results"] if r["mem0_correct"])

    # Paired: hybrid vs mem0
    both = sum(1 for r in state["results"] if r["hybrid_correct"] and r["mem0_correct"])
    h_only = sum(1 for r in state["results"] if r["hybrid_correct"] and not r["mem0_correct"])
    m_only = sum(1 for r in state["results"] if not r["hybrid_correct"] and r["mem0_correct"])
    both_w = sum(1 for r in state["results"] if not r["hybrid_correct"] and not r["mem0_correct"])

    summary = {
        "method": state["method"],
        "kdf_keep_rate": state["kdf_keep_rate"],
        "n_questions": n_total,
        "hybrid_accuracy": n_correct / max(n_total, 1),
        "mem0_accuracy": n_mem0 / max(n_total, 1),
        "contingency_hybrid_vs_mem0": {
            "both_correct": both,
            "hybrid_only": h_only,
            "mem0_only": m_only,
            "both_wrong": both_w,
        },
        "total_tokens_in": state["total_tokens_in"],
        "total_tokens_out": state["total_tokens_out"],
        "cost_usd": state["est_cost_usd"],
        "results": state["results"],
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)

    print("\n=== W10 Hybrid Summary ===")
    print(f"Hybrid accuracy: {summary['hybrid_accuracy']:.4f}")
    print(f"Mem0   accuracy: {summary['mem0_accuracy']:.4f}  (reference)")
    print(f"Hybrid − Mem0  : {summary['hybrid_accuracy'] - summary['mem0_accuracy']:+.4f}")
    print(f"Paired: both={both}, hybrid_only={h_only}, mem0_only={m_only}, both_wrong={both_w}")
    print(f"Cost: ${summary['cost_usd']:.4f}")


if __name__ == "__main__":
    main()
