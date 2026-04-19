//! Public dataset loaders.
//!
//! Data files are NOT redistributed with this repo. Place them under
//! `benchmarks/real_data/data/` according to the schemas below.
//! Any missing file yields `None` so the bench skips that dataset gracefully.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::Dataset;

fn data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// FB15K-237 loader.
///
/// Expected files under `data/fb15k-237/`:
///   train.txt / valid.txt / test.txt   (tab-separated: head \t relation \t tail)
///
/// Rare ground truth: relations whose global frequency ≤ `rare_freq_max`.
/// Nodes = entities (head/tail union).
pub fn load_fb15k_237(rare_freq_max: usize) -> Option<Dataset> {
    let root = data_root().join("fb15k-237");
    let files = ["train.txt", "valid.txt", "test.txt"];
    let mut all_rows: Vec<(String, String, String)> = Vec::new();
    for f in files {
        let p = root.join(f);
        if !p.exists() { return None; }
        for line in std::fs::read_to_string(&p).ok()?.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() == 3 {
                all_rows.push((parts[0].to_string(), parts[1].to_string(), parts[2].to_string()));
            }
        }
    }

    let mut entity_ids: HashMap<String, u32> = HashMap::new();
    let mut relation_counts: HashMap<String, u32> = HashMap::new();
    for (h, r, t) in &all_rows {
        let next = entity_ids.len() as u32;
        entity_ids.entry(h.clone()).or_insert(next);
        let next = entity_ids.len() as u32;
        entity_ids.entry(t.clone()).or_insert(next);
        *relation_counts.entry(r.clone()).or_insert(0) += 1;
    }

    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut rare_entities: HashSet<u32> = HashSet::new();
    for (h, r, t) in &all_rows {
        let hu = *entity_ids.get(h).unwrap();
        let tu = *entity_ids.get(t).unwrap();
        edges.push((hu, tu, 1.0));
        if relation_counts[r] as usize <= rare_freq_max {
            rare_entities.insert(hu);
            rare_entities.insert(tu);
        }
    }

    Some(Dataset {
        name: "FB15K-237".to_string(),
        n_nodes: entity_ids.len(),
        edges,
        rare_ground_truth: rare_entities,
        description: format!(
            "FB15K-237 with rare = entities touching relations of frequency ≤ {}",
            rare_freq_max
        ),
    })
}

/// ogbn-arxiv loader.
///
/// Expected files under `data/ogbn-arxiv/`:
///   edges.csv          (source_id,target_id)
///   node_year.csv      (node_id,year) — optional
///   citation_count.csv (node_id,citations) — rare truth = citations ≤ N
pub fn load_ogbn_arxiv(rare_citation_max: usize) -> Option<Dataset> {
    let root = data_root().join("ogbn-arxiv");
    let edges_p = root.join("edges.csv");
    let cites_p = root.join("citation_count.csv");
    if !edges_p.exists() || !cites_p.exists() { return None; }

    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut max_id = 0u32;
    for (i, line) in std::fs::read_to_string(&edges_p).ok()?.lines().enumerate() {
        if i == 0 { continue; } // header
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 2 { continue; }
        let s: u32 = parts[0].parse().ok()?;
        let t: u32 = parts[1].parse().ok()?;
        max_id = max_id.max(s).max(t);
        edges.push((s, t, 1.0));
    }

    let mut rare_ground_truth: HashSet<u32> = HashSet::new();
    for (i, line) in std::fs::read_to_string(&cites_p).ok()?.lines().enumerate() {
        if i == 0 { continue; }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() != 2 { continue; }
        let id: u32 = parts[0].parse().ok()?;
        let c: usize = parts[1].parse().unwrap_or(0);
        if c <= rare_citation_max { rare_ground_truth.insert(id); }
    }

    Some(Dataset {
        name: "ogbn-arxiv".to_string(),
        n_nodes: (max_id as usize) + 1,
        edges,
        rare_ground_truth,
        description: format!("ogbn-arxiv with rare = papers with citations ≤ {}", rare_citation_max),
    })
}

/// NASA HTTP log loader (simplified).
///
/// Expected file: `data/nasa-http/access.log` (Common Log Format).
/// Graph: nodes = client IPs ∪ requested resources, edges = hits.
/// Rare ground truth: resources with status codes in `rare_status_codes`
/// (e.g., 4xx/5xx rare errors).
pub fn load_nasa_log(rare_status_codes: &HashSet<u16>) -> Option<Dataset> {
    let p = data_root().join("nasa-http").join("access.log");
    if !p.exists() { return None; }
    let mut entity_ids: HashMap<String, u32> = HashMap::new();
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut rare: HashSet<u32> = HashSet::new();
    let re = regex::Regex::new(r#"^(\S+) .* "(?:GET|POST|HEAD) (\S+) .*" (\d+) "#).unwrap();
    for line in std::fs::read_to_string(&p).ok()?.lines() {
        if let Some(cap) = re.captures(line) {
            let client = cap[1].to_string();
            let resource = cap[2].to_string();
            let status: u16 = cap[3].parse().unwrap_or(0);
            let next = entity_ids.len() as u32;
            let src = *entity_ids.entry(format!("ip:{}", client)).or_insert(next);
            let next = entity_ids.len() as u32;
            let dst = *entity_ids.entry(format!("res:{}", resource)).or_insert(next);
            edges.push((src, dst, 1.0));
            if rare_status_codes.contains(&status) { rare.insert(dst); }
        }
    }
    Some(Dataset {
        name: "NASA-HTTP".to_string(),
        n_nodes: entity_ids.len(),
        edges,
        rare_ground_truth: rare,
        description: "NASA HTTP log; rare = resources triggering 4xx/5xx".to_string(),
    })
}

pub fn download_instructions() -> &'static str {
    "
Phase 6 public datasets are NOT redistributed. To enable benchmarks, place:

  benchmarks/real_data/data/fb15k-237/{train,valid,test}.txt
     https://www.microsoft.com/en-us/download/details.aspx?id=52312

  benchmarks/real_data/data/ogbn-arxiv/{edges.csv, citation_count.csv}
     via `ogb` Python package, or mirrored release

  benchmarks/real_data/data/nasa-http/access.log
     https://ita.ee.lbl.gov/html/contrib/NASA-HTTP.html
"
}
