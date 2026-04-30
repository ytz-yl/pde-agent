/// LLM-based classifier: generate embeddings and extract PDE tags from a paper.
///
/// Calls the OpenAI-compatible embeddings API (works with OpenAI, local Ollama, etc.).
/// Env vars:
///   OPENAI_API_KEY   — API key (required for OpenAI)
///   OPENAI_API_BASE  — override base URL (default: https://api.openai.com/v1)
///   EMBEDDING_MODEL  — model name (default: text-embedding-3-small)
///   CHAT_MODEL       — chat model for tag extraction (default: gpt-4o-mini)
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::store::schema::{PaperTag, TagType};

// ── Config ────────────────────────────────────────────────────────────────────

pub struct LlmConfig {
    pub api_base: String,
    pub api_key: String,
    pub embedding_model: String,
    pub chat_model: String,
}

impl LlmConfig {
    pub fn from_env() -> Self {
        LlmConfig {
            api_base: std::env::var("OPENAI_API_BASE")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            api_key: std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            embedding_model: std::env::var("EMBEDDING_MODEL")
                .unwrap_or_else(|_| "text-embedding-3-small".into()),
            chat_model: std::env::var("CHAT_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".into()),
        }
    }
}

// ── Embedding ─────────────────────────────────────────────────────────────────

/// Request body for the embeddings API.
#[derive(Serialize)]
struct EmbeddingRequest<'a> {
    input: &'a str,
    model: &'a str,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// Generate an embedding vector for `text`.
pub async fn embed_text(
    client: &Client,
    cfg: &LlmConfig,
    text: &str,
) -> Result<Vec<f32>> {
    let req = EmbeddingRequest {
        input: text,
        model: &cfg.embedding_model,
    };

    let resp: EmbeddingResponse = client
        .post(format!("{}/embeddings", cfg.api_base))
        .bearer_auth(&cfg.api_key)
        .json(&req)
        .send()
        .await
        .context("embedding API request")?
        .error_for_status()
        .context("embedding API status")?
        .json()
        .await
        .context("embedding API response")?;

    resp.data
        .into_iter()
        .next()
        .map(|d| d.embedding)
        .context("empty embedding response")
}

// ── Tag extraction ────────────────────────────────────────────────────────────

/// Request body for the chat completions API.
#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    temperature: f32,
    response_format: ResponseFormat,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ResponseFormat {
    r#type: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage2,
}

#[derive(Deserialize)]
struct ChatMessage2 {
    content: String,
}

/// Structured tags extracted by the LLM.
#[derive(Debug, Deserialize, Default)]
pub struct ExtractedTags {
    /// PDE types identified in the paper, e.g. ["navier_stokes", "wave_equation"]
    #[serde(default)]
    pub pde_types: Vec<String>,
    /// Numerical/ML methods used or proposed, e.g. ["fno", "pinns"]
    #[serde(default)]
    pub methods: Vec<String>,
    /// Application domains, e.g. ["fluid_dynamics", "heat_transfer"]
    #[serde(default)]
    pub domains: Vec<String>,
    /// Benchmark datasets/problems mentioned, e.g. ["navier_stokes_2d", "burgers_1d"]
    #[serde(default)]
    pub benchmarks: Vec<String>,
}

impl ExtractedTags {
    /// Convert to the flat `PaperTag` list used by the store.
    pub fn to_paper_tags(&self) -> Vec<PaperTag> {
        let mut tags = Vec::new();
        for v in &self.pde_types {
            tags.push(PaperTag { tag_type: TagType::PdeType, tag_value: v.clone() });
        }
        for v in &self.methods {
            tags.push(PaperTag { tag_type: TagType::Method, tag_value: v.clone() });
        }
        for v in &self.domains {
            tags.push(PaperTag { tag_type: TagType::Domain, tag_value: v.clone() });
        }
        for v in &self.benchmarks {
            tags.push(PaperTag { tag_type: TagType::Benchmark, tag_value: v.clone() });
        }
        tags
    }
}

/// Extract PDE tags from a paper title + abstract using the LLM.
pub async fn extract_tags(
    client: &Client,
    cfg: &LlmConfig,
    title: &str,
    abstract_text: &str,
) -> Result<ExtractedTags> {
    let system_prompt = r#"You are a scientific metadata extractor specialising in partial differential equations (PDEs).
Given a paper title and abstract, extract structured tags and return them as JSON.

Return ONLY valid JSON with these fields (all are arrays of lowercase_snake_case strings):
{
  "pde_types": [...],    // e.g. "navier_stokes", "heat_equation", "wave_equation", "poisson"
  "methods": [...],      // e.g. "fno", "pinns", "deeponet", "fem", "fdm", "fvm", "spectral"
  "domains": [...],      // e.g. "fluid_dynamics", "heat_transfer", "elasticity", "electromagnetics"
  "benchmarks": [...]    // e.g. "burgers_1d", "navier_stokes_2d", "darcy_flow"
}
Use empty arrays if a category has no relevant entries. Do not add explanations."#;

    let user_content = format!("Title: {}\n\nAbstract: {}", title, abstract_text);

    let req = ChatRequest {
        model: &cfg.chat_model,
        messages: vec![
            ChatMessage { role: "system".into(), content: system_prompt.into() },
            ChatMessage { role: "user".into(), content: user_content },
        ],
        temperature: 0.0,
        response_format: ResponseFormat { r#type: "json_object".into() },
    };

    let resp: ChatResponse = client
        .post(format!("{}/chat/completions", cfg.api_base))
        .bearer_auth(&cfg.api_key)
        .json(&req)
        .send()
        .await
        .context("chat API request")?
        .error_for_status()
        .context("chat API status")?
        .json()
        .await
        .context("chat API response")?;

    let content = resp
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .context("empty chat response")?;

    serde_json::from_str::<ExtractedTags>(&content)
        .context("parse extracted tags JSON")
}
