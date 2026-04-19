"""
D9 Step 6: Fetch T1 (past) period papers for backtest validation.

T1: 2014-2018 (5 years, past)
T2: 2020-2024 (5 years, "present" — already fetched as papers_raw.json)

Same 4 fields, same sampling strategy. Used by step 7 to compute
transition base rates: T1 KDF-layer × T2 outcome.

Cost: $0. Runtime: ~2-3 min (OpenAlex rate limit).
"""
from __future__ import annotations

import json
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

# Same fields as T2
FIELDS = {
    "AI_ML": "C154945302",
    "Materials_SemiCond": "C192562407",
    "Biomed_Pharma": "C71924100",
    "Automotive": "C127413603",
}
PER_FIELD = 500
OPENALEX_BASE = "https://api.openalex.org/works"


def _strip(s):
    if s is None:
        return ""
    return str(s).replace("https://openalex.org/", "")


def fetch_field_range(concept_id, date_from, date_to, limit):
    papers = []
    per_page = 200
    cursor = "*"
    remaining = limit
    while remaining > 0:
        params = {
            "filter": f"concepts.id:{concept_id},from_publication_date:{date_from},to_publication_date:{date_to}",
            "sort": "cited_by_count:desc",
            "per-page": min(per_page, remaining),
            "cursor": cursor,
        }
        url = f"{OPENALEX_BASE}?{urllib.parse.urlencode(params)}"
        req = urllib.request.Request(url, headers={"User-Agent": "kdf-research-exp/0.1 (mailto:garden.of.knowledge.chai@gmail.com)"})
        with urllib.request.urlopen(req, timeout=60) as resp:
            data = json.load(resp)
        results = data.get("results", [])
        if not results:
            break
        papers.extend(results)
        remaining -= len(results)
        next_cursor = data.get("meta", {}).get("next_cursor")
        if not next_cursor or next_cursor == cursor:
            break
        cursor = next_cursor
        time.sleep(0.3)
    return papers[:limit]


def extract(paper, field_tag):
    authorships = []
    for a in paper.get("authorships", []) or []:
        author = a.get("author") or {}
        insts = []
        for inst in a.get("institutions", []) or []:
            iid = inst.get("id")
            if not iid:
                continue
            insts.append({
                "id": _strip(iid),
                "name": inst.get("display_name"),
                "country": inst.get("country_code"),
                "type": inst.get("type"),
            })
        authorships.append({
            "author_id": _strip(author.get("id")),
            "author_name": author.get("display_name"),
            "institutions": insts,
        })
    return {
        "id": _strip(paper.get("id")),
        "title": paper.get("title") or paper.get("display_name"),
        "publication_year": paper.get("publication_year"),
        "cited_by_count": paper.get("cited_by_count"),
        "field_tag": field_tag,
        "authorships": authorships,
    }


def main():
    out_dir = Path("demos/D9_corporate_network/out")
    out_dir.mkdir(parents=True, exist_ok=True)

    all_papers = []
    for field, cid in FIELDS.items():
        print(f"Fetching T1={2014}-{2018} for {field}...", file=sys.stderr)
        papers = fetch_field_range(cid, "2014-01-01", "2018-12-31", PER_FIELD)
        print(f"  got {len(papers)}", file=sys.stderr)
        for p in papers:
            all_papers.append(extract(p, field))

    out = out_dir / "papers_t1_2014_2018.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump(all_papers, f, indent=None, ensure_ascii=False)
    print(f"\nSaved T1 data: {out} ({len(all_papers)} papers)")

    # Unique institutions
    insts = set()
    for p in all_papers:
        for a in p["authorships"]:
            for i in a["institutions"]:
                insts.add(i["id"])
    print(f"Unique T1 institutions: {len(insts)}")


if __name__ == "__main__":
    main()
