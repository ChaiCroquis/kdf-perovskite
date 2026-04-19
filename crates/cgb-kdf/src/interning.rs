//! Node ID Interning for String ↔ u32 boundary conversion
//!
//! Provides efficient conversion between string node names and integer IDs.
//! String operations happen only at API boundaries; internal computation uses u32.

use std::collections::HashMap;

/// Bidirectional mapping between node names and integer IDs
///
/// # Design
/// - Strings are stored once
/// - All internal operations use u32 IDs
/// - O(1) lookup in both directions
///
/// # Example
/// ```
/// use cgb_kdf::interning::NodeIdMap;
///
/// let mut map = NodeIdMap::new();
/// let id = map.get_or_insert("node_a");
/// assert_eq!(map.get_name(id), Some("node_a"));
/// ```
#[derive(Clone, Debug, Default)]
pub struct NodeIdMap {
    /// String → ID mapping
    to_id: HashMap<String, u32>,
    /// ID → String mapping (indexed by ID)
    to_name: Vec<String>,
}

impl NodeIdMap {
    /// Create an empty mapping
    pub fn new() -> Self {
        Self {
            to_id: HashMap::new(),
            to_name: Vec::new(),
        }
    }

    /// Create with pre-allocated capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            to_id: HashMap::with_capacity(capacity),
            to_name: Vec::with_capacity(capacity),
        }
    }

    /// Get or insert a node name, returning its ID
    pub fn get_or_insert(&mut self, name: &str) -> u32 {
        if let Some(&id) = self.to_id.get(name) {
            id
        } else {
            let id = self.to_name.len() as u32;
            self.to_name.push(name.to_string());
            self.to_id.insert(name.to_string(), id);
            id
        }
    }

    /// Get ID for a name (if exists)
    pub fn get_id(&self, name: &str) -> Option<u32> {
        self.to_id.get(name).copied()
    }

    /// Get name for an ID (if exists)
    pub fn get_name(&self, id: u32) -> Option<&str> {
        self.to_name.get(id as usize).map(|s| s.as_str())
    }

    /// Number of interned nodes
    pub fn len(&self) -> usize {
        self.to_name.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.to_name.is_empty()
    }

    /// Iterate over all (id, name) pairs
    pub fn iter(&self) -> impl Iterator<Item = (u32, &str)> {
        self.to_name
            .iter()
            .enumerate()
            .map(|(id, name)| (id as u32, name.as_str()))
    }

    /// Convert string edges to ID-based edges
    ///
    /// Returns (converted_edges, self) for chaining
    pub fn intern_edges(&mut self, edges: &[(String, String, f64)]) -> Vec<(u32, u32, f64)> {
        edges
            .iter()
            .map(|(u, v, w)| {
                let u_id = self.get_or_insert(u);
                let v_id = self.get_or_insert(v);
                (u_id, v_id, *w)
            })
            .collect()
    }

    /// Convert string partition to ID-based partition
    pub fn intern_partition(&mut self, partition: &HashMap<String, u32>) -> Vec<u32> {
        let mut result = vec![0u32; self.len()];
        for (name, &module) in partition {
            if let Some(id) = self.get_id(name) {
                result[id as usize] = module;
            }
        }
        result
    }

    /// Convert ID-based partition back to string partition
    pub fn extern_partition(&self, partition: &[u32]) -> HashMap<String, u32> {
        partition
            .iter()
            .enumerate()
            .filter_map(|(id, &module)| {
                self.get_name(id as u32)
                    .map(|name| (name.to_string(), module))
            })
            .collect()
    }
}

/// Internal edge representation using IDs
pub type InternedEdge = (u32, u32, f64);

/// Internal partition representation using IDs
pub type InternedPartition = Vec<u32>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_or_insert() {
        let mut map = NodeIdMap::new();
        let id1 = map.get_or_insert("node_a");
        let id2 = map.get_or_insert("node_b");
        let id1_again = map.get_or_insert("node_a");

        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(id1_again, 0); // Same ID returned
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_bidirectional_lookup() {
        let mut map = NodeIdMap::new();
        let id = map.get_or_insert("test_node");

        assert_eq!(map.get_id("test_node"), Some(id));
        assert_eq!(map.get_name(id), Some("test_node"));
        assert_eq!(map.get_id("nonexistent"), None);
        assert_eq!(map.get_name(999), None);
    }

    #[test]
    fn test_intern_edges() {
        let mut map = NodeIdMap::new();
        let edges = vec![
            ("a".to_string(), "b".to_string(), 1.0),
            ("b".to_string(), "c".to_string(), 2.0),
            ("a".to_string(), "c".to_string(), 1.5),
        ];

        let interned = map.intern_edges(&edges);

        assert_eq!(interned.len(), 3);
        assert_eq!(map.len(), 3); // a, b, c

        let a = map.get_id("a").unwrap();
        let b = map.get_id("b").unwrap();
        let c = map.get_id("c").unwrap();

        assert_eq!(interned[0], (a, b, 1.0));
        assert_eq!(interned[1], (b, c, 2.0));
        assert_eq!(interned[2], (a, c, 1.5));
    }

    #[test]
    fn test_partition_conversion() {
        let mut map = NodeIdMap::new();
        map.get_or_insert("a");
        map.get_or_insert("b");
        map.get_or_insert("c");

        let partition: HashMap<String, u32> = [
            ("a".to_string(), 0),
            ("b".to_string(), 0),
            ("c".to_string(), 1),
        ]
        .into_iter()
        .collect();

        let interned = map.intern_partition(&partition);
        assert_eq!(interned, vec![0, 0, 1]);

        let externed = map.extern_partition(&interned);
        assert_eq!(externed, partition);
    }

    #[test]
    fn test_iter() {
        let mut map = NodeIdMap::new();
        map.get_or_insert("x");
        map.get_or_insert("y");

        let pairs: Vec<_> = map.iter().collect();
        assert_eq!(pairs, vec![(0, "x"), (1, "y")]);
    }

    #[test]
    fn test_with_capacity() {
        let map = NodeIdMap::with_capacity(100);
        assert!(map.is_empty());
    }

    #[test]
    fn test_is_empty() {
        let mut map = NodeIdMap::new();
        assert!(map.is_empty());

        map.get_or_insert("node");
        assert!(!map.is_empty());
    }

    #[test]
    fn test_intern_partition_missing_node() {
        let mut map = NodeIdMap::new();
        map.get_or_insert("a");
        map.get_or_insert("b");

        let partition: HashMap<String, u32> = [
            ("a".to_string(), 0),
            ("c".to_string(), 1), // "c" not in map
        ]
        .into_iter()
        .collect();

        let interned = map.intern_partition(&partition);
        // Only "a" should be set, "c" is ignored
        assert_eq!(interned[0], 0);
        // "b" was not in partition, defaults to 0
        assert_eq!(interned[1], 0);
    }

    #[test]
    fn test_extern_partition_partial() {
        let mut map = NodeIdMap::new();
        map.get_or_insert("a");
        map.get_or_insert("b");

        let interned = vec![0, 1];
        let externed = map.extern_partition(&interned);

        assert_eq!(externed.len(), 2);
        assert_eq!(externed.get("a"), Some(&0));
        assert_eq!(externed.get("b"), Some(&1));
    }

    #[test]
    fn test_large_graph() {
        let mut map = NodeIdMap::new();
        for i in 0..1000 {
            let name = format!("node_{}", i);
            let id = map.get_or_insert(&name);
            assert_eq!(id, i);
        }
        assert_eq!(map.len(), 1000);
    }
}
