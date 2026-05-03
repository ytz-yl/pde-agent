# 知识库 API 调用技巧

端点基础路径：知识库服务（默认端口 3001）

知识库是一个图数据库（Neo4j + SQLite），每次查询的起点是**节点 id**，查询通过图关系在节点之间跳转。

---

## 端点速览

| 端点 | 方法 | 功能 |
|---|---|---|
| `/health` | GET | 服务健康检查 |
| `/search?q=` | GET | 按名称跨全节点类型搜索 |
| `/equations` | GET | 列出方程（可按 `pde_type` 过滤） |
| `/equations/:id` | GET | 获取单个方程详情 |
| `/equations/:id/solvers` | GET | 该方程关联的 AI 模型 + 数值方法 |
| `/equations/:id/conditions` | GET | 该方程的边界/初始条件 |
| `/equations/:id/datasets` | GET | 该方程的基准数据集 |
| `/equations/:id/papers` | GET | 研究该方程的论文 |
| `/ai-models` | GET | 列出 AI 模型（可按 `training_type` 过滤） |
| `/ai-models/:id` | GET | 获取单个 AI 模型详情 |
| `/ai-models/:id/profile` | GET | AI 模型完整图谱（多跳） |
| `/ai-models/:id/equations` | GET | 该模型能求解的方程列表 |
| `/ai-models/:id/papers` | GET | 提出该模型的论文 |
| `/numerical-methods` | GET | 列出所有数值方法 |
| `/numerical-methods/:id` | GET | 获取单个数值方法详情 |
| `/numerical-methods/:id/papers` | GET | 提出该数值方法的论文 |
| `/papers` | GET | 列出论文（可按 `year` 过滤） |
| `/papers/:id` | GET | 论文基本信息 + 摘要 |
| `/papers/:id/profile` | GET | 论文完整图谱（多跳） |

---

## `/search` — 跨节点名称搜索

```
GET /search?q=<query>
```

对**所有节点类型**（Equation、AIModel、NumericalMethod、Paper 等）做正则名称匹配，返回最多 50 条结果。

**最适用场景**：不确定某个概念属于哪个节点类型时，用搜索找到节点 id，再进一步图遍历。

响应格式：
```json
[
  {
    "node_type": "ai_model",
    "id": "fno",
    "name": "Fourier Neural Operator"
  },
  {
    "node_type": "paper",
    "id": "2010.08895",
    "name": "Fourier Neural Operator for Parametric PDEs"
  }
]
```

**技巧**：搜索词越具体越好，如 `"fourier neural operator"` 优于 `"neural network"`。

---

## `/equations` — 方程

### 列出方程

```
GET /equations?pde_type=<type>
```

| `pde_type` 值 | 含义 |
|---|---|
| `parabolic` | 抛物型（如热方程、扩散方程） |
| `elliptic` | 椭圆型（如泊松方程、拉普拉斯方程） |
| `hyperbolic` | 双曲型（如波动方程、对流方程） |
| `mixed` | 混合型 |
| 省略 | 返回所有方程 |

### 预置方程 id 速查

知识库内置以下方程节点，可直接用 id 查询：

| id | 方程名 | pde_type |
|---|---|---|
| `heat_equation` | 热方程 | parabolic |
| `wave_equation` | 波动方程 | hyperbolic |
| `poisson_equation` | 泊松方程 | elliptic |
| `navier_stokes` | Navier-Stokes 方程 | mixed |
| `burgers_equation` | Burgers 方程 | hyperbolic |
| `schrodinger_equation` | 薛定谔方程 | parabolic |
| `allen_cahn` | Allen-Cahn 方程 | parabolic |

### 响应格式（单个方程）

```json
{
  "id": "heat_equation",
  "name": "Heat Equation",
  "pde_type": "parabolic",
  "variables": ["t", "x", "y"],
  "time_dependent": true,
  "operator": "laplacian",
  "description": "...",
  "tags": ["diffusion", "heat"]
}
```

### `/equations/:id/solvers` — 查询方程的求解器

这是**最核心的图遍历端点**，返回通过 `SOLVES` 关系连接到该方程的所有 AI 模型和数值方法：

```json
{
  "ai_models": [
    {
      "id": "fno",
      "name": "Fourier Neural Operator",
      "architecture": "FNO",
      "training_type": "operator_learning",
      ...
    }
  ],
  "numerical_methods": [
    {
      "id": "fdm",
      "name": "Finite Difference Method",
      "method_type": "grid_based",
      ...
    }
  ]
}
```

---

## `/ai-models` — AI 模型

### 列出 AI 模型

```
GET /ai-models?training_type=<type>
```

| `training_type` 值 | 含义 |
|---|---|
| `supervised` | 监督学习 |
| `unsupervised` | 无监督学习 |
| `self_supervised` | 自监督学习 |
| `physics_informed` | 物理信息神经网络（如 PINNs） |
| `operator_learning` | 算子学习（如 FNO、DeepONet） |
| 省略 | 返回所有 AI 模型 |

### 预置 AI 模型 id 速查

| id | 模型名 | training_type |
|---|---|---|
| `pinns` | Physics-Informed Neural Networks | physics_informed |
| `deeponet` | Deep Operator Network | operator_learning |
| `fno` | Fourier Neural Operator | operator_learning |
| `pdeformer` | PDEformer | supervised |

### `/ai-models/:id/profile` — 模型完整图谱

多跳查询，一次返回模型的全部上下文：

```json
{
  "model": { ... },
  "equations": [{ "id": "...", "name": "..." }],
  "loss_functions": [{ "id": "...", "name": "...", "loss_type": "physics" }],
  "metrics": [{ "id": "...", "name": "..." }],
  "datasets": [{ "id": "...", "name": "..." }],
  "papers": [{ "id": "...", "title": "..." }]
}
```

---

## `/numerical-methods` — 数值方法

无过滤参数，`GET /numerical-methods` 返回全部数值方法列表。

### 预置数值方法 id 速查

| id | 方法名 | method_type |
|---|---|---|
| `fdm` | Finite Difference Method | grid_based |
| `fem` | Finite Element Method | mesh_based |
| `fvm` | Finite Volume Method | mesh_based |
| `spectral` | Spectral Method | spectral_based |
| `rbf` | Radial Basis Function Method | mesh_free |

### 响应格式（单个数值方法）

```json
{
  "id": "fem",
  "name": "Finite Element Method",
  "method_type": "mesh_based",
  "order": 2,
  "description": "...",
  "tags": ["classical", "mesh_based"]
}
```

---

## `/papers` — 论文

### 列出论文

```
GET /papers?year=<year>
```

按 `year` 过滤（4 位整数），省略则返回所有论文，按标题排序。

### 获取论文详情

```
GET /papers/:id
```

`id` 优先使用 arXiv id（如 `"2010.08895"`）。响应合并了 Neo4j 结构字段和 SQLite 中的摘要：

```json
{
  "paper": {
    "id": "2010.08895",
    "title": "Fourier Neural Operator for Parametric PDEs",
    "authors": ["Li, Z.", "Kovachki, N."],
    "published_year": 2021,
    "arxiv_id": "2010.08895",
    "doi": null,
    "tags": ["operator_learning", "fourier"]
  },
  "abstract": "We propose...",
  "notes": null
}
```

### `/papers/:id/profile` — 论文完整图谱

多跳查询，返回该论文提出/研究/引用的全部节点：

```json
{
  "paper": { ... },
  "proposes": [
    { "node_type": "ai_model", "id": "fno", "name": "..." }
  ],
  "studies": [
    { "node_type": "equation", "id": "navier_stokes", "name": "..." }
  ],
  "uses_datasets": [{ "id": "...", "name": "..." }],
  "cites": [{ "id": "...", "title": "..." }],
  "cited_by": [{ "id": "...", "title": "..." }]
}
```

---

## 写入 API（`/internal/`）

供知识录入使用，不用于普通查询：

| 端点 | 方法 | 功能 |
|---|---|---|
| `/internal/nodes` | POST | 新增或更新节点（MERGE） |
| `/internal/nodes/:label/:id` | DELETE | 删除节点（级联删除关系） |
| `/internal/relations` | POST | 新增或更新关系 |
| `/internal/relations` | DELETE | 删除关系 |
| `/internal/content` | POST | 写入 SQLite 长文本（摘要/注释） |

写入节点示例：
```json
POST /internal/nodes
{
  "node_type": "paper",
  "data": {
    "id": "2301.12345",
    "title": "...",
    "authors": ["Wang, S."],
    "published_year": 2023,
    "arxiv_id": "2301.12345",
    "tags": ["pinns"]
  }
}
```
