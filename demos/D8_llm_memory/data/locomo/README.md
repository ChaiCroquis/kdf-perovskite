# LoCoMo data (not redistributed)

The LoCoMo benchmark (Snap Research, ACL 2024) is licensed CC BY-NC 4.0
by its authors; we do not redistribute it from this repository.

## To reproduce F-056 (W5 LoCoMo benchmark)

1. Download the dataset:
   ```bash
   curl -sL -o demos/D8_llm_memory/data/locomo/locomo10.json \
       https://raw.githubusercontent.com/snap-research/locomo/main/data/locomo10.json
   ```
2. Generate the sampled oracle (200 Q balanced across 4 non-adversarial categories):
   ```bash
   python demos/D8_llm_memory/scripts/locomo_adapter.py --n 200
   ```
   → writes `locomo_oracle_sampled.json` in this directory.
3. Dump real KDF selected turns:
   ```bash
   cargo run --release -p demo-d8-llm-memory --bin phase_w3_real_kdf_turns -- \
       --keep-rate 0.30 --method KDF \
       --input demos/D8_llm_memory/data/locomo/locomo_oracle_sampled.json \
       --out demos/D8_llm_memory/out/w5_locomo_real_kdf_turns_030.json
   ```
4. Run the Mem0 vs Real-KDF benchmark (requires OPENAI_API_KEY):
   ```bash
   python demos/D8_llm_memory/scripts/w5_locomo_mem0_vs_kdf.py
   ```
   → ~105 min runtime, ~$0.20 on gpt-4o-mini

## Dataset license

- LoCoMo: CC BY-NC 4.0
- Citation: Maharana, A. et al. "Evaluating Very Long-Term Conversational
  Memory of LLM Agents." ACL 2024.
  [arXiv:2402.17753](https://arxiv.org/abs/2402.17753)
- Upstream: https://github.com/snap-research/locomo
