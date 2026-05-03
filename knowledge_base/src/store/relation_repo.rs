/// Relation CRUD operations against Neo4j.
///
/// Uses MERGE semantics so all writes are idempotent.
/// Relation types match the constants in schema.rs.

use anyhow::{Context, Result};
use neo4rs::Graph;

use crate::store::schema::{Relation, REL_SOLVES, REL_REQUIRES, REL_HAS_CONDITION,
    REL_APPLIES_TO, REL_TRAINED_BY, REL_EVALUATED_BY, REL_REPRESENTS,
    REL_BASED_ON, REL_TESTED_ON, REL_VARIANT_OF,
    REL_PROPOSES, REL_STUDIES, REL_USES_DATASET, REL_REPORTS_METRIC, REL_CITES};

// ── Validation ────────────────────────────────────────────────────────────────

/// All valid relation type strings accepted by the write API.
pub const VALID_RELATION_TYPES: &[&str] = &[
    REL_SOLVES,
    REL_REQUIRES,
    REL_HAS_CONDITION,
    REL_APPLIES_TO,
    REL_TRAINED_BY,
    REL_EVALUATED_BY,
    REL_REPRESENTS,
    REL_BASED_ON,
    REL_TESTED_ON,
    REL_VARIANT_OF,
    // Paper relations
    REL_PROPOSES,
    REL_STUDIES,
    REL_USES_DATASET,
    REL_REPORTS_METRIC,
    REL_CITES,
];

pub fn is_valid_relation_type(rel_type: &str) -> bool {
    VALID_RELATION_TYPES.contains(&rel_type)
}

// ── Upsert ────────────────────────────────────────────────────────────────────

/// Create or update a relation between two nodes.
///
/// The Cypher uses dynamic label and relation-type strings constructed from
/// the Relation struct. This is safe because `rel_type` is validated against
/// `VALID_RELATION_TYPES` before reaching this function.
pub async fn upsert_relation(graph: &Graph, rel: &Relation) -> Result<()> {
    // Build a property SET clause from the optional JSON properties bag.
    // For simplicity we store the entire bag as a `properties` string on the edge.
    let props_clause = if rel.properties.is_some() {
        ", r.properties = $props"
    } else {
        ""
    };

    let cypher = format!(
        "MATCH (a:{fl} {{id: $from_id}}), (b:{tl} {{id: $to_id}}) \
         MERGE (a)-[r:{rel_type}]->(b) \
         SET r.created = coalesce(r.created, datetime()){props}",
        fl = rel.from_label,
        tl = rel.to_label,
        rel_type = rel.relation_type,
        props = props_clause,
    );

    let mut q = neo4rs::query(&cypher)
        .param("from_id", rel.from_id.as_str())
        .param("to_id", rel.to_id.as_str());

    if let Some(ref p) = rel.properties {
        q = q.param("props", p.to_string().as_str());
    }

    graph.run(q).await.context("upsert_relation")
}

/// Delete a specific relation between two nodes.
/// Returns true if at least one relation was deleted.
pub async fn delete_relation(
    graph: &Graph,
    from_label: &str,
    from_id: &str,
    to_label: &str,
    to_id: &str,
    rel_type: &str,
) -> Result<bool> {
    let cypher = format!(
        "MATCH (a:{fl} {{id: $from_id}})-[r:{rel}]->(b:{tl} {{id: $to_id}}) \
         DELETE r RETURN count(r) AS deleted",
        fl = from_label, tl = to_label, rel = rel_type
    );

    let mut result = graph
        .execute(neo4rs::query(&cypher)
            .param("from_id", from_id)
            .param("to_id", to_id))
        .await
        .context("delete_relation execute")?;

    if let Some(row) = result.next().await.context("delete_relation next")? {
        let deleted: i64 = row.get("deleted").unwrap_or(0);
        Ok(deleted > 0)
    } else {
        Ok(false)
    }
}

// ── Query helpers for the retrieval layer ─────────────────────────────────────

/// Fetch the ids and labels of all nodes on the OTHER end of a relation
/// starting from a given node.
///
/// Example: all nodes that solve `heat_equation`:
///   outgoing_neighbors(graph, "Equation", "heat_equation", "SOLVES", Direction::Incoming)
pub enum Direction {
    Outgoing,
    Incoming,
}

pub struct NeighborRef {
    pub id: String,
    pub label: String,
    pub name: Option<String>,
}

pub async fn neighbors(
    graph: &Graph,
    node_label: &str,
    node_id: &str,
    rel_type: &str,
    direction: Direction,
) -> Result<Vec<NeighborRef>> {
    let cypher = match direction {
        Direction::Outgoing => format!(
            "MATCH (a:{nl} {{id: $id}})-[:{rt}]->(b) \
             RETURN labels(b)[0] AS label, b.id AS id, b.name AS name",
            nl = node_label, rt = rel_type
        ),
        Direction::Incoming => format!(
            "MATCH (b)-[:{rt}]->(a:{nl} {{id: $id}}) \
             RETURN labels(b)[0] AS label, b.id AS id, b.name AS name",
            nl = node_label, rt = rel_type
        ),
    };

    let mut result = graph
        .execute(neo4rs::query(&cypher).param("id", node_id))
        .await
        .context("neighbors execute")?;

    let mut out = Vec::new();
    while let Some(row) = result.next().await.context("neighbors row")? {
        let id: String = row.get("id").unwrap_or_default();
        let label: String = row.get("label").unwrap_or_default();
        let name: Option<String> = row.get("name").ok().filter(|s: &String| !s.is_empty());
        out.push(NeighborRef { id, label, name });
    }
    Ok(out)
}
