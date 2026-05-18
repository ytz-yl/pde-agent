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

// ---------------------------------------------------------------------------
// PDE specification — supports single-variable and multi-variable problems
// ---------------------------------------------------------------------------

/// Full PDE specification supporting multi-variable / multi-equation systems.
///
/// # Single-variable shorthand (backward-compatible)
/// If `variables` is absent or empty and `equations` is absent or empty, the
/// legacy `equation` + `initial_condition` + `boundary_condition` fields are
/// used.
///
/// # Multi-variable / multi-equation
/// `variables` names the unknown fields (e.g. `["u", "v", "p"]`).
/// `equations` is a list of equation strings, one per constraint.
/// `initial_conditions` maps variable name → flat grid array (length n*n,
/// row-major on a uniform n×n grid over [0,1]²).
/// If a variable needs `u_t(0)=0` as a second IC, encode it as
/// `{"u.dt": [0.0, ...]}` (a flat array of zeros) or the special token
/// `"zero"`.
///
/// # Coefficient fields
/// `coef_fields` maps a name used in equations → flat 128×128 array that the
/// solver will pass to `pde.new_coef_field(...)`.
///
/// # Domain / boundary via SDF
/// `domains` is a list of named SDF specifications.  Each entry has:
///   - `name`  – identifier referenced in the equation string
///   - `sdf`   – flat n×n array of signed-distance-function values
///   - `role`  – `"interior"` | `"boundary_dirichlet"` | `"boundary_neumann"` |
///               `"boundary_mur"`
///
/// # Boundary conditions list
/// `bcs` is a list of boundary condition specs, each with:
///   - `domain`  – name of the domain (from `domains`) on which BC is applied
///   - `vars`    – list of variable names (or expressions) summed to zero
///   - `bc_type` – `"dirichlet"` | `"neumann"` | `"mur"` | `"periodic"`
///
/// # History input (data-driven models)
/// `history` provides a previously-uploaded tensor file as the model input,
/// instead of (or in addition to) `initial_conditions`.  The file is
/// referenced by its `file_id` returned from `POST /files`.
/// When `history` is present, `initial_condition` / `initial_conditions` are
/// ignored by solvers that support the history path.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PdeSpec {
    // ── Legacy single-variable fields (kept for backward compat) ────────────
    /// Human-readable equation string, e.g. "u_t + (u^2)_x + (-0.3*u)_y = 0"
    pub equation: String,
    /// Initial condition on a uniform n×n grid (row-major).  n is inferred
    /// from sqrt(len).  Required for time-dependent problems.
    pub initial_condition: Option<Vec<f64>>,
    /// Boundary condition type: "periodic" | "dirichlet" | "neumann"
    pub boundary_condition: Option<String>,
    /// Free scalar parameters referenced in the equation (name → value).
    pub parameters: Option<serde_json::Value>,

    // ── Multi-variable / multi-equation extensions ───────────────────────────
    /// Names of the unknown field variables, e.g. ["u"], ["u","v"], ["h","u","v"].
    #[serde(default)]
    pub variables: Vec<String>,

    /// One equation string per constraint.  Uses the same mini-DSL as the
    /// legacy `equation` field; variable names come from `variables`.
    #[serde(default)]
    pub equations: Vec<String>,

    /// Initial conditions per variable (and optionally time-derivative IC).
    /// Keys: variable name (e.g. "u") or "u.dt" for ∂u/∂t at t=0.
    /// Values: flat n×n array, or the special string "zero" (all zeros),
    ///         or the special string "grf" (Gaussian random field sample).
    #[serde(default)]
    pub initial_conditions: std::collections::HashMap<String, IcValue>,

    /// Named scalar coefficient fields — key is a name used in equations,
    /// value is a flat n×n array evaluated on the same grid as ICs.
    #[serde(default)]
    pub coef_fields: std::collections::HashMap<String, Vec<f64>>,

    /// Named SDF domains for non-periodic / complex-geometry BCs.
    #[serde(default)]
    pub domains: Vec<SdfDomain>,

    /// Explicit boundary conditions (applied after the PDE equations).
    #[serde(default)]
    pub bcs: Vec<BcSpec>,

    // ── History input (data-driven / autoregressive models) ─────────────────
    /// Reference to a previously uploaded tensor file (from POST /files).
    /// When present, the solver uses these historical snapshots as input
    /// instead of `initial_condition` / `initial_conditions`.
    pub history: Option<HistorySpec>,
}

/// Reference to an uploaded tensor file providing historical time-step data.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct HistorySpec {
    /// ID returned by `POST /files`.
    pub file_id: String,

    /// File format hint: "hdf5" | "npy" | "npz" | "pt".
    /// If omitted, the server infers from the stored file extension.
    pub format: Option<String>,

    /// Key / dataset path inside the file (for HDF5 or npz).
    /// For HDF5: e.g. "/data/u" or "snapshots".
    /// For npz:  the array name inside the archive.
    /// Omit for plain .npy or .pt tensors (single array).
    pub dataset_key: Option<String>,

    /// Which time-step indices from the file to use as the conditioning
    /// window. e.g. [0, 1, 2] selects the first three snapshots.
    /// If omitted, all time steps in the file are used.
    pub input_timesteps: Option<Vec<usize>>,

    /// Variable names corresponding to the last dimension of the loaded
    /// tensor. e.g. ["u", "v", "p"].
    /// If omitted, defaults to ["u"] for single-channel data.
    #[serde(default)]
    pub variables: Vec<String>,
}

/// Initial-condition value: either a flat float array, or a keyword token.
#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum IcValue {
    /// Flat n×n grid values (row-major).
    Array(Vec<f64>),
    /// Keyword: "zero" → all zeros; "grf" → Gaussian random field sample.
    Token(String),
}

/// Signed-distance-function domain specification.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SdfDomain {
    /// Identifier used in equations / bcs to reference this domain.
    pub name: String,
    /// Flat n×n SDF values (row-major, same n as IC grids).
    pub sdf: Vec<f64>,
    /// Semantic role of this domain.
    pub role: SdfRole,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SdfRole {
    /// Domain interior (defines where the PDE holds).
    Interior,
    /// Dirichlet boundary.
    BoundaryDirichlet,
    /// Neumann boundary.
    BoundaryNeumann,
    /// Absorbing / Mur boundary.
    BoundaryMur,
}

/// One boundary condition entry.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BcSpec {
    /// Name of the domain (must match an entry in `PdeSpec::domains`).
    pub domain: String,
    /// Variable expressions that sum to zero on this boundary, e.g.
    /// `["u"]` for u=0, or `["u", "g"]` for u+g=0.
    pub vars: Vec<String>,
    /// BC type: "dirichlet" | "neumann" | "mur" | "robin"
    pub bc_type: String,
    /// Optional coefficient used in Mur / Robin BCs (e.g. wave speed c).
    pub coef: Option<f64>,
}

// ---------------------------------------------------------------------------
// Query specification
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Solver response
// ---------------------------------------------------------------------------

/// Solver response.
#[derive(Debug, Serialize)]
pub struct SolveResponse {
    pub solver_used: String,
    /// Variable names corresponding to the last dimension (n_vars).
    /// For single-variable problems this is ["u"].
    pub variables: Vec<String>,
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
