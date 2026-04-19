#!/usr/bin/env python3
"""
Phase M2 - PKM multi-corpus suitability test.

P2 (F-012/F-017) proved KDF works on the inventor's own Obsidian vault
(2,182 notes). The open question: does this generalize to other PKM-shape
corpora, or is it specific to that one vault?

This test uses the **Welsh Wikipedia** pilot data from V6 (50 articles)
as a proxy "minority-language PKM corpus":
  - Rare ground truth = Welsh-only articles (no enwiki link, cultural
    concepts specific to Welsh)
  - Majority = cross-lingual articles

If KDF preserves Welsh-only minority at a rate significantly above random,
PKM suitability is reinforced.
"""

import json, sys
from pathlib import Path
from collections import defaultdict

def main():
    # Get Welsh articles and their QIDs (pre-fetched in V6 via Rust)
    # We'll re-fetch live to keep the script self-contained, limited to 30 to
    # avoid rate-limit concerns.
    import urllib.request
    import urllib.parse

    UA = "KDF-research/0.1 (github.com/ChaiCroquis/kdf-perovskite)"
    def fetch(u):
        req = urllib.request.Request(u, headers={'User-Agent': UA})
        with urllib.request.urlopen(req) as r:
            return json.loads(r.read())

    print("Phase M2 - PKM multi-corpus suitability test\n", file=sys.stderr)
    print("Fetching 30 random Welsh Wikipedia articles + their Wikidata QIDs...", file=sys.stderr)

    # Get random Welsh article titles
    url = "https://cy.wikipedia.org/w/api.php?action=query&list=random&rnnamespace=0&rnlimit=30&format=json"
    data = fetch(url)
    titles = [a['title'] for a in data['query']['random']]

    # Get Wikidata QIDs
    titles_param = '|'.join(titles)
    encoded = urllib.parse.quote(titles_param)
    url2 = f"https://cy.wikipedia.org/w/api.php?action=query&titles={encoded}&prop=pageprops&ppprop=wikibase_item&format=json"
    data2 = fetch(url2)

    title_to_qid = {}
    for _, page in data2['query']['pages'].items():
        title = page.get('title')
        props = page.get('pageprops', {})
        qid = props.get('wikibase_item')
        if title and qid:
            title_to_qid[title] = qid

    # Check enwiki sitelinks via Wikidata
    qids = list(title_to_qid.values())
    ids_param = '|'.join(qids)
    url3 = f"https://www.wikidata.org/w/api.php?action=wbgetentities&ids={ids_param}&props=sitelinks&sitefilter=enwiki&format=json"
    data3 = fetch(url3)

    qid_has_enwiki = {}
    for qid, entity in data3.get('entities', {}).items():
        has_en = 'enwiki' in entity.get('sitelinks', {})
        qid_has_enwiki[qid] = has_en

    # Now get article abstracts (short text) for graph construction
    # Use the extracts API (short summary per article)
    print("Fetching article extracts...", file=sys.stderr)
    article_data = {}
    for i in range(0, len(titles), 20):
        batch_titles = titles[i:i+20]
        batch_param = '|'.join(batch_titles)
        encoded = urllib.parse.quote(batch_param)
        url4 = f"https://cy.wikipedia.org/w/api.php?action=query&titles={encoded}&prop=extracts&exintro=1&explaintext=1&exlimit=20&format=json"
        data4 = fetch(url4)
        for _, page in data4.get('query', {}).get('pages', {}).items():
            title = page.get('title')
            extract = page.get('extract', '')
            if title and extract:
                article_data[title] = extract

    # Build corpus
    corpus_titles = list(article_data.keys())
    corpus_texts = [article_data[t] for t in corpus_titles]
    n = len(corpus_texts)

    # Rare ground truth = Welsh-only (no enwiki sitelink)
    rare_indices = []
    for i, title in enumerate(corpus_titles):
        qid = title_to_qid.get(title)
        if qid and not qid_has_enwiki.get(qid, True):
            rare_indices.append(i)

    print(f"Corpus: {n} articles", file=sys.stderr)
    print(f"Welsh-only (minority): {len(rare_indices)} / {n}", file=sys.stderr)
    if not rare_indices:
        print("No minority articles this run; aborting", file=sys.stderr)
        sys.exit(0)

    # Build shingle graph
    def shingles(text, k=4):
        t = text.lower()
        return set(t[i:i+k] for i in range(len(t) - k + 1))

    shingle_to_doc = defaultdict(list)
    for i, text in enumerate(corpus_texts):
        for s in list(shingles(text))[:200]:
            shingle_to_doc[s].append(i)

    edges = set()
    for docs in shingle_to_doc.values():
        if len(docs) < 2 or len(docs) > 10: continue
        for a in range(len(docs)):
            for b in range(a+1, min(a+3, len(docs))):
                edges.add((min(docs[a], docs[b]), max(docs[a], docs[b])))

    # Compute degrees
    degrees = [0] * n
    for a, b in edges:
        degrees[a] += 1
        degrees[b] += 1

    # Simulate KDF-like "keep top-K" decision by structural rarity:
    # deg=1 nodes are "Rare-protected"; deg=0 are "Garbage"; else "Edge".
    # KDF-compatible retention strategy: keep all Rare + top-K of Edge by weight.
    keep_rate = 0.50  # compress to 50%
    keep_count = int(n * keep_rate + 0.5)

    # Strategy A: KDF-structural - prioritize deg=1 (rare), then deg increasing
    sorted_by_rarity = sorted(range(n), key=lambda i: (abs(degrees[i] - 1) if degrees[i] > 0 else 999, degrees[i]))
    kdf_kept = set(sorted_by_rarity[:keep_count])
    kdf_rare_retained = len(kdf_kept & set(rare_indices))
    kdf_recall = kdf_rare_retained / len(rare_indices)

    # Strategy B: Random (baseline)
    import random
    random.seed(42)
    rand_recalls = []
    for trial in range(20):
        random.seed(42 + trial)
        rand_kept = set(random.sample(range(n), keep_count))
        rand_rare_retained = len(rand_kept & set(rare_indices))
        rand_recalls.append(rand_rare_retained / len(rare_indices))
    rand_recall = sum(rand_recalls) / len(rand_recalls)

    print(f"\n=== M2 PKM Welsh-article results ===")
    print(f"n_corpus = {n}, n_rare (Welsh-only) = {len(rare_indices)}")
    print(f"n_edges (shingle) = {len(edges)}")
    print(f"keep_rate = {keep_rate*100:.0f}%, keep_count = {keep_count}")
    print()
    print(f"KDF-like structural rarity retention: recall = {kdf_recall:.3f}")
    print(f"Random retention (20 seeds avg):      recall = {rand_recall:.3f}")
    ratio = kdf_recall / max(rand_recall, 1e-9)
    print(f"ratio (KDF/Random):                   x{ratio:.2f}")
    print()

    # Verdict
    if ratio >= 1.1:
        verdict = "PKM suitability CONFIRMED on 2nd corpus"
    elif ratio >= 1.0:
        verdict = "PKM suitability marginal; not brittle but not strong either"
    else:
        verdict = "PKM suitability NOT confirmed on 2nd corpus (KDF <= Random)"
    print(f"=> {verdict}")

    # Dump
    out = Path("demos/D8_llm_memory/out/m2_pkm_welsh_results.json")
    out.parent.mkdir(parents=True, exist_ok=True)
    with open(out, 'w', encoding='utf-8') as f:
        json.dump({
            "n_corpus": n,
            "n_rare": len(rare_indices),
            "n_edges": len(edges),
            "kdf_recall": kdf_recall,
            "random_recall_mean": rand_recall,
            "ratio": ratio,
            "verdict": verdict,
        }, f, indent=2)

if __name__ == '__main__':
    main()
