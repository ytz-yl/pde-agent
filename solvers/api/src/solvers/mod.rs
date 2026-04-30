// Solver trait and registry

pub mod classical;
pub mod pdeformer2;

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;

use crate::error::ApiError;
use crate::models::{SolveRequest, SolveResponse, SolverInfo};

/// Every solver backend implements this trait.
#[async_trait]
pub trait Solver: Send + Sync {
    fn info(&self) -> SolverInfo;
    async fn solve(&self, req: &SolveRequest) -> Result<SolveResponse, ApiError>;
}

/// Global solver registry, built once at startup.
pub struct SolverRegistry {
    solvers: HashMap<String, Arc<dyn Solver>>,
}

impl SolverRegistry {
    pub fn new() -> Self {
        let mut reg = Self {
            solvers: HashMap::new(),
        };
        reg.register(Arc::new(pdeformer2::PDEformer2Solver::new()));
        reg.register(Arc::new(classical::ClassicalSolver::new()));
        reg
    }

    fn register(&mut self, solver: Arc<dyn Solver>) {
        let id = solver.info().id.clone();
        self.solvers.insert(id, solver);
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Solver>> {
        self.solvers.get(id).cloned()
    }

    pub fn list(&self) -> Vec<SolverInfo> {
        self.solvers.values().map(|s| s.info()).collect()
    }

    /// Returns the default solver id.
    pub fn default_id() -> &'static str {
        "pdeformer2"
    }
}
