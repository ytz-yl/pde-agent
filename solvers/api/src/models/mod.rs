// API data models (request / response types)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Common
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn err(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
            request_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }
}

// ---------------------------------------------------------------------------
// /solvers
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct SolverInfo {
    pub id: String,
    pub name: String,
    pub category: SolverCategory,
    pub description: String,
    pub supported_pde_types: Vec<String>,
    pub backend: String,
    pub available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SolverCategory {
    Classical,
    MachineLearning,
    Hybrid,
}

// ---------------------------------------------------------------------------
// /solve  (generic entry point, dispatches to a named solver)
// ---------------------------------------------------------------------------

/// A PDE problem submitted by the caller.
#[derive(Debug, Deserialize, Serialize)]
pub struct SolveRequest {
    /// Which solver to use. Defaults to "pdeformer2" if omitted.
    pub solver: Option<String>,
    /// The PDE definition.
    pub pde: PdeSpec,
    /// Where to evaluate the solution.
    pub query: QuerySpec,
    /// Solver-specific options (forwarded verbatim).
    pub options: Option<serde_json::Value>,
}

/// Symbolic + numeric description of the PDE.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PdeSpec {
    /// Human-readable equation string, e.g. "u_t + (u^2)_x + (-0.3*u)_y = 0"
    pub equation: String,
    /// Initial condition values on a uniform 128×128 grid (row-major, length
    /// 128*128). Required for time-dependent problems.
    pub initial_condition: Option<Vec<f64>>,
    /// Boundary condition type: "periodic" | "dirichlet" | "neumann"
    pub boundary_condition: Option<String>,
    /// Free scalar parameters referenced in the equation (name → value).
    pub parameters: Option<serde_json::Value>,
}

/// Coordinates at which the solution should be returned.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct QuerySpec {
    /// Spatial x-coordinates (length n_x).
    pub x: Vec<f64>,
    /// Spatial y-coordinates (length n_y).
    pub y: Vec<f64>,
    /// Time snapshots to evaluate (defaults to [0, 0.25, 0.5, 0.75, 1.0]).
    pub t: Option<Vec<f64>>,
}

/// Solver response.
#[derive(Debug, Serialize)]
pub struct SolveResponse {
    pub solver_used: String,
    /// Solution values with shape [n_t][n_x][n_y][n_vars].
    pub solution: Vec<Vec<Vec<Vec<f64>>>>,
    pub shape: SolutionShape,
    pub metadata: SolveMetadata,
}

#[derive(Debug, Serialize)]
pub struct SolutionShape {
    pub n_t: usize,
    pub n_x: usize,
    pub n_y: usize,
    pub n_vars: usize,
}

#[derive(Debug, Serialize)]
pub struct SolveMetadata {
    pub wall_time_ms: u64,
    pub backend: String,
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// /health
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub solvers_available: Vec<String>,
}
