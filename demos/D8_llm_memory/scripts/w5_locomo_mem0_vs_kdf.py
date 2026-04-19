"""
W5: LoCoMo benchmark — Real KDF vs Mem0 (gpt-4o-mini), apples-to-apples.

Adapts the F-044/F-053 pipeline to LoCoMo. Key differences from LongMemEval:
  - LoCoMo has 10 samples; each sample shares one haystack across many Q.
    => Build Mem0 memory ONCE per sample (not per-Q) to save time/cost.
  - Turn roles are speaker names (Caroline, Melanie) not user/assistant.
    => Alternate user/assistant mapping for Mem0 schema compliance.

Pipeline for each sampled Q:
  1. Ensure sample's Mem0 memory is built (one-time per sample).
  2. Mem0 search(q) → facts → Mem0 answer + judge.
  3. Real KDF turns (pre-computed by Rust binary) → KDF answer + judge.

Outputs:
  demos/D8_llm_memory/out/w5_locomo_results.json
  demos/D8_llm_memory/out/w5_locomo_checkpoint.json
"""
from __future__ import annotations

import argparse
import json
import os
import signal
import sys
import time
import uuid
from pathlib import Path
from collections import defaultdict

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


def parse_args():
    p = argparse.ArgumentParser()
    p.add_argument("--oracle", default="demos/D8_llm_memory/data/locomo/locomo_oracle_sampled.json")
    p.add_argument("--turns", default="demos/D8_llm_memory/out/w5_locomo_real_kdf_turns_030.json")
    p.add_argument("--model", default="gpt-4o-mini")
    p.add_argument("--out", default="demos/D8_llm_memory/out/w5_locomo_results.json")
    p.add_argument("--checkpoint", default="demos/D8_llm_memory/out/w5_locomo_checkpoint.json")
    p.add_argument("--checkpoint-every", type=int, default=10)
    p.add_argument("--qdrant-path", default="demos/D8_llm_memory/out/qdrant_locomo")
    p.add_argument("--qdrant-reuse", default="",
                   help="Reuse an existing Qdrant dir (skip re-ingest if samples_ingested match)")
    return p.parse_args()


def extract_flat(q):
    flat = []
    for sess in q["haystack_sessions"]:
        for t in sess:
            flat.append(t)
    return flat


def flatten_for_mem0(flat_turns, max_chars=2000):
    """Map LoCoMo speaker names to alternating user/assistant for Mem0."""
    speakers = list({t.get("role", "") for t in flat_turns})
    mapping = {}
    if len(speakers) >= 1:
        mapping[speakers[0]] = "user"
    if len(speakers) >= 2:
        mapping[speakers[1]] = "assistant"
    for s in speakers[2:]:
        mapping[s] = "user"  # fallback

    out = []
    for t in flat_turns:
        content = t.get("content", "")
        if len(content) > max_chars:
            content = content[:max_chars] + "... [truncated]"
        speaker = t.get("role", "user")
        out.append({"role": mapping.get(speaker, "user"), "content": f"[{speaker}] {content}"})
    return out


def build_mem0_for_sample(mem, sample_q_list, user_id):
    """Add all turns from the first Q's haystack into Mem0 under user_id.
    All Qs in the sample share the same haystack; use the first one."""
    q0 = sample_q_list[0]
    flat = extract_flat(q0)
    msgs = flatten_for_mem0(flat)
    t0 = time.time()
    BATCH = 4
    for i in range(0, len(msgs), BATCH):
        batch = msgs[i:i + BATCH]
        try:
            mem.add(batch, user_id=user_id)
        except Exception:
            for m in batch:
                try:
                    mem.add([m], user_id=user_id)
                except Exception as e:
                    print(f"[mem0 add err] {e}", file=sys.stderr)
    return (time.time() - t0) * 1000


def mem0_retrieve(mem, query, user_id, limit=50):
    t0 = time.time()
    try:
        retrieved = mem.search(query=query, filters={"user_id": user_id}, limit=limit)
    except TypeError:
        retrieved = mem.search(query=query, user_id=user_id, limit=limit)
    elapsed = (time.time() - t0) * 1000
    texts = []
    if isinstance(retrieved, dict) and "results" in retrieved:
        for r in retrieved["results"]:
            if isinstance(r, dict):
                texts.append(r.get("memory") or r.get("text") or r.get("content") or str(r))
            else:
                texts.append(str(r))
    elif isinstance(retrieved, list):
        for r in retrieved:
            if isinstance(r, dict):
                texts.append(r.get("memory") or r.get("text") or r.get("content") or str(r))
            else:
                texts.append(str(r))
    return texts, elapsed


def generate_answer(client, model, context, question):
    prompt = (
        f"You are a memory agent. Using only the facts below, answer the question concisely.\n\n"
        f"Facts:\n{context}\n\n"
        f"Question: {question}\n\n"
        f"Answer (1-2 sentences, just the fact):"
    )
    resp = client.chat.completions.create(model=model, messages=[{"role": "user", "content": prompt}],
                                          temperature=0, max_tokens=150)
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
    resp = client.chat.completions.create(model=model, messages=[{"role": "user", "content": prompt}],
                                          temperature=0, max_tokens=80)
    verdict = resp.choices[0].message.content.strip()
    correct = verdict.upper().startswith("YES")
    return correct, verdict, resp.usage.prompt_tokens, resp.usage.completion_tokens


def main():
    args = parse_args()
    if not os.environ.get("OPENAI_API_KEY"):
        print("ERROR: OPENAI_API_KEY not set", file=sys.stderr)
        sys.exit(1)

    # Qdrant storage: if --qdrant-reuse is given and non-empty, reuse that path
    # (must match samples already ingested); otherwise create a fresh subdir.
    qdrant_root = Path(args.qdrant_path)
    if args.qdrant_reuse and Path(args.qdrant_reuse).exists():
        run_qdrant = Path(args.qdrant_reuse)
        print(f"Reusing Qdrant at {run_qdrant}", file=sys.stderr)
    else:
        run_qdrant = qdrant_root / f"run_{uuid.uuid4().hex[:8]}"
        run_qdrant.mkdir(parents=True, exist_ok=True)

    from openai import OpenAI
    from mem0 import Memory

    client = OpenAI()

    mem_config = {
        "vector_store": {
            "provider": "qdrant",
            "config": {"path": str(run_qdrant), "collection_name": "locomo_bench"}
        },
        "llm": {"provider": "openai", "config": {"model": args.model, "temperature": 0}},
        "embedder": {"provider": "openai", "config": {"model": "text-embedding-3-small"}},
    }

    # Load oracle & turns
    with open(args.oracle, encoding="utf-8") as f:
        oracle = json.load(f)
    with open(args.turns, encoding="utf-8") as f:
        turns_data = json.load(f)
    turns_by_qid = {r["question_id"]: r for r in turns_data["results"]}
    print(f"Oracle: {len(oracle)} Q, KDF turns: method={turns_data['method']} @ {turns_data['keep_rate']}",
          file=sys.stderr)

    # Group by sample_id (extract from question_id like "locomo_0::q5")
    def sample_of(qid):
        return qid.split("::q")[0]
    by_sample = defaultdict(list)
    for q in oracle:
        by_sample[sample_of(q["question_id"])].append(q)
    print(f"Samples: {len(by_sample)}", file=sys.stderr)

    # Resume
    ckpt_path = Path(args.checkpoint)
    if ckpt_path.exists():
        with ckpt_path.open(encoding="utf-8") as f:
            state = json.load(f)
        print(f"Resuming: {len(state['completed_q_ids'])}/{len(oracle)} done", file=sys.stderr)
    else:
        state = {
            "completed_q_ids": [],
            "results": [],
            "total_tokens_in": 0,
            "total_tokens_out": 0,
            "est_cost_usd": 0.0,
            "samples_ingested": [],
        }
    done = set(state["completed_q_ids"])
    samples_done = set(state["samples_ingested"])

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

    # Single Memory instance; use different user_ids per sample to isolate
    mem = Memory.from_config(mem_config)

    t_start = time.time()
    n = 0
    for sample_id, qs in by_sample.items():
        user_id = f"loco_{sample_id.replace('::', '_')}"
        if sample_id not in samples_done:
            add_ms = build_mem0_for_sample(mem, qs, user_id)
            state["samples_ingested"].append(sample_id)
            samples_done.add(sample_id)
            save()
            print(f"[ingest] {sample_id}: {len(extract_flat(qs[0]))} turns in {add_ms/1000:.1f}s", file=sys.stderr)

        for q in qs:
            qid = q["question_id"]
            if qid in done:
                continue
            if qid not in turns_by_qid:
                print(f"[skip] {qid}: no KDF turns", file=sys.stderr)
                continue
            # Mem0 retrieve + answer + judge
            try:
                mem_texts, m_retrieve_ms = mem0_retrieve(mem, q["question"], user_id, limit=30)
                mem_ctx = "\n".join(f"- {t}" for t in mem_texts)
                mem_ans, m_tin, m_tout = generate_answer(client, args.model, mem_ctx, q["question"])
            except Exception as e:
                print(f"[err mem0 q={qid}] {e}", file=sys.stderr)
                time.sleep(5)
                continue
            try:
                gt = q["answer"]
                if not isinstance(gt, str):
                    gt = str(gt)
                m_correct, m_verdict, m_jin, m_jout = judge_answer(client, args.model, q["question"], gt, mem_ans)
            except Exception as e:
                print(f"[err judge mem0 q={qid}] {e}", file=sys.stderr)
                time.sleep(5)
                continue

            # Real KDF answer + judge
            flat = extract_flat(q)
            sel_idx = turns_by_qid[qid]["kept_turn_indices"]
            kdf_ctx = "\n".join(f"[{flat[i].get('role', '')}] {flat[i].get('content', '')[:500]}"
                                 for i in sel_idx if i < len(flat))
            try:
                kdf_ans, k_tin, k_tout = generate_answer(client, args.model, kdf_ctx, q["question"])
            except Exception as e:
                print(f"[err kdf q={qid}] {e}", file=sys.stderr)
                time.sleep(5)
                continue
            try:
                k_correct, k_verdict, k_jin, k_jout = judge_answer(client, args.model, q["question"], gt, kdf_ans)
            except Exception as e:
                print(f"[err judge kdf q={qid}] {e}", file=sys.stderr)
                time.sleep(5)
                continue

            total_in = m_tin + m_jin + k_tin + k_jin
            total_out = m_tout + m_jout + k_tout + k_jout
            state["total_tokens_in"] += total_in
            state["total_tokens_out"] += total_out
            state["est_cost_usd"] += total_in * price_in + total_out * price_out

            state["results"].append({
                "question_id": qid,
                "question_type": q["question_type"],
                "locomo_category": q.get("_locomo_category"),
                "mem0_answer": mem_ans,
                "mem0_correct": m_correct,
                "mem0_verdict": m_verdict,
                "mem0_n_retrieved": len(mem_texts),
                "mem0_retrieve_ms": m_retrieve_ms,
                "kdf_answer": kdf_ans,
                "kdf_correct": k_correct,
                "kdf_verdict": k_verdict,
                "kdf_real_turn_recall": turns_by_qid[qid]["answer_turn_recall"],
                "n_total_turns": turns_by_qid[qid]["n_total_turns"],
                "n_kept_turns": len(sel_idx),
                "tokens_in": total_in,
                "tokens_out": total_out,
            })
            state["completed_q_ids"].append(qid)
            done.add(qid)
            n += 1
            if n % args.checkpoint_every == 0:
                save()
                elapsed = time.time() - t_start
                rate = n / max(elapsed, 0.1)
                eta = (len(oracle) - len(done)) / max(rate, 0.01)
                ma = sum(1 for r in state["results"] if r["mem0_correct"]) / len(state["results"])
                ka = sum(1 for r in state["results"] if r["kdf_correct"]) / len(state["results"])
                print(f"[{len(done)}/{len(oracle)}] mem0={ma:.3f} kdf={ka:.3f} cost=${state['est_cost_usd']:.3f} eta={eta/60:.0f}m",
                      file=sys.stderr)
    save()

    # Summary
    nres = len(state["results"])
    m_acc = sum(1 for r in state["results"] if r["mem0_correct"]) / max(nres, 1)
    k_acc = sum(1 for r in state["results"] if r["kdf_correct"]) / max(nres, 1)
    both = sum(1 for r in state["results"] if r["mem0_correct"] and r["kdf_correct"])
    m_only = sum(1 for r in state["results"] if r["mem0_correct"] and not r["kdf_correct"])
    k_only = sum(1 for r in state["results"] if not r["mem0_correct"] and r["kdf_correct"])
    both_w = sum(1 for r in state["results"] if not r["mem0_correct"] and not r["kdf_correct"])

    summary = {
        "benchmark": "LoCoMo (balanced 200Q, non-adversarial)",
        "model": args.model,
        "n_questions": nres,
        "mem0_accuracy": m_acc,
        "kdf_accuracy": k_acc,
        "kdf_minus_mem0": k_acc - m_acc,
        "contingency": {"both_correct": both, "kdf_only": k_only, "mem0_only": m_only, "both_wrong": both_w},
        "total_tokens_in": state["total_tokens_in"],
        "total_tokens_out": state["total_tokens_out"],
        "cost_usd": state["est_cost_usd"],
        "results": state["results"],
    }
    Path(args.out).parent.mkdir(parents=True, exist_ok=True)
    with open(args.out, "w", encoding="utf-8") as f:
        json.dump(summary, f, indent=2, ensure_ascii=False)
    print("\n=== W5 LoCoMo summary ===")
    print(f"Mem0: {m_acc:.4f}  KDF: {k_acc:.4f}  KDF-Mem0: {k_acc-m_acc:+.4f}")
    print(f"Paired: both_ok={both}, kdf_only={k_only}, mem0_only={m_only}, both_wrong={both_w}")
    print(f"Cost: ${summary['cost_usd']:.4f}")


if __name__ == "__main__":
    main()
