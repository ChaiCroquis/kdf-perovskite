#!/usr/bin/env python3
"""Caveat 1 verification: KDF vs dense embedding on a general retrieval task.

BEIR SciFact: scientific claim verification against biomedical abstracts.
- Corpus: 5,183 abstracts
- Queries: 300 claims (test set)
- Binary relevance

This is a *general* retrieval task, very different from LongMemEval's
conversational memory. Goal: see if KDF's one-off-mention-preservation
advantage carries over, or is specific to conversational data.

For KDF-compatible evaluation:
- Build document × shingle bipartite graph (same as LongMemEval pipeline)
- Treat each document as a "turn"
- Rank by rare-protection signal
- Compare to dense embedding top-K

However, in retrieval the task is query-specific — a document isn't
universally "rare", it's rare w.r.t. a specific query. So KDF's
query-blind approach is expected to struggle.

Honest prediction: KDF will underperform on SciFact because
(a) the task requires query-document matching, and
(b) scientific abstracts don't have the conversational one-off-mention
    structure that KDF exploits.

**A negative result here would confirm caveat 1** (LongMemEval-specific)
and honestly bound the KDF sales pitch.
"""

import json, sys, os
from pathlib import Path

def main():
    # Download SciFact
    from beir import util
    from beir.datasets.data_loader import GenericDataLoader

    out_dir = Path("benchmarks/real_data/data/scifact")
    out_dir.parent.mkdir(parents=True, exist_ok=True)

    if not (out_dir / "corpus.jsonl").exists():
        print("Downloading BEIR SciFact...", file=sys.stderr)
        url = "https://public.ukp.informatik.tu-darmstadt.de/thakur/BEIR/datasets/scifact.zip"
        util.download_and_unzip(url, str(out_dir.parent))
    else:
        print(f"Already downloaded: {out_dir}", file=sys.stderr)

    corpus, queries, qrels = GenericDataLoader(data_folder=str(out_dir)).load(split="test")
    print(f"Corpus: {len(corpus)}, Queries: {len(queries)}, Qrels: {len(qrels)}",
          file=sys.stderr)

    # Flatten corpus to list with deterministic ordering
    corpus_ids = sorted(corpus.keys())
    corpus_texts = [corpus[cid]['title'] + '. ' + corpus[cid].get('text', '')
                    for cid in corpus_ids]
    corpus_id_to_idx = {cid: i for i, cid in enumerate(corpus_ids)}

    # For each query, compute recall@K with BGE-small
    from sentence_transformers import SentenceTransformer
    import numpy as np

    print("Loading BGE-small-en-v1.5...", file=sys.stderr)
    model = SentenceTransformer('BAAI/bge-small-en-v1.5')

    # Embed corpus (this is O(|corpus|) once)
    print(f"Embedding {len(corpus_texts)} docs...", file=sys.stderr)
    corpus_vecs = model.encode(corpus_texts, normalize_embeddings=True,
                               show_progress_bar=False, batch_size=32)
    print("  done", file=sys.stderr)

    # Compute metrics at different K
    ks = [10, 30, 50, 100]
    total_queries = 0
    recall_at_k = {k: 0.0 for k in ks}

    print(f"Running {len(queries)} queries...", file=sys.stderr)
    for q_idx, (qid, qtext) in enumerate(queries.items()):
        if q_idx % 50 == 0:
            print(f"  Q {q_idx}/{len(queries)}", file=sys.stderr, flush=True)
        if qid not in qrels or not qrels[qid]:
            continue
        relevant_cids = [cid for cid, rel in qrels[qid].items() if rel > 0]
        if not relevant_cids:
            continue
        relevant_idx = set(corpus_id_to_idx[cid] for cid in relevant_cids
                          if cid in corpus_id_to_idx)
        if not relevant_idx:
            continue

        # Embed query and dot product
        qvec = model.encode(qtext, normalize_embeddings=True, show_progress_bar=False)
        sims = corpus_vecs @ qvec
        ranked = np.argsort(-sims)

        for k in ks:
            top_k = set(ranked[:k].tolist())
            recall = len(top_k & relevant_idx) / len(relevant_idx)
            recall_at_k[k] += recall
        total_queries += 1

    print(f"\n## BGE-small-en-v1.5 on BEIR SciFact ({total_queries} test queries)")
    print(f"| k | recall@k |")
    print(f"|---:|---:|")
    for k in ks:
        print(f"| {k} | {recall_at_k[k] / total_queries:.4f} |")

    # KDF approach: query-blind rare preservation
    # For each query, we want to build a graph of the corpus + query-relevant
    # structure, then use KDF's layer classification to pick top-K.
    # But KDF is query-blind, so for general retrieval the natural framing is:
    #   Given a corpus-only view, which documents would KDF protect?
    # This maps to "does KDF's query-blind rare-preservation correlate with
    # task relevance?"

    # Build a simple shingle-based graph on the corpus (same style as LongMemEval)
    print("\nBuilding KDF shingle graph on corpus...", file=sys.stderr)
    from collections import defaultdict

    def shingles(text, k=5):
        text = text.lower()
        return set(text[i:i+k] for i in range(len(text) - k + 1))

    # Inverted index: shingle -> set of doc indices
    shingle_to_docs = defaultdict(list)
    for i, text in enumerate(corpus_texts):
        for s in list(shingles(text))[:100]:  # cap per doc for speed
            shingle_to_docs[s].append(i)

    # Edges: two docs connected if they share a shingle (cap per group)
    edges_set = set()
    for docs in shingle_to_docs.values():
        if len(docs) < 2 or len(docs) > 30:
            continue
        for a_idx in range(len(docs)):
            for b_idx in range(a_idx + 1, min(a_idx + 5, len(docs))):
                a, b = docs[a_idx], docs[b_idx]
                if a != b:
                    edges_set.add((min(a, b), max(a, b)))
    print(f"  edges: {len(edges_set)}", file=sys.stderr)

    # Compute degree for each doc
    degrees = defaultdict(int)
    for (a, b) in edges_set:
        degrees[a] += 1
        degrees[b] += 1

    # KDF-style: rank docs by "inverse degree" (more isolated = higher priority)
    # Query-blind ranking: order all docs by 1/(1 + degree)
    kdf_scores = np.array([1.0 / (1 + degrees.get(i, 0)) for i in range(len(corpus_texts))])
    kdf_ranked = np.argsort(-kdf_scores)

    # For each query, measure: how many relevant docs are in KDF's top-K?
    # This tests whether KDF's query-blind rarity correlates with relevance
    kdf_recall_at_k = {k: 0.0 for k in ks}
    relevant_count = 0
    for qid, qtext in queries.items():
        if qid not in qrels or not qrels[qid]:
            continue
        relevant_cids = [cid for cid, rel in qrels[qid].items() if rel > 0]
        relevant_idx = set(corpus_id_to_idx[cid] for cid in relevant_cids
                          if cid in corpus_id_to_idx)
        if not relevant_idx:
            continue
        for k in ks:
            top_k = set(kdf_ranked[:k].tolist())
            recall = len(top_k & relevant_idx) / len(relevant_idx)
            kdf_recall_at_k[k] += recall
        relevant_count += 1

    print(f"\n## KDF query-blind on SciFact ({relevant_count} queries)")
    print(f"| k | KDF recall@k | BGE recall@k | ratio |")
    print(f"|---:|---:|---:|---:|")
    for k in ks:
        k_kdf = kdf_recall_at_k[k] / relevant_count
        k_bge = recall_at_k[k] / total_queries
        ratio = k_kdf / k_bge if k_bge > 0 else 0
        print(f"| {k} | {k_kdf:.4f} | {k_bge:.4f} | ×{ratio:.2f} |")

    # Random baseline for context
    np.random.seed(42)
    rand_recall_at_k = {k: 0.0 for k in ks}
    for qid, qtext in queries.items():
        if qid not in qrels or not qrels[qid]:
            continue
        relevant_cids = [cid for cid, rel in qrels[qid].items() if rel > 0]
        relevant_idx = set(corpus_id_to_idx[cid] for cid in relevant_cids
                          if cid in corpus_id_to_idx)
        if not relevant_idx:
            continue
        # Random top-K
        idx = np.arange(len(corpus_texts))
        np.random.shuffle(idx)
        for k in ks:
            top_k = set(idx[:k].tolist())
            recall = len(top_k & relevant_idx) / len(relevant_idx)
            rand_recall_at_k[k] += recall

    print(f"\n## Random baseline")
    print(f"| k | Random recall@k |")
    print(f"|---:|---:|")
    for k in ks:
        print(f"| {k} | {rand_recall_at_k[k] / relevant_count:.4f} |")

    # Summary and honest assessment
    k10_kdf = kdf_recall_at_k[10] / relevant_count
    k10_bge = recall_at_k[10] / total_queries
    k10_rand = rand_recall_at_k[10] / relevant_count

    print("\n## 結論 (caveat 1 の honest assessment)")
    print(f"recall@10: KDF={k10_kdf:.3f}, BGE-small={k10_bge:.3f}, Random={k10_rand:.3f}")

    if k10_kdf > k10_bge:
        print("[PASS] KDF beat BGE-small on SciFact; LongMemEval-specific caveat partially resolved")
    elif k10_kdf > k10_rand + 0.01:
        print("[PARTIAL] KDF > Random but < BGE; LongMemEval-specific caveat holds")
        print(f"         (KDF/Random x{k10_kdf/k10_rand:.2f}, KDF/BGE x{k10_kdf/k10_bge:.2f})")
    else:
        print("[FAIL] KDF ~= Random; general retrieval requires query-aware methods.")
        print("       LongMemEval result does NOT generalize to semantic retrieval tasks.")

    out_path = Path('demos/D8_llm_memory/out/scifact_results.json')
    with open(out_path, 'w') as f:
        json.dump({
            'n_test_queries': total_queries,
            'ks': ks,
            'bge_small_recall_at_k': {str(k): recall_at_k[k] / total_queries for k in ks},
            'kdf_recall_at_k': {str(k): kdf_recall_at_k[k] / relevant_count for k in ks},
            'random_recall_at_k': {str(k): rand_recall_at_k[k] / relevant_count for k in ks},
        }, f, indent=2)

if __name__ == '__main__':
    main()
