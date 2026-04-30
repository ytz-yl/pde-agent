// Classical PDE solver backend (py-pde via Python subprocess bridge)

use std::time::Instant;

use anyhow::Context;
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use std::process::Stdio;
use tracing::{debug, info, warn};

use crate::error::ApiError;
use crate::models::{
    SolveMetadata, SolveRequest, SolveResponse, SolverCategory, SolverInfo, SolutionShape,
};
use super::Solver;

fn python_bin() -> String {
    std::env::var("CLASSICAL_PYTHON").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/miniconda3/envs/classical-pde/bin/python3", home)
    })
}

fn script_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("scripts/classical_solve.py");
    p
}

pub struct ClassicalSolver;

impl ClassicalSolver {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Solver for ClassicalSolver {
    fn info(&self) -> SolverInfo {
        SolverInfo {
            id: "classical".into(),
            name: "Classical FDM (py-pde)".into(),
            category: SolverCategory::Classical,
            description: "Finite difference / pseudospectral solver powered by the py-pde \
                           library. Supports diffusion, wave, Allen-Cahn, Cahn-Hilliard, and \
                           arbitrary symbolic PDEs on 2D Cartesian grids."
                .into(),
            supported_pde_types: vec![
                "diffusion".into(),
                "heat".into(),
                "wave".into(),
                "allen_cahn".into(),
                "cahn_hilliard".into(),
                "custom_symbolic".into(),
            ],
            backend: "py-pde / FDM / Python".into(),
            available: true,
        }
    }

    async fn solve(&self, req: &SolveRequest) -> Result<SolveResponse, ApiError> {
        let t0 = Instant::now();

        let payload = serde_json::to_string(req)
            .context("Failed to serialize solve request")
            .map_err(ApiError::Internal)?;

        let python = python_bin();
        let script = script_path();

        debug!(
            "Launching classical Python bridge: {} {}",
            python,
            script.display()
        );

        let mut child = Command::new(&python)
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn Python process: {}", python))
            .map_err(ApiError::Internal)?;

        if let Some(stdin) = child.stdin.take() {
            let mut stdin = stdin;
            stdin
                .write_all(payload.as_bytes())
                .await
                .context("Failed to write to Python stdin")
                .map_err(ApiError::Internal)?;
        }

        let output = child
            .wait_with_output()
            .await
            .context("Failed to wait for Python process")
            .map_err(ApiError::Internal)?;

        let wall_time_ms = t0.elapsed().as_millis() as u64;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("classical solver stderr:\n{}", stderr);
            return Err(ApiError::SolverError(format!(
                "Python process exited with {}: {}",
                output.status,
                stderr.lines().last().unwrap_or("(no output)")
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            info!("classical solver stderr:\n{}", stderr);
        }

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
            solver_used: "classical".into(),
            solution,
            shape: SolutionShape { n_t, n_x, n_y, n_vars },
            metadata: SolveMetadata {
                wall_time_ms,
                backend: "py-pde 0.54 / FDM".into(),
                notes,
            },
        })
    }
}
