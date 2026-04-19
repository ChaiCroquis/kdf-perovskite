#!/usr/bin/env python3
"""
Phase M1 — "Mem0-style" pipeline with local LLM vs KDF retrieval.

Since OpenAI API key is not set in this session, we implement a minimal
Mem0-style pipeline using:
  - Qwen2.5-0.5B-Instruct (local transformers, CPU) for fact extraction
  - BGE-small-en-v1.5 for embedding
  - Simple cosine retrieval for top-K

**Honest caveat**: Qwen2.5-0.5B is much weaker than Mem0's default
gpt-4o-mini. So this test approximates "weakest-possible-Mem0 vs KDF".
If KDF competes, it's a low-bar win. Full Mem0 with GPT-4o-mini requires
OpenAI key and would likely extract better facts.

Evaluation: LongMemEval first 20 questions.
"""

import json
import sys
import time
from pathlib import Path
import torch
import numpy as np

def main():
    data_path = Path("demos/D8_llm_memory/data/longmemeval_oracle.json")
    with open(data_path, encoding='utf-8') as f:
        questions = json.load(f)
    N_QUESTIONS = 20
    sample = questions[:N_QUESTIONS]
    print(f"Loaded {len(questions)} questions; using first {N_QUESTIONS}", file=sys.stderr)

    # Load LLM for fact extraction
    from transformers import AutoTokenizer, AutoModelForCausalLM
    print("Loading Qwen2.5-0.5B-Instruct for local fact extraction...", file=sys.stderr)
    llm_tok = AutoTokenizer.from_pretrained('Qwen/Qwen2.5-0.5B-Instruct')
    llm = AutoModelForCausalLM.from_pretrained(
        'Qwen/Qwen2.5-0.5B-Instruct', torch_dtype=torch.float32)
    llm.eval()

    # Embedding model
    print("Loading BGE-small-en-v1.5 for embedding...", file=sys.stderr)
    from sentence_transformers import SentenceTransformer
    embed_model = SentenceTransformer('BAAI/bge-small-en-v1.5')

    def extract_fact(turn_content, role):
        """Extract a short fact from a single turn using local LLM."""
        prompt = f"Extract one concise fact (1 sentence, max 20 words) from this {role} message. Output only the fact, no preamble.\n\nMessage: {turn_content[:500]}\n\nFact:"
        text = llm_tok.apply_chat_template(
            [{'role': 'user', 'content': prompt}], tokenize=False, add_generation_prompt=True)
        inputs = llm_tok(text, return_tensors='pt')
        with torch.no_grad():
            out = llm.generate(**inputs, max_new_tokens=50, do_sample=False,
                              pad_token_id=llm_tok.eos_token_id)
        fact = llm_tok.decode(out[0][inputs['input_ids'].shape[1]:], skip_special_tokens=True)
        return fact.strip().split('\n')[0][:200]  # first line, capped

    results = []
    total_t0 = time.time()

    for q_idx, q in enumerate(sample):
        q_t0 = time.time()
        print(f"\nQ {q_idx+1}/{N_QUESTIONS} (type: {q['question_type']})", file=sys.stderr)

        # Flatten turns
        turns_flat = []
        answer_idx = []
        for i, session in enumerate(q['haystack_sessions']):
            sid = q['haystack_session_ids'][i]
            is_ans = sid in q['answer_session_ids']
            for turn in session:
                if is_ans and turn.get('has_answer', False):
                    answer_idx.append(len(turns_flat))
                turns_flat.append({'role': turn['role'], 'content': turn['content']})
        if not answer_idx:
            continue

        # Fact extraction: one LLM call per turn (Mem0-style)
        print(f"  Extracting facts from {len(turns_flat)} turns...", file=sys.stderr)
        t0 = time.time()
        facts = []
        for t in turns_flat:
            fact = extract_fact(t['content'], t['role'])
            facts.append(fact)
        extract_elapsed = time.time() - t0
        print(f"    done in {extract_elapsed:.1f}s ({len(turns_flat)/extract_elapsed:.1f} turns/s)", file=sys.stderr)

        # Embed facts + query
        all_texts = facts + [q['question']]
        vecs = embed_model.encode(all_texts, normalize_embeddings=True,
                                  show_progress_bar=False, convert_to_numpy=True)
        fact_vecs = vecs[:-1]
        query_vec = vecs[-1]
        sims = fact_vecs @ query_vec

        # Retrieve top-30% (same budget as KDF)
        n = len(turns_flat)
        keep = max(1, int(n * 0.30 + 0.5))
        ranked = np.argsort(-sims)
        top = set(ranked[:keep].tolist())
        hit = len(top & set(answer_idx))
        recall = hit / len(answer_idx)

        # Also compare: BGE on raw turns (from F-043 data if available)
        raw_texts = [t['content'] for t in turns_flat] + [q['question']]
        raw_vecs = embed_model.encode(raw_texts, normalize_embeddings=True,
                                      show_progress_bar=False, convert_to_numpy=True)
        raw_sims = raw_vecs[:-1] @ raw_vecs[-1]
        raw_ranked = np.argsort(-raw_sims)
        raw_top = set(raw_ranked[:keep].tolist())
        raw_hit = len(raw_top & set(answer_idx))
        raw_recall = raw_hit / len(answer_idx)

        q_elapsed = time.time() - q_t0
        print(f"  mem0-style (Qwen + BGE) recall = {recall:.3f}", file=sys.stderr)
        print(f"  BGE-only (no LLM) recall        = {raw_recall:.3f}", file=sys.stderr)
        print(f"  q total: {q_elapsed:.1f}s", file=sys.stderr)

        results.append({
            'q_idx': q_idx,
            'question_type': q['question_type'],
            'n_turns': n,
            'n_answer': len(answer_idx),
            'recall_mem0_style': recall,
            'recall_bge_raw': raw_recall,
            'extract_s': extract_elapsed,
        })

    total_elapsed = time.time() - total_t0

    mean_m0 = sum(r['recall_mem0_style'] for r in results) / max(len(results), 1)
    mean_bge = sum(r['recall_bge_raw'] for r in results) / max(len(results), 1)

    print("\n" + "=" * 60)
    print(f"# M1 results on LongMemEval first {len(results)}Q")
    print("=" * 60)
    print(f"mem0-style (Qwen-0.5B fact extract + BGE retrieve): {mean_m0:.4f}")
    print(f"BGE-only on raw turns (F-043 replication):          {mean_bge:.4f}")
    print(f"KDF (from F-033/F-043, full 500Q average):          0.8210")
    print()
    print("Interpretation:")
    print(f"  - Mem0-style (weak LLM) vs KDF: {'KDF wins' if 0.821 > mean_m0 else 'Mem0-style wins'} by {abs(0.821 - mean_m0):.3f}")
    print(f"  - BGE-only vs KDF: {'KDF wins' if 0.821 > mean_bge else 'BGE wins'} by {abs(0.821 - mean_bge):.3f}")
    print()
    print("Caveats:")
    print("  - Qwen2.5-0.5B is much weaker than Mem0's default gpt-4o-mini")
    print("  - 20 questions is a small sample; results have high variance")
    print("  - Full Mem0 pipeline (with GPT-4o-mini) would likely score higher")
    print(f"  - Total runtime: {total_elapsed/60:.1f} min for {len(results)} Qs")

    out = Path("demos/D8_llm_memory/out/m1_mem0style_local_results.json")
    with open(out, 'w') as f:
        json.dump({
            'n_questions': len(results),
            'mean_recall_mem0_style': mean_m0,
            'mean_recall_bge_raw': mean_bge,
            'kdf_reference': 0.821,
            'per_question': results,
            'llm_model': 'Qwen/Qwen2.5-0.5B-Instruct',
            'embed_model': 'BAAI/bge-small-en-v1.5',
            'caveat': 'Local weak LLM is much weaker than Mem0 default GPT-4o-mini; this is a lower bound on Mem0 performance.',
        }, f, indent=2)
    print(f"\nSaved to {out}")

if __name__ == '__main__':
    main()
