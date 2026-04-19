#!/usr/bin/env python3
"""
Phase Route A (Q2 追加) — Dense embedding precompute for LongMemEval.

LongMemEval oracle 100 questions について、各 turn と question を
sentence-transformers で embedding し、JSON にダンプする。
Rust 側がこの embedding を読んで cosine retrieval を実行し、KDF と比較する。

Models used: all-MiniLM-L6-v2 (22MB, 384-dim, fast sentence transformer baseline)
"""

import json
import sys
import os
from pathlib import Path

def main():
    # Load data
    data_path = Path("demos/D8_llm_memory/data/longmemeval_oracle.json")
    if not data_path.exists():
        print(f"ERROR: {data_path} not found", file=sys.stderr)
        sys.exit(1)
    with open(data_path, encoding='utf-8') as f:
        questions = json.load(f)
    print(f"Loaded {len(questions)} questions", file=sys.stderr)

    # Take first 100 (deterministic, same as Rust side)
    sample_size = 100
    sample = questions[:sample_size]

    # Embed
    print("Loading sentence-transformers all-MiniLM-L6-v2...", file=sys.stderr)
    from sentence_transformers import SentenceTransformer
    model = SentenceTransformer('all-MiniLM-L6-v2')
    print(f"Model loaded. Embedding dim = {model.get_sentence_embedding_dimension()}", file=sys.stderr)

    output = []
    for q_idx, q in enumerate(sample):
        print(f"Processing question {q_idx+1}/{sample_size}...", end='\r', file=sys.stderr)

        # Flatten turns
        turns_text = []
        answer_turn_indices = []
        session_ids = []
        for i, session in enumerate(q['haystack_sessions']):
            sid = q['haystack_session_ids'][i]
            is_answer_sess = sid in q['answer_session_ids']
            for turn in session:
                global_idx = len(turns_text)
                turns_text.append(turn['content'])
                session_ids.append(sid)
                if is_answer_sess and turn.get('has_answer', False):
                    answer_turn_indices.append(global_idx)

        # Embed all turns + the question
        # Batch for speed
        all_texts = turns_text + [q['question']]
        vecs = model.encode(all_texts, normalize_embeddings=True,
                            show_progress_bar=False, convert_to_numpy=True)

        turn_vecs = vecs[:-1]  # (n_turns, 384)
        query_vec = vecs[-1]   # (384,)

        # Cosine similarity (already normalized → just dot product)
        import numpy as np
        similarities = turn_vecs @ query_vec  # (n_turns,)

        output.append({
            'question_id': q['question_id'],
            'n_turns': len(turns_text),
            'answer_turn_indices': answer_turn_indices,
            'similarities': similarities.tolist(),
        })
    print(f"\nDone. Writing output...", file=sys.stderr)

    # Write
    out_path = Path("demos/D8_llm_memory/out/dense_embedding_similarities.json")
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, 'w') as f:
        json.dump(output, f, indent=None, separators=(',', ':'))
    print(f"Wrote {out_path}", file=sys.stderr)

    # Compute recall at top-30% for each question
    hits = 0
    total_answers = 0
    for q_data in output:
        n = q_data['n_turns']
        keep = max(1, int(n * 0.30 + 0.5))
        sims = q_data['similarities']
        answers = set(q_data['answer_turn_indices'])
        if not answers:
            continue
        # top-K by similarity (descending)
        ranked = sorted(range(n), key=lambda i: -sims[i])
        top = set(ranked[:keep])
        hit = len(top & answers)
        hits += hit
        total_answers += len(answers)
    if total_answers > 0:
        # Per-question recall average
        recalls = []
        for q_data in output:
            n = q_data['n_turns']
            keep = max(1, int(n * 0.30 + 0.5))
            sims = q_data['similarities']
            answers = set(q_data['answer_turn_indices'])
            if not answers:
                continue
            ranked = sorted(range(n), key=lambda i: -sims[i])
            top = set(ranked[:keep])
            hit = len(top & answers)
            recalls.append(hit / len(answers))
        avg_recall = sum(recalls) / len(recalls)
        print(f"\nMiniLM-L6-v2 dense retrieval (keep 30%):")
        print(f"  answer_turn_recall (mean) = {avg_recall:.4f}  over {len(recalls)} questions")
        print(f"  aggregate turn-level: {hits}/{total_answers} = {hits/total_answers:.4f}")

if __name__ == '__main__':
    main()
