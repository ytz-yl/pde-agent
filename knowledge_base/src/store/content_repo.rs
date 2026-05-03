/// SQLite-backed content store for long-form text blobs.
///
/// Schema:
///   node_content (node_id TEXT, node_type TEXT, abstract TEXT, notes TEXT)
///
/// Keyed by (node_id, node_type) so ids from different node types never clash.
/// Only Paper uses abstract today; other nodes may use notes in future.

use anyhow::{Context, Result};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ── Open / migrate ────────────────────────────────────────────────────────────

/// Open (or create) the SQLite content database and run the migration.
pub fn open_content_db(path: impl AsRef<Path>) -> Result<Connection> {
    let conn = Connection::open(path.as_ref())
        .with_context(|| format!("open content db at {:?}", path.as_ref()))?;
    migrate(&conn)?;
    Ok(conn)
}

fn migrate(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS node_content (
             node_id    TEXT NOT NULL,
             node_type  TEXT NOT NULL,
             abstract   TEXT,
             notes      TEXT,
             updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
             PRIMARY KEY (node_id, node_type)
         );
        ",
    )
    .context("content db migration")
}

// ── Data types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NodeContent {
    pub node_id: String,
    pub node_type: String,
    pub abstract_text: Option<String>,
    pub notes: Option<String>,
}

// ── Write ─────────────────────────────────────────────────────────────────────

/// Upsert content for a node. Empty strings are stored as NULL.
pub fn upsert_content(conn: &Connection, content: &NodeContent) -> Result<()> {
    let abs = content.abstract_text.as_deref().filter(|s| !s.is_empty());
    let notes = content.notes.as_deref().filter(|s| !s.is_empty());

    conn.execute(
        "INSERT INTO node_content (node_id, node_type, abstract, notes, updated_at)
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%SZ','now'))
         ON CONFLICT(node_id, node_type) DO UPDATE SET
             abstract   = excluded.abstract,
             notes      = excluded.notes,
             updated_at = excluded.updated_at",
        params![content.node_id, content.node_type, abs, notes],
    )
    .context("upsert_content")?;
    Ok(())
}

/// Delete content for a node (e.g. when the node itself is deleted).
pub fn delete_content(conn: &Connection, node_id: &str, node_type: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM node_content WHERE node_id = ?1 AND node_type = ?2",
        params![node_id, node_type],
    )
    .context("delete_content")?;
    Ok(())
}

// ── Read ──────────────────────────────────────────────────────────────────────

/// Fetch content for a node. Returns None if no row exists.
pub fn get_content(
    conn: &Connection,
    node_id: &str,
    node_type: &str,
) -> Result<Option<NodeContent>> {
    let mut stmt = conn.prepare(
        "SELECT node_id, node_type, abstract, notes
         FROM node_content WHERE node_id = ?1 AND node_type = ?2",
    )?;

    stmt.query_row(params![node_id, node_type], |row| {
        Ok(NodeContent {
            node_id: row.get(0)?,
            node_type: row.get(1)?,
            abstract_text: row.get(2)?,
            notes: row.get(3)?,
        })
    })
    .optional()
    .context("get_content")
}
