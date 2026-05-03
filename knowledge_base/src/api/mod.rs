pub mod handlers;
pub mod routes;

use std::sync::Arc;
use neo4rs::Graph;
use rusqlite::Connection;
use tokio::sync::Mutex;

/// Shared application state passed to all axum handlers.
pub struct AppState {
    /// Neo4j graph connection pool.
    pub graph: Arc<Graph>,
    /// SQLite connection for long-form text content (abstract, notes).
    /// Serialised via async Mutex.
    pub content_db: Arc<Mutex<Connection>>,
}
