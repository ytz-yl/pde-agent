/// High-level retrieval queries for the PDE knowledge graph.

use anyhow::{Context, Result};
use neo4rs::Graph;
use serde::Serialize;

use crate::store::schema::{
    AIModel, BenchResult, Benchmark, Condition, NumericalMethod, Paper,
    LABEL_AI_MODEL, LABEL_BENCHMARK, LABEL_BENCH_RESULT, LABEL_DATASET, LABEL_EQUATION,
    LABEL_LOSS_FUNCTION, LABEL_METRIC, LABEL_NUMERICAL_METHOD, LABEL_PAPER,
    REL_EVALUATED_BY, REL_OF_METHOD, REL_ON_BENCHMARK, REL_ON_DATASET, REL_SOLVES,
    REL_TESTED_ON, REL_TRAINED_BY, REL_USES_METRIC,
    REL_PROPOSES, REL_STUDIES, REL_USES_DATASET, REL_CITES,
};

// ── Result types ──────────────────────────────────────────────────────────────

/// One group of solvers (either executable-locally or literature-only).
#[derive(Debug, Serialize, Default)]
pub struct SolverGroup {
    pub ai_models: Vec<AIModel>,
    pub numerical_methods: Vec<NumericalMethod>,
}

/// All solvers (AI models + numerical methods) that can handle an equation,
/// split by whether they are callable through the engines API.
///
/// The split is based on the `engine_id` field on each method node:
///   - `engine_id` set and non-empty  → `executable`
///   - otherwise (null / empty)        → `literature_only`
#[derive(Debug, Serialize)]
pub struct EquationSolvers {
    pub equation_id: String,
    pub equation_name: String,
    pub executable: SolverGroup,
    pub literature_only: SolverGroup,
}

/// Full profile of an AI model: what it solves, how it's trained, and metrics.
#[derive(Debug, Serialize)]
pub struct AIModelProfile {
    pub model: AIModel,
    /// Equations this model claims to solve.
    pub solves: Vec<EquationRef>,
    /// Loss functions used in training.
    pub trained_by: Vec<LossFunctionRef>,
    /// Evaluation metrics.
    pub evaluated_by: Vec<MetricRef>,
    /// Benchmark datasets.
    pub tested_on: Vec<DatasetRef>,
}

/// Conditions associated with an equation.
#[derive(Debug, Serialize)]
pub struct EquationConditions {
    pub equation_id: String,
    pub conditions: Vec<Condition>,
}

/// Lightweight reference to a named node.
#[derive(Debug, Clone, Serialize)]
pub struct EquationRef {
    pub id: String,
    pub name: String,
    pub pde_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LossFunctionRef {
    pub id: String,
    pub name: String,
    pub loss_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetricRef {
    pub id: String,
    pub name: String,
    pub metric_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DatasetRef {
    pub id: String,
    pub name: String,
    pub dimension: Option<String>,
}

// ── Queries ───────────────────────────────────────────────────────────────────

/// Return all AI models and numerical methods that SOLVES a given equation,
/// partitioned into `executable` (engine_id non-empty) and `literature_only`.
pub async fn solvers_for_equation(graph: &Graph, equation_id: &str) -> Result<EquationSolvers> {
    // First get the equation name
    let mut eq_result = graph
        .execute(neo4rs::query(
            "MATCH (e:Equation {id: $id}) RETURN e.id AS id, e.name AS name"
        )
        .param("id", equation_id))
        .await
        .context("solvers_for_equation: get equation")?;

    let (eq_id, eq_name) = if let Some(row) = eq_result.next().await? {
        let id: String = row.get("id").unwrap_or_default();
        let name: String = row.get("name").unwrap_or_default();
        (id, name)
    } else {
        return Ok(EquationSolvers {
            equation_id: equation_id.to_string(),
            equation_name: String::new(),
            executable: SolverGroup::default(),
            literature_only: SolverGroup::default(),
        });
    };

    // AI models that SOLVES the equation
    let mut ai_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl})-[:{rel}]->(e:{el} {{id: $id}}) RETURN m",
            fl = LABEL_AI_MODEL, rel = REL_SOLVES, el = LABEL_EQUATION
        ))
        .param("id", equation_id))
        .await
        .context("solvers_for_equation: ai models")?;

    let mut executable = SolverGroup::default();
    let mut literature_only = SolverGroup::default();

    while let Some(row) = ai_result.next().await.context("ai model row")? {
        if let Ok(m) = row_to_ai_model_from_row(&row) {
            if m.engine_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
                executable.ai_models.push(m);
            } else {
                literature_only.ai_models.push(m);
            }
        }
    }

    // Numerical methods that SOLVES the equation
    let mut nm_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl})-[:{rel}]->(e:{el} {{id: $id}}) RETURN m",
            fl = LABEL_NUMERICAL_METHOD, rel = REL_SOLVES, el = LABEL_EQUATION
        ))
        .param("id", equation_id))
        .await
        .context("solvers_for_equation: numerical methods")?;

    while let Some(row) = nm_result.next().await.context("nm row")? {
        if let Ok(m) = row_to_numerical_method_from_row(&row) {
            if m.engine_id.as_deref().map(|s| !s.is_empty()).unwrap_or(false) {
                executable.numerical_methods.push(m);
            } else {
                literature_only.numerical_methods.push(m);
            }
        }
    }

    Ok(EquationSolvers {
        equation_id: eq_id,
        equation_name: eq_name,
        executable,
        literature_only,
    })
}

/// Return the full profile of an AI model.
pub async fn ai_model_profile(graph: &Graph, model_id: &str) -> Result<Option<AIModelProfile>> {
    // Get the model itself
    let mut model_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n",
            label = LABEL_AI_MODEL
        ))
        .param("id", model_id))
        .await
        .context("ai_model_profile: get model")?;

    let model = match model_result.next().await.context("model row")? {
        Some(row) => row_to_ai_model_from_n(&row)?,
        None => return Ok(None),
    };

    // Equations it solves
    let mut eq_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl} {{id: $id}})-[:{rel}]->(e:{el}) \
             RETURN e.id AS id, e.name AS name, e.pde_type AS pde_type",
            fl = LABEL_AI_MODEL, rel = REL_SOLVES, el = LABEL_EQUATION
        ))
        .param("id", model_id))
        .await
        .context("ai_model_profile: solves")?;

    let mut solves = Vec::new();
    while let Some(row) = eq_result.next().await? {
        solves.push(EquationRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            pde_type: row.get("pde_type").unwrap_or_default(),
        });
    }

    // Loss functions
    let mut loss_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl} {{id: $id}})-[:{rel}]->(l:{ll}) \
             RETURN l.id AS id, l.name AS name, l.loss_type AS loss_type",
            fl = LABEL_AI_MODEL, rel = REL_TRAINED_BY, ll = LABEL_LOSS_FUNCTION
        ))
        .param("id", model_id))
        .await
        .context("ai_model_profile: trained_by")?;

    let mut trained_by = Vec::new();
    while let Some(row) = loss_result.next().await? {
        trained_by.push(LossFunctionRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            loss_type: row.get("loss_type").unwrap_or_default(),
        });
    }

    // Evaluation metrics
    let mut metric_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl} {{id: $id}})-[:{rel}]->(k:{ml}) \
             RETURN k.id AS id, k.name AS name, k.metric_type AS metric_type",
            fl = LABEL_AI_MODEL, rel = REL_EVALUATED_BY, ml = LABEL_METRIC
        ))
        .param("id", model_id))
        .await
        .context("ai_model_profile: evaluated_by")?;

    let mut evaluated_by = Vec::new();
    while let Some(row) = metric_result.next().await? {
        evaluated_by.push(MetricRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            metric_type: row.get("metric_type").unwrap_or_default(),
        });
    }

    // Datasets
    let mut ds_result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{fl} {{id: $id}})-[:{rel}]->(d:{dl}) \
             RETURN d.id AS id, d.name AS name, d.dimension AS dimension",
            fl = LABEL_AI_MODEL, rel = REL_TESTED_ON, dl = LABEL_DATASET
        ))
        .param("id", model_id))
        .await
        .context("ai_model_profile: tested_on")?;

    let mut tested_on = Vec::new();
    while let Some(row) = ds_result.next().await? {
        tested_on.push(DatasetRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            dimension: row.get("dimension").ok().filter(|s: &String| !s.is_empty()),
        });
    }

    Ok(Some(AIModelProfile {
        model,
        solves,
        trained_by,
        evaluated_by,
        tested_on,
    }))
}

/// Return conditions associated with an equation.
pub async fn conditions_for_equation(
    graph: &Graph,
    equation_id: &str,
) -> Result<EquationConditions> {
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (e:Equation {id: $id})-[:HAS_CONDITION]->(c:Condition) \
             RETURN c.id AS id, c.name AS name, c.condition_type AS ctype, \
                    c.form AS form, c.description AS desc"
        )
        .param("id", equation_id))
        .await
        .context("conditions_for_equation")?;

    let mut conditions = Vec::new();
    while let Some(row) = result.next().await? {
        conditions.push(Condition {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            condition_type: row.get::<String>("ctype")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(crate::store::schema::ConditionType::Other),
            form: row.get("form").ok().filter(|s: &String| !s.is_empty()),
            description: row.get("desc").ok().filter(|s: &String| !s.is_empty()),
        });
    }

    Ok(EquationConditions {
        equation_id: equation_id.to_string(),
        conditions,
    })
}

/// Search for nodes by name substring across all node types.
/// Returns a list of (label, id, name) triples.
#[derive(Debug, Serialize)]
pub struct SearchHit {
    pub label: String,
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

pub async fn search_by_name(graph: &Graph, query: &str) -> Result<Vec<SearchHit>> {
    let pattern = format!("(?i).*{}.*", regex_escape(query));

    let cypher = "CALL { \
        MATCH (n) WHERE n.name =~ $pattern \
        RETURN labels(n)[0] AS label, n.id AS id, n.name AS name, n.description AS desc \
    } RETURN label, id, name, desc ORDER BY name LIMIT 50";

    let mut result = graph
        .execute(neo4rs::query(cypher).param("pattern", pattern.as_str()))
        .await
        .context("search_by_name")?;

    let mut hits = Vec::new();
    while let Some(row) = result.next().await? {
        hits.push(SearchHit {
            label: row.get("label").unwrap_or_default(),
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            description: row.get("desc").ok().filter(|s: &String| !s.is_empty()),
        });
    }
    Ok(hits)
}

/// List all equations that a given AI model or NumericalMethod solves.
pub async fn equations_solved_by(graph: &Graph, solver_label: &str, solver_id: &str) -> Result<Vec<EquationRef>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (m:{sl} {{id: $id}})-[:SOLVES]->(e:Equation) \
             RETURN e.id AS id, e.name AS name, e.pde_type AS pde_type",
            sl = solver_label
        ))
        .param("id", solver_id))
        .await
        .context("equations_solved_by")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await? {
        out.push(EquationRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            pde_type: row.get("pde_type").unwrap_or_default(),
        });
    }
    Ok(out)
}

/// Return a list of datasets that benchmark a given equation.
pub async fn datasets_for_equation(graph: &Graph, equation_id: &str) -> Result<Vec<DatasetRef>> {
    let mut result = graph
        .execute(neo4rs::query(
            "MATCH (d:Dataset)-[:BASED_ON]->(e:Equation {id: $id}) \
             RETURN d.id AS id, d.name AS name, d.dimension AS dimension"
        )
        .param("id", equation_id))
        .await
        .context("datasets_for_equation")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await? {
        out.push(DatasetRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            dimension: row.get("dimension").ok().filter(|s: &String| !s.is_empty()),
        });
    }
    Ok(out)
}

// ── Paper queries ─────────────────────────────────────────────────────────────

/// Light reference to a paper (used in lists).
#[derive(Debug, Clone, Serialize)]
pub struct PaperRef {
    pub id: String,
    pub title: String,
    pub published_year: Option<u32>,
    pub arxiv_id: Option<String>,
}

/// Papers that PROPOSE a given AIModel or NumericalMethod.
pub async fn papers_proposing(graph: &Graph, target_label: &str, target_id: &str) -> Result<Vec<PaperRef>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl})-[:{rel}]->(t:{tl} {{id: $id}}) \
             RETURN p.id AS id, p.title AS title, \
                    p.published_year AS year, p.arxiv_id AS arxiv_id",
            pl = LABEL_PAPER, rel = REL_PROPOSES, tl = target_label
        ))
        .param("id", target_id))
        .await
        .context("papers_proposing")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await? {
        out.push(paper_ref_from_row(&row));
    }
    Ok(out)
}

/// Papers that STUDY a given Equation.
pub async fn papers_studying(graph: &Graph, equation_id: &str) -> Result<Vec<PaperRef>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl})-[:{rel}]->(e:Equation {{id: $id}}) \
             RETURN p.id AS id, p.title AS title, \
                    p.published_year AS year, p.arxiv_id AS arxiv_id",
            pl = LABEL_PAPER, rel = REL_STUDIES
        ))
        .param("id", equation_id))
        .await
        .context("papers_studying")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await? {
        out.push(paper_ref_from_row(&row));
    }
    Ok(out)
}

/// All relations of a paper: what it proposes, studies, uses, and cites.
#[derive(Debug, Serialize)]
pub struct PaperProfile {
    pub paper: Paper,
    pub proposes: Vec<serde_json::Value>,   // [{label, id, name}]
    pub studies: Vec<EquationRef>,
    pub uses_datasets: Vec<DatasetRef>,
    pub cites: Vec<PaperRef>,
    pub cited_by: Vec<PaperRef>,
}

pub async fn paper_profile(graph: &Graph, paper_id: &str) -> Result<Option<PaperProfile>> {
    // fetch the paper node itself
    let mut r = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n", label = LABEL_PAPER
        ))
        .param("id", paper_id))
        .await?;

    let paper = match r.next().await? {
        Some(row) => row_to_paper_from_row(&row)?,
        None => return Ok(None),
    };

    // what it proposes (any label)
    let mut r2 = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl} {{id: $id}})-[:{rel}]->(t) \
             RETURN labels(t)[0] AS label, t.id AS id, t.name AS name",
            pl = LABEL_PAPER, rel = REL_PROPOSES
        ))
        .param("id", paper_id))
        .await?;
    let mut proposes = Vec::new();
    while let Some(row) = r2.next().await? {
        let label: String = row.get("label").unwrap_or_default();
        let id: String = row.get("id").unwrap_or_default();
        let name: String = row.get("name").unwrap_or_default();
        proposes.push(serde_json::json!({"label": label, "id": id, "name": name}));
    }

    // equations it studies
    let mut r3 = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl} {{id: $id}})-[:{rel}]->(e:Equation) \
             RETURN e.id AS id, e.name AS name, e.pde_type AS pde_type",
            pl = LABEL_PAPER, rel = REL_STUDIES
        ))
        .param("id", paper_id))
        .await?;
    let mut studies = Vec::new();
    while let Some(row) = r3.next().await? {
        studies.push(EquationRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            pde_type: row.get("pde_type").unwrap_or_default(),
        });
    }

    // datasets used
    let mut r4 = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl} {{id: $id}})-[:{rel}]->(d:Dataset) \
             RETURN d.id AS id, d.name AS name, d.dimension AS dimension",
            pl = LABEL_PAPER, rel = REL_USES_DATASET
        ))
        .param("id", paper_id))
        .await?;
    let mut uses_datasets = Vec::new();
    while let Some(row) = r4.next().await? {
        uses_datasets.push(DatasetRef {
            id: row.get("id").unwrap_or_default(),
            name: row.get("name").unwrap_or_default(),
            dimension: row.get("dimension").ok().filter(|s: &String| !s.is_empty()),
        });
    }

    // papers it cites
    let mut r5 = graph
        .execute(neo4rs::query(&format!(
            "MATCH (p:{pl} {{id: $id}})-[:{rel}]->(c:{pl}) \
             RETURN c.id AS id, c.title AS title, \
                    c.published_year AS year, c.arxiv_id AS arxiv_id",
            pl = LABEL_PAPER, rel = REL_CITES
        ))
        .param("id", paper_id))
        .await?;
    let mut cites = Vec::new();
    while let Some(row) = r5.next().await? { cites.push(paper_ref_from_row(&row)); }

    // papers that cite this one
    let mut r6 = graph
        .execute(neo4rs::query(&format!(
            "MATCH (c:{pl})-[:{rel}]->(p:{pl} {{id: $id}}) \
             RETURN c.id AS id, c.title AS title, \
                    c.published_year AS year, c.arxiv_id AS arxiv_id",
            pl = LABEL_PAPER, rel = REL_CITES
        ))
        .param("id", paper_id))
        .await?;
    let mut cited_by = Vec::new();
    while let Some(row) = r6.next().await? { cited_by.push(paper_ref_from_row(&row)); }

    Ok(Some(PaperProfile { paper, proposes, studies, uses_datasets, cites, cited_by }))
}

fn paper_ref_from_row(row: &neo4rs::Row) -> PaperRef {
    let year = row.get::<i64>("year").unwrap_or(0);
    PaperRef {
        id: row.get("id").unwrap_or_default(),
        title: row.get("title").unwrap_or_default(),
        published_year: if year > 0 { Some(year as u32) } else { None },
        arxiv_id: row.get("arxiv_id").ok().filter(|s: &String| !s.is_empty()),
    }
}

fn row_to_paper_from_row(row: &neo4rs::Row) -> Result<Paper> {
    let n: neo4rs::Node = row.get("n").context("paper node 'n'")?;
    let year = n.get::<i64>("published_year").unwrap_or(0);
    Ok(Paper {
        id: n.get("id").unwrap_or_default(),
        title: n.get("title").unwrap_or_default(),
        authors: json_vec_node(&n, "authors"),
        published_year: if year > 0 { Some(year as u32) } else { None },
        arxiv_id: opt_str_node(&n, "arxiv_id"),
        doi: opt_str_node(&n, "doi"),
        pdf_path: opt_str_node(&n, "pdf_path"),
        tags: json_vec_node(&n, "tags"),
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Deserialise an AIModel from a row where the node is bound to alias `alias`.
fn ai_model_from_alias(row: &neo4rs::Row, alias: &str) -> Result<AIModel> {
    use crate::store::schema::TrainingType;
    use std::str::FromStr;

    let n: neo4rs::Node = row
        .get(alias)
        .with_context(|| format!("ai model node '{}'", alias))?;
    Ok(AIModel {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        architecture: n.get("architecture").unwrap_or_default(),
        input_vars: json_vec_node(&n, "input_vars"),
        output_vars: json_vec_node(&n, "output_vars"),
        training_type: TrainingType::from_str(
            &n.get::<String>("training_type").unwrap_or_default(),
        )?,
        description: opt_str_node(&n, "description"),
        paper_ref: opt_str_node(&n, "paper_ref"),
        tags: json_vec_node(&n, "tags"),
        engine_id: opt_str_node(&n, "engine_id"),
    })
}

/// RETURN m  variant (used in MATCH (m:AIModel)...)
fn row_to_ai_model_from_row(row: &neo4rs::Row) -> Result<AIModel> {
    ai_model_from_alias(row, "m")
}

/// RETURN n  variant (used in MATCH (n:AIModel)...)
fn row_to_ai_model_from_n(row: &neo4rs::Row) -> Result<AIModel> {
    ai_model_from_alias(row, "n")
}

fn row_to_numerical_method_from_row(row: &neo4rs::Row) -> Result<NumericalMethod> {
    use crate::store::schema::NumericalMethodType;
    use std::str::FromStr;

    let n: neo4rs::Node = row.get("m").context("numerical method node 'm'")?;
    let order = n.get::<i64>("order").unwrap_or(0);
    Ok(NumericalMethod {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        method_type: NumericalMethodType::from_str(
            &n.get::<String>("method_type").unwrap_or_default(),
        )?,
        order: if order > 0 { Some(order as u32) } else { None },
        description: opt_str_node(&n, "description"),
        tags: json_vec_node(&n, "tags"),
        engine_id: opt_str_node(&n, "engine_id"),
    })
}

fn opt_str_node(node: &neo4rs::Node, field: &str) -> Option<String> {
    node.get::<String>(field).ok().filter(|s| !s.is_empty())
}

fn json_vec_node(node: &neo4rs::Node, field: &str) -> Vec<String> {
    node.get::<String>(field)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

/// Minimal regex escaping for name search patterns.
fn regex_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if "\\^$.|?*+()[]{}".contains(c) {
                vec!['\\', c]
            } else {
                vec![c]
            }
        })
        .collect()
}

// ── Benchmark leaderboard ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ResultConfidence {
    Verified,
    Single,
    Disputed,
}

#[derive(Debug, Serialize)]
pub struct LeaderboardEntry {
    pub method_id: String,
    pub method_label: String,
    pub method_name: Option<String>,
    pub best_value: f64,
    pub all_values: Vec<f64>,
    pub n_independent_sources: usize,
    pub n_results: usize,
    pub confidence: ResultConfidence,
    pub latest_recorded_at: Option<String>,
    pub source_breakdown: std::collections::BTreeMap<String, usize>,
}

#[derive(Debug, Serialize)]
pub struct BenchmarkLeaderboard {
    pub benchmark: Benchmark,
    pub dataset_name: Option<String>,
    pub metric_name: Option<String>,
    pub entries: Vec<LeaderboardEntry>,
}

/// Build the leaderboard for a benchmark. Returns None if the benchmark doesn't exist.
///
/// Confidence rules per (method, benchmark):
///   - n_independent_sources == 1                            → Single
///   - n_independent_sources ≥ 2 and spread ≤ tolerance      → Verified
///   - n_independent_sources ≥ 2 and spread > tolerance      → Disputed
/// Where:
///   - independent source = distinct (source_type, source_paper_id) pair
///   - spread = (max - min) / |mean|, falling back to (max - min) when mean is 0
pub async fn benchmark_leaderboard(
    graph: &Graph,
    benchmark_id: &str,
) -> Result<Option<BenchmarkLeaderboard>> {
    // 1. Fetch benchmark + dataset/metric names.
    let mut bench_q = graph
        .execute(neo4rs::query(&format!(
            "MATCH (b:{bl} {{id: $id}}) \
             OPTIONAL MATCH (b)-[:{rd}]->(d:{dl}) \
             OPTIONAL MATCH (b)-[:{rm}]->(m:{ml}) \
             RETURN b, d.name AS ds_name, m.name AS metric_name",
            bl = LABEL_BENCHMARK, rd = REL_ON_DATASET, dl = LABEL_DATASET,
            rm = REL_USES_METRIC, ml = LABEL_METRIC
        ))
        .param("id", benchmark_id))
        .await
        .context("benchmark_leaderboard: get benchmark")?;

    let (benchmark, dataset_name, metric_name) = match bench_q.next().await? {
        Some(row) => {
            let b: neo4rs::Node = row.get("b").context("benchmark node 'b'")?;
            let benchmark = Benchmark {
                id: b.get("id").unwrap_or_default(),
                name: b.get("name").unwrap_or_default(),
                dataset_id: b.get("dataset_id").unwrap_or_default(),
                metric_id: b.get("metric_id").unwrap_or_default(),
                lower_is_better: b.get::<bool>("lower_is_better").unwrap_or(true),
                protocol: opt_str_node(&b, "protocol"),
                tolerance: b.get::<f64>("tolerance").ok(),
            };
            let ds_name: Option<String> = row.get("ds_name").ok().filter(|s: &String| !s.is_empty());
            let metric_name: Option<String> = row.get("metric_name").ok().filter(|s: &String| !s.is_empty());
            (benchmark, ds_name, metric_name)
        }
        None => return Ok(None),
    };

    // 2. Pull all results for this benchmark joined with the method node.
    let mut rows = graph
        .execute(neo4rs::query(&format!(
            "MATCH (r:{rl})-[:{ron}]->(b:{bl} {{id: $id}}) \
             MATCH (r)-[:{rof}]->(m) \
             RETURN m.id AS mid, labels(m)[0] AS mlabel, m.name AS mname, \
                    r.value AS value, r.source_type AS src, \
                    r.source_paper_id AS pid, r.recorded_at AS ts",
            rl = LABEL_BENCH_RESULT, ron = REL_ON_BENCHMARK, bl = LABEL_BENCHMARK,
            rof = REL_OF_METHOD
        ))
        .param("id", benchmark_id))
        .await
        .context("benchmark_leaderboard: get results")?;

    struct Row {
        mid: String,
        mlabel: String,
        mname: Option<String>,
        value: f64,
        src: String,
        pid: String,
        ts: Option<String>,
    }
    let mut all_rows: Vec<Row> = Vec::new();
    while let Some(row) = rows.next().await? {
        all_rows.push(Row {
            mid: row.get("mid").unwrap_or_default(),
            mlabel: row.get("mlabel").unwrap_or_default(),
            mname: row.get("mname").ok().filter(|s: &String| !s.is_empty()),
            value: row.get::<f64>("value").unwrap_or(0.0),
            src: row.get("src").unwrap_or_default(),
            pid: row.get("pid").unwrap_or_default(),
            ts: row.get("ts").ok().filter(|s: &String| !s.is_empty()),
        });
    }

    // 3. Group by (mid, mlabel).
    use std::collections::BTreeMap;
    let mut groups: BTreeMap<(String, String), Vec<Row>> = BTreeMap::new();
    for r in all_rows {
        groups.entry((r.mid.clone(), r.mlabel.clone())).or_default().push(r);
    }

    // 4. Build entries.
    let tolerance = benchmark.tolerance.unwrap_or(0.05);
    let mut entries: Vec<LeaderboardEntry> = Vec::with_capacity(groups.len());

    for ((mid, mlabel), rows) in groups {
        let method_name = rows.iter().find_map(|r| r.mname.clone());
        let n_results = rows.len();

        // Independent sources: distinct (src, pid) tuples. self_run with empty pid all collapse.
        let mut independent: std::collections::BTreeSet<(String, String)> =
            std::collections::BTreeSet::new();
        let mut breakdown: BTreeMap<String, usize> = BTreeMap::new();
        for r in &rows {
            independent.insert((r.src.clone(), r.pid.clone()));
            *breakdown.entry(r.src.clone()).or_insert(0) += 1;
        }
        let n_independent = independent.len();

        let values: Vec<f64> = rows.iter().map(|r| r.value).collect();
        let min = values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let abs_mean = mean.abs();
        let spread = if abs_mean > f64::EPSILON {
            (max - min) / abs_mean
        } else {
            max - min
        };

        let best_value = if benchmark.lower_is_better { min } else { max };

        let confidence = if n_independent >= 2 {
            if spread <= tolerance {
                ResultConfidence::Verified
            } else {
                ResultConfidence::Disputed
            }
        } else {
            ResultConfidence::Single
        };

        let latest_recorded_at = rows
            .iter()
            .filter_map(|r| r.ts.clone())
            .max(); // ISO-8601 strings sort lexically

        entries.push(LeaderboardEntry {
            method_id: mid,
            method_label: mlabel,
            method_name,
            best_value,
            all_values: values,
            n_independent_sources: n_independent,
            n_results,
            confidence,
            latest_recorded_at,
            source_breakdown: breakdown,
        });
    }

    // 5. Sort. Primary: best_value (asc if lower_is_better, desc otherwise).
    //    Tie-break: Verified > Single > Disputed.
    fn confidence_rank(c: &ResultConfidence) -> u8 {
        match c {
            ResultConfidence::Verified => 0,
            ResultConfidence::Single => 1,
            ResultConfidence::Disputed => 2,
        }
    }
    entries.sort_by(|a, b| {
        let primary = if benchmark.lower_is_better {
            a.best_value.partial_cmp(&b.best_value).unwrap_or(std::cmp::Ordering::Equal)
        } else {
            b.best_value.partial_cmp(&a.best_value).unwrap_or(std::cmp::Ordering::Equal)
        };
        primary.then_with(|| confidence_rank(&a.confidence).cmp(&confidence_rank(&b.confidence)))
    });

    Ok(Some(BenchmarkLeaderboard {
        benchmark,
        dataset_name,
        metric_name,
        entries,
    }))
}

/// All BenchResults attributable to a given AIModel or NumericalMethod, newest first.
pub async fn results_for_method(
    graph: &Graph,
    method_label: &str,
    method_id: &str,
) -> Result<Vec<BenchResult>> {
    crate::store::node_repo::list_results_for_method(graph, method_label, method_id).await
}
