/// Semantic search: find papers/methods by vector similarity.
use anyhow::Result;
use reqwest::Client;
use rusqlite::Connection;

use crate::{
    ingestion::classifier::{embed_text, LlmConfig},
    store::{paper_repo, schema::Paper, vector_index::VectorIndex},
};

/// A single semantic search hit.
#[derive(Debug, serde::Serialize)]
pub struct SemanticHit {
    pub id: String,
    pub score: f32,
}

/// Search for papers semantically similar to `query`.
///
/// Flow:
///   1. Embed `query` using the LLM API
///   2. Run ANN search in the HNSW index → list of (id, score) pairs
///   3. Fetch full Paper records from SQLite for the top-k ids
pub async fn search_papers(
    conn: &Connection,
    vector_index: &VectorIndex,
    client: &Client,
    llm_cfg: &LlmConfig,
    query: &str,
    k: usize,
) -> Result<Vec<(Paper, f32)>> {
    let embedding = embed_text(client, llm_cfg, query).await?;
    let hits = vector_index.search(&embedding, k)?;

    let mut results = Vec::with_capacity(hits.len());
    for (id, score) in hits {
        if let Some(paper) = paper_repo::get_paper(conn, &id)? {
            results.push((paper, score));
        }
    }
    // Already sorted by score descending from the index
    Ok(results)
}

/// Hybrid search: combine vector similarity with FTS5 full-text score.
///
/// Strategy: run both searches, merge by id (average scores), re-rank.
pub async fn hybrid_search_papers(
    conn: &Connection,
    vector_index: &VectorIndex,
    client: &Client,
    llm_cfg: &LlmConfig,
    query: &str,
    k: usize,
) -> Result<Vec<(Paper, f32)>> {
    // --- vector hits ---
    let embedding = embed_text(client, llm_cfg, query).await?;
    let vec_hits = vector_index.search(&embedding, k * 2)?;

    // --- FTS hits ---
    // Sanitise query for FTS5: wrap in quotes to prevent syntax errors
    let fts_query = sanitise_fts_query(query);
    let fts_papers = paper_repo::search_papers_fts(conn, &fts_query, k * 2)
        .unwrap_or_default();

    // Normalise FTS rank into [0, 1] (higher is better)
    let fts_max = fts_papers.len() as f32;
    let fts_scores: Vec<(String, f32)> = fts_papers
        .into_iter()
        .enumerate()
        .map(|(i, p)| {
            let score = if fts_max > 0.0 {
                (fts_max - i as f32) / fts_max
            } else {
                0.0
            };
            (p.id, score)
        })
        .collect();

    // Merge: collect all ids, combine scores
    use std::collections::HashMap;
    let mut combined: HashMap<String, f32> = HashMap::new();
    for (id, score) in &vec_hits {
        *combined.entry(id.clone()).or_default() += score * 0.7;
    }
    for (id, score) in &fts_scores {
        *combined.entry(id.clone()).or_default() += score * 0.3;
    }

    let mut ranked: Vec<(String, f32)> = combined.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(k);

    let mut results = Vec::with_capacity(ranked.len());
    for (id, score) in ranked {
        if let Some(paper) = paper_repo::get_paper(conn, &id)? {
            results.push((paper, score));
        }
    }
    Ok(results)
}

pub fn sanitise_fts_query_pub(q: &str) -> String {
    sanitise_fts_query(q)
}

fn sanitise_fts_query(q: &str) -> String {
    // Escape double quotes and wrap each space-separated word
    q.split_whitespace()
        .map(|w| format!("\"{}\"", w.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ")
}
