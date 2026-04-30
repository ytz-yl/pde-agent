/// In-process HNSW vector index backed by `usearch`.
///
/// This module wraps `usearch::Index` to provide a simple key→vector store
/// that can be queried by approximate nearest-neighbour search.  The index
/// is persisted to a single binary file on disk and can be rebuilt from the
/// SQLite embeddings at any time.
///
/// Key design decisions:
/// - All integer keys are derived by hashing the string id so that the index
///   stays in sync with the SQLite primary keys without a separate id→int map.
/// - The index file is written atomically (write to temp, then rename).
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use anyhow::{Context, Result};
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use super::schema::EMBEDDING_DIM;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Build the usearch `IndexOptions` used for all indexes in this crate.
fn index_options() -> IndexOptions {
    IndexOptions {
        dimensions: EMBEDDING_DIM,
        metric: MetricKind::Cos,   // cosine similarity
        quantization: ScalarKind::F32,
        connectivity: 16,          // HNSW M parameter
        expansion_add: 128,        // ef_construction
        expansion_search: 64,      // ef at query time
        multi: false,
    }
}

// ── VectorIndex ───────────────────────────────────────────────────────────────

/// Thread-safe HNSW index with a string-key → usearch u64 key mapping.
pub struct VectorIndex {
    inner: Arc<RwLock<IndexInner>>,
}

struct IndexInner {
    index: Index,
    /// Map: string id  →  usearch key (u64 = first 8 bytes of FNV-1a hash)
    key_map: HashMap<String, u64>,
    /// Reverse map: usearch key → string id
    rev_map: HashMap<u64, String>,
    /// Path where the index is persisted (may be in-memory if None)
    path: Option<PathBuf>,
}

impl VectorIndex {
    /// Create a new in-memory index.
    pub fn new_in_memory() -> Result<Self> {
        let index = Index::new(&index_options()).context("create usearch index")?;
        Ok(VectorIndex {
            inner: Arc::new(RwLock::new(IndexInner {
                index,
                key_map: HashMap::new(),
                rev_map: HashMap::new(),
                path: None,
            })),
        })
    }

    /// Load an existing index from `path`, or create a new one if the file
    /// does not exist.
    pub fn open_or_create(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let (index, key_map, rev_map) = if path.exists() {
            let index = Index::new(&index_options()).context("create usearch index")?;
            index.load(path.to_str().unwrap()).context("load index")?;
            // key_map / rev_map are not persisted by usearch; they are
            // rebuilt from SQLite when `rebuild_from_entries` is called.
            (index, HashMap::new(), HashMap::new())
        } else {
            let index = Index::new(&index_options()).context("create usearch index")?;
            index.reserve(1024).context("reserve capacity")?;
            (index, HashMap::new(), HashMap::new())
        };
        Ok(VectorIndex {
            inner: Arc::new(RwLock::new(IndexInner {
                index,
                key_map,
                rev_map,
                path: Some(path),
            })),
        })
    }

    /// Rebuild the entire index from a list of (id, embedding) pairs.
    /// This is called after a fresh start or a full re-ingestion.
    pub fn rebuild_from_entries(&self, entries: Vec<(String, Vec<f32>)>) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        // Reset
        inner.index = Index::new(&index_options()).context("recreate index")?;
        inner.index.reserve(entries.len().max(64)).context("reserve")?;
        inner.key_map.clear();
        inner.rev_map.clear();

        for (id, embedding) in entries {
            let key = fnv1a_u64(&id);
            inner
                .index
                .add(key, &embedding)
                .context("add to index")?;
            inner.key_map.insert(id.clone(), key);
            inner.rev_map.insert(key, id);
        }

        if let Some(ref path) = inner.path {
            save_index(&inner.index, path)?;
        }
        Ok(())
    }

    /// Add or update a single entry. If the key already exists in usearch it
    /// will be overwritten.
    pub fn upsert(&self, id: &str, embedding: &[f32]) -> Result<()> {
        let key = fnv1a_u64(id);
        let mut inner = self.inner.write().unwrap();
        // usearch does not support update in-place; remove + re-add.
        if inner.key_map.contains_key(id) {
            inner.index.remove(key).context("remove old vector")?;
        } else {
            // Grow capacity if needed (double current or add 256, whichever is bigger)
            let cap = inner.index.capacity();
            let size = inner.index.size();
            if size + 1 >= cap {
                let new_cap = (cap * 2).max(cap + 256);
                inner.index.reserve(new_cap).context("reserve")?;
            }
        }
        inner.index.add(key, embedding).context("add vector")?;
        inner.key_map.insert(id.to_string(), key);
        inner.rev_map.insert(key, id.to_string());

        if let Some(ref path) = inner.path {
            save_index(&inner.index, path)?;
        }
        Ok(())
    }

    /// Remove an entry by string id.
    pub fn remove(&self, id: &str) -> Result<bool> {
        let mut inner = self.inner.write().unwrap();
        if let Some(&key) = inner.key_map.get(id) {
            inner.index.remove(key).context("remove vector")?;
            inner.key_map.remove(id);
            inner.rev_map.remove(&key);
            if let Some(ref path) = inner.path {
                save_index(&inner.index, path)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Approximate nearest-neighbour search.
    /// Returns up to `k` (id, score) pairs ordered by descending cosine similarity.
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(String, f32)>> {
        let inner = self.inner.read().unwrap();
        let results = inner
            .index
            .search(query, k)
            .context("usearch search")?;

        let hits = results
            .keys
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(&key, &dist)| {
                inner.rev_map.get(&key).map(|id| {
                    // usearch returns cosine *distance* (1 - similarity); convert to similarity
                    let similarity = 1.0 - dist;
                    (id.clone(), similarity)
                })
            })
            .collect();
        Ok(hits)
    }

    /// Number of vectors currently in the index.
    pub fn size(&self) -> usize {
        self.inner.read().unwrap().index.size()
    }
}

// ── Persistence helpers ───────────────────────────────────────────────────────

/// Atomically save the index: write to a temp file, then rename.
fn save_index(index: &Index, path: &Path) -> Result<()> {
    let tmp = path.with_extension("tmp");
    index
        .save(tmp.to_str().unwrap())
        .context("save index to temp")?;
    std::fs::rename(&tmp, path).context("rename temp to index file")?;
    Ok(())
}

// ── FNV-1a 64-bit hash ────────────────────────────────────────────────────────

/// Deterministic string → u64 mapping used as usearch keys.
fn fnv1a_u64(s: &str) -> u64 {
    const FNV_OFFSET: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;
    let mut hash = FNV_OFFSET;
    for byte in s.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn random_vec(dim: usize, seed: u64) -> Vec<f32> {
        // Simple LCG for deterministic test vectors
        let mut x = seed;
        let mut v = Vec::with_capacity(dim);
        for _ in 0..dim {
            x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            v.push((x >> 33) as f32 / (u32::MAX as f32) - 0.5);
        }
        // L2-normalise so cosine distance is well-defined
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        v.iter_mut().for_each(|x| *x /= norm);
        v
    }

    #[test]
    fn test_upsert_and_search() {
        let idx = VectorIndex::new_in_memory().unwrap();
        let v1 = random_vec(EMBEDDING_DIM, 1);
        let v2 = random_vec(EMBEDDING_DIM, 2);
        let v3 = random_vec(EMBEDDING_DIM, 3);
        idx.upsert("paper:a", &v1).unwrap();
        idx.upsert("paper:b", &v2).unwrap();
        idx.upsert("paper:c", &v3).unwrap();
        assert_eq!(idx.size(), 3);

        let results = idx.search(&v1, 2).unwrap();
        assert!(!results.is_empty());
        // The best hit should be the vector itself
        assert_eq!(results[0].0, "paper:a");
    }

    #[test]
    fn test_remove() {
        let idx = VectorIndex::new_in_memory().unwrap();
        let v = random_vec(EMBEDDING_DIM, 42);
        idx.upsert("x", &v).unwrap();
        assert_eq!(idx.size(), 1);
        assert!(idx.remove("x").unwrap());
        assert_eq!(idx.size(), 0);
        assert!(!idx.remove("x").unwrap());
    }
}
