#!/usr/bin/env python3
"""BGE-small version - for robustness check."""

import json, sys
from pathlib import Path

def main():
    data_path = Path("demos/D8_llm_memory/data/longmemeval_oracle.json")
    with open(data_path, encoding='utf-8') as f:
        questions = json.load(f)
    sample = questions[:100]

    print("Loading BAAI/bge-small-en-v1.5...", file=sys.stderr)
    from sentence_transformers import SentenceTransformer
    model = SentenceTransformer('BAAI/bge-small-en-v1.5')

    recalls = []
    for q_idx, q in enumerate(sample):
        print(f"Q {q_idx+1}/100", end='\r', file=sys.stderr)
        turns_text, answer_idx = [], []
        for i, session in enumerate(q['haystack_sessions']):
            sid = q['haystack_session_ids'][i]
            is_ans_sess = sid in q['answer_session_ids']
            for turn in session:
                if is_ans_sess and turn.get('has_answer', False):
                    answer_idx.append(len(turns_text))
                turns_text.append(turn['content'])
        if not answer_idx:
            continue

        all_texts = turns_text + [q['question']]
        vecs = model.encode(all_texts, normalize_embeddings=True, show_progress_bar=False)
        turn_vecs = vecs[:-1]
        query_vec = vecs[-1]
        import numpy as np
        sims = turn_vecs @ query_vec

        n = len(turns_text)
        keep = max(1, int(n * 0.30 + 0.5))
        ranked = sorted(range(n), key=lambda i: -sims[i])
        top = set(ranked[:keep])
        hit = len(top & set(answer_idx))
        recalls.append(hit / len(answer_idx))

    print(f"\nBGE-small-en-v1.5 dense retrieval (keep 30%):")
    print(f"  answer_turn_recall (mean) = {sum(recalls)/len(recalls):.4f}  over {len(recalls)} questions")

if __name__ == '__main__':
    main()
