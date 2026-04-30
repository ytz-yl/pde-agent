/// Structured (SQL-backed) queries for papers and methods.
use anyhow::Result;
use rusqlite::Connection;

use crate::store::{
    method_repo,
    paper_repo,
    schema::{Method, MethodRelation, Paper, TagType},
};

// ── Paper queries ─────────────────────────────────────────────────────────────

/// Parameters for a structured paper query.
#[derive(Debug, Default, serde::Deserialize)]
pub struct PaperQuery {
    /// Filter by PDE type tag (e.g. "navier_stokes").
    pub pde_type: Option<String>,
    /// Filter by method tag (e.g. "fno").
    pub method: Option<String>,
    /// Filter by application domain tag (e.g. "fluid_dynamics").
    pub domain: Option<String>,
    /// Filter by benchmark tag.
    pub benchmark: Option<String>,
    /// Maximum number of results.
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    20
}

/// Execute a structured paper query.
///
/// If multiple tag filters are provided, the most specific one is used
/// (priority: pde_type > method > domain > benchmark).
pub fn query_papers(conn: &Connection, q: &PaperQuery) -> Result<Vec<Paper>> {
    if let Some(ref v) = q.pde_type {
        paper_repo::papers_by_tag(conn, TagType::PdeType, v, q.limit)
    } else if let Some(ref v) = q.method {
        paper_repo::papers_by_tag(conn, TagType::Method, v, q.limit)
    } else if let Some(ref v) = q.domain {
        paper_repo::papers_by_tag(conn, TagType::Domain, v, q.limit)
    } else if let Some(ref v) = q.benchmark {
        paper_repo::papers_by_tag(conn, TagType::Benchmark, v, q.limit)
    } else {
        paper_repo::get_recent_papers(conn, None, q.limit)
    }
}

// ── Method queries ────────────────────────────────────────────────────────────

/// List all methods, optionally filtered by category.
pub fn list_methods(conn: &Connection, category: Option<&str>) -> Result<Vec<Method>> {
    method_repo::list_methods(conn, category)
}

/// Get a method by id.
pub fn get_method(conn: &Connection, id: &str) -> Result<Option<Method>> {
    method_repo::get_method(conn, id)
}

/// Get all methods related to `id`, with their relation metadata.
pub fn get_related_methods(
    conn: &Connection,
    id: &str,
) -> Result<Vec<(Method, MethodRelation)>> {
    method_repo::get_related_methods(conn, id, None)
}

/// Get papers that use or propose a given method.
pub fn papers_for_method(conn: &Connection, method_id: &str, limit: usize) -> Result<Vec<Paper>> {
    paper_repo::papers_by_tag(conn, TagType::Method, method_id, limit)
}

/// Get recent papers, optionally filtered by domain.
pub fn recent_papers(
    conn: &Connection,
    domain: Option<&str>,
    limit: usize,
) -> Result<Vec<Paper>> {
    paper_repo::get_recent_papers(conn, domain, limit)
}
