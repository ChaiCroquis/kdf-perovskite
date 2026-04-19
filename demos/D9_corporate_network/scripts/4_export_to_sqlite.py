"""
D9 Step 4: Export institutions + edges to SQLite with Obsidian-inspired schema.

Idea: Use Obsidian's semantic model (notes + tags + links) but store in SQLite
for indexed querying and minimal storage overhead. No Obsidian app required.

Schema(obsidian-flavored):
  - notes: id, name, note_type, body_json, last_updated
  - tags:  note_id, tag              (e.g. 'field/AI_ML', 'country/JP')
  - edges: src_id, tgt_id, edge_type, weight

Query power examples (tag-filtered sub-graph extraction):
  "Get all institutions tagged 'field/AI_ML' AND 'country/JP' and their edges"
  → small sub-graph → re-run KDF → local boundary spanners in Japanese AI research

Cost: $0. File size: 5.6 MB JSON → ~500 KB SQLite (estimated 10x reduction).
"""
from __future__ import annotations

import json
import sqlite3
import sys
from collections import defaultdict
from itertools import combinations
from pathlib import Path

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

DB_PATH = Path("demos/D9_corporate_network/out/corporate_graph.db")


SCHEMA = """
CREATE TABLE IF NOT EXISTS notes (
    id            TEXT PRIMARY KEY,
    name          TEXT,
    note_type     TEXT,
    body_json     TEXT,
    last_updated  TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    note_id TEXT,
    tag     TEXT,
    PRIMARY KEY (note_id, tag),
    FOREIGN KEY (note_id) REFERENCES notes(id)
);
CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);

CREATE TABLE IF NOT EXISTS edges (
    src_id    TEXT,
    tgt_id    TEXT,
    edge_type TEXT,
    weight    REAL,
    PRIMARY KEY (src_id, tgt_id, edge_type),
    FOREIGN KEY (src_id) REFERENCES notes(id),
    FOREIGN KEY (tgt_id) REFERENCES notes(id)
);
CREATE INDEX IF NOT EXISTS idx_edges_src ON edges(src_id);
CREATE INDEX IF NOT EXISTS idx_edges_tgt ON edges(tgt_id);
CREATE INDEX IF NOT EXISTS idx_edges_type ON edges(edge_type);

CREATE TABLE IF NOT EXISTS metadata (
    key   TEXT PRIMARY KEY,
    value TEXT
);
"""


def build_tags_for_institution(inst: dict) -> list[str]:
    """Produce Obsidian-style hierarchical tags for one institution note."""
    tags = []
    country = inst.get("country")
    if country:
        tags.append(f"country/{country}")
    ty = inst.get("type")
    if ty:
        tags.append(f"type/{ty}")
    for f in inst.get("fields_spanned", []):
        tags.append(f"field/{f}")
    layer = inst.get("kdf30_layer")
    if layer:
        tags.append(f"kdf/{layer.lower()}")
    nf = inst.get("n_fields", 0)
    if nf >= 3:
        tags.append("span/multi_3plus")
    if nf == 4:
        tags.append("span/all_4")
    # Degree bucket
    deg = inst.get("degree", 0)
    if deg == 0:
        tags.append("deg/zero")
    elif deg <= 10:
        tags.append("deg/low_1_10")
    elif deg <= 100:
        tags.append("deg/mid_11_100")
    else:
        tags.append("deg/high_100plus")
    return tags


def main():
    DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    if DB_PATH.exists():
        DB_PATH.unlink()

    conn = sqlite3.connect(DB_PATH)
    conn.executescript(SCHEMA)

    # Load institutions
    print("Loading institutions_ranked.json...")
    with open("demos/D9_corporate_network/out/institutions_ranked.json", encoding="utf-8") as f:
        inst_data = json.load(f)
    institutions = inst_data["institutions"]
    print(f"  {len(institutions)} institutions")

    # Insert notes + tags
    print("Inserting notes + tags...")
    cur = conn.cursor()
    tag_rows = []
    for inst in institutions:
        body = {
            "n_fields": inst.get("n_fields"),
            "n_papers": inst.get("n_papers"),
            "degree": inst.get("degree"),
            "fields_spanned": inst.get("fields_spanned"),
            "kdf30_layer": inst.get("kdf30_layer"),
            "in_kdf30": inst.get("in_kdf30"),
            "in_kdf50": inst.get("in_kdf50"),
            "in_topdegree30": inst.get("in_topdegree30"),
            "boundary_score": inst.get("boundary_score"),
        }
        cur.execute(
            "INSERT OR REPLACE INTO notes (id, name, note_type, body_json, last_updated) VALUES (?,?,?,?,date('now'))",
            (inst["id"], inst.get("name"), "institution", json.dumps(body, ensure_ascii=False))
        )
        for tag in build_tags_for_institution(inst):
            tag_rows.append((inst["id"], tag))
    cur.executemany("INSERT OR IGNORE INTO tags (note_id, tag) VALUES (?,?)", tag_rows)
    conn.commit()
    print(f"  {cur.execute('SELECT COUNT(*) FROM tags').fetchone()[0]} tag rows")

    # Rebuild edges from papers
    print("Rebuilding co-authorship edges from papers...")
    with open("demos/D9_corporate_network/out/papers_raw.json", encoding="utf-8") as f:
        papers = json.load(f)
    edge_weights = defaultdict(float)
    for paper in papers:
        paper_insts = set()
        for a in paper.get("authorships", []):
            for inst in a.get("institutions", []):
                iid = inst.get("id")
                if iid:
                    paper_insts.add(iid)
        for u, v in combinations(sorted(paper_insts), 2):
            edge_weights[(u, v)] += 1
    cur.executemany(
        "INSERT OR REPLACE INTO edges (src_id, tgt_id, edge_type, weight) VALUES (?,?,?,?)",
        ((u, v, "coauthored", w) for (u, v), w in edge_weights.items())
    )
    conn.commit()
    n_edges = cur.execute("SELECT COUNT(*) FROM edges").fetchone()[0]
    print(f"  {n_edges} coauthored edges")

    # Metadata
    cur.execute("INSERT OR REPLACE INTO metadata VALUES ('source', 'OpenAlex 2020-2024 top cited, 4 fields × 500 papers')")
    cur.execute("INSERT OR REPLACE INTO metadata VALUES ('n_papers', ?)", (str(len(papers)),))
    conn.commit()
    conn.close()

    size_kb = DB_PATH.stat().st_size / 1024
    print(f"\nDB file: {DB_PATH} ({size_kb:.1f} KB)")

    # Compare
    import os
    json_size = os.path.getsize("demos/D9_corporate_network/out/institutions_ranked.json") / 1024
    papers_size = os.path.getsize("demos/D9_corporate_network/out/papers_raw.json") / 1024
    print(f"  Original JSON: institutions {json_size:.0f} KB + papers {papers_size:.0f} KB = {json_size+papers_size:.0f} KB")
    print(f"  SQLite:        {size_kb:.0f} KB  ({size_kb/(json_size+papers_size)*100:.1f}% of JSON)")


if __name__ == "__main__":
    main()
