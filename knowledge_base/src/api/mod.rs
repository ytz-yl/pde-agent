pub mod handlers;
pub mod routes;

use std::sync::Arc;
use reqwest::Client;
use rusqlite::Connection;
use tokio::sync::Mutex;

use crate::{
    ingestion::classifier::LlmConfig,
    store::vector_index::VectorIndex,
};

/// All mutable state is wrapped in `Arc<Mutex<_>>` so handlers can share it.
pub struct AppState {
    /// SQLite connection — serialised access via async Mutex.
    pub db: Arc<Mutex<Connection>>,
    /// HNSW vector index.
    pub vector_index: Arc<VectorIndex>,
    /// Reusable HTTP client for arXiv + LLM API calls.
    pub http_client: Client,
    /// LLM configuration (API keys, model names).
    pub llm_cfg: Arc<LlmConfig>,
}
