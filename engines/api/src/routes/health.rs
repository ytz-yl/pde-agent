use axum::Json;

use crate::models::{ApiResponse, HealthResponse};

/// GET /health
pub async fn health() -> Json<ApiResponse<HealthResponse>> {
    Json(ApiResponse::ok(HealthResponse {
        status: "ok".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        solvers_available: vec!["pdeformer2".into(), "classical".into()],
    }))
}
