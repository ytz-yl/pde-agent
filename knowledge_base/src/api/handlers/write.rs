/// Write handlers — internal endpoints for agent-driven knowledge ingestion.
///
/// Route summary:
///   POST   /internal/nodes               upsert any node (KnowledgeNode JSON)
///   DELETE /internal/nodes/:label/:id    delete a node (+ all its relations)
///   POST   /internal/relations           upsert a relation between two nodes
///   DELETE /internal/relations           delete a specific relation
///   POST   /internal/content             upsert abstract/notes for any node
///
/// For Paper nodes, the upsert body may include optional top-level fields
/// `"abstract"` and `"notes"` which are stored in SQLite (not Neo4j).

use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::{
    api::AppState,
    store::{
        content_repo::{delete_content, upsert_content, NodeContent},
        node_repo::{delete_node, upsert_bench_result, upsert_node},
        relation_repo::{delete_relation, is_valid_relation_type, upsert_relation},
        schema::{BenchResult, KnowledgeNode, LABEL_BENCH_RESULT, Relation, SourceType},
    },
};

use std::str::FromStr;

use super::query::AppError;

// ── Node upsert ───────────────────────────────────────────────────────────────

/// Upsert a knowledge node.
///
/// For Paper nodes the body may also carry `"abstract"` and `"notes"` string
/// fields at the top level; these are forwarded to SQLite content_repo.
///
/// Example body (Paper):
/// ```json
/// {
///   "node_type": "paper",
///   "id": "2010.08895",
///   "title": "Fourier Neural Operator for Parametric PDEs",
///   "authors": ["Li, Z.", "Kovachki, N."],
///   "published_year": 2021,
///   "arxiv_id": "2010.08895",
///   "tags": ["operator-learning"],
///   "abstract": "We introduce the Fourier neural operator …",
///   "notes": "Key result: FNO on Navier-Stokes."
/// }
/// ```
pub async fn upsert_node_handler(
    State(state): State<Arc<AppState>>,
    Json(mut body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Pull out optional content fields before deserialising the node.
    let abstract_text = body
        .as_object_mut()
        .and_then(|o| o.remove("abstract"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));
    let notes = body
        .as_object_mut()
        .and_then(|o| o.remove("notes"))
        .and_then(|v| v.as_str().map(|s| s.to_string()));

    let node: KnowledgeNode = serde_json::from_value(body)
        .map_err(|e| anyhow::anyhow!("invalid node body: {}", e))?;

    let label = node.label().to_string();

    // Write structural data to Neo4j. The dispatcher returns the resolved id —
    // important for BenchResult, where the id may have been auto-generated.
    let id = upsert_node(&state.graph, &node).await?;

    // If any content fields were provided, persist them to SQLite.
    if abstract_text.is_some() || notes.is_some() {
        let content = NodeContent {
            node_id: id.clone(),
            node_type: label.clone(),
            abstract_text,
            notes,
        };
        let db = state.content_db.lock().await;
        upsert_content(&db, &content)?;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "action": "upserted",
            "label": label,
            "id": id,
        })),
    ))
}

// ── BenchResult convenience submission ────────────────────────────────────────

#[derive(Deserialize)]
pub struct SubmitResultRequest {
    pub method_id: String,
    pub method_label: String,
    pub benchmark_id: String,
    pub value: f64,
    pub source_type: String,
    #[serde(default)]
    pub source_paper_id: Option<String>,
    #[serde(default)]
    pub hardware: Option<String>,
    #[serde(default)]
    pub code_ref: Option<String>,
    #[serde(default)]
    pub recorded_at: Option<String>,
    /// Free-form long-form notes — stored to SQLite content_repo.
    #[serde(default)]
    pub notes: Option<String>,
}

/// POST /internal/results — single-call submission of one BenchResult.
///
/// Wires the OF_METHOD / ON_BENCHMARK / REPORTED_IN edges automatically.
/// Saves under an auto-generated id; the response returns it.
pub async fn submit_result_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitResultRequest>,
) -> Result<impl IntoResponse, AppError> {
    let source_type = SourceType::from_str(&req.source_type)
        .map_err(|e| anyhow::anyhow!("invalid source_type: {}", e))?;

    let result = BenchResult {
        id: None,
        method_id: req.method_id,
        method_label: req.method_label,
        benchmark_id: req.benchmark_id,
        value: req.value,
        source_type,
        source_paper_id: req.source_paper_id.filter(|s| !s.is_empty()),
        hardware: req.hardware.filter(|s| !s.is_empty()),
        code_ref: req.code_ref.filter(|s| !s.is_empty()),
        recorded_at: req.recorded_at.filter(|s| !s.is_empty()),
    };

    let id = upsert_bench_result(&state.graph, &result).await?;

    if let Some(notes) = req.notes.filter(|s| !s.is_empty()) {
        let content = NodeContent {
            node_id: id.clone(),
            node_type: LABEL_BENCH_RESULT.to_string(),
            abstract_text: None,
            notes: Some(notes),
        };
        let db = state.content_db.lock().await;
        upsert_content(&db, &content)?;
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "action": "upserted",
            "label": LABEL_BENCH_RESULT,
            "id": id,
        })),
    ))
}

// ── Node delete ───────────────────────────────────────────────────────────────

pub async fn delete_node_handler(
    State(state): State<Arc<AppState>>,
    Path((label, id)): Path<(String, String)>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = delete_node(&state.graph, &label, &id).await?;
    // Also remove content from SQLite.
    if deleted {
        let db = state.content_db.lock().await;
        delete_content(&db, &id, &label)?;
        Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok", "deleted": true })))
            .into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "not found" })))
            .into_response())
    }
}

// ── Relation upsert ───────────────────────────────────────────────────────────

pub async fn upsert_relation_handler(
    State(state): State<Arc<AppState>>,
    Json(rel): Json<Relation>,
) -> Result<impl IntoResponse, AppError> {
    if !is_valid_relation_type(&rel.relation_type) {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": format!("unknown relation_type: {}", rel.relation_type),
                "valid_types": crate::store::relation_repo::VALID_RELATION_TYPES,
            })),
        )
            .into_response());
    }
    upsert_relation(&state.graph, &rel).await?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "action": "upserted",
            "relation": rel.relation_type,
            "from": format!("{}:{}", rel.from_label, rel.from_id),
            "to":   format!("{}:{}", rel.to_label, rel.to_id),
        })),
    )
        .into_response())
}

// ── Relation delete ───────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DeleteRelationRequest {
    pub from_label: String,
    pub from_id: String,
    pub to_label: String,
    pub to_id: String,
    pub relation_type: String,
}

pub async fn delete_relation_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DeleteRelationRequest>,
) -> Result<impl IntoResponse, AppError> {
    let deleted = delete_relation(
        &state.graph,
        &req.from_label,
        &req.from_id,
        &req.to_label,
        &req.to_id,
        &req.relation_type,
    )
    .await?;

    if deleted {
        Ok((StatusCode::OK, Json(serde_json::json!({ "status": "ok", "deleted": true })))
            .into_response())
    } else {
        Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({ "error": "relation not found" })))
            .into_response())
    }
}

// ── Content upsert (standalone) ───────────────────────────────────────────────

/// Upsert abstract/notes independently of the node upsert.
/// Useful when an agent wants to update only the text without touching Neo4j.
///
/// Body: `{ "node_id": "…", "node_type": "Paper", "abstract": "…", "notes": "…" }`
pub async fn upsert_content_handler(
    State(state): State<Arc<AppState>>,
    Json(content): Json<NodeContent>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.content_db.lock().await;
    upsert_content(&db, &content)?;
    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "ok",
            "node_id": content.node_id,
            "node_type": content.node_type,
        })),
    ))
}
