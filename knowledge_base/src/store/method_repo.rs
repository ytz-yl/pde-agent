/// SQLite-backed method repository.
use std::str::FromStr;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use super::schema::{
    blob_to_embedding, embedding_to_blob, Method, MethodCategory, MethodRelation, RelationKind,
};

// ── Create / Update ───────────────────────────────────────────────────────────

pub fn upsert_method(conn: &Connection, method: &Method) -> Result<()> {
    let tags_json = serde_json::to_string(&method.tags)?;
    let embedding_blob = method.embedding.as_deref().map(embedding_to_blob);

    conn.execute(
        r#"
        INSERT INTO methods (id, name, category, description, embedding, tags, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        ON CONFLICT(id) DO UPDATE SET
            name        = excluded.name,
            category    = excluded.category,
            description = excluded.description,
            embedding   = excluded.embedding,
            tags        = excluded.tags,
            updated_at  = excluded.updated_at
        "#,
        params![
            method.id,
            method.name,
            method.category.as_str(),
            method.description,
            embedding_blob,
            tags_json,
            method.created_at.to_rfc3339(),
            method.updated_at.to_rfc3339(),
        ],
    )
    .context("upsert_method")?;
    Ok(())
}

pub fn upsert_relation(conn: &Connection, rel: &MethodRelation) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO method_relations (from_method, to_method, relation, weight)
        VALUES (?1, ?2, ?3, ?4)
        ON CONFLICT(from_method, to_method, relation) DO UPDATE SET
            weight = excluded.weight
        "#,
        params![
            rel.from_method,
            rel.to_method,
            rel.relation.as_str(),
            rel.weight,
        ],
    )
    .context("upsert_relation")?;
    Ok(())
}

// ── Read ──────────────────────────────────────────────────────────────────────

pub fn get_method(conn: &Connection, id: &str) -> Result<Option<Method>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, category, description, embedding, tags, created_at, updated_at
        FROM methods WHERE id = ?1
        "#,
    )?;
    stmt.query_row(params![id], row_to_method)
        .optional()
        .context("get_method")
}

pub fn list_methods(conn: &Connection, category: Option<&str>) -> Result<Vec<Method>> {
    if let Some(cat) = category {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, category, description, embedding, tags, created_at, updated_at
            FROM methods WHERE category = ?1 ORDER BY name
            "#,
        )?;
        let rows = stmt.query_map(params![cat], row_to_method)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("list_methods (filtered)")?;
        Ok(rows)
    } else {
        let mut stmt = conn.prepare(
            r#"
            SELECT id, name, category, description, embedding, tags, created_at, updated_at
            FROM methods ORDER BY name
            "#,
        )?;
        let rows = stmt.query_map([], row_to_method)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("list_methods")?;
        Ok(rows)
    }
}

/// Get all methods related to `id`, optionally filtered by relation kind.
pub fn get_related_methods(
    conn: &Connection,
    id: &str,
    relation: Option<RelationKind>,
) -> Result<Vec<(Method, MethodRelation)>> {
    let rels: Vec<MethodRelation> = if let Some(rel_kind) = relation {
        let mut stmt = conn.prepare(
            r#"
            SELECT from_method, to_method, relation, weight
            FROM method_relations
            WHERE (from_method = ?1 OR to_method = ?1) AND relation = ?2
            "#,
        )?;
        let rows = stmt.query_map(params![id, rel_kind.as_str()], row_to_relation)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("get_related_methods (filtered)")?;
        rows
    } else {
        let mut stmt = conn.prepare(
            r#"
            SELECT from_method, to_method, relation, weight
            FROM method_relations
            WHERE from_method = ?1 OR to_method = ?1
            "#,
        )?;
        let rows = stmt.query_map(params![id], row_to_relation)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .context("get_related_methods")?;
        rows
    };

    rels.into_iter()
        .filter_map(|rel| {
            let other_id = if rel.from_method == id {
                &rel.to_method
            } else {
                &rel.from_method
            };
            match get_method(conn, other_id) {
                Ok(Some(m)) => Some(Ok((m, rel))),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        })
        .collect()
}

/// Return all methods that have embeddings (for vector index rebuild).
pub fn all_methods_with_embeddings(conn: &Connection) -> Result<Vec<(String, Vec<f32>)>> {
    let mut stmt = conn
        .prepare("SELECT id, embedding FROM methods WHERE embedding IS NOT NULL")?;
    let rows: Vec<(String, Vec<u8>)> = stmt.query_map([], |row| {
        let id: String = row.get(0)?;
        let blob: Vec<u8> = row.get(1)?;
        Ok((id, blob))
    })?
    .collect::<rusqlite::Result<Vec<_>>>()
    .context("all_methods_with_embeddings")?;

    rows.into_iter()
        .map(|(id, blob)| {
            let vec = blob_to_embedding(&blob).context("decode embedding")?;
            Ok((id, vec))
        })
        .collect()
}

// ── Delete ────────────────────────────────────────────────────────────────────

pub fn delete_method(conn: &Connection, id: &str) -> Result<bool> {
    let n = conn
        .execute("DELETE FROM methods WHERE id = ?1", params![id])
        .context("delete_method")?;
    Ok(n > 0)
}

// ── Internal helpers ──────────────────────────────────────────────────────────

fn row_to_method(row: &rusqlite::Row) -> rusqlite::Result<Method> {
    let id: String = row.get(0)?;
    let name: String = row.get(1)?;
    let category_str: String = row.get(2)?;
    let description: Option<String> = row.get(3)?;
    let embedding_blob: Option<Vec<u8>> = row.get(4)?;
    let tags_json: String = row.get(5)?;
    let created_at_str: String = row.get(6)?;
    let updated_at_str: String = row.get(7)?;

    let category =
        MethodCategory::from_str(&category_str).unwrap_or(MethodCategory::Classical);
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let embedding = embedding_blob.and_then(|b| blob_to_embedding(&b).ok());

    let created_at = DateTime::parse_from_rfc3339(&created_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());
    let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now());

    Ok(Method {
        id,
        name,
        category,
        description,
        embedding,
        tags,
        created_at,
        updated_at,
    })
}

fn row_to_relation(row: &rusqlite::Row) -> rusqlite::Result<MethodRelation> {
    let from_method: String = row.get(0)?;
    let to_method: String = row.get(1)?;
    let relation_str: String = row.get(2)?;
    let weight: f64 = row.get(3)?;

    let relation = RelationKind::from_str(&relation_str)
        .unwrap_or(RelationKind::CompetesWith);

    Ok(MethodRelation {
        from_method,
        to_method,
        relation,
        weight: weight as f32,
    })
}
