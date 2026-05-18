# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## System Overview

PDE Agent is a domain-specific service infrastructure for AI agents to solve and research Partial Differential Equation problems. It is **not** an agent framework — it provides callable services that any agent framework can consume.

Three backend services plus a React frontend:

| Service | Port | Language | Directory |
|---|---|---|---|
| Knowledge Base | 3001 | Rust + Neo4j + SQLite | `knowledge_base/` |
| Solver API | 3000 (start.sh) / 8080 (standalone) | Rust + Python subprocesses | `engines/api/` |
| Frontend | 5173 | React + TypeScript (Vite) | `frontend/` |

Neo4j must be running before the Knowledge Base starts. Start it with `./start-neo4j.sh`.

## Commands

### Run everything
```bash
./start.sh            # build + start all three services, tail logs
./start.sh --stop     # stop all
./start.sh --logs     # tail logs only
```

### Knowledge Base (Rust)
```bash
cd knowledge_base
NEO4J_PASSWORD=password cargo run                 # dev mode, port 3001
cargo build --release                              # production build
cargo test                                         # run tests
```

### Solver API (Rust)
```bash
cd engines/api
cargo run                                          # dev mode, port 8080
cargo build --release
cargo test
```

### Frontend
```bash
cd frontend
npm install
npm run dev        # Vite dev server, port 5173 (proxies /api/* to backends)
npm run build      # tsc + vite build
npm run lint       # eslint
```

## Environment Variables

### Knowledge Base
| Variable | Default | Description |
|---|---|---|
| `NEO4J_URI` | `bolt://localhost:7687` | Neo4j Bolt URI |
| `NEO4J_USER` | `neo4j` | |
| `NEO4J_PASSWORD` | `password` | |
| `KB_BIND_ADDR` | `0.0.0.0:3000` | HTTP listen address |
| `KB_CONTENT_DB` | `content.db` | SQLite path |
| `KB_SEED_DATA` | `true` | Set `"false"` to skip seeding |

### Solver API
| Variable | Default | Description |
|---|---|---|
| `LISTEN_ADDR` | `0.0.0.0:8080` | HTTP listen address |
| `PDEFORMER2_DIR` | `../ml/pdeformer-2` (relative to `engines/api/`) | PDEformer-2 repo root |
| `PDEFORMER2_PYTHON` | `$HOME/miniconda3/envs/pdeformer2/bin/python` | Python for ML solver |
| `CLASSICAL_PYTHON` | `$HOME/miniconda3/envs/pdeformer2/bin/python` | Python for classical solver |
| `SOLVER_UPLOAD_DIR` | `/tmp/pde-solver-uploads/` | Uploaded tensor file storage |

## Architecture

### Knowledge Base (`knowledge_base/src/`)

Dual-storage design:
- **Neo4j**: structural fields (id, name, enums), graph relationships — fast traversal via Cypher
- **SQLite** (`content.db`): long-form text (paper abstracts, notes) keyed by `(node_id, node_type)`

Layer structure:
- `store/schema.rs` — domain types: 9 node structs, 15 relation type constants
- `store/graph.rs` — Neo4j connect, schema constraints, seed data (idempotent MERGE)
- `store/node_repo.rs` — node CRUD
- `store/relation_repo.rs` — relation CRUD + neighbor queries
- `store/content_repo.rs` — SQLite abstract/notes upsert/get
- `retrieval/query.rs` — high-level graph traversal (equation solvers, model profiles, paper profiles, search)
- `api/routes.rs` — axum route registration (public query + internal write)
- `api/handlers/query.rs` / `write.rs` — HTTP handlers

Public routes are read-only. Write routes live under `/internal/` (no auth — network-level restriction recommended for production).

### Solver API (`engines/api/src/`)

Rust HTTP layer dispatches to Python subprocesses:
```
POST /solve → SolverRegistry.get(solver_id) → spawn Python subprocess
  stdin: JSON SolveRequest → Python bridge script → stdout: JSON SolveResponse
```

- `engines/mod.rs` — `Solver` trait + `SolverRegistry` (build once at startup)
- `engines/pdeformer2.rs` — ML backend; spawns `scripts/pdeformer2_infer.py` in `pdeformer2` conda env
- `engines/classical.rs` — classical backend; spawns `scripts/classical_solve.py` in `classical-pde` conda env
- `models/mod.rs` — all request/response types (`SolveRequest`, `PdeSpec`, `SolveResponse`)

The `SolveRequest.pde` field supports both legacy single-variable syntax and multi-variable/multi-equation systems. See `models/mod.rs` for the full `PdeSpec` documentation.

`POST /files` accepts multipart upload (field name must be `file`) and returns a `file_id` for use in `pde.history.file_id`.

### Frontend (`frontend/src/`)

React + React Router v6. Routes: `/` (Home), `/knowledge` (Knowledge), `/solvers` (Solver), `/skills` (Skills).

- `lib/api.ts` — typed API clients for both backends (`knowledgeApi`, `solverApi`, `skillsApi`)
- `i18n/context.tsx` + `i18n/translations.ts` — EN/ZH toggle via `useI18n()` hook; `I18nProvider` wraps the app
- `components/ui/` — shadcn-style components (Badge, Button, Card, Input, Select, Spinner, Textarea)

The Vite dev server proxies:
- `/api/knowledge/*` → `http://localhost:3001` (strip prefix)
- `/api/engines/*` → `http://localhost:3000` (strip prefix)
- `/api/skills/*` → served by `frontend/plugins/skillsPlugin.ts` directly (reads `skills/` directory)

### Skills (`skills/` + `frontend/skills/`)

Agent skill packages live under `skills/` (knowledge ingestion skill) and `frontend/skills/` (pde-skill for solving). Each package is a directory of Markdown files. The Skills UI in the frontend lets users browse and download them as `.zip` archives.

`skills/pde-ingest-skill/` documents how an agent should write to the knowledge base graph. Key references:
- `SKILL.md` — full ingestion workflow (discover → extract → map → dedup → write → verify)
- `ingest-api.md` — complete write API reference with node/relation schemas and pre-seeded node IDs

## Adding a New Solver Backend

1. Add the solver library as a git submodule under `engines/classical/` or `engines/ml/`
2. Create a bridge script at `engines/api/scripts/<name>.py` (reads JSON from stdin, writes JSON to stdout)
3. Create `engines/api/src/engines/<name>.rs` implementing the `Solver` trait
4. Register it in `SolverRegistry::new()` in `engines/api/src/engines/mod.rs`

## Known Gotchas

**Knowledge Base:**
- `Paper` nodes require `"authors": [...]` — missing this field returns 500, not 400
- Health check: `GET /health` returns `{"service":"pde-knowledge-base","status":"ok"}`; root `/` returns 404
- `GET /equations/:id/solvers` returns `{ ai_models: [...], numerical_methods: [...] }` — both types in one response
- `Dataset → Equation` relationship direction is `BASED_ON` with `from=Dataset, to=Equation`
- Unknown `training_type` or `method_type` values silently fall back to `supervised`/`other` — to add new enum variants, edit `store/schema.rs` and recompile
- All writes use MERGE semantics (idempotent); safe to repeat

**Solver API:**
- `POST /files` multipart field name must be `"file"` (400 otherwise)
- When `pde.history` is present, `initial_condition`/`initial_conditions` are ignored
- The `classical` solver only uses the first variable channel (`arr[-1, :, :, 0]`) from multi-variable history files
- `cargo build` may show spurious "Rust 2015" linter warnings from patch tools; `Cargo.toml` correctly declares `edition = "2021"` and builds fine
