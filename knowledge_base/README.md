# PDE Knowledge Base

PDE Agent 系统的知识库服务组件。用 Rust 实现，提供 HTTP REST API，支持 PDE 论文的语义搜索、结构化查询、自动摄入和方法推荐。

## 功能概览

| 功能 | 说明 |
|---|---|
| **语义搜索** | 自然语言查询，向量相似度 + FTS5 混合排序 |
| **结构化查询** | 按 PDE 类型、方法、领域、基准数据集过滤 |
| **论文摄入** | 手动录入或从 arXiv 批量拉取，LLM 自动打标签 |
| **方法推荐** | 给定 PDE 类型和约束，推荐最适合的求解方法 |
| **方法对比** | 查询两种方法之间的已知关系（扩展、竞争等） |

## 技术选型

| 组件 | 选型 | 原因 |
|---|---|---|
| HTTP 框架 | `axum 0.7` | 类型安全，async-native |
| 关系存储 | `SQLite`（rusqlite bundled）| 零部署，支持 FTS5 全文搜索 |
| 向量索引 | `usearch`（HNSW） | 纯进程内，余弦相似度 ANN |
| 嵌入 / 分类 | OpenAI 兼容 API | 支持 OpenAI 或本地 Ollama |
| XML 解析 | `quick-xml` | arXiv Atom feed 解析 |

## 目录结构

```
knowledge_base/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
├── migrations/
│   └── 001_initial.sql        # 建表 DDL + FTS5 触发器 + 8 种 PDE 方法 seed 数据
└── src/
    ├── lib.rs                 # crate 根，公开所有子模块
    ├── main.rs                # 二进制入口：读配置、初始化存储、启动 HTTP 服务
    │
    ├── store/                 # 持久化层
    │   ├── mod.rs             # open_db / open_db_in_memory（含自动 migration）
    │   ├── schema.rs          # 共享类型：Paper, Method, TagType, RelationKind
    │   │                      # 及 embedding blob 序列化工具函数
    │   ├── paper_repo.rs      # 论文 CRUD、FTS5 搜索、tag 过滤
    │   ├── method_repo.rs     # 方法 CRUD、关系图查询
    │   └── vector_index.rs    # usearch HNSW 封装（线程安全、磁盘持久化）
    │
    ├── ingestion/             # 数据摄入层
    │   ├── mod.rs
    │   ├── arxiv_fetcher.rs   # 调用 arXiv API，解析 Atom/XML，返回 Paper 列表
    │   ├── classifier.rs      # 调用 LLM：生成 embedding、提取 PDE 标签
    │   └── pipeline.rs        # 摄入流水线：fetch → classify → embed → upsert
    │
    ├── retrieval/             # 检索层
    │   ├── mod.rs
    │   ├── semantic.rs        # 向量 + FTS5 混合搜索
    │   ├── structured.rs      # 纯 SQL 结构化查询（按标签、日期等）
    │   └── recommender.rs     # 基于规则的方法推荐、方法对比报告
    │
    └── api/                   # HTTP 层
        ├── mod.rs             # AppState（DB、向量索引、HTTP 客户端、LLM 配置）
        ├── routes.rs          # axum 路由注册表
        └── handlers.rs        # 所有请求处理函数
```

## 快速启动

### 环境变量

```bash
# 必填（如需语义搜索和论文摄入）
OPENAI_API_KEY=sk-...

# 可选，覆盖默认值
OPENAI_API_BASE=https://api.openai.com/v1   # 也可以填 Ollama 地址
EMBEDDING_MODEL=text-embedding-3-small
CHAT_MODEL=gpt-4o-mini
KB_DB_PATH=knowledge_base.db               # SQLite 文件路径
KB_INDEX_PATH=vector_index.bin             # HNSW 索引文件路径
KB_BIND_ADDR=0.0.0.0:3000                  # HTTP 监听地址
```

### 运行

```bash
cd knowledge_base

# 开发模式
OPENAI_API_KEY=sk-... cargo run

# 发布模式
cargo build --release
OPENAI_API_KEY=sk-... ./target/release/knowledge-base
```

服务启动后会自动执行数据库 migration 并重建向量索引（从 SQLite 中已有的 embedding 重载）。

### 使用本地 Ollama

```bash
OPENAI_API_BASE=http://localhost:11434/v1 \
OPENAI_API_KEY=ollama \
EMBEDDING_MODEL=nomic-embed-text \
CHAT_MODEL=llama3 \
cargo run
```

## API 接口

### 搜索

```
GET /search?q=<query>[&pde_type=<tag>][&method=<tag>][&domain=<tag>][&limit=10]
```

混合搜索（向量 + FTS5），可叠加结构化过滤条件。返回按相关度降序排列的论文列表。

**示例：**
```bash
curl "http://localhost:3000/search?q=neural+operator+for+fluid+dynamics&domain=fluid_dynamics&limit=5"
```

### 最新论文

```
GET /papers/recent[?domain=<tag>][&limit=20]
```

### 论文详情

```
GET /papers/<id>
```

其中 `id` 为 arXiv ID（如 `2301.12345`）或 DOI。

### 方法列表

```
GET /methods[?category=classical|ml|hybrid]
```

### 方法详情 / 关联方法 / 方法对比

```
GET /methods/<id>
GET /methods/<id>/related
GET /methods/compare?a=<id_a>&b=<id_b>
```

### 方法推荐

```
POST /recommend
Content-Type: application/json

{
  "pde_type": "navier_stokes",
  "domain": "fluid_dynamics",
  "constraints": ["irregular_domain", "fast_inference"],
  "top_k": 3
}
```

`constraints` 支持的关键词：`irregular_domain`、`inverse_problem`、`high_dimensional`、`parametric`、`fast_inference`、`high_accuracy`、`complex_geometry`、`conservation_laws`、`guaranteed_convergence`。

### 手动摄入论文

```
POST /ingest/paper
Content-Type: application/json

{
  "id": "2301.12345",
  "title": "...",
  "abstract_text": "...",
  "authors": ["Alice", "Bob"],
  "published": "2023-01-01T00:00:00Z",
  "source_url": "https://arxiv.org/abs/2301.12345",
  "pdf_url": "https://arxiv.org/pdf/2301.12345"
}
```

### 从 arXiv 批量拉取

```
POST /ingest/fetch-arxiv
Content-Type: application/json

{
  "query": "fourier neural operator partial differential equations",
  "max_results": 25
}
```

### 健康检查

```
GET /health
```

## 数据库 Schema

核心表：

| 表名 | 说明 |
|---|---|
| `papers` | 论文主表，含 embedding BLOB |
| `papers_fts` | FTS5 虚拟表（title + abstract），自动与 `papers` 同步 |
| `paper_tags` | 论文标签（pde_type / method / domain / benchmark） |
| `methods` | PDE 方法条目，预置 8 种方法 |
| `method_relations` | 方法间关系（extends / competes_with / combines_with） |
| `paper_methods` | 论文与方法的多对多关联 |

预置方法：`fdm`, `fem`, `fvm`, `spectral`, `pinns`, `deeponet`, `fno`, `pdeformer`。

## 向量索引说明

- 使用 HNSW 算法（usearch crate），余弦相似度，维度 1536（对应 `text-embedding-3-small`）
- 索引持久化到 `vector_index.bin`（原子写入，先写临时文件再 rename）
- 重启时从 SQLite 中已有 embedding 自动重建 key 映射（usearch 只持久化向量，不持久化 string key）
- 如需更换 embedding 模型（维度变化），需修改 `store/schema.rs` 中的 `EMBEDDING_DIM` 常量并重建索引

## 演进路径

当前为小规模起步方案，可按需升级：

| 阶段 | 存储升级 |
|---|---|
| 当前（<1 万篇） | SQLite + usearch，单文件，零部署 |
| 中等规模（1–10 万篇） | SQLite → PostgreSQL（换 `sqlx` driver），usearch → Qdrant |
| 大规模（>10 万篇） | 加消息队列（异步摄入），引入 Neo4j（真正的知识图谱） |

SQLite 到 PostgreSQL 的迁移成本低：`sqlx` 对两者提供统一接口，主要改动在 `store/mod.rs` 和各 repo 文件的 SQL 语法。
