/// Method recommender: given a PDE type and constraints, suggest the best methods.
use anyhow::Result;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

use crate::store::{method_repo, schema::Method};

// ── Request / Response ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RecommendRequest {
    /// PDE type, e.g. "navier_stokes", "heat_equation", "wave_equation".
    pub pde_type: String,
    /// Application domain hint, e.g. "fluid_dynamics", "elasticity".
    pub domain: Option<String>,
    /// Additional constraints as free-text keywords, e.g. ["irregular_domain", "inverse_problem"].
    #[serde(default)]
    pub constraints: Vec<String>,
    /// Maximum number of recommendations.
    #[serde(default = "default_top_k")]
    pub top_k: usize,
}

fn default_top_k() -> usize { 3 }

#[derive(Debug, Serialize)]
pub struct Recommendation {
    pub method: Method,
    /// Human-readable justification for the recommendation.
    pub reason: String,
    /// Confidence score in [0, 1].
    pub score: f32,
}

// ── Core logic ────────────────────────────────────────────────────────────────

/// Score all known methods against the request and return the top-k.
///
/// This is a rule-based heuristic designed to be replaced or augmented by
/// an LLM-based ranker later.  Rules are intentionally explicit so they can
/// be audited and extended.
pub fn recommend(conn: &Connection, req: &RecommendRequest) -> Result<Vec<Recommendation>> {
    let all_methods = method_repo::list_methods(conn, None)?;
    let constraints_lower: Vec<String> = req
        .constraints
        .iter()
        .map(|s| s.to_lowercase())
        .collect();

    let mut scored: Vec<(Method, f32, String)> = all_methods
        .into_iter()
        .map(|m| {
            let (score, reason) = score_method(&m, req, &constraints_lower);
            (m, score, reason)
        })
        .collect();

    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(req.top_k);

    Ok(scored
        .into_iter()
        .map(|(method, score, reason)| Recommendation { method, score, reason })
        .collect())
}

/// Score a single method against the recommendation request.
/// Returns (score, human-readable reason).
fn score_method(method: &Method, req: &RecommendRequest, constraints: &[String]) -> (f32, String) {
    let mut score: f32 = 0.0;
    let mut reasons: Vec<&str> = Vec::new();

    let pde = req.pde_type.to_lowercase();
    let domain = req.domain.as_deref().map(str::to_lowercase).unwrap_or_default();
    let method_tags: Vec<String> = method.tags.iter().map(|t| t.to_lowercase()).collect();
    let method_id = method.id.as_str();

    // ── Category penalties / bonuses ─────────────────────────────────────────

    // Prefer ML methods for high-dimensional, parametric, or inverse problems
    let wants_ml = constraints.iter().any(|c| {
        matches!(c.as_str(), "inverse_problem" | "high_dimensional" | "parametric" | "fast_inference")
    });
    if wants_ml && method.category == crate::store::schema::MethodCategory::Ml {
        score += 0.3;
        reasons.push("ML methods excel at the requested constraint");
    }

    // Prefer classical methods for highly accurate, complex-geometry, or conservation-law problems
    let wants_classical = constraints.iter().any(|c| {
        matches!(c.as_str(), "high_accuracy" | "complex_geometry" | "conservation_laws" | "guaranteed_convergence")
    });
    if wants_classical && method.category == crate::store::schema::MethodCategory::Classical {
        score += 0.3;
        reasons.push("Classical methods are preferred for accuracy/convergence guarantees");
    }

    // ── PDE-type specific rules ───────────────────────────────────────────────

    match pde.as_str() {
        "navier_stokes" | "fluid_dynamics" => {
            if matches!(method_id, "fvm" | "fem") {
                score += 0.4;
                reasons.push("FVM/FEM are industry standard for Navier-Stokes");
            }
            if method_id == "fno" {
                score += 0.3;
                reasons.push("FNO has shown strong results on Navier-Stokes benchmarks");
            }
        }
        "heat_equation" | "diffusion" => {
            if matches!(method_id, "fdm" | "fem") {
                score += 0.4;
                reasons.push("FDM/FEM are efficient and well-understood for parabolic problems");
            }
        }
        "wave_equation" | "hyperbolic" => {
            if matches!(method_id, "fdm" | "spectral") {
                score += 0.4;
                reasons.push("FDM/Spectral methods handle wave propagation well");
            }
        }
        "poisson" | "elliptic" => {
            if matches!(method_id, "fem" | "spectral") {
                score += 0.4;
                reasons.push("FEM/Spectral are optimal for elliptic problems");
            }
        }
        _ => {
            // Generic: FNO and PDEformer are universal
            if matches!(method_id, "fno" | "pdeformer") {
                score += 0.2;
                reasons.push("Universal ML operators handle diverse PDE families");
            }
        }
    }

    // ── Domain hints ──────────────────────────────────────────────────────────

    if domain.contains("fluid") && method_tags.iter().any(|t| t.contains("cfd")) {
        score += 0.15;
        reasons.push("Method is designed for CFD applications");
    }

    // ── Irregular domain constraint ───────────────────────────────────────────

    if constraints.contains(&"irregular_domain".to_string()) {
        if method_id == "fem" {
            score += 0.3;
            reasons.push("FEM handles unstructured/irregular meshes natively");
        }
        if matches!(method_id, "pinns" | "deeponet") {
            score += 0.2;
            reasons.push("Mesh-free methods naturally handle irregular domains");
        }
        if method_id == "fdm" {
            score -= 0.2;
            // Not a reason to show — just a penalty
        }
    }

    let reason = if reasons.is_empty() {
        "General-purpose method applicable to this PDE class.".to_string()
    } else {
        reasons.join("; ")
    };

    (score.max(0.0), reason)
}

// ── Method comparison ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ComparisonReport {
    pub method_a: Method,
    pub method_b: Method,
    pub relations: Vec<RelationSummary>,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub struct RelationSummary {
    pub kind: String,
    pub weight: f32,
}

/// Compare two methods: fetch their relation edges and produce a summary.
pub fn compare_methods(
    conn: &Connection,
    id_a: &str,
    id_b: &str,
) -> Result<Option<ComparisonReport>> {
    let method_a = match method_repo::get_method(conn, id_a)? {
        Some(m) => m,
        None => return Ok(None),
    };
    let method_b = match method_repo::get_method(conn, id_b)? {
        Some(m) => m,
        None => return Ok(None),
    };

    // Find direct relations between the two methods
    let related = method_repo::get_related_methods(conn, id_a, None)?;
    let relations: Vec<RelationSummary> = related
        .into_iter()
        .filter(|(m, _)| m.id == id_b)
        .map(|(_, rel)| RelationSummary {
            kind: rel.relation.as_str().to_string(),
            weight: rel.weight,
        })
        .collect();

    let summary = if relations.is_empty() {
        format!(
            "{} and {} have no direct relationship recorded in the knowledge base.",
            method_a.name, method_b.name
        )
    } else {
        let kinds: Vec<&str> = relations.iter().map(|r| r.kind.as_str()).collect();
        format!(
            "{} and {} are related by: {}.",
            method_a.name,
            method_b.name,
            kinds.join(", ")
        )
    };

    Ok(Some(ComparisonReport {
        method_a,
        method_b,
        relations,
        summary,
    }))
}
