/// axum route definitions.
///
/// Public (query) routes:
///   GET  /health
///   GET  /equations[?pde_type=]
///   GET  /equations/:id
///   GET  /equations/:id/solvers
///   GET  /equations/:id/conditions
///   GET  /equations/:id/datasets
///   GET  /equations/:id/papers
///   GET  /ai-models[?training_type=]
///   GET  /ai-models/:id
///   GET  /ai-models/:id/profile
///   GET  /ai-models/:id/equations
///   GET  /ai-models/:id/papers
///   GET  /numerical-methods
///   GET  /numerical-methods/:id
///   GET  /numerical-methods/:id/papers
///   GET  /papers[?year=]
///   GET  /papers/:id
///   GET  /papers/:id/profile
///   GET  /search?q=
///
/// Internal (write) routes:
///   POST   /internal/nodes
///   DELETE /internal/nodes/:label/:id
///   POST   /internal/relations
///   DELETE /internal/relations
///   POST   /internal/content

use std::sync::Arc;

use axum::{
    routing::{delete, get, post},
    Router,
};

use super::{
    AppState,
    handlers::{
        query::{
            ai_model_equations_handler, ai_model_papers_handler, ai_model_profile_handler,
            benchmark_leaderboard_handler, equation_conditions_handler,
            equation_datasets_handler, equation_papers_handler, equation_solvers_handler,
            get_ai_model_handler, get_benchmark_handler, get_equation_handler,
            get_numerical_method_handler, get_paper_handler, health,
            list_ai_models_handler, list_benchmarks_handler, list_equations_handler,
            list_numerical_methods_handler, list_papers_handler,
            numerical_method_papers_handler, paper_profile_handler,
            results_for_ai_model_handler, results_for_numerical_method_handler,
            search_handler,
        },
        write::{
            delete_node_handler, delete_relation_handler, submit_result_handler,
            upsert_content_handler, upsert_node_handler, upsert_relation_handler,
        },
    },
};

pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        // ── Health ─────────────────────────────────────────────────────────
        .route("/health",                              get(health))
        // ── Equations ──────────────────────────────────────────────────────
        .route("/equations",                           get(list_equations_handler))
        .route("/equations/:id",                      get(get_equation_handler))
        .route("/equations/:id/solvers",              get(equation_solvers_handler))
        .route("/equations/:id/conditions",           get(equation_conditions_handler))
        .route("/equations/:id/datasets",             get(equation_datasets_handler))
        .route("/equations/:id/papers",               get(equation_papers_handler))
        // ── AI Models ──────────────────────────────────────────────────────
        .route("/ai-models",                          get(list_ai_models_handler))
        .route("/ai-models/:id",                     get(get_ai_model_handler))
        .route("/ai-models/:id/profile",             get(ai_model_profile_handler))
        .route("/ai-models/:id/equations",           get(ai_model_equations_handler))
        .route("/ai-models/:id/papers",              get(ai_model_papers_handler))
        // ── Numerical Methods ──────────────────────────────────────────────
        .route("/numerical-methods",                  get(list_numerical_methods_handler))
        .route("/numerical-methods/:id",             get(get_numerical_method_handler))
        .route("/numerical-methods/:id/papers",      get(numerical_method_papers_handler))
        // ── Papers ─────────────────────────────────────────────────────────
        .route("/papers",                             get(list_papers_handler))
        .route("/papers/:id",                        get(get_paper_handler))
        .route("/papers/:id/profile",                get(paper_profile_handler))
        // ── Benchmarks ─────────────────────────────────────────────────────
        .route("/benchmarks",                         get(list_benchmarks_handler))
        .route("/benchmarks/:id",                    get(get_benchmark_handler))
        .route("/benchmarks/:id/leaderboard",        get(benchmark_leaderboard_handler))
        .route("/ai-models/:id/results",             get(results_for_ai_model_handler))
        .route("/numerical-methods/:id/results",     get(results_for_numerical_method_handler))
        // ── Search ─────────────────────────────────────────────────────────
        .route("/search",                             get(search_handler))
        // ── Internal write API ─────────────────────────────────────────────
        .route("/internal/nodes",                     post(upsert_node_handler))
        .route("/internal/nodes/:label/:id",         delete(delete_node_handler))
        .route("/internal/relations",                 post(upsert_relation_handler))
        .route("/internal/relations",                 delete(delete_relation_handler))
        .route("/internal/content",                   post(upsert_content_handler))
        .route("/internal/results",                   post(submit_result_handler))
        .with_state(state)
}
