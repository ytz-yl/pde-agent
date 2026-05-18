use axum::{extract::State, Json};
use std::sync::Arc;
use tracing::info;

use crate::error::ApiError;
use crate::models::{ApiResponse, SolveRequest, SolveResponse};
use crate::routes::files::file_path_for_id;
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
///     "parameters": {},
///
///     // --- OR use uploaded tensor file as history input ---
///     "history": {
///       "file_id": "<id from POST /files>",
///       "format": "hdf5",            // optional, inferred from extension
///       "dataset_key": "/snapshots", // optional, for HDF5/npz
///       "input_timesteps": [0,1,2],  // optional, defaults to all
///       "variables": ["u"]           // optional, defaults to ["u"]
///     }
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
///     "variables": ["u"],
///     "shape": { "n_t": 5, "n_x": 32, "n_y": 32, "n_vars": 1 },
///     "metadata": { "wall_time_ms": 1234, "backend": "...", "notes": [] }
///   }
/// }
/// ```
pub async fn solve(
    State(registry): State<Arc<SolverRegistry>>,
    Json(mut req): Json<SolveRequest>,
) -> Result<Json<ApiResponse<SolveResponse>>, ApiError> {
    // If a history file_id is provided, resolve it to an absolute path so
    // the Python bridge can open the file directly.  We do this in the Rust
    // layer so the Python script never has to know about the upload directory.
    if let Some(ref mut history) = req.pde.history {
        let fid = &history.file_id;
        let path = file_path_for_id(fid).ok_or_else(|| {
            ApiError::FileNotFound(format!(
                "No uploaded file with id '{}'. Upload it first via POST /files.",
                fid
            ))
        })?;

        // Inject the resolved path as a special field the Python bridge reads.
        // We (ab)use the `file_id` field to carry the absolute path string here.
        // The Python bridge checks whether the value is a valid path and uses it.
        history.file_id = path.to_string_lossy().to_string();

        // Also infer format from extension if the caller didn't specify one.
        if history.format.is_none() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                history.format = Some(match ext.to_lowercase().as_str() {
                    "h5" | "hdf5" => "hdf5",
                    "npy"         => "npy",
                    "npz"         => "npz",
                    "pt" | "pth"  => "pt",
                    other         => other,
                }.to_string());
            }
        }

        info!(
            file_path = %history.file_id,
            format    = ?history.format,
            "Resolved history file_id to path"
        );
    }

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
