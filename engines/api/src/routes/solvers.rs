use axum::{extract::State, Json};
use std::sync::Arc;

use crate::error::ApiError;
use crate::models::{ApiResponse, SolverInfo};
use crate::solvers::SolverRegistry;

/// GET /solvers
/// List all registered solver backends.
pub async fn list_solvers(
    State(registry): State<Arc<SolverRegistry>>,
) -> Result<Json<ApiResponse<Vec<SolverInfo>>>, ApiError> {
    let list = registry.list();
    Ok(Json(ApiResponse::ok(list)))
}
