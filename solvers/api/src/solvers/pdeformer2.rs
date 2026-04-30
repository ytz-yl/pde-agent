// PDEformer-2 solver backend
//
// Architecture: the Rust server spawns a Python subprocess using the
// `pdeformer2` conda environment. The subprocess reads a JSON problem
// description from stdin and writes a JSON result to stdout.
//
// This keeps the hot HTTP path in Rust (fast, concurrent) while the heavy
// MindSpore/numpy computation runs in a separate Python process.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Instant;

use anyhow::Context;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, info, warn};

use crate::error::ApiError;
use crate::models::{
    SolveMetadata, SolveRequest, SolveResponse, SolverCategory, SolverInfo, SolutionShape,
};
use super::Solver;

/// Path to the PDEformer-2 repository (relative to the workspace root or
/// absolute). Can be overridden via env var PDEFORMER2_DIR.
fn pdeformer2_dir() -> PathBuf {
    std::env::var("PDEFORMER2_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // Default: two levels up from solvers/api → project root,
            // then into solvers/ml/pdeformer-2
            let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            p.push("../ml/pdeformer-2");
            p
        })
}

/// Conda environment name (or full path to python binary).
fn python_bin() -> String {
    std::env::var("PDEFORMER2_PYTHON").unwrap_or_else(|_| {
        // Try to locate the conda env python
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/miniconda3/envs/pdeformer2/bin/python", home)
    })
}

pub struct PDEformer2Solver;

impl PDEformer2Solver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Solver for PDEformer2Solver {
    fn info(&self) -> SolverInfo {
        SolverInfo {
            id: "pdeformer2".into(),
            name: "PDEformer-2".into(),
            category: SolverCategory::MachineLearning,
            description: "Foundation model for 2D PDEs pretrained on ~40 TB of simulation data. \
                           Handles arbitrary PDE forms, domains, boundary conditions and time \
                           dependencies. Outputs solution at any spatio-temporal coordinate."
                .into(),
            supported_pde_types: vec![
                "elliptic".into(),
                "parabolic".into(),
                "hyperbolic".into(),
                "nonlinear_conservation_law".into(),
                "reaction_diffusion".into(),
                "navier_stokes".into(),
            ],
            backend: "MindSpore / Python".into(),
            available: true,
        }
    }

    async fn solve(&self, req: &SolveRequest) -> Result<SolveResponse, ApiError> {
        let t0 = Instant::now();

        // Serialize the request to JSON — the Python script reads it from stdin
        let payload = serde_json::to_string(req)
            .context("Failed to serialize solve request")
            .map_err(ApiError::Internal)?;

        let pdeformer2_dir = pdeformer2_dir();
        let python = python_bin();
        let script = pdeformer2_dir.join("../../api/scripts/pdeformer2_infer.py");

        debug!(
            "Launching Python bridge: {} {} (cwd: {})",
            python,
            script.display(),
            pdeformer2_dir.display()
        );

        let mut child = Command::new(&python)
            .arg(&script)
            .current_dir(&pdeformer2_dir)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn Python process: {}", python))
            .map_err(ApiError::Internal)?;

        // Write request JSON to stdin
        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin
                .write_all(payload.as_bytes())
                .await
                .context("Failed to write to Python stdin")
                .map_err(ApiError::Internal)?;
            // stdin is dropped here → EOF sent to Python
        }

        let output = child
            .wait_with_output()
            .await
            .context("Failed to wait for Python process")
            .map_err(ApiError::Internal)?;

        let wall_time_ms = t0.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("PDEformer-2 stderr:\n{}", stderr);
            return Err(ApiError::SolverError(format!(
                "Python process exited with {}: {}",
                output.status,
                stderr.lines().last().unwrap_or("(no output)")
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Log any Python warnings/info written to stderr
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            info!("PDEformer-2 stderr:\n{}", stderr);
        }

        // Parse the JSON result produced by the Python script
        let raw: serde_json::Value = serde_json::from_str(&stdout)
            .with_context(|| format!("Failed to parse Python output as JSON:\n{}", stdout))
            .map_err(ApiError::Internal)?;

        if let Some(err) = raw.get("error").and_then(|v| v.as_str()) {
            return Err(ApiError::SolverError(err.to_string()));
        }

        let solution: Vec<Vec<Vec<Vec<f64>>>> =
            serde_json::from_value(raw["solution"].clone())
                .context("Failed to parse 'solution' field")
                .map_err(ApiError::Internal)?;

        let n_t = solution.len();
        let n_x = solution.first().map(|t| t.len()).unwrap_or(0);
        let n_y = solution.first().and_then(|t| t.first()).map(|x| x.len()).unwrap_or(0);
        let n_vars = solution
            .first()
            .and_then(|t| t.first())
            .and_then(|x| x.first())
            .map(|y| y.len())
            .unwrap_or(0);

        let notes: Vec<String> = raw["notes"]
            .as_array()
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(str::to_owned)).collect())
            .unwrap_or_default();

        Ok(SolveResponse {
            solver_used: "pdeformer2".into(),
            solution,
            shape: SolutionShape { n_t, n_x, n_y, n_vars },
            metadata: SolveMetadata {
                wall_time_ms,
                backend: "MindSpore 2.8 / CPU".into(),
                notes,
            },
        })
    }
}
