#!/usr/bin/env python3
"""Full 500 LongMemEval vs 3 dense embedding models + per-category breakdown.

Addresses F-043 caveat: "100 question sample" → verify on full 500."""

import json, sys
from pathlib import Path
from collections import defaultdict

def main():
    data_path = Path("demos/D8_llm_memory/data/longmemeval_oracle.json")
    with open(data_path, encoding='utf-8') as f:
        questions = json.load(f)
    n_total = len(questions)
    print(f"Total: {n_total} questions", file=sys.stderr)

    from sentence_transformers import SentenceTransformer
    import numpy as np

    models_to_test = [
        ('MiniLM-L6-v2', 'all-MiniLM-L6-v2'),
        ('BGE-small-en-v1.5', 'BAAI/bge-small-en-v1.5'),
        # mpnet-base-v2 skipped for speed (420MB × 500 questions is too slow)
    ]

    # Preload all models once
    models = {}
    for (short, full) in models_to_test:
        print(f"Loading {short}...", file=sys.stderr)
        models[short] = SentenceTransformer(full)

    # Per-question results
    per_question = []  # list of dicts: q_id, q_type, minilm_recall, bge_recall

    for q_idx, q in enumerate(questions):
        if q_idx % 25 == 0:
            print(f"Q {q_idx}/{n_total}", file=sys.stderr, flush=True)

        turns_text = []
        answer_idx = []
        for i, session in enumerate(q['haystack_sessions']):
            sid = q['haystack_session_ids'][i]
            is_ans = sid in q['answer_session_ids']
            for turn in session:
                if is_ans and turn.get('has_answer', False):
                    answer_idx.append(len(turns_text))
                turns_text.append(turn['content'])
        if not answer_idx:
            continue

        n = len(turns_text)
        keep = max(1, int(n * 0.30 + 0.5))

        q_record = {
            'q_idx': q_idx,
            'question_id': q['question_id'],
            'question_type': q['question_type'],
            'n_turns': n,
            'n_answer_turns': len(answer_idx),
        }

        all_texts = turns_text + [q['question']]
        for short, model in models.items():
            vecs = model.encode(all_texts, normalize_embeddings=True,
                               show_progress_bar=False, convert_to_numpy=True)
            sims = vecs[:-1] @ vecs[-1]
            ranked = sorted(range(n), key=lambda i: -sims[i])
            top = set(ranked[:keep])
            hit = len(top & set(answer_idx))
            q_record[f'{short}_recall'] = hit / len(answer_idx)

        per_question.append(q_record)

    # Aggregate by overall + by question type
    def stats(recalls):
        if not recalls: return 0.0
        return sum(recalls) / len(recalls)

    print("\n" + "=" * 70)
    print(f"# Caveat 2 verification: full {len(per_question)}Q (answer-bearing)")
    print("=" * 70)

    print("\n## Overall")
    for short, _ in models_to_test:
        recalls = [r[f'{short}_recall'] for r in per_question]
        print(f"{short:20s}: recall = {stats(recalls):.4f}  (n={len(recalls)})")

    # Per-type breakdown
    print("\n## Per-category breakdown")
    types = set(r['question_type'] for r in per_question)
    print(f"| Type | n | MiniLM-L6-v2 | BGE-small |")
    print("|---|---:|---:|---:|")
    for t in sorted(types):
        subset = [r for r in per_question if r['question_type'] == t]
        rec_minilm = stats([r['MiniLM-L6-v2_recall'] for r in subset])
        rec_bge = stats([r['BGE-small-en-v1.5_recall'] for r in subset])
        print(f"| {t} | {len(subset)} | {rec_minilm:.3f} | {rec_bge:.3f} |")

    # Dump
    out_path = Path('demos/D8_llm_memory/out/full_500_dense_recalls.json')
    out_path.parent.mkdir(parents=True, exist_ok=True)
    with open(out_path, 'w') as f:
        json.dump({
            'n_total_questions': n_total,
            'n_with_answers': len(per_question),
            'overall': {short: stats([r[f'{short}_recall'] for r in per_question])
                        for short, _ in models_to_test},
            'per_type': {
                t: {short: stats([r[f'{short}_recall'] for r in per_question if r['question_type'] == t])
                    for short, _ in models_to_test}
                for t in sorted(types)
            },
            'per_question': per_question,
        }, f, indent=2)
    print(f"\nDumped to {out_path}")

if __name__ == '__main__':
    main()
