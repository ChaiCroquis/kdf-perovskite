"""Variant: emit ALL 321 non-adversarial temporal (category=2) LoCoMo Q for W5b reproducibility check."""
import json, sys
from locomo_adapter import convert_sample

if sys.platform == "win32":
    sys.stdout.reconfigure(encoding="utf-8")

with open("demos/D8_llm_memory/data/locomo/locomo10.json", encoding="utf-8") as f:
    data = json.load(f)

all_q = []
for idx, sample in enumerate(data):
    _, q_entries = convert_sample(sample, idx)
    all_q.extend(q_entries)

temporal = [q for q in all_q if q["_locomo_category"] == 2]
print(f"Temporal (cat=2) non-adversarial: {len(temporal)}", file=sys.stderr)

with open("demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json", "w", encoding="utf-8") as f:
    json.dump(temporal, f, ensure_ascii=False)
print("Wrote demos/D8_llm_memory/data/locomo/locomo_oracle_temporal_all.json", file=sys.stderr)
