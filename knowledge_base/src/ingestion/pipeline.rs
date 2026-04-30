/// Ingestion pipeline: fetch → classify → store → index.
use anyhow::Result;
use reqwest::Client;
use rusqlite::Connection;

use crate::store::{paper_repo, vector_index::VectorIndex};
use super::{
    arxiv_fetcher::{fetch_papers, ArxivQuery},
    classifier::{embed_text, extract_tags, LlmConfig},
};

/// Ingest papers from arXiv matching `query` into the database and vector index.
///
/// For each fetched paper:
///  1. Extract tags via LLM
///  2. Generate embedding vector
///  3. Upsert into SQLite
///  4. Upsert into HNSW vector index
pub async fn ingest_arxiv(
    conn: &Connection,
    vector_index: &VectorIndex,
    client: &Client,
    llm_cfg: &LlmConfig,
    query: ArxivQuery,
) -> Result<IngestReport> {
    let papers = fetch_papers(client, &query).await?;
    let total_fetched = papers.len();
    let mut stored = 0usize;
    let mut errors = Vec::new();

    for mut paper in papers {
        let id = paper.id.clone();
        let title = paper.title.clone();
        let abstract_text = paper
            .abstract_text
            .clone()
            .unwrap_or_default();

        // Tag extraction
        match extract_tags(client, llm_cfg, &title, &abstract_text).await {
            Ok(tags) => paper.tags = tags.to_paper_tags(),
            Err(e) => {
                tracing::warn!("tag extraction failed for {}: {}", id, e);
                // Continue without tags rather than aborting
            }
        }

        // Embedding generation — use title + abstract as the text to embed
        let embed_text_content = format!("{}\n\n{}", title, abstract_text);
        match embed_text(client, llm_cfg, &embed_text_content).await {
            Ok(embedding) => {
                // Store in vector index before saving (index upsert is idempotent)
                if let Err(e) = vector_index.upsert(&id, &embedding) {
                    tracing::warn!("vector index upsert failed for {}: {}", id, e);
                }
                paper.embedding = Some(embedding);
            }
            Err(e) => {
                tracing::warn!("embedding failed for {}: {}", id, e);
            }
        }

        // Persist to SQLite
        match paper_repo::upsert_paper(conn, &paper) {
            Ok(()) => stored += 1,
            Err(e) => {
                tracing::error!("DB upsert failed for {}: {}", id, e);
                errors.push(format!("{}: {}", id, e));
            }
        }
    }

    Ok(IngestReport {
        total_fetched,
        stored,
        errors,
    })
}

/// Rebuild the vector index from all papers currently in the database.
/// Call this on startup or after a migration.
pub fn rebuild_vector_index(
    conn: &Connection,
    vector_index: &VectorIndex,
) -> Result<usize> {
    let entries = paper_repo::all_papers_with_embeddings(conn)?;
    let count = entries.len();
    vector_index.rebuild_from_entries(entries)?;
    Ok(count)
}

/// Summary returned after an ingestion run.
#[derive(Debug, serde::Serialize)]
pub struct IngestReport {
    pub total_fetched: usize,
    pub stored: usize,
    pub errors: Vec<String>,
}
