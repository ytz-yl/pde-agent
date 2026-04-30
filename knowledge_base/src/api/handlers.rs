/// axum request handlers.
use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    ingestion::{
        arxiv_fetcher::ArxivQuery,
        classifier::{embed_text, extract_tags},
    },
    retrieval::{
        recommender::{
            compare_methods as do_compare, recommend as do_recommend, RecommendRequest,
        },
        structured::{
            get_method as do_get_method, get_related_methods as do_related,
            list_methods as do_list_methods, query_papers, recent_papers as do_recent_papers,
            PaperQuery,
        },
    },
    store::{
        paper_repo::upsert_paper,
        schema::{Paper, PaperTag},
    },
};

use super::AppState;

// ── Health ────────────────────────────────────────────────────────────────────

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({ "status": "ok", "service": "knowledge-base" }))
}

// ── Search ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SearchParams {
    /// Free-text query (required).
    pub q: String,
    /// Optional PDE type filter.
    pub pde_type: Option<String>,
    /// Optional method filter.
    pub method: Option<String>,
    /// Optional domain filter.
    pub domain: Option<String>,
    /// Number of results (default 10).
    #[serde(default = "default_limit")]
    pub limit: usize,
    /// Use hybrid (vector + FTS) search; default true.
    #[serde(default = "default_true")]
    pub hybrid: bool,
}

fn default_limit() -> usize { 10 }
fn default_true() -> bool { true }

pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SearchParams>,
) -> Result<impl IntoResponse, AppError> {
    // Step 1: structured pre-filter (sync DB work, no await).
    let structured_ids: Option<std::collections::HashSet<String>> = {
        let db = state.db.lock().await;
        if params.pde_type.is_some() || params.method.is_some() || params.domain.is_some() {
            let pq = PaperQuery {
                pde_type: params.pde_type.clone(),
                method: params.method.clone(),
                domain: params.domain.clone(),
                benchmark: None,
                limit: 500,
            };
            let papers = query_papers(&db, &pq)?;
            Some(papers.into_iter().map(|p| p.id).collect())
        } else {
            None
        }
        // MutexGuard dropped here before any .await
    };

    // Step 2: embed the query (network, no lock held).
    use crate::ingestion::classifier::embed_text as do_embed;
    let embedding = do_embed(&state.http_client, &state.llm_cfg, &params.q).await?;

    // Step 3: ANN search (in-memory, no lock).
    let vec_hits = state.vector_index.search(&embedding, params.limit * 2)?;

    // Step 4: FTS search + fetch papers (sync DB work).
    let results: Vec<(Paper, f32)> = {
        use crate::retrieval::semantic::sanitise_fts_query_pub;
        use crate::store::paper_repo;

        let db = state.db.lock().await;
        let fts_query = sanitise_fts_query_pub(&params.q);
        let fts_papers = paper_repo::search_papers_fts(&db, &fts_query, params.limit * 2)
            .unwrap_or_default();

        // Merge vector + FTS scores
        let fts_max = fts_papers.len() as f32;
        let mut combined: std::collections::HashMap<String, f32> = std::collections::HashMap::new();
        for (id, score) in &vec_hits {
            *combined.entry(id.clone()).or_default() += score * 0.7;
        }
        for (i, p) in fts_papers.iter().enumerate() {
            let score = if fts_max > 0.0 { (fts_max - i as f32) / fts_max } else { 0.0 };
            *combined.entry(p.id.clone()).or_default() += score * 0.3;
        }

        let mut ranked: Vec<(String, f32)> = combined.into_iter().collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(params.limit);

        let mut out = Vec::with_capacity(ranked.len());
        for (id, score) in ranked {
            if let Some(paper) = paper_repo::get_paper(&db, &id)? {
                out.push((paper, score));
            }
        }
        out
        // MutexGuard dropped here
    };

    // Step 5: apply structural filter.
    let results: Vec<_> = results
        .into_iter()
        .filter(|(p, _)| {
            structured_ids
                .as_ref()
                .map(|ids| ids.contains(&p.id))
                .unwrap_or(true)
        })
        .collect();

    #[derive(Serialize)]
    struct Hit {
        score: f32,
        paper: Paper,
    }

    let hits: Vec<Hit> = results
        .into_iter()
        .map(|(paper, score)| Hit { score, paper })
        .collect();

    Ok(Json(hits))
}

// ── Papers ────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct RecentParams {
    pub domain: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: usize,
}

pub async fn recent_papers(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RecentParams>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    let papers = do_recent_papers(&db, params.domain.as_deref(), params.limit)?;
    Ok(Json(papers))
}

pub async fn get_paper(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    match crate::store::paper_repo::get_paper(&db, &id)? {
        Some(p) => Ok(Json(p).into_response()),
        None => Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not found"}))).into_response()),
    }
}

// ── Methods ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct MethodListParams {
    pub category: Option<String>,
}

pub async fn list_methods(
    State(state): State<Arc<AppState>>,
    Query(params): Query<MethodListParams>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    let methods = do_list_methods(&db, params.category.as_deref())?;
    Ok(Json(methods))
}

pub async fn get_method(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    match do_get_method(&db, &id)? {
        Some(m) => Ok(Json(m).into_response()),
        None => Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "not found"}))).into_response()),
    }
}

pub async fn related_methods(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    let related = do_related(&db, &id)?;

    #[derive(Serialize)]
    struct RelatedEntry {
        relation: String,
        weight: f32,
        method: crate::store::schema::Method,
    }

    let entries: Vec<RelatedEntry> = related
        .into_iter()
        .map(|(method, rel)| RelatedEntry {
            relation: rel.relation.as_str().to_string(),
            weight: rel.weight,
            method,
        })
        .collect();

    Ok(Json(entries))
}

#[derive(Deserialize)]
pub struct CompareParams {
    pub a: String,
    pub b: String,
}

pub async fn compare_methods(
    State(state): State<Arc<AppState>>,
    Query(params): Query<CompareParams>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    match do_compare(&db, &params.a, &params.b)? {
        Some(report) => Ok(Json(report).into_response()),
        None => Ok((StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "one or both methods not found"}))).into_response()),
    }
}

// ── Recommendations ───────────────────────────────────────────────────────────

pub async fn recommend(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecommendRequest>,
) -> Result<impl IntoResponse, AppError> {
    let db = state.db.lock().await;
    let recs = do_recommend(&db, &req)?;
    Ok(Json(recs))
}

// ── Ingestion ─────────────────────────────────────────────────────────────────

/// Manually ingest a single paper (e.g. a paper not on arXiv).
#[derive(Deserialize)]
pub struct IngestPaperRequest {
    pub id: String,
    pub title: String,
    pub abstract_text: Option<String>,
    pub authors: Option<Vec<String>>,
    pub published: Option<String>,
    pub source_url: Option<String>,
    pub pdf_url: Option<String>,
}

pub async fn ingest_paper(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestPaperRequest>,
) -> Result<impl IntoResponse, AppError> {
    let abstract_text = req.abstract_text.clone().unwrap_or_default();
    let title = req.title.clone();

    // Generate tags
    let tags_result = extract_tags(
        &state.http_client,
        &state.llm_cfg,
        &title,
        &abstract_text,
    )
    .await;

    let tags: Vec<PaperTag> = tags_result
        .map(|t| t.to_paper_tags())
        .unwrap_or_default();

    // Generate embedding
    let embed_input = format!("{}\n\n{}", title, abstract_text);
    let embedding = embed_text(&state.http_client, &state.llm_cfg, &embed_input)
        .await
        .ok();

    let now = Utc::now();
    let paper = Paper {
        id: req.id.clone(),
        title,
        abstract_text: req.abstract_text,
        authors: req.authors.unwrap_or_default(),
        published: req
            .published
            .as_deref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        source_url: req.source_url,
        pdf_url: req.pdf_url,
        embedding: embedding.clone(),
        created_at: now,
        updated_at: now,
        tags,
    };

    // Update vector index
    if let Some(ref emb) = embedding {
        state.vector_index.upsert(&req.id, emb)?;
    }

    let db = state.db.lock().await;
    upsert_paper(&db, &paper)?;

    Ok((StatusCode::CREATED, Json(serde_json::json!({"id": req.id, "status": "ingested"}))))
}

/// Trigger an arXiv batch fetch.
#[derive(Deserialize)]
pub struct FetchArxivRequest {
    /// Search terms passed to the arXiv API.
    pub query: String,
    /// Maximum number of papers to fetch.
    #[serde(default = "default_arxiv_max")]
    pub max_results: usize,
}

fn default_arxiv_max() -> usize { 25 }

pub async fn fetch_arxiv(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FetchArxivRequest>,
) -> Result<impl IntoResponse, AppError> {
    let arxiv_query = ArxivQuery::new(req.query).max_results(req.max_results);
    // ingest_arxiv acquires its own db lock internally, but it's an async fn
    // that holds the lock across awaits — use the same pattern: pass a Connection
    // reference by acquiring lock then releasing after each sync operation.
    // Since ingest_arxiv takes &Connection (sync) we need to restructure slightly.
    // Simple approach: acquire the lock only around the DB writes inside a block,
    // do network calls outside. We replicate the pipeline inline here.
    use crate::ingestion::{
        arxiv_fetcher::fetch_papers,
        classifier::{embed_text as do_embed, extract_tags as do_tags},
    };
    use crate::store::paper_repo::upsert_paper as do_upsert;

    let mut papers = fetch_papers(&state.http_client, &arxiv_query).await?;
    let total_fetched = papers.len();
    let mut stored = 0usize;
    let mut errors: Vec<String> = Vec::new();

    for paper in papers.iter_mut() {
        let id = paper.id.clone();
        let title = paper.title.clone();
        let abstract_text = paper.abstract_text.clone().unwrap_or_default();

        // Network: tag extraction
        match do_tags(&state.http_client, &state.llm_cfg, &title, &abstract_text).await {
            Ok(t) => paper.tags = t.to_paper_tags(),
            Err(e) => tracing::warn!("tag extraction failed for {}: {}", id, e),
        }

        // Network: embedding
        let embed_input = format!("{}\n\n{}", title, abstract_text);
        match do_embed(&state.http_client, &state.llm_cfg, &embed_input).await {
            Ok(emb) => {
                let _ = state.vector_index.upsert(&id, &emb);
                paper.embedding = Some(emb);
            }
            Err(e) => tracing::warn!("embedding failed for {}: {}", id, e),
        }

        // Sync DB write — lock scope
        let db = state.db.lock().await;
        match do_upsert(&db, paper) {
            Ok(()) => stored += 1,
            Err(e) => errors.push(format!("{}: {}", id, e)),
        }
        // lock released here
    }

    Ok(Json(serde_json::json!({
        "total_fetched": total_fetched,
        "stored": stored,
        "errors": errors,
    })))
}

// ── Error type ────────────────────────────────────────────────────────────────

/// Wrapper that converts `anyhow::Error` into a 500 JSON response.
pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("handler error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": self.0.to_string()})),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(e: E) -> Self {
        AppError(e.into())
    }
}
