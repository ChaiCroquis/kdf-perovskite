"""
D9: Corporate Network Boundary-Spanner Detection (遊び検証)

Step 1: Fetch recent research papers from OpenAlex across multiple fields.

Approach: Use research papers as a proxy for patents — co-authorship is
structurally analogous to co-inventorship. Institutions (companies +
universities) that co-author across MULTIPLE research fields are
"boundary spanners" = KDF's structural-rarity target per F-061/F-062.

Fields targeted (chosen for Japanese industry relevance):
  - AI / machine learning
  - Semiconductor / materials science
  - Biomedical / pharmaceutical
  - Automotive / mobility

Output: demos/D9_corporate_network/out/papers_raw.json
Cost: $0 (OpenAlex is free, no auth).
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

# OpenAlex concept IDs (from https://api.openalex.org/concepts)
# Picked 4 fields with strong Japanese industrial presence
FIELDS = {
    "AI_ML": "C154945302",           # Artificial intelligence
    "Materials_SemiCond": "C192562407",  # Materials science
    "Biomed_Pharma": "C71924100",    # Medicine
    "Automotive": "C127413603",      # Engineering (automotive/mobility proxy)
}

# Limit to recent papers, English OR Japanese, cap per field
PER_FIELD_LIMIT = 500  # top cited in the last 5 years in that field
PUBLICATION_YEAR_MIN = 2020

OPENALEX_BASE = "https://api.openalex.org/works"


def fetch_field(field_name: str, concept_id: str, limit: int = 500) -> list[dict]:
    """Fetch top cited papers in the given field from last 5 years."""
    papers = []
    per_page = 200  # OpenAlex max
    cursor = "*"
    remaining = limit
    while remaining > 0:
        params = {
            "filter": f"concepts.id:{concept_id},from_publication_date:2020-01-01,to_publication_date:2024-12-31",
            "sort": "cited_by_count:desc",
            "per-page": min(per_page, remaining),
            "cursor": cursor,
        }
        url = f"{OPENALEX_BASE}?{urllib.parse.urlencode(params)}"
        req = urllib.request.Request(
            url,
            headers={"User-Agent": "kdf-research-exp/0.1 (mailto:garden.of.knowledge.chai@gmail.com)"},
        )
        print(f"  [GET] {field_name}: have {len(papers)}, fetching next {min(per_page, remaining)}...", file=sys.stderr)
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
        time.sleep(0.3)  # polite rate limit
    return papers[:limit]


def _strip_prefix(s) -> str:
    if s is None:
        return ""
    return str(s).replace("https://openalex.org/", "")


def extract_compact(paper: dict, field_tag: str) -> dict:
    """Extract only the fields we need (defensive vs None)."""
    authorships = []
    for a in paper.get("authorships", []) or []:
        author = a.get("author") or {}
        insts_out = []
        for inst in a.get("institutions", []) or []:
            inst_id = inst.get("id")
            if not inst_id:
                continue
            insts_out.append({
                "id": _strip_prefix(inst_id),
                "name": inst.get("display_name"),
                "country": inst.get("country_code"),
                "type": inst.get("type"),
            })
        authorships.append({
            "author_id": _strip_prefix(author.get("id")),
            "author_name": author.get("display_name"),
            "institutions": insts_out,
        })
    return {
        "id": _strip_prefix(paper.get("id")),
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
    for field_name, concept_id in FIELDS.items():
        print(f"\n=== Field: {field_name} (concept {concept_id}) ===")
        papers = fetch_field(field_name, concept_id, limit=PER_FIELD_LIMIT)
        print(f"  fetched {len(papers)} papers")
        for p in papers:
            all_papers.append(extract_compact(p, field_name))

    out = out_dir / "papers_raw.json"
    with out.open("w", encoding="utf-8") as f:
        json.dump(all_papers, f, indent=None, ensure_ascii=False)
    print(f"\nSaved: {out}")
    print(f"Total papers: {len(all_papers)}")
    # Quick stats
    unique_insts = set()
    for p in all_papers:
        for a in p["authorships"]:
            for inst in a["institutions"]:
                unique_insts.add((inst["id"], inst["name"], inst["country"], inst["type"]))
    print(f"Unique institutions: {len(unique_insts)}")
    jp_count = sum(1 for i in unique_insts if i[2] == "JP")
    print(f"  Japanese institutions: {jp_count}")


if __name__ == "__main__":
    main()
