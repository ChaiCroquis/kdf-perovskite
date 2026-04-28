//! Generic KDF graph selection binary — takes an arbitrary graph (nodes + edges)
//! as JSON and returns KDF-selected node IDs. This enables validation
//! experiments (classical algorithm revival, git pruning, etc.) without
//! requiring domain-specific Rust wiring.
//!
//! Input JSON schema:
//!   {
//!     "n": <u32>,
//!     "edges": [[u, v, w], ...],  // u, v: u32 node ids; w: f64 weight
//!     "node_ids": ["id1", "id2", ...]  // optional: original ids for mapping
//!   }
//!
//! Output JSON schema:
//!   {
//!     "selected_node_indices": [idx1, idx2, ...],  // sorted
//!     "selected_node_ids": ["id1", ...],           // if node_ids provided
//!     "layers": {"Rare": [...], "Core": [...], "Edge": [...], "Garbage": [...]},
//!     "method": "KDF",
//!     "keep_rate": 0.30,
//!     "n_kept": N,
//!     "n_total": M
//!   }
//!
//! Usage:
//!   cargo run --release -p demo-d8-llm-memory --bin kdf_select_generic -- \
//!       --input graph.json --out selected.json --keep-rate 0.30

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize)]
struct GraphInput {
    n: u32,
    edges: Vec<(u32, u32, f64)>,
    #[serde(default)]
    node_ids: Option<Vec<String>>,
}

#[derive(Serialize)]
struct SelectionOutput {
    selected_node_indices: Vec<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selected_node_ids: Option<Vec<String>>,
    layers: HashMap<String, Vec<u32>>,
    method: String,
    keep_rate: f64,
    n_kept: usize,
    n_total: u32,
}

fn kdf_select(
    n: u32,
    edges: &[(u32, u32, f64)],
    keep: usize,
) -> (Vec<u32>, HashMap<String, Vec<u32>>) {
    use cgb_kdf::{Layer, NodeClassifier};
    let mut c = NodeClassifier::default();
    let class = c.classify(n as usize, edges);

    let score = |l: Layer| -> i32 {
        match l {
            Layer::Rare => 3,
            Layer::Core => 2,
            Layer::Edge => 1,
            Layer::Garbage => 0,
        }
    };

    let mut scored: Vec<(u32, i32)> = (0..n)
        .map(|id| {
            (
                id,
                score(class.layers.get(&id).copied().unwrap_or(Layer::Edge)),
            )
        })
        .collect();
    scored.sort_by_key(|x| (std::cmp::Reverse(x.1), x.0));

    let selected: Vec<u32> = scored.iter().take(keep).map(|(id, _)| *id).collect();

    let mut layers_map: HashMap<String, Vec<u32>> = HashMap::new();
    for (id, layer) in &class.layers {
        let name = match layer {
            Layer::Rare => "Rare",
            Layer::Core => "Core",
            Layer::Edge => "Edge",
            Layer::Garbage => "Garbage",
        };
        layers_map.entry(name.to_string()).or_default().push(*id);
    }
    for v in layers_map.values_mut() {
        v.sort();
    }

    let mut selected_sorted = selected.clone();
    selected_sorted.sort();
    (selected_sorted, layers_map)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut input_path: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut keep_rate: f64 = 0.30;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--input" => {
                input_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--out" => {
                out_path = Some(args[i + 1].clone());
                i += 2;
            }
            "--keep-rate" => {
                keep_rate = args[i + 1].parse().expect("keep_rate must be float");
                i += 2;
            }
            _ => i += 1,
        }
    }
    let input_path = input_path.expect("--input required");
    let out_path = out_path.expect("--out required");

    eprintln!("Loading {} ...", input_path);
    let raw = std::fs::read_to_string(&input_path).expect("read input");
    let input: GraphInput = serde_json::from_str(&raw).expect("parse JSON");
    eprintln!("Graph: n={}, edges={}", input.n, input.edges.len());

    let keep = ((input.n as f64) * keep_rate).ceil() as usize;
    let keep = keep.max(1).min(input.n as usize);
    eprintln!(
        "keep={} ({}%)",
        keep,
        (keep as f64) / (input.n as f64) * 100.0
    );

    let (selected, layers_map) = kdf_select(input.n, &input.edges, keep);
    eprintln!("Selected {} nodes", selected.len());
    eprintln!(
        "Layers: Rare={}, Core={}, Edge={}, Garbage={}",
        layers_map.get("Rare").map(|v| v.len()).unwrap_or(0),
        layers_map.get("Core").map(|v| v.len()).unwrap_or(0),
        layers_map.get("Edge").map(|v| v.len()).unwrap_or(0),
        layers_map.get("Garbage").map(|v| v.len()).unwrap_or(0)
    );

    let selected_ids: Option<Vec<String>> = input.node_ids.as_ref().map(|ids| {
        selected
            .iter()
            .filter_map(|&idx| ids.get(idx as usize).cloned())
            .collect()
    });

    let output = SelectionOutput {
        selected_node_indices: selected,
        selected_node_ids: selected_ids,
        layers: layers_map,
        method: "KDF".to_string(),
        keep_rate,
        n_kept: keep,
        n_total: input.n,
    };

    let json = serde_json::to_string(&output).expect("serialize");
    std::fs::write(&out_path, json).expect("write output");
    eprintln!("Wrote: {}", out_path);
    println!("{}", out_path);
}
