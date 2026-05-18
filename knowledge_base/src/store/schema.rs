/// PDE Knowledge Base — Core domain types.
///
/// Node types (Neo4j labels):
///   Equation, Condition, Theorem, NumericalMethod, AIModel,
///   LossFunction, Metric, Dataset, Paper
///
/// Relation types (Neo4j edge types):
///   SOLVES, REQUIRES, HAS_CONDITION, APPLIES_TO, TRAINED_BY,
///   EVALUATED_BY, REPRESENTS, BASED_ON, TESTED_ON, VARIANT_OF,
///   PROPOSES, STUDIES, USES_DATASET, REPORTS_METRIC, CITES
///
/// Storage layout:
///   Neo4j  — structural fields (id, name, short enums, pdf_path …)
///   SQLite — long text blobs (abstract, notes) keyed by (node_id, node_type)

use serde::{Deserialize, Serialize};

// ── Node label constants ──────────────────────────────────────────────────────

pub const LABEL_EQUATION: &str = "Equation";
pub const LABEL_CONDITION: &str = "Condition";
pub const LABEL_THEOREM: &str = "Theorem";
pub const LABEL_NUMERICAL_METHOD: &str = "NumericalMethod";
pub const LABEL_AI_MODEL: &str = "AIModel";
pub const LABEL_LOSS_FUNCTION: &str = "LossFunction";
pub const LABEL_METRIC: &str = "Metric";
pub const LABEL_DATASET: &str = "Dataset";
pub const LABEL_PAPER: &str = "Paper";
pub const LABEL_BENCHMARK: &str = "Benchmark";
pub const LABEL_BENCH_RESULT: &str = "BenchResult";

// ── Relation type constants ───────────────────────────────────────────────────

pub const REL_SOLVES: &str = "SOLVES";
pub const REL_REQUIRES: &str = "REQUIRES";
pub const REL_HAS_CONDITION: &str = "HAS_CONDITION";
pub const REL_APPLIES_TO: &str = "APPLIES_TO";
pub const REL_TRAINED_BY: &str = "TRAINED_BY";
pub const REL_EVALUATED_BY: &str = "EVALUATED_BY";
pub const REL_REPRESENTS: &str = "REPRESENTS";
pub const REL_BASED_ON: &str = "BASED_ON";
pub const REL_TESTED_ON: &str = "TESTED_ON";
pub const REL_VARIANT_OF: &str = "VARIANT_OF";
// Paper relations
pub const REL_PROPOSES: &str = "PROPOSES";
pub const REL_STUDIES: &str = "STUDIES";
pub const REL_USES_DATASET: &str = "USES_DATASET";
pub const REL_REPORTS_METRIC: &str = "REPORTS_METRIC";
pub const REL_CITES: &str = "CITES";
// Benchmark / BenchResult relations
pub const REL_ON_DATASET: &str = "ON_DATASET";
pub const REL_USES_METRIC: &str = "USES_METRIC";
pub const REL_OF_METHOD: &str = "OF_METHOD";
pub const REL_ON_BENCHMARK: &str = "ON_BENCHMARK";
pub const REL_REPORTED_IN: &str = "REPORTED_IN";

// ── Equation ──────────────────────────────────────────────────────────────────

/// PDE classification types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PdeType {
    Parabolic,
    Elliptic,
    Hyperbolic,
    Mixed,
    Other,
}

impl PdeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PdeType::Parabolic => "parabolic",
            PdeType::Elliptic => "elliptic",
            PdeType::Hyperbolic => "hyperbolic",
            PdeType::Mixed => "mixed",
            PdeType::Other => "other",
        }
    }
}

impl std::str::FromStr for PdeType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "parabolic" => Ok(PdeType::Parabolic),
            "elliptic" => Ok(PdeType::Elliptic),
            "hyperbolic" => Ok(PdeType::Hyperbolic),
            "mixed" => Ok(PdeType::Mixed),
            _ => Ok(PdeType::Other),
        }
    }
}

/// A PDE equation node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equation {
    /// Unique identifier, e.g. "heat_equation", "navier_stokes".
    pub id: String,
    pub name: String,
    /// Parabolic / elliptic / hyperbolic / mixed.
    pub pde_type: PdeType,
    /// Mathematical variables involved, e.g. ["t", "x", "y"].
    pub variables: Vec<String>,
    /// Whether the equation is time-dependent.
    pub time_dependent: bool,
    /// Differential operator type, e.g. "laplacian", "gradient".
    pub operator: Option<String>,
    /// Free-text description.
    pub description: Option<String>,
    /// Tags for additional categorisation, e.g. ["diffusion", "heat"].
    pub tags: Vec<String>,
}

// ── Condition ─────────────────────────────────────────────────────────────────

/// Condition type classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    /// Boundary condition.
    Boundary,
    /// Initial condition.
    Initial,
    /// Domain constraint.
    Domain,
    /// Regularity assumption.
    Regularity,
    /// Other constraint.
    Other,
}

impl ConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConditionType::Boundary => "boundary",
            ConditionType::Initial => "initial",
            ConditionType::Domain => "domain",
            ConditionType::Regularity => "regularity",
            ConditionType::Other => "other",
        }
    }
}

impl std::str::FromStr for ConditionType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "boundary" => Ok(ConditionType::Boundary),
            "initial" => Ok(ConditionType::Initial),
            "domain" => Ok(ConditionType::Domain),
            "regularity" => Ok(ConditionType::Regularity),
            _ => Ok(ConditionType::Other),
        }
    }
}

/// A mathematical condition/constraint node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub id: String,
    pub name: String,
    pub condition_type: ConditionType,
    /// Mathematical form, e.g. "u = 0 on boundary".
    pub form: Option<String>,
    pub description: Option<String>,
}

// ── Theorem ───────────────────────────────────────────────────────────────────

/// A mathematical theorem node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theorem {
    pub id: String,
    pub name: String,
    /// The conclusion / result statement.
    pub result: String,
    /// Confidence that this theorem is correctly classified [0, 1].
    pub confidence: f32,
    pub description: Option<String>,
    /// Source reference, e.g. paper title or textbook.
    pub source: Option<String>,
}

// ── NumericalMethod ───────────────────────────────────────────────────────────

/// Category of a numerical method.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumericalMethodType {
    GridBased,
    MeshBased,
    SpectralBased,
    MeshFree,
    Other,
}

impl NumericalMethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NumericalMethodType::GridBased => "grid_based",
            NumericalMethodType::MeshBased => "mesh_based",
            NumericalMethodType::SpectralBased => "spectral_based",
            NumericalMethodType::MeshFree => "mesh_free",
            NumericalMethodType::Other => "other",
        }
    }
}

impl std::str::FromStr for NumericalMethodType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "grid_based" => Ok(NumericalMethodType::GridBased),
            "mesh_based" => Ok(NumericalMethodType::MeshBased),
            "spectral_based" => Ok(NumericalMethodType::SpectralBased),
            "mesh_free" => Ok(NumericalMethodType::MeshFree),
            _ => Ok(NumericalMethodType::Other),
        }
    }
}

/// A classical/numerical PDE solving method node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericalMethod {
    pub id: String,
    pub name: String,
    pub method_type: NumericalMethodType,
    /// Convergence order, e.g. 2 for second-order FDM.
    pub order: Option<u32>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    /// Bridge to engines API: solver id at `GET /solvers` (e.g. "classical").
    /// `Some(_)` means the method is callable via the local engines service;
    /// `None` means it exists only as literature reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_id: Option<String>,
}

// ── AIModel ───────────────────────────────────────────────────────────────────

/// Training paradigm for an AI model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingType {
    Supervised,
    Unsupervised,
    SelfSupervised,
    PhysicsInformed,
    OperatorLearning,
}

impl TrainingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrainingType::Supervised => "supervised",
            TrainingType::Unsupervised => "unsupervised",
            TrainingType::SelfSupervised => "self_supervised",
            TrainingType::PhysicsInformed => "physics_informed",
            TrainingType::OperatorLearning => "operator_learning",
        }
    }
}

impl std::str::FromStr for TrainingType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "supervised" => Ok(TrainingType::Supervised),
            "unsupervised" => Ok(TrainingType::Unsupervised),
            "self_supervised" => Ok(TrainingType::SelfSupervised),
            "physics_informed" => Ok(TrainingType::PhysicsInformed),
            "operator_learning" => Ok(TrainingType::OperatorLearning),
            _ => Ok(TrainingType::Supervised),
        }
    }
}

/// An AI/ML model for solving PDEs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    /// Neural architecture, e.g. "MLP", "CNN", "Transformer", "FNO".
    pub architecture: String,
    /// Input variable names, e.g. ["x", "t"].
    pub input_vars: Vec<String>,
    /// Output variable names, e.g. ["u"].
    pub output_vars: Vec<String>,
    pub training_type: TrainingType,
    pub description: Option<String>,
    /// Reference paper id or citation.
    pub paper_ref: Option<String>,
    pub tags: Vec<String>,
    /// Bridge to engines API: solver id at `GET /solvers` (e.g. "pdeformer2").
    /// `Some(_)` means the model is callable via the local engines service;
    /// `None` means it exists only as literature reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_id: Option<String>,
}

// ── LossFunction ──────────────────────────────────────────────────────────────

/// Category of a loss function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossType {
    Physics,
    DataDriven,
    Boundary,
    Combined,
    Other,
}

impl LossType {
    pub fn as_str(&self) -> &'static str {
        match self {
            LossType::Physics => "physics",
            LossType::DataDriven => "data_driven",
            LossType::Boundary => "boundary",
            LossType::Combined => "combined",
            LossType::Other => "other",
        }
    }
}

impl std::str::FromStr for LossType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "physics" => Ok(LossType::Physics),
            "data_driven" => Ok(LossType::DataDriven),
            "boundary" => Ok(LossType::Boundary),
            "combined" => Ok(LossType::Combined),
            _ => Ok(LossType::Other),
        }
    }
}

/// A loss / objective function node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossFunction {
    pub id: String,
    pub name: String,
    pub loss_type: LossType,
    /// Mathematical formulation description.
    pub formulation: Option<String>,
    pub description: Option<String>,
}

// ── Metric ────────────────────────────────────────────────────────────────────

/// What aspect a metric measures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Accuracy,
    Efficiency,
    Stability,
    Generalisation,
    Other,
}

impl MetricType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::Accuracy => "accuracy",
            MetricType::Efficiency => "efficiency",
            MetricType::Stability => "stability",
            MetricType::Generalisation => "generalisation",
            MetricType::Other => "other",
        }
    }
}

impl std::str::FromStr for MetricType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "accuracy" => Ok(MetricType::Accuracy),
            "efficiency" => Ok(MetricType::Efficiency),
            "stability" => Ok(MetricType::Stability),
            "generalisation" => Ok(MetricType::Generalisation),
            _ => Ok(MetricType::Other),
        }
    }
}

/// An evaluation metric node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: String,
    pub name: String,
    pub metric_type: MetricType,
    /// Unit or scale, e.g. "dimensionless", "seconds".
    pub unit: Option<String>,
    pub description: Option<String>,
}

// ── Dataset ───────────────────────────────────────────────────────────────────

/// A benchmark dataset node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    /// Dimensionality string, e.g. "1D", "2D", "3D".
    pub dimension: Option<String>,
    /// Number of samples if known.
    pub num_samples: Option<u64>,
    pub description: Option<String>,
    /// URL to dataset or paper.
    pub url: Option<String>,
}

// ── Paper ─────────────────────────────────────────────────────────────────────

/// A research paper node.
///
/// Long-form text (abstract, notes) is stored in SQLite `node_content`,
/// not here. `pdf_path` points to the local file system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    /// Stable identifier. Prefer arXiv id (e.g. "2301.12345") or DOI.
    pub id: String,
    pub title: String,
    /// Author list, e.g. ["Li, Z.", "Kovachki, N."].
    pub authors: Vec<String>,
    /// 4-digit year, e.g. 2021.
    pub published_year: Option<u32>,
    /// arXiv identifier if available, e.g. "2010.08895".
    pub arxiv_id: Option<String>,
    /// DOI string, e.g. "10.1145/3524.3521".
    pub doi: Option<String>,
    /// Absolute path to the downloaded PDF on the local filesystem.
    pub pdf_path: Option<String>,
    pub tags: Vec<String>,
}

// ── Benchmark / BenchResult ───────────────────────────────────────────────────

/// Provenance category for a single measurement.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// Value lifted from a paper's reported numbers.
    PaperReported,
    /// Locally re-run by the maintainer / agent.
    SelfRun,
    /// Independent third-party reproduction (different paper).
    ThirdPartyReproduction,
}

impl SourceType {
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceType::PaperReported => "paper_reported",
            SourceType::SelfRun => "self_run",
            SourceType::ThirdPartyReproduction => "third_party_reproduction",
        }
    }

    /// Compact slug used inside auto-generated BenchResult ids.
    pub fn short(&self) -> &'static str {
        match self {
            SourceType::PaperReported => "paper",
            SourceType::SelfRun => "self",
            SourceType::ThirdPartyReproduction => "third",
        }
    }
}

impl std::str::FromStr for SourceType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "paper_reported" => Ok(SourceType::PaperReported),
            "self_run" => Ok(SourceType::SelfRun),
            "third_party_reproduction" => Ok(SourceType::ThirdPartyReproduction),
            // Fall back to PaperReported (most conservative for ranking).
            _ => Ok(SourceType::PaperReported),
        }
    }
}

/// A reusable evaluation protocol: a metric situated on a specific dataset.
///
/// Many BenchResults can target the same Benchmark; the Benchmark itself is
/// defined once and serves as the comparison axis for ranking.
///
/// On upsert, two graph edges are also wired:
///   `(Benchmark)-[:ON_DATASET]->(Dataset)`
///   `(Benchmark)-[:USES_METRIC]->(Metric)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Benchmark {
    /// Stable id, e.g. "pdebench_ns2d_rel_l2".
    pub id: String,
    pub name: String,
    /// Dataset this benchmark evaluates on. Must match an existing Dataset.id.
    pub dataset_id: String,
    /// Metric used for scoring. Must match an existing Metric.id.
    pub metric_id: String,
    /// Whether smaller values are better (true for error metrics, false for speedup).
    pub lower_is_better: bool,
    /// Short protocol summary; long form goes to SQLite via /internal/content.
    pub protocol: Option<String>,
    /// Relative tolerance for cross-source agreement; default 0.05 if None.
    pub tolerance: Option<f64>,
}

/// One measured value of (method, benchmark) → value, with provenance.
///
/// Reliability comes from accumulating multiple BenchResults for the same
/// (method, benchmark) pair and aggregating at query time — never overwritten.
///
/// On upsert, three graph edges are also wired (REPORTED_IN only when set):
///   `(BenchResult)-[:OF_METHOD]->(<method_label>)`
///   `(BenchResult)-[:ON_BENCHMARK]->(Benchmark)`
///   `(BenchResult)-[:REPORTED_IN]->(Paper)`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchResult {
    /// Stable id. If `None` on upsert, the server generates one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Method node id (must exist).
    pub method_id: String,
    /// Method node label: "AIModel" or "NumericalMethod".
    pub method_label: String,
    /// Benchmark node id (must exist).
    pub benchmark_id: String,
    /// The measured numeric value, in the unit declared by the metric.
    pub value: f64,
    /// Where the value came from.
    pub source_type: SourceType,
    /// Required when source_type ∈ {paper_reported, third_party_reproduction}.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_paper_id: Option<String>,
    /// Free-form hardware string, e.g. "1x A100 80G".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware: Option<String>,
    /// Git commit, repo URL, or script path that reproduces the value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code_ref: Option<String>,
    /// ISO-8601 timestamp; auto-set to now on upsert if None.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_at: Option<String>,
}

// ── Generic node wrapper (for API responses) ──────────────────────────────────

/// All possible node variants returned by the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "node_type", rename_all = "snake_case")]
pub enum KnowledgeNode {
    Equation(Equation),
    Condition(Condition),
    Theorem(Theorem),
    NumericalMethod(NumericalMethod),
    AiModel(AIModel),
    LossFunction(LossFunction),
    Metric(Metric),
    Dataset(Dataset),
    Paper(Paper),
    Benchmark(Benchmark),
    BenchResult(BenchResult),
}

impl KnowledgeNode {
    pub fn node_id(&self) -> &str {
        match self {
            KnowledgeNode::Equation(n) => &n.id,
            KnowledgeNode::Condition(n) => &n.id,
            KnowledgeNode::Theorem(n) => &n.id,
            KnowledgeNode::NumericalMethod(n) => &n.id,
            KnowledgeNode::AiModel(n) => &n.id,
            KnowledgeNode::LossFunction(n) => &n.id,
            KnowledgeNode::Metric(n) => &n.id,
            KnowledgeNode::Dataset(n) => &n.id,
            KnowledgeNode::Paper(n) => &n.id,
            KnowledgeNode::Benchmark(n) => &n.id,
            KnowledgeNode::BenchResult(n) => n.id.as_deref().unwrap_or(""),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            KnowledgeNode::Equation(_) => LABEL_EQUATION,
            KnowledgeNode::Condition(_) => LABEL_CONDITION,
            KnowledgeNode::Theorem(_) => LABEL_THEOREM,
            KnowledgeNode::NumericalMethod(_) => LABEL_NUMERICAL_METHOD,
            KnowledgeNode::AiModel(_) => LABEL_AI_MODEL,
            KnowledgeNode::LossFunction(_) => LABEL_LOSS_FUNCTION,
            KnowledgeNode::Metric(_) => LABEL_METRIC,
            KnowledgeNode::Dataset(_) => LABEL_DATASET,
            KnowledgeNode::Paper(_) => LABEL_PAPER,
            KnowledgeNode::Benchmark(_) => LABEL_BENCHMARK,
            KnowledgeNode::BenchResult(_) => LABEL_BENCH_RESULT,
        }
    }
}

// ── Relation ──────────────────────────────────────────────────────────────────

/// A directed relation between two nodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Source node id.
    pub from_id: String,
    /// Source node label (for disambiguation when ids collide across types).
    pub from_label: String,
    /// Target node id.
    pub to_id: String,
    /// Target node label.
    pub to_label: String,
    /// Relation type, e.g. "SOLVES", "REQUIRES".
    pub relation_type: String,
    /// Optional properties bag (serialised as JSON).
    pub properties: Option<serde_json::Value>,
}

// ── NodeType enum (for write API routing) ─────────────────────────────────────

/// Discriminator used in write requests to identify which node type to create.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Equation,
    Condition,
    Theorem,
    NumericalMethod,
    AiModel,
    LossFunction,
    Metric,
    Dataset,
    Paper,
    Benchmark,
    BenchResult,
}

impl NodeType {
    pub fn as_label(&self) -> &'static str {
        match self {
            NodeType::Equation => LABEL_EQUATION,
            NodeType::Condition => LABEL_CONDITION,
            NodeType::Theorem => LABEL_THEOREM,
            NodeType::NumericalMethod => LABEL_NUMERICAL_METHOD,
            NodeType::AiModel => LABEL_AI_MODEL,
            NodeType::LossFunction => LABEL_LOSS_FUNCTION,
            NodeType::Metric => LABEL_METRIC,
            NodeType::Dataset => LABEL_DATASET,
            NodeType::Paper => LABEL_PAPER,
            NodeType::Benchmark => LABEL_BENCHMARK,
            NodeType::BenchResult => LABEL_BENCH_RESULT,
        }
    }
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_label())
    }
}
