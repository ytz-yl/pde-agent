# PDE Knowledge Base

PDE Agent 系统的知识库服务组件。用 Rust 实现，提供 HTTP REST API，以**知识图谱**的形式存储和查询 PDE 领域知识，涵盖方程、条件、定理、数值方法、AI 模型、损失函数、评估指标、数据集与论文。

## 架构总览

知识库采用**双存储**设计：

```
┌─────────────────────────────────────────────────────┐
│                  HTTP API (axum)                    │
│  /equations  /ai-models  /numerical-methods         │
│  /papers     /search     /internal (write)          │
└────────────────┬────────────────────────────────────┘
                 │
        ┌────────┴────────┐
        │                 │
   ┌────▼─────┐    ┌──────▼──────┐
   │  Neo4j   │    │   SQLite    │
   │  图数据库 │    │  内容数据库  │
   │          │    │             │
   │ 节点与关系 │    │ 长文本摘要  │
   │ 结构化字段 │    │ (abstract, │
   │ 快速遍历  │    │  notes)     │
   └──────────┘    └─────────────┘
```

- **Neo4j**：存储所有节点的结构化字段（id、name、枚举类型等）及节点间的有向关系，支持图遍历查询
- **SQLite**：存储长文本内容（论文摘要、注释），以 `(node_id, node_type)` 为主键，避免在图数据库中存储大文本

## 知识图谱模型

### 节点类型（Neo4j Labels）

| 标签 | 说明 |
|---|---|
| `Equation` | PDE 方程（如 Heat Equation、Navier-Stokes） |
| `Condition` | 数学条件（边界条件、初始条件、域约束、正则性假设） |
| `Theorem` | 数学定理（收敛性、存在性、唯一性等） |
| `NumericalMethod` | 经典数值方法（FDM、FEM、FVM、谱方法） |
| `AIModel` | AI/ML 求解模型（PINNs、DeepONet、FNO、PDEformer） |
| `LossFunction` | 损失 / 目标函数（PDE 残差损失、边界损失、MSE） |
| `Metric` | 评估指标（L2 误差、L∞ 误差、推理时间） |
| `Dataset` | 基准数据集（Burgers 1D、Navier-Stokes 2D、Darcy Flow） |
| `Paper` | 研究论文（arXiv ID / DOI，结构字段存图、摘要存 SQLite） |

### 关系类型（Neo4j Edge Types）

| 关系 | 含义 |
|---|---|
| `SOLVES` | (AIModel\|NumericalMethod) → Equation |
| `REQUIRES` | 节点依赖另一节点 |
| `HAS_CONDITION` | Equation → Condition |
| `APPLIES_TO` | 方法适用于某方程/域 |
| `TRAINED_BY` | AIModel → LossFunction |
| `EVALUATED_BY` | AIModel → Metric |
| `REPRESENTS` | LossFunction → Equation（该损失编码了哪个 PDE） |
| `BASED_ON` | Dataset → Equation |
| `TESTED_ON` | AIModel → Dataset |
| `VARIANT_OF` | 方法/模型是另一个的变体 |
| `PROPOSES` | Paper → (AIModel\|NumericalMethod) |
| `STUDIES` | Paper → Equation |
| `USES_DATASET` | Paper → Dataset |
| `REPORTS_METRIC` | Paper → Metric |
| `CITES` | Paper → Paper |

### 预置 Seed 数据

服务启动时自动 MERGE（幂等）以下初始知识节点：

- **方程**：Heat、Wave、Poisson、Navier-Stokes、Burgers、Schrödinger、Allen-Cahn
- **条件**：Dirichlet BC、Neumann BC、Periodic BC、Zero IC、Bounded Domain、Smooth Coefficients
- **数值方法**：FDM、FEM、FVM、Spectral
- **AI 模型**：PINN、DeepONet、FNO、PDEformer、DeepXDE
- **损失函数**：PDE 残差损失、边界条件损失、数据 MSE 损失、Combined PINN 损失
- **评估指标**：L2 相对误差、L∞ 误差、MSE、推理时间、训练时间、参数量、泛化误差
- **数据集**：Burgers 1D、Navier-Stokes 2D、Heat 2D、Darcy Flow
- **关系**：AIModel/NumericalMethod SOLVES Equation、AIModel TRAINED_BY LossFunction、AIModel EVALUATED_BY Metric、AIModel TESTED_ON Dataset、Equation HAS_CONDITION Condition、Dataset BASED_ON Equation

## 技术选型

| 组件 | 选型 | 原因 |
|---|---|---|
| HTTP 框架 | `axum 0.7` | 类型安全，async-native |
| 图数据库 | `Neo4j`（neo4rs 0.8） | 原生图查询，Cypher 语言，支持复杂关系遍历 |
| 内容存储 | `SQLite`（rusqlite bundled）| 零部署，存储长文本摘要 |
| 序列化 | `serde / serde_json` | JSON API |
| 异步运行时 | `tokio` | 全异步 I/O |

## 目录结构

```
knowledge_base/
├── Cargo.toml
├── Cargo.lock
├── .gitignore
└── src/
    ├── lib.rs                       # crate 根，公开所有子模块
    ├── main.rs                      # 入口：读配置、连接 Neo4j、初始化 SQLite、启动 HTTP
    │
    ├── store/                       # 持久化层
    │   ├── mod.rs                   # 子模块声明
    │   ├── schema.rs                # 核心领域类型：节点结构体、枚举、关系常量
    │   ├── graph.rs                 # Neo4j 连接、Schema 约束初始化、Seed 数据写入
    │   ├── node_repo.rs             # 各类节点的 CRUD（MERGE/GET/LIST），基于 Neo4j
    │   ├── relation_repo.rs         # 关系 CRUD（MERGE/DELETE）及邻居查询工具
    │   └── content_repo.rs          # SQLite 内容库：abstract/notes 的 upsert/get/delete
    │
    ├── retrieval/                   # 检索层
    │   ├── mod.rs
    │   └── query.rs                 # 高层图遍历查询：方程求解器、AI 模型 Profile、论文 Profile、全文搜索
    │
    └── api/                         # HTTP 层
        ├── mod.rs                   # AppState（Neo4j Graph pool + SQLite Mutex）
        ├── routes.rs                # axum 路由注册（公开查询 + 内部写 API）
        └── handlers/
            ├── mod.rs
            ├── query.rs             # 所有只读请求处理函数
            └── write.rs             # 内部写请求处理（upsert/delete 节点与关系）
```

## 快速启动

### 前置依赖

需要运行中的 Neo4j 实例（4.x 或 5.x）。可使用项目根目录的脚本：

```bash
# 在项目根目录
./start-neo4j.sh
```

或直接用 Docker 启动：

```bash
docker run -d \
  --name pde-neo4j \
  -p 7687:7687 -p 7474:7474 \
  -e NEO4J_AUTH=neo4j/password \
  neo4j:5
```

### 环境变量

```bash
# Neo4j 连接（必填）
NEO4J_URI=bolt://localhost:7687     # Bolt 协议 URI
NEO4J_USER=neo4j                    # 用户名
NEO4J_PASSWORD=password             # 密码

# 可选，覆盖默认值
KB_BIND_ADDR=0.0.0.0:3000           # HTTP 监听地址
KB_CONTENT_DB=content.db            # SQLite 内容数据库文件路径
KB_SEED_DATA=true                   # 设为 "false" 可跳过 seed 数据写入
```

### 运行

```bash
cd knowledge_base

# 开发模式
NEO4J_PASSWORD=password cargo run

# 发布模式
cargo build --release
NEO4J_PASSWORD=password ./target/release/knowledge-base
```

服务启动时会自动：
1. 连接 Neo4j 并创建唯一性约束（IF NOT EXISTS，幂等）
2. 写入 Seed 数据（MERGE，幂等）
3. 打开/创建 SQLite 内容数据库并执行 migration
4. 在 `KB_BIND_ADDR` 监听 HTTP 请求

## API 接口

### 健康检查

```
GET /health
```

返回：`{"status": "ok", "service": "pde-knowledge-base"}`

---

### 方程（Equations）

```
GET /equations[?pde_type=parabolic|elliptic|hyperbolic|mixed]
GET /equations/:id
GET /equations/:id/solvers        # 返回所有能求解该方程的 AI 模型 + 数值方法
GET /equations/:id/conditions     # 该方程关联的边界/初始条件
GET /equations/:id/datasets       # 基于该方程的基准数据集
GET /equations/:id/papers         # 研究该方程的论文
```

**示例：**
```bash
curl "http://localhost:3000/equations?pde_type=parabolic"
curl "http://localhost:3000/equations/navier_stokes/solvers"
```

---

### AI 模型（AI Models）

```
GET /ai-models[?training_type=supervised|physics_informed|operator_learning|...]
GET /ai-models/:id
GET /ai-models/:id/profile        # 完整 Profile：求解方程、训练损失、评估指标、测试数据集
GET /ai-models/:id/equations      # 该模型能求解的方程列表
GET /ai-models/:id/papers         # 提出该模型的论文
```

**示例：**
```bash
curl "http://localhost:3000/ai-models?training_type=physics_informed"
curl "http://localhost:3000/ai-models/fno/profile"
```

---

### 数值方法（Numerical Methods）

```
GET /numerical-methods
GET /numerical-methods/:id
GET /numerical-methods/:id/papers  # 提出该方法的论文
```

---

### 论文（Papers）

```
GET /papers[?year=2024]
GET /papers/:id                    # 节点字段 + SQLite 中的摘要/注释
GET /papers/:id/profile            # 完整 Profile：proposes/studies/uses/cites/cited_by
```

---

### 全文搜索

```
GET /search?q=<query>
```

对所有节点的 `name` 字段进行正则模式匹配（大小写不敏感），返回最多 50 条结果，包含节点类型、id、name 和 description。

**示例：**
```bash
curl "http://localhost:3000/search?q=neural+operator"
```

---

### 内部写 API（Internal Write）

> 供知识构建 Agent 调用，不对外暴露（建议通过网络策略限制访问）。

#### 创建/更新节点

```
POST /internal/nodes
Content-Type: application/json

{
  "node_type": "equation",        // 见 NodeType 枚举
  "equation": {                   // 对应 node_type 的节点数据
    "id": "kdv",
    "name": "Korteweg-de Vries Equation",
    "pde_type": "hyperbolic",
    "variables": ["t", "x"],
    "time_dependent": true,
    "description": "Nonlinear dispersive PDE. du/dt + 6u du/dx + d3u/dx3 = 0.",
    "tags": ["soliton", "dispersive"]
  }
}
```

`node_type` 支持：`equation` / `condition` / `theorem` / `numerical_method` / `ai_model` / `loss_function` / `metric` / `dataset` / `paper`。

#### 删除节点

```
DELETE /internal/nodes/:label/:id
```

使用 `DETACH DELETE`，同时删除所有关联边。

#### 创建/更新关系

```
POST /internal/relations
Content-Type: application/json

{
  "from_id": "pinn",
  "from_label": "AIModel",
  "to_id": "kdv",
  "to_label": "Equation",
  "relation_type": "SOLVES",
  "properties": null
}
```

`relation_type` 必须是已定义的合法关系类型之一（见 `relation_repo.rs` 中的 `VALID_RELATION_TYPES`）。

#### 删除关系

```
DELETE /internal/relations
Content-Type: application/json

{
  "from_id": "pinn",
  "from_label": "AIModel",
  "to_id": "kdv",
  "to_label": "Equation",
  "relation_type": "SOLVES"
}
```

## 数据存储说明

### Neo4j 约束

服务启动时自动为所有 9 种节点类型创建 `id` 字段的唯一性约束：

```cypher
CREATE CONSTRAINT equation_id IF NOT EXISTS FOR (n:Equation) REQUIRE n.id IS UNIQUE
-- ... 其余 8 种类型同理
```

### SQLite 内容表

```sql
CREATE TABLE node_content (
    node_id    TEXT NOT NULL,
    node_type  TEXT NOT NULL,
    abstract   TEXT,            -- 论文摘要
    notes      TEXT,            -- 自由格式注释
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ','now')),
    PRIMARY KEY (node_id, node_type)
);
```

## 演进路径

当前方案以 Neo4j 为核心图存储，已具备生产级图查询能力。未来可按需扩展：

| 方向 | 升级方案 |
|---|---|
| 语义搜索 | 为节点 embedding 增加向量索引（Neo4j Vector Index 或外接 Qdrant） |
| 论文自动摄入 | 增加 ingestion 模块（arXiv fetcher + LLM 分类器），通过内部写 API 写入 |
| 大规模内容存储 | SQLite → PostgreSQL（`sqlx` 提供统一接口，迁移成本低） |
| 访问控制 | 为 `/internal` 路由添加 API Key 鉴权中间件 |
