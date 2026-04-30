use axum::{extract::State, Json};
use std::sync::Arc;
use tracing::info;

use crate::error::ApiError;
use crate::models::{ApiResponse, SolveRequest, SolveResponse};
use crate::solvers::SolverRegistry;

/// POST /solve
///
/// Dispatch a PDE solve request to the appropriate backend.
///
/// # Request body (JSON)
/// ```json
/// {
///   "solver": "pdeformer2",          // optional, defaults to "pdeformer2"
///   "pde": {
///     "equation": "u_t + (u^2)_x + (-0.3*u)_y = 0",
///     "initial_condition": [...],    // flat 128*128 array, row-major
///     "boundary_condition": "periodic",
///     "parameters": {}
///   },
///   "query": {
///     "x": [0.0, 0.03125, ...],      // n_x values in [0,1]
///     "y": [0.0, 0.03125, ...],      // n_y values in [0,1]
///     "t": [0.0, 0.25, 0.5, 0.75, 1.0]
///   },
///   "options": {}
/// }
/// ```
///
/// # Response body (JSON)
/// ```json
/// {
///   "success": true,
///   "data": {
///     "solver_used": "pdeformer2",
///     "solution": [[[[...]]]], // [n_t][n_x][n_y][n_vars]
///     "shape": { "n_t": 5, "n_x": 32, "n_y": 32, "n_vars": 1 },
///     "metadata": { "wall_time_ms": 1234, "backend": "...", "notes": [] }
///   },
///   ...
/// }
/// ```
pub async fn solve(
    State(registry): State<Arc<SolverRegistry>>,
    Json(req): Json<SolveRequest>,
) -> Result<Json<ApiResponse<SolveResponse>>, ApiError> {
    let solver_id = req
        .solver
        .as_deref()
        .unwrap_or(SolverRegistry::default_id())
        .to_owned();

    info!(solver = %solver_id, equation = %req.pde.equation, "Received solve request");

    let solver = registry
        .get(&solver_id)
        .ok_or_else(|| ApiError::SolverNotFound(solver_id.clone()))?;

    let result = solver.solve(&req).await?;

    info!(
        solver = %solver_id,
        wall_ms = result.metadata.wall_time_ms,
        "Solve completed"
    );

    Ok(Json(ApiResponse::ok(result)))
}
