-- PDE Knowledge Base - Initial Schema
-- Migration: 001_initial.sql

PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ============================================================
-- Papers table
-- ============================================================
CREATE TABLE IF NOT EXISTS papers (
    id           TEXT PRIMARY KEY,          -- arXiv ID (e.g. "2301.12345") or DOI
    title        TEXT NOT NULL,
    abstract     TEXT,
    authors      TEXT NOT NULL DEFAULT '[]', -- JSON array of author name strings
    published    TEXT,                       -- ISO-8601 date string
    source_url   TEXT,                       -- link to abstract page
    pdf_url      TEXT,
    -- embedding stored as raw little-endian float32 bytes
    embedding    BLOB,
    created_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at   TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- Full-text search on title + abstract
CREATE VIRTUAL TABLE IF NOT EXISTS papers_fts USING fts5(
    title,
    abstract,
    content='papers',
    content_rowid='rowid'
);

-- Keep FTS in sync
CREATE TRIGGER IF NOT EXISTS papers_fts_insert AFTER INSERT ON papers BEGIN
    INSERT INTO papers_fts(rowid, title, abstract)
    VALUES (new.rowid, new.title, new.abstract);
END;

CREATE TRIGGER IF NOT EXISTS papers_fts_delete AFTER DELETE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, abstract)
    VALUES ('delete', old.rowid, old.title, old.abstract);
END;

CREATE TRIGGER IF NOT EXISTS papers_fts_update AFTER UPDATE ON papers BEGIN
    INSERT INTO papers_fts(papers_fts, rowid, title, abstract)
    VALUES ('delete', old.rowid, old.title, old.abstract);
    INSERT INTO papers_fts(rowid, title, abstract)
    VALUES (new.rowid, new.title, new.abstract);
END;

-- ============================================================
-- Tags table  (multi-value labels per paper)
-- tag_type: "pde_type" | "method" | "domain" | "benchmark"
-- ============================================================
CREATE TABLE IF NOT EXISTS paper_tags (
    paper_id    TEXT NOT NULL REFERENCES papers(id) ON DELETE CASCADE,
    tag_type    TEXT NOT NULL,
    tag_value   TEXT NOT NULL,
    PRIMARY KEY (paper_id, tag_type, tag_value)
);

CREATE INDEX IF NOT EXISTS idx_paper_tags_type_value
    ON paper_tags (tag_type, tag_value);

-- ============================================================
-- PDE Methods table
-- ============================================================
CREATE TABLE IF NOT EXISTS methods (
    id          TEXT PRIMARY KEY,           -- e.g. "fem", "fno", "deeponet"
    name        TEXT NOT NULL,
    category    TEXT NOT NULL,              -- "classical" | "ml" | "hybrid"
    description TEXT,
    embedding   BLOB,
    tags        TEXT NOT NULL DEFAULT '[]', -- JSON array of tag strings
    created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- ============================================================
-- Method relations table (lightweight knowledge graph)
-- relation: "extends" | "competes_with" | "combines_with" | "requires"
-- ============================================================
CREATE TABLE IF NOT EXISTS method_relations (
    from_method TEXT NOT NULL REFERENCES methods(id) ON DELETE CASCADE,
    to_method   TEXT NOT NULL REFERENCES methods(id) ON DELETE CASCADE,
    relation    TEXT NOT NULL,
    weight      REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (from_method, to_method, relation)
);

CREATE INDEX IF NOT EXISTS idx_method_relations_from
    ON method_relations (from_method);
CREATE INDEX IF NOT EXISTS idx_method_relations_to
    ON method_relations (to_method);

-- ============================================================
-- Paper <-> Method links
-- ============================================================
CREATE TABLE IF NOT EXISTS paper_methods (
    paper_id    TEXT NOT NULL REFERENCES papers(id) ON DELETE CASCADE,
    method_id   TEXT NOT NULL REFERENCES methods(id) ON DELETE CASCADE,
    role        TEXT NOT NULL DEFAULT 'proposes', -- "proposes" | "evaluates" | "uses"
    PRIMARY KEY (paper_id, method_id, role)
);

-- ============================================================
-- Seed data: core PDE methods
-- ============================================================
INSERT OR IGNORE INTO methods (id, name, category, description, tags) VALUES
    ('fdm',      'Finite Difference Method',         'classical', 'Approximates derivatives by finite differences on structured grids. Simple to implement, best for regular domains.', '["grid","structured","explicit","implicit"]'),
    ('fem',      'Finite Element Method',            'classical', 'Variational formulation on unstructured meshes. Handles complex geometries and adaptive refinement well.', '["mesh","unstructured","variational","adaptive"]'),
    ('fvm',      'Finite Volume Method',             'classical', 'Integral form of conservation laws on control volumes. Widely used in CFD (fluids, heat transfer).', '["conservation","cfd","unstructured"]'),
    ('spectral', 'Spectral Methods',                 'classical', 'Global basis functions (Fourier, Chebyshev). Exponential convergence for smooth solutions on simple domains.', '["fourier","chebyshev","high-order","smooth"]'),
    ('pinns',    'Physics-Informed Neural Networks', 'ml',        'Embeds PDE residuals into the loss function of a neural network. Mesh-free, good for inverse problems.', '["mesh-free","inverse","neural-network","collocation"]'),
    ('deeponet', 'Deep Operator Network',            'ml',        'Learns mappings between function spaces. Efficient for parametric PDEs where the operator is fixed.', '["operator-learning","parametric","neural-network"]'),
    ('fno',      'Fourier Neural Operator',          'ml',        'Learns solution operators in Fourier space. Fast inference, handles varying discretisations.', '["operator-learning","fourier","fast","resolution-invariant"]'),
    ('pdeformer','PDEformer',                        'ml',        'Transformer-based universal PDE solver using symbolic DAG representation. Supports diverse PDE families.', '["transformer","symbolic","dag","universal"]');

INSERT OR IGNORE INTO method_relations (from_method, to_method, relation, weight) VALUES
    ('fno',      'deeponet', 'competes_with', 1.0),
    ('fno',      'pinns',    'competes_with', 0.8),
    ('deeponet', 'pinns',    'competes_with', 0.8),
    ('pdeformer','fno',      'extends',       1.0),
    ('pdeformer','deeponet', 'extends',       0.9),
    ('fem',      'fdm',      'extends',       0.7),
    ('pinns',    'fdm',      'competes_with', 0.6),
    ('fvm',      'fdm',      'extends',       0.6);
