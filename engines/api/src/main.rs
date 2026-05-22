mod error;
mod models;
mod routes;
mod solvers;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use routes::files::upload_file;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use solvers::SolverRegistry;

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,pde_solver_api=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Build solver registry (all backends registered here)
    let registry = Arc::new(SolverRegistry::new());

    // CORS: allow any origin (tighten in production)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Router
    let app = Router::new()
        .route("/health",  get(routes::health::health))
        .route("/solvers", get(routes::solvers::list_solvers))
        .route("/solve",   post(routes::solve::solve))
        .route("/files",   post(upload_file))
        .with_state(registry)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into());
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    info!("PDE Solver API listening on {}", addr);

    axum::serve(listener, app).await.unwrap();
}
