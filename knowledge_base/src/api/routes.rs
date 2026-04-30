/// axum route definitions.
use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use super::{AppState, handlers};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ── Search / retrieval ─────────────────────────────────────────────
        .route("/search",                     get(handlers::search))
        .route("/papers/recent",              get(handlers::recent_papers))
        .route("/papers/:id",                 get(handlers::get_paper))
        // ── Methods ────────────────────────────────────────────────────────
        .route("/methods",                    get(handlers::list_methods))
        .route("/methods/:id",                get(handlers::get_method))
        .route("/methods/:id/related",        get(handlers::related_methods))
        .route("/methods/compare",            get(handlers::compare_methods))
        // ── Recommendations ─────────────────────────────────────────────
        .route("/recommend",                  post(handlers::recommend))
        // ── Ingestion ──────────────────────────────────────────────────────
        .route("/ingest/paper",               post(handlers::ingest_paper))
        .route("/ingest/fetch-arxiv",         post(handlers::fetch_arxiv))
        // ── Health ─────────────────────────────────────────────────────────
        .route("/health",                     get(handlers::health))
        .with_state(state)
}
