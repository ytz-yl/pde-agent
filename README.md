# PDE Agent System

A service infrastructure for AI agents to solve and research Partial Differential Equation (PDE) problems. Rather than building another agent framework, this project provides domain-specific PDE services that any agent can leverage.

---

## Motivation

Existing agent frameworks (LangChain, AutoGen, CrewAI, etc.) are mature enough. The gap is not in orchestration — it is in domain capability. When an agent needs to solve a PDE problem or conduct PDE algorithm research, it lacks:

- Structured, up-to-date knowledge of PDE theory and methods
- Reliable numerical solvers for diverse PDE scenarios
- Clear guidance on which method to use and how to invoke it

This project fills that gap.

---

## System Overview

```
┌─────────────────────────────────────────────────────────┐
│                      Agent (external)                   │
│          LangChain / AutoGen / CrewAI / custom          │
└────────────────────┬────────────────────────────────────┘
                     │  uses
         ┌───────────▼───────────┐
         │    PDE Agent Services │
         │  ┌─────────────────┐  │
         │  │  Knowledge Base │  │
         │  └─────────────────┘  │
         │  ┌─────────────────┐  │
         │  │   PDE Solvers   │  │
         │  └─────────────────┘  │
         │  ┌─────────────────┐  │
         │  │   Skill Tree    │  │
         │  └─────────────────┘  │
         └───────────────────────┘
                     │
         ┌───────────▼───────────┐
         │     Frontend UI       │
         └───────────────────────┘
```

The system has three core components and a frontend layer.

---

## Component 1: PDE Knowledge Base

A graph-structured knowledge base built on Neo4j, covering PDE theory, numerical methods, AI models, and research literature as an interconnected knowledge graph.

### Knowledge graph model

The knowledge base represents PDE domain knowledge as a property graph with nine node types and fifteen relation types:

**Node types**: `Equation`, `Condition`, `Theorem`, `NumericalMethod`, `AIModel`, `LossFunction`, `Metric`, `Dataset`, `Paper`

**Key relation types**:
- `SOLVES` — links solvers (AI models or numerical methods) to equations they handle
- `HAS_CONDITION` — links equations to their boundary/initial conditions
- `TRAINED_BY` — links AI models to their loss functions
- `EVALUATED_BY` / `TESTED_ON` — links AI models to metrics and benchmark datasets
- `PROPOSES` / `STUDIES` / `CITES` — links papers to the knowledge they contribute

### Storage design

```
Neo4j (graph database)
  └── structural fields: id, name, enums, relationships
      fast graph traversal, Cypher queries

SQLite (content store)
  └── long-form text: paper abstracts, notes
      keyed by (node_id, node_type)
```

### Pre-seeded knowledge

The service ships with seed data covering: Heat / Wave / Poisson / Navier-Stokes / Burgers / Schrödinger / Allen-Cahn equations; Dirichlet / Neumann / Periodic boundary conditions; FDM / FEM / FVM / Spectral methods; PINN / DeepONet / FNO / PDEformer / DeepXDE models; standard loss functions, evaluation metrics, and benchmark datasets — with all relationships pre-wired.

### API surface

- Read endpoints: browse equations, AI models, numerical methods, papers; traverse graph relationships (solvers for an equation, full model profile, paper citation graph)
- Write endpoints (`/internal/`): upsert and delete nodes and relations, for use by knowledge-building agents
- Search: name-based search across all node types

---

## Component 2: PDE Solver Service

A curated library of state-of-the-art PDE solvers, exposed as callable services. Agents do not need to implement solvers themselves — they call this service.

### Scope of solvers

| Category | Examples |
|---|---|
| Classical numerical | FDM (explicit/implicit), FEM (linear/nonlinear), FVM, spectral |
| Modern ML-based | PINNs, DeepONet, Fourier Neural Operator (FNO) |
| Hybrid | Physics-constrained neural networks, adaptive mesh refinement + ML |
| Specialized | Stokes / Navier-Stokes, Schrödinger, Maxwell, elasticity, etc. |

### Service design

Each solver is wrapped as a service with a uniform interface:

```
Input:
  - PDE specification (equation, domain, boundary/initial conditions, parameters)
  - Solver config (method, resolution, tolerance, device)

Output:
  - Solution field (numerical or analytical approximation)
  - Metadata (convergence info, runtime, estimated error)
  - Visualization-ready data
```

Solvers are versioned and documented. New state-of-the-art methods are added as the field advances.

---

## Component 3: Agent Skill

A structured skill specification that tells agents **what services exist**, **when to use them**, and **how to call them**.

Located at [`frontend/skills/pde-skill/`](./frontend/skills/pde-skill/), the skill consists of:

- **[`SKILL.md`](./frontend/skills/pde-skill/SKILL.md)**: maps user intent to specific API endpoints, defines recommended call sequences for common scenarios
- **[`solve-api.md`](./frontend/skills/pde-skill/solve-api.md)**: detailed guide for constructing `POST /solve` requests — equation syntax, initial condition format, response parsing
- **[`knowledge-api.md`](./frontend/skills/pde-skill/knowledge-api.md)**: guide for all knowledge base endpoints — query parameters, filter usage, `constraints` keyword reference for `/recommend`

Skills are also browsable and downloadable directly from the frontend UI under the **Skills** tab.

---

## Component 4: Frontend

A single-page React + TypeScript application that makes the knowledge base and solver library visible and explorable. Supports **English / Chinese** language switching.

### Knowledge Base UI

- Semantic and full-text search over papers and methods
- Paper list with score-ranked results, abstract preview, tags, and source links
- Method browser: list, detail, related methods, side-by-side comparison
- Method recommendation form powered by `POST /recommend`

### Solver UI

- Catalog of available solvers with supported PDE types
- Interactive form to build and submit a `POST /solve` request
- Result visualization: heatmap of the solution field at each time snapshot
- Metadata display: solver used, wall time, backend info

### Skills UI

- File-tree browser for all skill packages under `frontend/skills/`
- Rendered Markdown preview with GitHub-Flavored Markdown support
- One-click download of any skill package as a `.zip` archive

---

## Use Cases

### PDE problem solving
An agent receives a problem: *"Simulate 2D heat diffusion on an irregular domain with Dirichlet boundary conditions."* It queries the skill tree, selects FEM, calls the solver service, and returns the solution with error estimates.

### PDE research assistance
An agent is asked: *"What are the most effective methods for solving the Navier-Stokes equations at high Reynolds number?"* It queries the knowledge base for recent papers, compares methods, and synthesizes a structured response with citations.

### Method benchmarking
An agent benchmarks PINNs vs FNO on a specific problem by calling `benchmark_methods`, pulling results from the solver service, and retrieving relevant comparison studies from the knowledge base.

### Algorithm research
An agent proposes improvements to an existing solver by combining knowledge retrieval (what has been tried, what are known failure modes) with solver benchmarking (empirical validation of the proposed change).

---

## Project Structure

```
pde-agent/
├── knowledge_base/          # PDE knowledge graph service (Rust)
│   └── src/
│       ├── store/           # Neo4j node/relation CRUD, SQLite content store
│       ├── retrieval/       # high-level graph traversal queries
│       └── api/             # axum HTTP handlers and routes
├── engines/                 # PDE solver implementations and wrappers
│   ├── classical/           # FDM, FEM, FVM, spectral
│   ├── ml/                  # PINNs, DeepONet, FNO
│   └── api/                 # solver service endpoints
├── frontend/                # web UI (React + TypeScript)
│   ├── skills/              # agent skill packages
│   │   └── pde-skill/       # SKILL.md + sub-guides
│   └── src/                 # application source
├── start-neo4j.sh           # starts the Neo4j instance
├── start.sh                 # one-command launcher
└── README.md
```

---

## Design Principles

- **Service-first, not framework-first**: this project provides services, not an agent runtime. Any agent framework can consume these services via API.
- **Structured knowledge over raw retrieval**: papers are not just indexed — they are classified, summarized, and linked into a coherent knowledge structure.
- **Method diversity**: no single solver paradigm is privileged. Classical numerical methods and modern ML-based methods coexist.
- **Transparency**: the solver UI shows concrete implementations; the knowledge base shows source papers. Users and agents can inspect what is actually happening.
- **Incremental growth**: new solvers and papers are added continuously without breaking existing interfaces.
