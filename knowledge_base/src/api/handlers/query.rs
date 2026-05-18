/// Query handlers — read-only endpoints for agents and users.
///
/// Route summary:
///   GET  /equations                       list equations (optional ?pde_type=)
///   GET  /equations/:id                   get equation by id
///   GET  /equations/:id/solvers           which AI + numerical methods solve it
///   GET  /equations/:id/conditions        conditions associated with the equation
///   GET  /equations/:id/datasets          benchmark datasets for the equation
///   GET  /equations/:id/papers            papers studying this equation
///   GET  /ai-models                       list AI models (optional ?training_type=)
///   GET  /ai-models/:id                   get AI model by id
///   GET  /ai-models/:id/profile           full AI model profile
///   GET  /ai-models/:id/equations         equations an AI model solves
///   GET  /ai-models/:id/papers            papers that proposed this model
///   GET  /numerical-methods               list numerical methods
///   GET  /numerical-methods/:id           get numerical method by id
///   GET  /numerical-methods/:id/papers    papers that proposed this method
///   GET  /papers                          list papers (optional ?year=)
///   GET  /papers/:id                      get paper node + abstract
///   GET  /papers/:id/profile              full paper profile (proposes/studies/cites)
///   GET  /search                          name search across all nodes

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    api::AppState,
    retrieval::query::{
        ai_model_profile, benchmark_leaderboard, conditions_for_equation, datasets_for_equation,
        equations_solved_by, paper_profile, papers_proposing, papers_studying, results_for_method,
        search_by_name, solvers_for_equation,
    },
    store::{
        content_repo::get_content,
        node_repo::{
            get_ai_model, get_benchmark, get_equation, get_numerical_method, get_paper,
            list_ai_models, list_benchmarks, list_equations, list_numerical_methods, list_papers,
        },
        schema::{LABEL_AI_MODEL, LABEL_BENCHMARK, LABEL_NUMERICAL_METHOD, LABEL_PAPER},
    },
};

// ── Health ────────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "pde-knowledge-base" }))
}

// ── Equations ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListEquationsParams {
    pub pde_type: Option<String>,
}

pub async fn list_equations_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListEquationsParams>,
) -> Result<impl IntoResponse, AppError> {
    let eqs = list_equations(&state.graph, params.pde_type.as_deref()).await?;
    Ok(Json(eqs))
}

pub async fn get_equation_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match get_equation(&state.graph, &id).await? {
        Some(eq) => Ok(Json(eq).into_response()),
        None => Ok(not_found("equation")),
    }
}

pub async fn equation_solvers_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(solvers_for_equation(&state.graph, &id).await?))
}

pub async fn equation_conditions_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(conditions_for_equation(&state.graph, &id).await?))
}

pub async fn equation_datasets_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(datasets_for_equation(&state.graph, &id).await?))
}

pub async fn equation_papers_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(papers_studying(&state.graph, &id).await?))
}

// ── AI Models ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListAIModelsParams {
    pub training_type: Option<String>,
}

pub async fn list_ai_models_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListAIModelsParams>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(list_ai_models(&state.graph, params.training_type.as_deref()).await?))
}

pub async fn get_ai_model_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match get_ai_model(&state.graph, &id).await? {
        Some(m) => Ok(Json(m).into_response()),
        None => Ok(not_found("ai_model")),
    }
}

pub async fn ai_model_profile_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match ai_model_profile(&state.graph, &id).await? {
        Some(p) => Ok(Json(p).into_response()),
        None => Ok(not_found("ai_model")),
    }
}

pub async fn ai_model_equations_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(equations_solved_by(&state.graph, LABEL_AI_MODEL, &id).await?))
}

pub async fn ai_model_papers_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(papers_proposing(&state.graph, LABEL_AI_MODEL, &id).await?))
}

// ── Numerical Methods ─────────────────────────────────────────────────────────

pub async fn list_numerical_methods_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(list_numerical_methods(&state.graph).await?))
}

pub async fn get_numerical_method_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match get_numerical_method(&state.graph, &id).await? {
        Some(m) => Ok(Json(m).into_response()),
        None => Ok(not_found("numerical_method")),
    }
}

pub async fn numerical_method_papers_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(papers_proposing(&state.graph, LABEL_NUMERICAL_METHOD, &id).await?))
}

// ── Papers ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct ListPapersParams {
    pub year: Option<u32>,
}

pub async fn list_papers_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListPapersParams>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(list_papers(&state.graph, params.year).await?))
}

/// GET /papers/:id — returns the paper node merged with its abstract from SQLite.
pub async fn get_paper_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let paper = match get_paper(&state.graph, &id).await? {
        Some(p) => p,
        None => return Ok(not_found("paper")),
    };
    // Attach abstract if available
    let content = {
        let db = state.content_db.lock().await;
        get_content(&db, &id, LABEL_PAPER).unwrap_or(None)
    };
    let abstract_text = content.as_ref().and_then(|c| c.abstract_text.clone());
    let notes = content.as_ref().and_then(|c| c.notes.clone());

    Ok(Json(serde_json::json!({
        "paper": paper,
        "abstract": abstract_text,
        "notes": notes,
    }))
    .into_response())
}

pub async fn paper_profile_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match paper_profile(&state.graph, &id).await? {
        Some(p) => Ok(Json(p).into_response()),
        None => Ok(not_found("paper")),
    }
}

// ── Benchmarks ────────────────────────────────────────────────────────────────

pub async fn list_benchmarks_handler(
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(list_benchmarks(&state.graph).await?))
}

/// GET /benchmarks/:id — returns the Benchmark node merged with its long-form
/// protocol from SQLite content (if any).
pub async fn get_benchmark_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let benchmark = match get_benchmark(&state.graph, &id).await? {
        Some(b) => b,
        None => return Ok(not_found("benchmark")),
    };
    let content = {
        let db = state.content_db.lock().await;
        get_content(&db, &id, LABEL_BENCHMARK).unwrap_or(None)
    };
    let notes = content.as_ref().and_then(|c| c.notes.clone());

    Ok(Json(serde_json::json!({
        "benchmark": benchmark,
        "notes": notes,
    }))
    .into_response())
}

pub async fn benchmark_leaderboard_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    match benchmark_leaderboard(&state.graph, &id).await? {
        Some(lb) => Ok(Json(lb).into_response()),
        None => Ok(not_found("benchmark")),
    }
}

pub async fn results_for_ai_model_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(results_for_method(&state.graph, LABEL_AI_MODEL, &id).await?))
}

pub async fn results_for_numerical_method_handler(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(results_for_method(&state.graph, LABEL_NUMERICAL_METHOD, &id).await?))
}

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchParams {
    pub q: String,
}

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    Ok(Json(search_by_name(&state.graph, &params.q).await?))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn not_found(entity: &str) -> axum::response::Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "not found", "entity": entity })),
    )
        .into_response()
}

// ── Error type ────────────────────────────────────────────────────────────────

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("handler error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": self.0.to_string() })),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}
