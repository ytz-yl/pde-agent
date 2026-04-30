/// PDE Knowledge Base — HTTP service entry point.
///
/// Configuration via environment variables:
///   KB_DB_PATH         Path to the SQLite database file (default: knowledge_base.db)
///   KB_INDEX_PATH      Path to the HNSW vector index file (default: vector_index.bin)
///   KB_BIND_ADDR       Address to bind the HTTP server (default: 0.0.0.0:3000)
///   OPENAI_API_KEY     API key for LLM / embedding calls
///   OPENAI_API_BASE    LLM API base URL (default: https://api.openai.com/v1)
///   EMBEDDING_MODEL    Embedding model name (default: text-embedding-3-small)
///   CHAT_MODEL         Chat model name (default: gpt-4o-mini)

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use knowledge_base::{
    api::{routes::build_router, AppState},
    ingestion::{classifier::LlmConfig, pipeline::rebuild_vector_index},
    store::{open_db, vector_index::VectorIndex},
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env if present (dev convenience)
    dotenvy::dotenv().ok();

    // Initialise tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "knowledge_base=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // ── Config ─────────────────────────────────────────────────────────────
    let db_path = std::env::var("KB_DB_PATH")
        .unwrap_or_else(|_| "knowledge_base.db".into());
    let index_path = std::env::var("KB_INDEX_PATH")
        .unwrap_or_else(|_| "vector_index.bin".into());
    let bind_addr = std::env::var("KB_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into());

    // ── Storage ────────────────────────────────────────────────────────────
    tracing::info!("opening database at {}", db_path);
    let conn = open_db(&db_path)?;

    tracing::info!("loading/creating vector index at {}", index_path);
    let vector_index = Arc::new(VectorIndex::open_or_create(&index_path)?);

    // Rebuild key maps from SQLite on startup (usearch only persists vectors,
    // not the string→u64 id mapping)
    {
        let count = rebuild_vector_index(&conn, &vector_index)?;
        tracing::info!("vector index ready ({} embeddings)", count);
    }

    // ── App state ──────────────────────────────────────────────────────────
    let state = Arc::new(AppState {
        db: Arc::new(Mutex::new(conn)),
        vector_index,
        http_client: reqwest::Client::new(),
        llm_cfg: Arc::new(LlmConfig::from_env()),
    });

    // ── Router ─────────────────────────────────────────────────────────────
    let app = build_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // ── Serve ──────────────────────────────────────────────────────────────
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("knowledge-base listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
