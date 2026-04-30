/// Core data types shared across the knowledge base crate.
///
/// These mirror the SQLite schema defined in migrations/001_initial.sql.
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ── Vector dimension ─────────────────────────────────────────────────────────

/// Dimension of embedding vectors (OpenAI text-embedding-3-small = 1536,
/// or any local model output; must match whatever the ingestion layer uses).
pub const EMBEDDING_DIM: usize = 1536;

// ── Paper ─────────────────────────────────────────────────────────────────────

/// A research paper stored in the knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    /// arXiv ID (e.g. "2301.12345") or DOI. Serves as primary key.
    pub id: String,
    pub title: String,
    pub abstract_text: Option<String>,
    /// JSON-serialised list of author names.
    pub authors: Vec<String>,
    pub published: Option<DateTime<Utc>>,
    pub source_url: Option<String>,
    pub pdf_url: Option<String>,
    /// Embedding vector; `None` if not yet generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Tags attached to this paper (populated by JOIN queries).
    #[serde(default)]
    pub tags: Vec<PaperTag>,
}

/// A single tag on a paper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct PaperTag {
    pub tag_type: TagType,
    pub tag_value: String,
}

/// Controlled vocabulary for tag types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TagType {
    PdeType,
    Method,
    Domain,
    Benchmark,
}

impl TagType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TagType::PdeType => "pde_type",
            TagType::Method => "method",
            TagType::Domain => "domain",
            TagType::Benchmark => "benchmark",
        }
    }
}

impl std::str::FromStr for TagType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pde_type" => Ok(TagType::PdeType),
            "method" => Ok(TagType::Method),
            "domain" => Ok(TagType::Domain),
            "benchmark" => Ok(TagType::Benchmark),
            other => Err(anyhow::anyhow!("unknown tag type: {}", other)),
        }
    }
}

// ── Method ────────────────────────────────────────────────────────────────────

/// A PDE method entry in the knowledge base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    /// Short identifier, e.g. "fem", "fno".
    pub id: String,
    pub name: String,
    pub category: MethodCategory,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding: Option<Vec<f32>>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MethodCategory {
    Classical,
    Ml,
    Hybrid,
}

impl MethodCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            MethodCategory::Classical => "classical",
            MethodCategory::Ml => "ml",
            MethodCategory::Hybrid => "hybrid",
        }
    }
}

impl std::str::FromStr for MethodCategory {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "classical" => Ok(MethodCategory::Classical),
            "ml" => Ok(MethodCategory::Ml),
            "hybrid" => Ok(MethodCategory::Hybrid),
            other => Err(anyhow::anyhow!("unknown method category: {}", other)),
        }
    }
}

/// A directed relation between two methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodRelation {
    pub from_method: String,
    pub to_method: String,
    pub relation: RelationKind,
    pub weight: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Extends,
    CompetesWith,
    CombinesWith,
    Requires,
}

impl RelationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationKind::Extends => "extends",
            RelationKind::CompetesWith => "competes_with",
            RelationKind::CombinesWith => "combines_with",
            RelationKind::Requires => "requires",
        }
    }
}

impl std::str::FromStr for RelationKind {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "extends" => Ok(RelationKind::Extends),
            "competes_with" => Ok(RelationKind::CompetesWith),
            "combines_with" => Ok(RelationKind::CombinesWith),
            "requires" => Ok(RelationKind::Requires),
            other => Err(anyhow::anyhow!("unknown relation kind: {}", other)),
        }
    }
}

// ── Helper: embedding serialisation ──────────────────────────────────────────

/// Serialise a float32 embedding vector to raw little-endian bytes for SQLite BLOB storage.
pub fn embedding_to_blob(v: &[f32]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(v.len() * 4);
    for &x in v {
        buf.extend_from_slice(&x.to_le_bytes());
    }
    buf
}

/// Deserialise a raw bytes BLOB back to a float32 vector.
pub fn blob_to_embedding(blob: &[u8]) -> anyhow::Result<Vec<f32>> {
    if blob.len() % 4 != 0 {
        anyhow::bail!("embedding blob length {} is not a multiple of 4", blob.len());
    }
    Ok(blob
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect())
}
