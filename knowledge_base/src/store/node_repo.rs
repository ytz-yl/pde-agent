/// Node CRUD operations against Neo4j.
///
/// Each function accepts a `&neo4rs::Graph` and returns `anyhow::Result<_>`.
/// All mutations use MERGE to be idempotent.

use anyhow::{Context, Result};
use neo4rs::{Graph, Row};

use crate::store::schema::{
    AIModel, BenchResult, Benchmark, Condition, Dataset, Equation, LossFunction, Metric,
    NumericalMethod, NumericalMethodType, Paper, PdeType, SourceType, Theorem, TrainingType,
    KnowledgeNode,
    LABEL_AI_MODEL, LABEL_BENCH_RESULT, LABEL_BENCHMARK, LABEL_CONDITION, LABEL_DATASET,
    LABEL_EQUATION, LABEL_LOSS_FUNCTION, LABEL_METRIC, LABEL_NUMERICAL_METHOD,
    LABEL_PAPER, LABEL_THEOREM,
    REL_OF_METHOD, REL_ON_BENCHMARK, REL_ON_DATASET, REL_REPORTED_IN, REL_USES_METRIC,
};

use std::str::FromStr;

// ── Upsert helpers ────────────────────────────────────────────────────────────

/// Upsert an Equation node.
pub async fn upsert_equation(graph: &Graph, eq: &Equation) -> Result<()> {
    let vars = serde_json::to_string(&eq.variables)?;
    let tags = serde_json::to_string(&eq.tags)?;
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.pde_type = $pde_type, \
                 n.variables = $vars, \
                 n.time_dependent = $time_dep, \
                 n.operator = $operator, \
                 n.description = $desc, \
                 n.tags = $tags",
            label = LABEL_EQUATION
        ))
        .param("id", eq.id.as_str())
        .param("name", eq.name.as_str())
        .param("pde_type", eq.pde_type.as_str())
        .param("vars", vars.as_str())
        .param("time_dep", eq.time_dependent)
        .param("operator", eq.operator.as_deref().unwrap_or(""))
        .param("desc", eq.description.as_deref().unwrap_or(""))
        .param("tags", tags.as_str()))
        .await
        .context("upsert_equation")
}

/// Upsert a Condition node.
pub async fn upsert_condition(graph: &Graph, c: &Condition) -> Result<()> {
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.condition_type = $ctype, \
                 n.form = $form, \
                 n.description = $desc",
            label = LABEL_CONDITION
        ))
        .param("id", c.id.as_str())
        .param("name", c.name.as_str())
        .param("ctype", c.condition_type.as_str())
        .param("form", c.form.as_deref().unwrap_or(""))
        .param("desc", c.description.as_deref().unwrap_or("")))
        .await
        .context("upsert_condition")
}

/// Upsert a Theorem node.
pub async fn upsert_theorem(graph: &Graph, t: &Theorem) -> Result<()> {
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.result = $result, \
                 n.confidence = $confidence, \
                 n.description = $desc, \
                 n.source = $source",
            label = LABEL_THEOREM
        ))
        .param("id", t.id.as_str())
        .param("name", t.name.as_str())
        .param("result", t.result.as_str())
        .param("confidence", t.confidence as f64)
        .param("desc", t.description.as_deref().unwrap_or(""))
        .param("source", t.source.as_deref().unwrap_or("")))
        .await
        .context("upsert_theorem")
}

/// Upsert a NumericalMethod node.
pub async fn upsert_numerical_method(graph: &Graph, m: &NumericalMethod) -> Result<()> {
    let tags = serde_json::to_string(&m.tags)?;
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.method_type = $mtype, \
                 n.order = $order, \
                 n.description = $desc, \
                 n.tags = $tags, \
                 n.engine_id = $engine_id",
            label = LABEL_NUMERICAL_METHOD
        ))
        .param("id", m.id.as_str())
        .param("name", m.name.as_str())
        .param("mtype", m.method_type.as_str())
        .param("order", m.order.unwrap_or(0) as i64)
        .param("desc", m.description.as_deref().unwrap_or(""))
        .param("tags", tags.as_str())
        .param("engine_id", m.engine_id.as_deref().unwrap_or("")))
        .await
        .context("upsert_numerical_method")
}

/// Upsert an AIModel node.
pub async fn upsert_ai_model(graph: &Graph, m: &AIModel) -> Result<()> {
    let input_vars = serde_json::to_string(&m.input_vars)?;
    let output_vars = serde_json::to_string(&m.output_vars)?;
    let tags = serde_json::to_string(&m.tags)?;
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.architecture = $arch, \
                 n.input_vars = $input_vars, \
                 n.output_vars = $output_vars, \
                 n.training_type = $training, \
                 n.description = $desc, \
                 n.paper_ref = $paper_ref, \
                 n.tags = $tags, \
                 n.engine_id = $engine_id",
            label = LABEL_AI_MODEL
        ))
        .param("id", m.id.as_str())
        .param("name", m.name.as_str())
        .param("arch", m.architecture.as_str())
        .param("input_vars", input_vars.as_str())
        .param("output_vars", output_vars.as_str())
        .param("training", m.training_type.as_str())
        .param("desc", m.description.as_deref().unwrap_or(""))
        .param("paper_ref", m.paper_ref.as_deref().unwrap_or(""))
        .param("tags", tags.as_str())
        .param("engine_id", m.engine_id.as_deref().unwrap_or("")))
        .await
        .context("upsert_ai_model")
}

/// Upsert a LossFunction node.
pub async fn upsert_loss_function(graph: &Graph, l: &LossFunction) -> Result<()> {
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.loss_type = $ltype, \
                 n.formulation = $form, \
                 n.description = $desc",
            label = LABEL_LOSS_FUNCTION
        ))
        .param("id", l.id.as_str())
        .param("name", l.name.as_str())
        .param("ltype", l.loss_type.as_str())
        .param("form", l.formulation.as_deref().unwrap_or(""))
        .param("desc", l.description.as_deref().unwrap_or("")))
        .await
        .context("upsert_loss_function")
}

/// Upsert a Metric node.
pub async fn upsert_metric(graph: &Graph, m: &Metric) -> Result<()> {
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.metric_type = $mtype, \
                 n.unit = $unit, \
                 n.description = $desc",
            label = LABEL_METRIC
        ))
        .param("id", m.id.as_str())
        .param("name", m.name.as_str())
        .param("mtype", m.metric_type.as_str())
        .param("unit", m.unit.as_deref().unwrap_or(""))
        .param("desc", m.description.as_deref().unwrap_or("")))
        .await
        .context("upsert_metric")
}

/// Upsert a Dataset node.
pub async fn upsert_dataset(graph: &Graph, d: &Dataset) -> Result<()> {
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.name = $name, \
                 n.dimension = $dim, \
                 n.num_samples = $samples, \
                 n.description = $desc, \
                 n.url = $url",
            label = LABEL_DATASET
        ))
        .param("id", d.id.as_str())
        .param("name", d.name.as_str())
        .param("dim", d.dimension.as_deref().unwrap_or(""))
        .param("samples", d.num_samples.unwrap_or(0) as i64)
        .param("desc", d.description.as_deref().unwrap_or(""))
        .param("url", d.url.as_deref().unwrap_or("")))
        .await
        .context("upsert_dataset")
}

/// Upsert a Paper node (only structural fields; abstract goes to SQLite content_repo).
pub async fn upsert_paper(graph: &Graph, p: &Paper) -> Result<()> {
    let authors = serde_json::to_string(&p.authors)?;
    let tags = serde_json::to_string(&p.tags)?;
    graph
        .run(neo4rs::query(&format!(
            "MERGE (n:{label} {{id: $id}}) \
             SET n.title        = $title, \
                 n.authors      = $authors, \
                 n.published_year = $year, \
                 n.arxiv_id     = $arxiv_id, \
                 n.doi          = $doi, \
                 n.pdf_path     = $pdf_path, \
                 n.tags         = $tags",
            label = LABEL_PAPER
        ))
        .param("id", p.id.as_str())
        .param("title", p.title.as_str())
        .param("authors", authors.as_str())
        .param("year", p.published_year.unwrap_or(0) as i64)
        .param("arxiv_id", p.arxiv_id.as_deref().unwrap_or(""))
        .param("doi", p.doi.as_deref().unwrap_or(""))
        .param("pdf_path", p.pdf_path.as_deref().unwrap_or(""))
        .param("tags", tags.as_str()))
        .await
        .context("upsert_paper")
}

/// Fetch a Paper by id.
pub async fn get_paper(graph: &Graph, id: &str) -> Result<Option<Paper>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n",
            label = LABEL_PAPER
        ))
        .param("id", id))
        .await
        .context("get_paper")?;

    if let Some(row) = result.next().await.context("get_paper next")? {
        Ok(Some(row_to_paper(&row)?))
    } else {
        Ok(None)
    }
}

/// List Paper nodes, optionally filtered by published_year.
pub async fn list_papers(graph: &Graph, year: Option<u32>) -> Result<Vec<Paper>> {
    let cypher = if year.is_some() {
        format!(
            "MATCH (n:{label}) WHERE n.published_year = $year RETURN n ORDER BY n.title",
            label = LABEL_PAPER
        )
    } else {
        format!("MATCH (n:{label}) RETURN n ORDER BY n.title", label = LABEL_PAPER)
    };

    let mut q = neo4rs::query(&cypher);
    if let Some(y) = year {
        q = q.param("year", y as i64);
    }

    let mut result = graph.execute(q).await.context("list_papers")?;
    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_papers row")? {
        out.push(row_to_paper(&row)?);
    }
    Ok(out)
}

fn row_to_paper(row: &Row) -> Result<Paper> {
    let n: neo4rs::Node = row.get("n").context("paper node")?;
    let year = n.get::<i64>("published_year").unwrap_or(0);
    Ok(Paper {
        id: n.get("id").unwrap_or_default(),
        title: n.get("title").unwrap_or_default(),
        authors: json_vec_field(&n, "authors"),
        published_year: if year > 0 { Some(year as u32) } else { None },
        arxiv_id: opt_str_field(&n, "arxiv_id"),
        doi: opt_str_field(&n, "doi"),
        pdf_path: opt_str_field(&n, "pdf_path"),
        tags: json_vec_field(&n, "tags"),
    })
}

/// Dispatch upsert to the correct typed function based on the KnowledgeNode variant.
///
/// Returns the resolved id of the upserted node. Most node types have a
/// caller-provided id and pass it through; `BenchResult` may have id=None,
/// in which case one is generated server-side and returned here.
pub async fn upsert_node(graph: &Graph, node: &KnowledgeNode) -> Result<String> {
    match node {
        KnowledgeNode::Equation(n)        => { upsert_equation(graph, n).await?;        Ok(n.id.clone()) }
        KnowledgeNode::Condition(n)       => { upsert_condition(graph, n).await?;       Ok(n.id.clone()) }
        KnowledgeNode::Theorem(n)         => { upsert_theorem(graph, n).await?;         Ok(n.id.clone()) }
        KnowledgeNode::NumericalMethod(n) => { upsert_numerical_method(graph, n).await?; Ok(n.id.clone()) }
        KnowledgeNode::AiModel(n)         => { upsert_ai_model(graph, n).await?;         Ok(n.id.clone()) }
        KnowledgeNode::LossFunction(n)    => { upsert_loss_function(graph, n).await?;    Ok(n.id.clone()) }
        KnowledgeNode::Metric(n)          => { upsert_metric(graph, n).await?;           Ok(n.id.clone()) }
        KnowledgeNode::Dataset(n)         => { upsert_dataset(graph, n).await?;          Ok(n.id.clone()) }
        KnowledgeNode::Paper(n)           => { upsert_paper(graph, n).await?;            Ok(n.id.clone()) }
        KnowledgeNode::Benchmark(n)       => { upsert_benchmark(graph, n).await?;        Ok(n.id.clone()) }
        KnowledgeNode::BenchResult(n)     => upsert_bench_result(graph, n).await,
    }
}

// ── Delete ────────────────────────────────────────────────────────────────────

/// Delete a node by its label and id. Returns true if a node was deleted.
pub async fn delete_node(graph: &Graph, label: &str, id: &str) -> Result<bool> {
    // Use DETACH DELETE to also remove all incident relations.
    let cypher = format!(
        "MATCH (n:{label} {{id: $id}}) DETACH DELETE n RETURN count(n) AS deleted"
    );
    let mut result = graph
        .execute(neo4rs::query(&cypher).param("id", id))
        .await
        .context("delete_node execute")?;

    if let Some(row) = result.next().await.context("delete_node next")? {
        let deleted: i64 = row.get("deleted").unwrap_or(0);
        Ok(deleted > 0)
    } else {
        Ok(false)
    }
}

// ── Fetch helpers ─────────────────────────────────────────────────────────────

/// Fetch an Equation by id.
pub async fn get_equation(graph: &Graph, id: &str) -> Result<Option<Equation>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n",
            label = LABEL_EQUATION
        ))
        .param("id", id))
        .await
        .context("get_equation")?;

    if let Some(row) = result.next().await.context("get_equation next")? {
        Ok(Some(row_to_equation(&row)?))
    } else {
        Ok(None)
    }
}

/// List all Equation nodes, optionally filtered by pde_type.
pub async fn list_equations(graph: &Graph, pde_type: Option<&str>) -> Result<Vec<Equation>> {
    let cypher = if pde_type.is_some() {
        format!(
            "MATCH (n:{label}) WHERE n.pde_type = $pde_type RETURN n ORDER BY n.name",
            label = LABEL_EQUATION
        )
    } else {
        format!("MATCH (n:{label}) RETURN n ORDER BY n.name", label = LABEL_EQUATION)
    };

    let mut q = neo4rs::query(&cypher);
    if let Some(pt) = pde_type {
        q = q.param("pde_type", pt);
    }

    let mut result = graph.execute(q).await.context("list_equations")?;
    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_equations row")? {
        out.push(row_to_equation(&row)?);
    }
    Ok(out)
}

/// Fetch an AIModel by id.
pub async fn get_ai_model(graph: &Graph, id: &str) -> Result<Option<AIModel>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n",
            label = LABEL_AI_MODEL
        ))
        .param("id", id))
        .await
        .context("get_ai_model")?;

    if let Some(row) = result.next().await.context("get_ai_model next")? {
        Ok(Some(row_to_ai_model(&row)?))
    } else {
        Ok(None)
    }
}

/// List all AIModel nodes, optionally filtered by training_type.
pub async fn list_ai_models(graph: &Graph, training_type: Option<&str>) -> Result<Vec<AIModel>> {
    let cypher = if training_type.is_some() {
        format!(
            "MATCH (n:{label}) WHERE n.training_type = $training RETURN n ORDER BY n.name",
            label = LABEL_AI_MODEL
        )
    } else {
        format!("MATCH (n:{label}) RETURN n ORDER BY n.name", label = LABEL_AI_MODEL)
    };

    let mut q = neo4rs::query(&cypher);
    if let Some(t) = training_type {
        q = q.param("training", t);
    }

    let mut result = graph.execute(q).await.context("list_ai_models")?;
    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_ai_models row")? {
        out.push(row_to_ai_model(&row)?);
    }
    Ok(out)
}

/// Fetch a NumericalMethod by id.
pub async fn get_numerical_method(graph: &Graph, id: &str) -> Result<Option<NumericalMethod>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n",
            label = LABEL_NUMERICAL_METHOD
        ))
        .param("id", id))
        .await
        .context("get_numerical_method")?;

    if let Some(row) = result.next().await.context("get_numerical_method next")? {
        Ok(Some(row_to_numerical_method(&row)?))
    } else {
        Ok(None)
    }
}

/// List all NumericalMethod nodes.
pub async fn list_numerical_methods(graph: &Graph) -> Result<Vec<NumericalMethod>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label}) RETURN n ORDER BY n.name",
            label = LABEL_NUMERICAL_METHOD
        )))
        .await
        .context("list_numerical_methods")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_numerical_methods row")? {
        out.push(row_to_numerical_method(&row)?);
    }
    Ok(out)
}

// ── Row deserialisation helpers ───────────────────────────────────────────────

pub fn opt_str_field(node: &neo4rs::Node, field: &str) -> Option<String> {
    node.get::<String>(field).ok().filter(|s| !s.is_empty())
}

fn bool_field(node: &neo4rs::Node, field: &str) -> bool {
    node.get::<bool>(field).unwrap_or(false)
}

fn i64_field(node: &neo4rs::Node, field: &str) -> i64 {
    node.get::<i64>(field).unwrap_or(0)
}

pub fn json_vec_field(node: &neo4rs::Node, field: &str) -> Vec<String> {
    node.get::<String>(field)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn row_to_equation(row: &Row) -> Result<Equation> {
    let n: neo4rs::Node = row.get("n").context("equation node")?;
    Ok(Equation {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        pde_type: PdeType::from_str(&n.get::<String>("pde_type").unwrap_or_default())?,
        variables: json_vec_field(&n, "variables"),
        time_dependent: bool_field(&n, "time_dependent"),
        operator: opt_str_field(&n, "operator"),
        description: opt_str_field(&n, "description"),
        tags: json_vec_field(&n, "tags"),
    })
}

pub fn row_to_ai_model(row: &Row) -> Result<AIModel> {
    let n: neo4rs::Node = row.get("n").context("ai_model node")?;
    Ok(AIModel {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        architecture: n.get("architecture").unwrap_or_default(),
        input_vars: json_vec_field(&n, "input_vars"),
        output_vars: json_vec_field(&n, "output_vars"),
        training_type: TrainingType::from_str(
            &n.get::<String>("training_type").unwrap_or_default(),
        )?,
        description: opt_str_field(&n, "description"),
        paper_ref: opt_str_field(&n, "paper_ref"),
        tags: json_vec_field(&n, "tags"),
        engine_id: opt_str_field(&n, "engine_id"),
    })
}

pub fn row_to_numerical_method(row: &Row) -> Result<NumericalMethod> {
    let n: neo4rs::Node = row.get("n").context("numerical_method node")?;
    let order = i64_field(&n, "order");
    Ok(NumericalMethod {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        method_type: NumericalMethodType::from_str(
            &n.get::<String>("method_type").unwrap_or_default(),
        )?,
        order: if order > 0 { Some(order as u32) } else { None },
        description: opt_str_field(&n, "description"),
        tags: json_vec_field(&n, "tags"),
        engine_id: opt_str_field(&n, "engine_id"),
    })
}

// ── Benchmark / BenchResult ───────────────────────────────────────────────────

/// Upsert a Benchmark node. Also wires `[:ON_DATASET]` and `[:USES_METRIC]`
/// edges to the referenced Dataset and Metric, which must already exist.
pub async fn upsert_benchmark(graph: &Graph, b: &Benchmark) -> Result<()> {
    graph.run(neo4rs::query(&format!(
        "MERGE (n:{label} {{id: $id}}) \
         SET n.name = $name, \
             n.dataset_id = $dataset_id, \
             n.metric_id = $metric_id, \
             n.lower_is_better = $lower, \
             n.protocol = $protocol, \
             n.tolerance = $tolerance",
        label = LABEL_BENCHMARK
    ))
    .param("id", b.id.as_str())
    .param("name", b.name.as_str())
    .param("dataset_id", b.dataset_id.as_str())
    .param("metric_id", b.metric_id.as_str())
    .param("lower", b.lower_is_better)
    .param("protocol", b.protocol.as_deref().unwrap_or(""))
    .param("tolerance", b.tolerance.unwrap_or(0.05)))
    .await
    .context("upsert_benchmark: node")?;

    graph.run(neo4rs::query(&format!(
        "MATCH (b:{bl} {{id: $bid}}), (d:{dl} {{id: $did}}) \
         MERGE (b)-[:{rel}]->(d)",
        bl = LABEL_BENCHMARK, dl = LABEL_DATASET, rel = REL_ON_DATASET
    ))
    .param("bid", b.id.as_str())
    .param("did", b.dataset_id.as_str()))
    .await
    .context("upsert_benchmark: ON_DATASET")?;

    graph.run(neo4rs::query(&format!(
        "MATCH (b:{bl} {{id: $bid}}), (m:{ml} {{id: $mid}}) \
         MERGE (b)-[:{rel}]->(m)",
        bl = LABEL_BENCHMARK, ml = LABEL_METRIC, rel = REL_USES_METRIC
    ))
    .param("bid", b.id.as_str())
    .param("mid", b.metric_id.as_str()))
    .await
    .context("upsert_benchmark: USES_METRIC")?;

    Ok(())
}

/// Auto-id format: `{method}__{benchmark}__{src_short}__{nanos_hex}`.
/// Nanoseconds give effective uniqueness for sequential calls; concurrent
/// callers may collide but MERGE makes the upsert idempotent on collision.
fn generate_bench_result_id(method_id: &str, benchmark_id: &str, source: &SourceType) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{}__{}__{}__{:x}", method_id, benchmark_id, source.short(), nanos)
}

/// Upsert a BenchResult. Returns the resolved id (auto-generated if input was None).
///
/// Validation:
///   `source_type ∈ {paper_reported, third_party_reproduction}` requires `source_paper_id`.
///
/// Wires three edges:
///   `[:OF_METHOD]` → AIModel | NumericalMethod
///   `[:ON_BENCHMARK]` → Benchmark
///   `[:REPORTED_IN]` → Paper (only when source_paper_id is non-empty)
pub async fn upsert_bench_result(graph: &Graph, r: &BenchResult) -> Result<String> {
    if matches!(
        r.source_type,
        SourceType::PaperReported | SourceType::ThirdPartyReproduction
    ) && r.source_paper_id.as_deref().unwrap_or("").is_empty()
    {
        anyhow::bail!(
            "source_paper_id is required when source_type is paper_reported or third_party_reproduction"
        );
    }
    if r.method_label != LABEL_AI_MODEL && r.method_label != LABEL_NUMERICAL_METHOD {
        anyhow::bail!(
            "method_label must be \"AIModel\" or \"NumericalMethod\", got \"{}\"",
            r.method_label
        );
    }

    let id = match r.id.as_deref() {
        Some(s) if !s.is_empty() => s.to_string(),
        _ => generate_bench_result_id(&r.method_id, &r.benchmark_id, &r.source_type),
    };
    let recorded_at = r.recorded_at.as_deref().unwrap_or("").to_string();

    // Use Cypher's datetime() to fill timestamp when caller didn't provide one.
    graph.run(neo4rs::query(&format!(
        "MERGE (n:{label} {{id: $id}}) \
         SET n.method_id = $mid, \
             n.method_label = $mlabel, \
             n.benchmark_id = $bid, \
             n.value = $value, \
             n.source_type = $stype, \
             n.source_paper_id = $pid, \
             n.hardware = $hw, \
             n.code_ref = $code, \
             n.recorded_at = CASE WHEN $ts <> \"\" THEN $ts ELSE toString(datetime()) END",
        label = LABEL_BENCH_RESULT
    ))
    .param("id", id.as_str())
    .param("mid", r.method_id.as_str())
    .param("mlabel", r.method_label.as_str())
    .param("bid", r.benchmark_id.as_str())
    .param("value", r.value)
    .param("stype", r.source_type.as_str())
    .param("pid", r.source_paper_id.as_deref().unwrap_or(""))
    .param("hw", r.hardware.as_deref().unwrap_or(""))
    .param("code", r.code_ref.as_deref().unwrap_or(""))
    .param("ts", recorded_at.as_str()))
    .await
    .context("upsert_bench_result: node")?;

    graph.run(neo4rs::query(&format!(
        "MATCH (r:{rl} {{id: $rid}}), (m:{ml} {{id: $mid}}) \
         MERGE (r)-[:{rel}]->(m)",
        rl = LABEL_BENCH_RESULT, ml = r.method_label, rel = REL_OF_METHOD
    ))
    .param("rid", id.as_str())
    .param("mid", r.method_id.as_str()))
    .await
    .context("upsert_bench_result: OF_METHOD")?;

    graph.run(neo4rs::query(&format!(
        "MATCH (r:{rl} {{id: $rid}}), (b:{bl} {{id: $bid}}) \
         MERGE (r)-[:{rel}]->(b)",
        rl = LABEL_BENCH_RESULT, bl = LABEL_BENCHMARK, rel = REL_ON_BENCHMARK
    ))
    .param("rid", id.as_str())
    .param("bid", r.benchmark_id.as_str()))
    .await
    .context("upsert_bench_result: ON_BENCHMARK")?;

    if let Some(pid) = r.source_paper_id.as_deref() {
        if !pid.is_empty() {
            graph.run(neo4rs::query(&format!(
                "MATCH (r:{rl} {{id: $rid}}), (p:{pl} {{id: $pid}}) \
                 MERGE (r)-[:{rel}]->(p)",
                rl = LABEL_BENCH_RESULT, pl = LABEL_PAPER, rel = REL_REPORTED_IN
            ))
            .param("rid", id.as_str())
            .param("pid", pid))
            .await
            .context("upsert_bench_result: REPORTED_IN")?;
        }
    }

    Ok(id)
}

pub async fn get_benchmark(graph: &Graph, id: &str) -> Result<Option<Benchmark>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n", label = LABEL_BENCHMARK
        ))
        .param("id", id))
        .await
        .context("get_benchmark")?;

    if let Some(row) = result.next().await.context("get_benchmark next")? {
        Ok(Some(row_to_benchmark(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_benchmarks(graph: &Graph) -> Result<Vec<Benchmark>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label}) RETURN n ORDER BY n.name", label = LABEL_BENCHMARK
        )))
        .await
        .context("list_benchmarks")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_benchmarks row")? {
        out.push(row_to_benchmark(&row)?);
    }
    Ok(out)
}

pub async fn get_bench_result(graph: &Graph, id: &str) -> Result<Option<BenchResult>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{label} {{id: $id}}) RETURN n", label = LABEL_BENCH_RESULT
        ))
        .param("id", id))
        .await
        .context("get_bench_result")?;

    if let Some(row) = result.next().await.context("get_bench_result next")? {
        Ok(Some(row_to_bench_result(&row)?))
    } else {
        Ok(None)
    }
}

pub async fn list_results_for_method(
    graph: &Graph,
    method_label: &str,
    method_id: &str,
) -> Result<Vec<BenchResult>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{rl})-[:{rel}]->(m:{ml} {{id: $id}}) RETURN n ORDER BY n.recorded_at DESC",
            rl = LABEL_BENCH_RESULT, rel = REL_OF_METHOD, ml = method_label
        ))
        .param("id", method_id))
        .await
        .context("list_results_for_method")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_results_for_method row")? {
        out.push(row_to_bench_result(&row)?);
    }
    Ok(out)
}

pub async fn list_results_for_benchmark(
    graph: &Graph,
    benchmark_id: &str,
) -> Result<Vec<BenchResult>> {
    let mut result = graph
        .execute(neo4rs::query(&format!(
            "MATCH (n:{rl})-[:{rel}]->(b:{bl} {{id: $id}}) RETURN n ORDER BY n.recorded_at DESC",
            rl = LABEL_BENCH_RESULT, rel = REL_ON_BENCHMARK, bl = LABEL_BENCHMARK
        ))
        .param("id", benchmark_id))
        .await
        .context("list_results_for_benchmark")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("list_results_for_benchmark row")? {
        out.push(row_to_bench_result(&row)?);
    }
    Ok(out)
}

pub fn row_to_benchmark(row: &Row) -> Result<Benchmark> {
    let n: neo4rs::Node = row.get("n").context("benchmark node")?;
    Ok(Benchmark {
        id: n.get("id").unwrap_or_default(),
        name: n.get("name").unwrap_or_default(),
        dataset_id: n.get("dataset_id").unwrap_or_default(),
        metric_id: n.get("metric_id").unwrap_or_default(),
        lower_is_better: bool_field(&n, "lower_is_better"),
        protocol: opt_str_field(&n, "protocol"),
        tolerance: n.get::<f64>("tolerance").ok(),
    })
}

pub fn row_to_bench_result(row: &Row) -> Result<BenchResult> {
    let n: neo4rs::Node = row.get("n").context("bench_result node")?;
    Ok(BenchResult {
        id: opt_str_field(&n, "id").or_else(|| n.get::<String>("id").ok()),
        method_id: n.get("method_id").unwrap_or_default(),
        method_label: n.get("method_label").unwrap_or_default(),
        benchmark_id: n.get("benchmark_id").unwrap_or_default(),
        value: n.get::<f64>("value").unwrap_or(0.0),
        source_type: SourceType::from_str(
            &n.get::<String>("source_type").unwrap_or_default(),
        )?,
        source_paper_id: opt_str_field(&n, "source_paper_id"),
        hardware: opt_str_field(&n, "hardware"),
        code_ref: opt_str_field(&n, "code_ref"),
        recorded_at: opt_str_field(&n, "recorded_at"),
    })
}
