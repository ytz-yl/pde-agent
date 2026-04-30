/// SQLite-backed paper repository.
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use super::schema::{
    blob_to_embedding, embedding_to_blob, Paper, PaperTag, TagType,
};

// ── Create / Update ───────────────────────────────────────────────────────────

/// Insert or replace a paper (upsert by id).
pub fn upsert_paper(conn: &Connection, paper: &Paper) -> Result<()> {
    let authors_json = serde_json::to_string(&paper.authors)?;
    let published = paper.published.map(|dt| dt.to_rfc3339());
    let embedding_blob = paper.embedding.as_deref().map(embedding_to_blob);

    conn.execute(
        r#"
        INSERT INTO papers (id, title, abstract, authors, published, source_url, pdf_url, embedding,
                            created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
        ON CONFLICT(id) DO UPDATE SET
            title        = excluded.title,
            abstract     = excluded.abstract,
            authors      = excluded.authors,
            published    = excluded.published,
            source_url   = excluded.source_url,
            pdf_url      = excluded.pdf_url,
            embedding    = excluded.embedding,
            updated_at   = excluded.updated_at
        "#,
        params![
            paper.id,
            paper.title,
            paper.abstract_text,
            authors_json,
            published,
            paper.source_url,
            paper.pdf_url,
            embedding_blob,
            paper.created_at.to_rfc3339(),
            paper.updated_at.to_rfc3339(),
        ],
    )
    .context("upsert paper")?;

    // Replace all tags for this paper
    replace_tags(conn, &paper.id, &paper.tags)?;

    Ok(())
}

/// Replace all tags for a paper (delete + insert).
pub fn replace_tags(conn: &Connection, paper_id: &str, tags: &[PaperTag]) -> Result<()> {
    conn.execute(
        "DELETE FROM paper_tags WHERE paper_id = ?1",
        params![paper_id],
    )
    .context("delete old tags")?;

    for tag in tags {
        conn.execute(
            "INSERT OR IGNORE INTO paper_tags (paper_id, tag_type, tag_value) VALUES (?1, ?2, ?3)",
            params![paper_id, tag.tag_type.as_str(), tag.tag_value],
        )
        .context("insert tag")?;
    }
    Ok(())
}

// ── Read ──────────────────────────────────────────────────────────────────────

/// Fetch a paper by id, including its tags.
pub fn get_paper(conn: &Connection, id: &str) -> Result<Option<Paper>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, abstract, authors, published, source_url, pdf_url, embedding,
               created_at, updated_at
        FROM papers WHERE id = ?1
        "#,
    )?;

    let paper_opt = stmt
        .query_row(params![id], |row| row_to_paper(row))
        .optional()
        .context("get_paper query")?;

    if let Some(mut paper) = paper_opt {
        paper.tags = load_tags(conn, &paper.id)?;
        Ok(Some(paper))
    } else {
        Ok(None)
    }
}

/// Fetch recent papers, optionally filtered by domain tag, ordered by published date.
pub fn get_recent_papers(
    conn: &Connection,
    domain: Option<&str>,
    limit: usize,
) -> Result<Vec<Paper>> {
    let papers: Vec<Paper> = if let Some(domain) = domain {
        let mut stmt = conn.prepare(
            r#"
            SELECT p.id, p.title, p.abstract, p.authors, p.published, p.source_url, p.pdf_url,
                   p.embedding, p.created_at, p.updated_at
            FROM papers p
            JOIN paper_tags t ON t.paper_id = p.id
            WHERE t.tag_type = 'domain' AND t.tag_value = ?1
            ORDER BY p.published DESC
            LIMIT ?2
            "#,
        )?;
        let rows = stmt.query_map(params![domain, limit as i64], |row| row_to_paper(row))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("get_recent_papers (domain filtered)")?;
        rows
    } else {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, title, abstract, authors, published, source_url, pdf_url,
                   embedding, created_at, updated_at
            FROM papers
            ORDER BY published DESC
            LIMIT ?1
            "#,
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| row_to_paper(row))?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("get_recent_papers")?;
        rows
    };

    papers
        .into_iter()
        .map(|mut p| {
            p.tags = load_tags(conn, &p.id)?;
            Ok(p)
        })
        .collect()
}

/// Full-text search using the FTS5 virtual table. Returns papers ranked by BM25.
pub fn search_papers_fts(conn: &Connection, query: &str, limit: usize) -> Result<Vec<Paper>> {
    let papers: Vec<Paper> = {
        let mut stmt = conn.prepare(
            r#"
        SELECT p.id, p.title, p.abstract, p.authors, p.published, p.source_url, p.pdf_url,
               p.embedding, p.created_at, p.updated_at
        FROM papers p
        JOIN papers_fts f ON f.rowid = p.rowid
        WHERE papers_fts MATCH ?1
        ORDER BY rank
        LIMIT ?2
        "#,
        )?;
        let collected: rusqlite::Result<Vec<Paper>> =
            stmt.query_map(params![query, limit as i64], |row| row_to_paper(row))?
                .collect();
        collected.context("search_papers_fts")?
    };

    papers
        .into_iter()
        .map(|mut p| {
            p.tags = load_tags(conn, &p.id)?;
            Ok(p)
        })
        .collect()
}

/// List all papers that have a given tag.
pub fn papers_by_tag(
    conn: &Connection,
    tag_type: TagType,
    tag_value: &str,
    limit: usize,
) -> Result<Vec<Paper>> {
    let papers: Vec<Paper> = {
        let mut stmt = conn.prepare(
            r#"
        SELECT p.id, p.title, p.abstract, p.authors, p.published, p.source_url, p.pdf_url,
               p.embedding, p.created_at, p.updated_at
        FROM papers p
        JOIN paper_tags t ON t.paper_id = p.id
        WHERE t.tag_type = ?1 AND t.tag_value = ?2
        ORDER BY p.published DESC
        LIMIT ?3
        "#,
        )?;
        let collected: rusqlite::Result<Vec<Paper>> = stmt
            .query_map(
                params![tag_type.as_str(), tag_value, limit as i64],
                |row| row_to_paper(row),
            )?
            .collect();
        collected.context("papers_by_tag")?
    };

    papers
        .into_iter()
        .map(|mut p| {
            p.tags = load_tags(conn, &p.id)?;
            Ok(p)
        })
        .collect()
}

/// Return all papers that have embeddings (needed by the vector index builder).
pub fn all_papers_with_embeddings(conn: &Connection) -> Result<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn.prepare(
        "SELECT id, embedding FROM papers WHERE embedding IS NOT NULL",
    )?;
    let rows: Vec<(String, Vec<u8>)> = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((id, blob))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()
    .context("all_papers_with_embeddings")?;

    rows.into_iter()
        .map(|(id, blob)| {
            let vec = blob_to_embedding(&blob).context("decode embedding")?;
            Ok((id, vec))
        })
        .collect()
}

// ── Delete ────────────────────────────────────────────────────────────────────

pub fn delete_paper(conn: &Connection, id: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM papers WHERE id = ?1", params![id])
        .context("delete_paper")?;
    Ok(n > 0)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn row_to_paper(row: &rusqlite::Row) -> rusqlite::Result<Paper> {
    let id: String = row.get(0)?;
    let title: String = row.get(1)?;
    let abstract_text: Option<String> = row.get(2)?;
    let authors_json: String = row.get(3)?;
    let published_str: Option<String> = row.get(4)?;
    let source_url: Option<String> = row.get(5)?;
    let pdf_url: Option<String> = row.get(6)?;
    let embedding_blob: Option<Vec<u8>> = row.get(7)?;
    let created_at_str: String = row.get(8)?;
    let updated_at_str: String = row.get(9)?;

    let authors: Vec<String> =
        serde_json::from_str(&authors_json).unwrap_or_default();

    let published = published_str.and_then(|s| {
        DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    });

    let embedding = embedding_blob.and_then(|b| blob_to_embedding(&b).ok());

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Paper {
        id,
        title,
        abstract_text,
        authors,
        published,
        source_url,
        pdf_url,
        embedding,
        created_at,
        updated_at,
        tags: vec![],
    })
}

fn load_tags(conn: &Connection, paper_id: &str) -> Result<Vec<PaperTag>> {
    let mut stmt = conn.prepare(
        "SELECT tag_type, tag_value FROM paper_tags WHERE paper_id = ?1",
    )?;
    let rows: Vec<(String, String)> = stmt.query_map(params![paper_id], |row| {
        let tt: String = row.get(0)?;
        let tv: String = row.get(1)?;
        Ok((tt, tv))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()
    .context("load_tags")?;

    rows.into_iter()
        .map(|(tt, tv)| {
            let tag_type = TagType::from_str(&tt).context("parse tag_type")?;
            Ok(PaperTag {
                tag_type,
                tag_value: tv,
            })
        })
        .collect()
}
