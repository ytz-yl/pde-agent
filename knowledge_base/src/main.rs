/// PDE Knowledge Base — HTTP service entry point.
///
/// Configuration via environment variables:
///   NEO4J_URI       Bolt URI (default: bolt://localhost:7687)
///   NEO4J_USER      Username (default: neo4j)
///   NEO4J_PASSWORD  Password (default: password)
///   KB_BIND_ADDR    Address to bind the HTTP server (default: 0.0.0.0:3000)
///   KB_CONTENT_DB   Path to the SQLite content database (default: content.db)
///   KB_SEED_DATA    Set to "false" to skip seeding initial data (default: true)

use std::sync::Arc;

use anyhow::Result;
use tokio::sync::Mutex;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use knowledge_base::{
    api::{routes::build_router, AppState},
    store::{
        content_repo::open_content_db,
        graph::{connect, init_schema, seed_data},
    },
};

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "knowledge_base=debug,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let bind_addr = std::env::var("KB_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".into());
    let content_db_path = std::env::var("KB_CONTENT_DB")
        .unwrap_or_else(|_| "content.db".into());
    let seed = std::env::var("KB_SEED_DATA")
        .map(|v| v.to_lowercase() != "false")
        .unwrap_or(true);

    // ── Neo4j ──────────────────────────────────────────────────────────────
    let graph = connect().await?;
    init_schema(&graph).await?;
    if seed {
        seed_data(&graph).await?;
    }

    // ── SQLite content db ──────────────────────────────────────────────────
    tracing::info!("opening content db at {}", content_db_path);
    let content_db = open_content_db(&content_db_path)?;

    // ── App state ──────────────────────────────────────────────────────────
    let state = Arc::new(AppState {
        graph: Arc::new(graph),
        content_db: Arc::new(Mutex::new(content_db)),
    });

    // ── Router ─────────────────────────────────────────────────────────────
    let app = build_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());

    // ── Serve ──────────────────────────────────────────────────────────────
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("pde-knowledge-base listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
