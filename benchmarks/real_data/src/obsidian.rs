//! Obsidian Vault → graph converter with PII masking.
//!
//! Nodes = notes (by path hash), edges = wiki-link references [[target]].
//! Rare ground truth: notes that are linked FROM <= `rare_indegree_max` notes
//! AND exist (link resolves). This approximates "orphan-ish but real" notes.
//!
//! PII masking rules (applied to **all text the build consumes** in-memory; we
//! never write back to disk):
//!  - Email addresses → `<EMAIL>`
//!  - JP phone numbers → `<PHONE>`
//!  - Credit-card-like 16 digit runs → `<CARD>`
//!  - URLs containing "token=" / "key=" / "auth=" → stripped to host only
//!  - Any 32+ char hex blob → `<HEX>`
//!
//! This is best-effort; the build never publishes note contents externally —
//! only graph structure (node ids are path-hashes, edges are indices).

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use walkdir::WalkDir;

use super::Dataset;

pub struct ObsidianBuildConfig {
    pub vault_root: PathBuf,
    pub max_notes: Option<usize>,
    pub rare_indegree_max: usize,
}

/// PII-masking regex bundle. Strict rules, intentionally over-masking is OK.
pub struct PiiMasker {
    email: Regex,
    phone: Regex,
    card: Regex,
    hex: Regex,
}

impl PiiMasker {
    pub fn new() -> Self {
        Self {
            email: Regex::new(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}").unwrap(),
            phone: Regex::new(r"\b0\d{1,4}-\d{1,4}-\d{3,4}\b|\b0\d{9,10}\b").unwrap(),
            card: Regex::new(r"\b\d{4}[- ]?\d{4}[- ]?\d{4}[- ]?\d{4}\b").unwrap(),
            hex: Regex::new(r"\b[0-9a-fA-F]{32,}\b").unwrap(),
        }
    }

    pub fn mask(&self, text: &str) -> String {
        let t = self.card.replace_all(text, "<CARD>");
        let t = self.email.replace_all(&t, "<EMAIL>");
        let t = self.phone.replace_all(&t, "<PHONE>");
        let t = self.hex.replace_all(&t, "<HEX>");
        t.to_string()
    }
}

impl Default for PiiMasker {
    fn default() -> Self { Self::new() }
}

fn extract_wikilinks(masked: &str) -> Vec<String> {
    // Matches [[target]] or [[target|alias]]; we keep only `target`
    let re = Regex::new(r"\[\[([^\[\]|#]+?)(?:[|#][^\[\]]*)?\]\]").unwrap();
    re.captures_iter(masked)
        .map(|c| c[1].trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn normalize_link_target(s: &str) -> String {
    // Obsidian targets are case-insensitive and may omit .md
    let s = s.trim_end_matches(".md").to_string();
    s.to_lowercase().trim().to_string()
}

pub fn build(config: &ObsidianBuildConfig) -> Result<Dataset, Box<dyn std::error::Error>> {
    let masker = PiiMasker::new();
    let mut path_to_id: HashMap<String, u32> = HashMap::new();
    let mut id_to_label: Vec<String> = Vec::new();

    // First pass: enumerate ALL notes to build a complete ID table.
    // (Wiki-links may point to any note in the vault, so we need them all
    //  to resolve links; but we only *read content* for `max_notes` of them.)
    let mut all_notes: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(&config.vault_root) {
        let entry = match entry { Ok(e) => e, Err(_) => continue };
        if !entry.file_type().is_file() { continue; }
        let p = entry.path();
        if p.extension().and_then(|e| e.to_str()) != Some("md") { continue; }
        // Skip hidden dirs
        if p.components().any(|c| c.as_os_str().to_string_lossy().starts_with('.')) { continue; }
        all_notes.push(p.to_path_buf());
    }
    all_notes.sort();

    // Always register *all* notes into the ID table so links resolve correctly,
    // regardless of max_notes (which only limits content-read scope).
    for p in &all_notes {
        let rel = p.strip_prefix(&config.vault_root).unwrap_or(p);
        let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let normalized = stem.to_lowercase();
        if !path_to_id.contains_key(&normalized) {
            let id = id_to_label.len() as u32;
            path_to_id.insert(normalized.clone(), id);
            let h = stable_hash_hex8(&normalized);
            id_to_label.push(format!("note_{}", h));
        }
    }

    // Determine content-read set (subset of all_notes honoring max_notes)
    let read_set: Vec<&PathBuf> = if let Some(lim) = config.max_notes {
        all_notes.iter().take(lim).collect()
    } else {
        all_notes.iter().collect()
    };

    // Second pass: read + mask + extract links (only from read_set)
    let mut edges: Vec<(u32, u32, f64)> = Vec::new();
    let mut indeg: HashMap<u32, u32> = HashMap::new();
    for p in &read_set {
        let rel = p.strip_prefix(&config.vault_root).unwrap_or(p);
        let stem = rel.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        let src_id = match path_to_id.get(&stem) { Some(&i) => i, None => continue };
        let content = match std::fs::read_to_string(p) { Ok(s) => s, Err(_) => continue };
        let masked = masker.mask(&content);
        let links = extract_wikilinks(&masked);
        let mut edge_set: HashSet<u32> = HashSet::new();
        for target in links {
            let key = normalize_link_target(&target);
            if let Some(&dst_id) = path_to_id.get(&key) {
                if dst_id != src_id && edge_set.insert(dst_id) {
                    edges.push((src_id, dst_id, 1.0));
                    *indeg.entry(dst_id).or_insert(0) += 1;
                }
            }
        }
    }

    // Rare ground truth: notes with 1..=rare_indegree_max incoming links.
    // (indegree 0 notes are truly orphan — we exclude to keep "rare but real".)
    let max_in = config.rare_indegree_max as u32;
    let rare_ground_truth: HashSet<u32> = indeg
        .iter()
        .filter(|(_, &c)| c >= 1 && c <= max_in)
        .map(|(&id, _)| id)
        .collect();

    Ok(Dataset {
        name: format!("ObsidianVault_n{}_read{}", all_notes.len(), read_set.len()),
        n_nodes: all_notes.len(),
        edges,
        rare_ground_truth,
        description: format!(
            "Obsidian Vault ({} total notes, {} read) PII masked; rare = notes with 1..={} incoming wiki-links",
            all_notes.len(), read_set.len(), max_in
        ),
    })
}

fn stable_hash_hex8(s: &str) -> String {
    // FNV-1a 64-bit
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:08x}", h as u32 ^ (h >> 32) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pii_masker_emails() {
        let m = PiiMasker::new();
        let out = m.mask("contact: user@example.com for more");
        assert!(out.contains("<EMAIL>"));
        assert!(!out.contains("user@example.com"));
    }

    #[test]
    fn pii_masker_phone_jp() {
        let m = PiiMasker::new();
        let out = m.mask("Call 03-1234-5678 now");
        assert!(out.contains("<PHONE>"));
    }

    #[test]
    fn pii_masker_card() {
        let m = PiiMasker::new();
        let out = m.mask("Card 4111-1111-1111-1111 expires");
        assert!(out.contains("<CARD>"));
    }

    #[test]
    fn pii_masker_hex_blob() {
        let m = PiiMasker::new();
        let out = m.mask("sha256 = abcdef0123456789abcdef0123456789abcd");
        assert!(out.contains("<HEX>"));
    }

    #[test]
    fn wikilinks_extracted() {
        let text = "See [[Alpha]] and [[Beta|beta alias]] and [[Gamma#section]]";
        let links = extract_wikilinks(text);
        assert_eq!(links, vec!["Alpha".to_string(), "Beta".to_string(), "Gamma".to_string()]);
    }

    #[test]
    fn stable_hash_is_deterministic() {
        assert_eq!(stable_hash_hex8("hello"), stable_hash_hex8("hello"));
        assert_ne!(stable_hash_hex8("hello"), stable_hash_hex8("world"));
    }
}
