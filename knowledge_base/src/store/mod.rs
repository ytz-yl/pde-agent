pub mod schema;
pub mod paper_repo;
pub mod method_repo;
pub mod vector_index;

use std::path::Path;
use anyhow::{Context, Result};
use rusqlite::Connection;

/// Open (or create) the SQLite database and run all pending migrations.
pub fn open_db(path: impl AsRef<Path>) -> Result<Connection> {
    let conn = Connection::open(path.as_ref())
        .with_context(|| format!("open SQLite at {:?}", path.as_ref()))?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Open an in-memory SQLite database (useful for tests).
pub fn open_db_in_memory() -> Result<Connection> {
    let conn = Connection::open_in_memory().context("open in-memory SQLite")?;
    run_migrations(&conn)?;
    Ok(conn)
}

/// Apply the embedded migration SQL.
fn run_migrations(conn: &Connection) -> Result<()> {
    conn.execute_batch(include_str!("../../migrations/001_initial.sql"))
        .context("run migration 001_initial.sql")
}
