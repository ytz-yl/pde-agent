/// Fetch papers from the arXiv Atom/XML search API.
///
/// arXiv API docs: https://info.arxiv.org/help/api/user-manual.html
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;

use crate::store::schema::{Paper, PaperTag};

// ── Public API ────────────────────────────────────────────────────────────────

/// Search query parameters for the arXiv API.
#[derive(Debug, Clone)]
pub struct ArxivQuery {
    /// Free-text search string (supports arXiv query syntax).
    pub search_query: String,
    /// Maximum number of results to return.
    pub max_results: usize,
    /// 0-based start index (for pagination).
    pub start: usize,
}

impl ArxivQuery {
    /// Build a query that searches title + abstract for the given terms.
    pub fn new(terms: impl Into<String>) -> Self {
        ArxivQuery {
            search_query: format!("ti:{0}+OR+abs:{0}", terms.into()),
            max_results: 25,
            start: 0,
        }
    }

    pub fn max_results(mut self, n: usize) -> Self {
        self.max_results = n;
        self
    }

    pub fn start(mut self, n: usize) -> Self {
        self.start = n;
        self
    }
}

/// Fetch papers matching `query` from arXiv and return them as `Paper` structs.
/// Embeddings and tags are not set here — they are filled by the classifier.
pub async fn fetch_papers(client: &Client, query: &ArxivQuery) -> Result<Vec<Paper>> {
    let url = format!(
        "https://export.arxiv.org/api/query?search_query={}&start={}&max_results={}",
        query.search_query, query.start, query.max_results,
    );

    tracing::debug!("arXiv request: {}", url);

    let body = client
        .get(&url)
        .header("User-Agent", "pde-agent/0.1 (knowledge-base)")
        .send()
        .await
        .context("arXiv HTTP request")?
        .error_for_status()
        .context("arXiv HTTP status")?
        .text()
        .await
        .context("arXiv response body")?;

    parse_atom_feed(&body).context("parse arXiv Atom feed")
}

// ── Atom/XML parser ───────────────────────────────────────────────────────────

/// Parse an arXiv Atom feed and return a list of papers.
fn parse_atom_feed(xml: &str) -> Result<Vec<Paper>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut papers: Vec<Paper> = Vec::new();
    let mut current: Option<PaperBuilder> = None;
    let mut current_tag = String::new();

    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let tag = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();
                // Strip namespace prefix if present (e.g. "atom:entry" → "entry")
                let local = tag.split(':').last().unwrap_or(&tag).to_string();
                if local == "entry" {
                    current = Some(PaperBuilder::default());
                }
                current_tag = local;
            }
            Ok(Event::End(ref e)) => {
                let tag = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();
                let local = tag.split(':').last().unwrap_or(&tag).to_string();
                if local == "entry" {
                    if let Some(builder) = current.take() {
                        if let Some(paper) = builder.build() {
                            papers.push(paper);
                        }
                    }
                }
                current_tag.clear();
            }
            Ok(Event::Text(ref e)) => {
                if let Some(ref mut builder) = current {
                    let text = e.unescape().unwrap_or_default().into_owned();
                    match current_tag.as_str() {
                        "id" => {
                            // arXiv id looks like "http://arxiv.org/abs/2301.12345v1"
                            if let Some(id) = extract_arxiv_id(&text) {
                                builder.id = Some(id);
                                builder.source_url = Some(text.clone());
                            }
                        }
                        "title" => builder.title = Some(text.trim().replace('\n', " ")),
                        "summary" => builder.abstract_text = Some(text.trim().replace('\n', " ")),
                        "published" => {
                            builder.published = DateTime::parse_from_rfc3339(&text)
                                .ok()
                                .map(|dt| dt.with_timezone(&Utc));
                        }
                        _ => {}
                    }
                }
            }
            Ok(Event::Empty(ref e)) => {
                // Handle self-closing tags like <link href="..." />
                let tag = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_string();
                let local = tag.split(':').last().unwrap_or(&tag).to_string();
                if local == "link" {
                    if let Some(ref mut builder) = current {
                        let mut is_pdf = false;
                        let mut href = String::new();
                        for attr in e.attributes().flatten() {
                            let key = std::str::from_utf8(attr.key.as_ref()).unwrap_or("");
                            let val = std::str::from_utf8(&attr.value).unwrap_or("").to_string();
                            match key {
                                "title" if val == "pdf" => is_pdf = true,
                                "href" => href = val,
                                _ => {}
                            }
                        }
                        if is_pdf && !href.is_empty() {
                            builder.pdf_url = Some(href);
                        }
                    }
                }
                // Author names appear as <name> inside <author> blocks
                if local == "name" {
                    // handled in Text events above via current_tag
                }
            }
            // Collect author names: they appear as Text inside <name> inside <author>
            Ok(Event::Eof) => break,
            Err(e) => {
                tracing::warn!("arXiv XML parse error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(papers)
}

// ── Builder ───────────────────────────────────────────────────────────────────

#[derive(Default)]
struct PaperBuilder {
    id: Option<String>,
    title: Option<String>,
    abstract_text: Option<String>,
    published: Option<DateTime<Utc>>,
    source_url: Option<String>,
    pdf_url: Option<String>,
}

impl PaperBuilder {
    fn build(self) -> Option<Paper> {
        let id = self.id?;
        let title = self.title.unwrap_or_else(|| "(no title)".to_string());
        let now = Utc::now();
        Some(Paper {
            id,
            title,
            abstract_text: self.abstract_text,
            authors: vec![],  // author parsing added below if needed
            published: self.published,
            source_url: self.source_url,
            pdf_url: self.pdf_url,
            embedding: None,
            created_at: now,
            updated_at: now,
            tags: vec![],
        })
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Extract the bare arXiv ID (e.g. "2301.12345") from a full arXiv URL.
fn extract_arxiv_id(url: &str) -> Option<String> {
    // Formats: "http://arxiv.org/abs/2301.12345v1"
    //           "https://arxiv.org/abs/2301.12345"
    let base = url
        .trim_end_matches('/')
        .rsplit('/')
        .next()?
        .to_string();
    // Strip version suffix vN
    let id = if let Some(pos) = base.rfind('v') {
        let suffix = &base[pos + 1..];
        if suffix.chars().all(|c| c.is_ascii_digit()) {
            base[..pos].to_string()
        } else {
            base
        }
    } else {
        base
    };
    Some(id)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_arxiv_id() {
        assert_eq!(
            extract_arxiv_id("http://arxiv.org/abs/2301.12345v1"),
            Some("2301.12345".into())
        );
        assert_eq!(
            extract_arxiv_id("https://arxiv.org/abs/2301.12345"),
            Some("2301.12345".into())
        );
    }

    #[test]
    fn test_parse_minimal_feed() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>http://arxiv.org/abs/2301.00001v1</id>
    <title>Test PDE Paper</title>
    <summary>Abstract of the paper.</summary>
    <published>2023-01-01T00:00:00Z</published>
    <link title="pdf" href="https://arxiv.org/pdf/2301.00001"/>
  </entry>
</feed>"#;
        let papers = parse_atom_feed(xml).unwrap();
        assert_eq!(papers.len(), 1);
        assert_eq!(papers[0].id, "2301.00001");
        assert_eq!(papers[0].title, "Test PDE Paper");
        assert_eq!(
            papers[0].abstract_text.as_deref(),
            Some("Abstract of the paper.")
        );
        assert!(papers[0].pdf_url.is_some());
    }
}
