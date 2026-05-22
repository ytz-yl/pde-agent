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
| `/equations/:id/solvers` | GET | 该方程关联的 AI 模型 + 数值方法（按 `engine_id` 分两组） |
| `/equations/:id/conditions` | GET | 该方程的边界/初始条件 |
| `/equations/:id/datasets` | GET | 该方程的基准数据集 |
| `/equations/:id/papers` | GET | 研究该方程的论文 |
| `/ai-models` | GET | 列出 AI 模型（可按 `training_type` 过滤） |
| `/ai-models/:id` | GET | 获取单个 AI 模型详情 |
| `/ai-models/:id/profile` | GET | AI 模型完整图谱（多跳） |
| `/ai-models/:id/equations` | GET | 该模型能求解的方程列表 |
| `/ai-models/:id/papers` | GET | 提出该模型的论文 |
| `/ai-models/:id/results` | GET | 该模型在所有 benchmark 上的实测记录 |
| `/numerical-methods` | GET | 列出所有数值方法 |
| `/numerical-methods/:id` | GET | 获取单个数值方法详情 |
| `/numerical-methods/:id/papers` | GET | 提出该数值方法的论文 |
| `/numerical-methods/:id/results` | GET | 该数值方法在所有 benchmark 上的实测记录 |
| `/papers` | GET | 列出论文（可按 `year` 过滤） |
| `/papers/:id` | GET | 论文基本信息 + 摘要 |
| `/papers/:id/profile` | GET | 论文完整图谱（多跳） |
| `/benchmarks` | GET | 列出所有评测口径（metric × dataset × 协议）|
| `/benchmarks/:id` | GET | 单个评测的定义 + 长版协议说明 |
| `/benchmarks/:id/leaderboard` | GET | 该评测的实时排行榜（含 confidence 聚合）|

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
    "label": "AIModel",
    "id": "fno",
    "name": "Fourier Neural Operator",
    "description": "Learns solution operators in Fourier space..."
  },
  {
    "label": "Paper",
    "id": "2010.08895",
    "name": "Fourier Neural Operator for Parametric PDEs",
    "description": null
  }
]
```

字段说明：
- `label` 是 Neo4j 标签的 **PascalCase** 形式（`Equation`、`AIModel`、`NumericalMethod`、`LossFunction`、`Metric`、`Dataset`、`Paper`、`Benchmark`、`BenchResult`、`Theorem`、`Condition`），不是 `node_type`。
- `description` 为 `null` 时表示节点没有描述字段（如 Paper）。
- 后续要去拿详情时，把 label 转回端点前缀：`AIModel` → `/ai-models/:id`、`NumericalMethod` → `/numerical-methods/:id` 等。

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

知识库 seed 时内置以下 7 个方程节点，可直接用 id 查询：

| id | 方程名 | pde_type |
|---|---|---|
| `heat_equation` | 热方程 | parabolic |
| `wave_equation` | 波动方程 | hyperbolic |
| `poisson` | 泊松方程 | elliptic |
| `navier_stokes` | Navier-Stokes 方程 | mixed |
| `burgers` | Burgers 方程 | hyperbolic |
| `schrodinger` | 薛定谔方程 | parabolic |
| `allen_cahn` | Allen-Cahn 方程 | parabolic |

> **注意 id 的规范**：seed 出来的方程**不带 `_equation` 后缀**（`burgers` 不是 `burgers_equation`，`poisson` 不是 `poisson_equation`）。如果走 ingest 流程批量录入，DB 里可能同时存在两种命名（重复节点是历史遗留），用 seed id 更稳。其它方程通过 `GET /equations` 全量列出。

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

这是**最核心的图遍历端点**，返回通过 `SOLVES` 关系连接到该方程的所有 AI 模型和数值方法。**响应按 `engine_id` 字段分两组**：

```json
{
  "equation_id": "heat_equation",
  "equation_name": "Heat Equation",
  "executable": {
    "ai_models": [
      {
        "id": "pdeformer",
        "name": "PDEformer",
        "architecture": "Transformer",
        "training_type": "supervised",
        "engine_id": "pdeformer2",
        ...
      }
    ],
    "numerical_methods": [
      {
        "id": "fdm",
        "name": "Finite Difference Method",
        "method_type": "grid_based",
        "engine_id": "classical",
        ...
      }
    ]
  },
  "literature_only": {
    "ai_models": [
      {
        "id": "fno",
        "name": "Fourier Neural Operator",
        "engine_id": null,
        ...
      }
    ],
    "numerical_methods": [
      { "id": "fem", "engine_id": null, ... }
    ]
  }
}
```

**两组的语义**：
- **`executable`**：方法节点的 `engine_id` 非空，意味着求解器服务（`localhost:3000`）注册了对应后端，**可以直接 `POST /solve` 调用**（用 `engine_id` 作为 `solver` 字段值）。
- **`literature_only`**：方法节点存在于知识图谱里（论文写过），但求解器服务没注册对应后端 —— **只能查询不能跑**。

**Agent 决策口径**：用户要"求解"时只能从 `executable` 取；用户要"调研有哪些方法"时两组都看。

> `engine_id` 字段在两组返回的方法节点上都会出现：`executable` 一定非空，`literature_only` 一定为 null/缺失。

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

| id | 模型名 | training_type | engine_id |
|---|---|---|---|
| `pinn` | Physics-Informed Neural Network | physics_informed | — |
| `deeponet` | Deep Operator Network | operator_learning | — |
| `fno` | Fourier Neural Operator | operator_learning | — |
| `pdeformer` | PDEformer | supervised | `pdeformer2` ✅ |
| `deepxde_net` | DeepXDE Network | physics_informed | — |

> id 是 `pinn`（无 s），不是 `pinns`。`engine_id` 列里有值的代表本地求解器服务可调用，发到 `POST /solve` 时把这个 id 填进 `solver` 字段。`—` 表示该模型只是图谱里的"知识"，没有可执行后端。

### `/ai-models/:id/profile` — 模型完整图谱

多跳查询，一次返回模型的全部上下文：

```json
{
  "model": { /* AIModel 节点完整字段，含 engine_id */ },
  "solves":       [{ "id": "heat_equation", "name": "...", "pde_type": "parabolic" }],
  "trained_by":   [{ "id": "combined_pinn_loss", "name": "...", "loss_type": "combined" }],
  "evaluated_by": [{ "id": "l2_error", "name": "...", "metric_type": "accuracy" }],
  "tested_on":    [{ "id": "navier_stokes_2d", "name": "...", "dimension": "2D" }]
}
```

字段对应的图关系：
- `solves` ← `(:AIModel)-[:SOLVES]->(:Equation)`
- `trained_by` ← `(:AIModel)-[:TRAINED_BY]->(:LossFunction)`
- `evaluated_by` ← `(:AIModel)-[:EVALUATED_BY]->(:Metric)`
- `tested_on` ← `(:AIModel)-[:TESTED_ON]->(:Dataset)`

> **注意**：profile 不包含 papers 字段。要拿提出该模型的论文，单独调 `GET /ai-models/:id/papers`。要拿该模型在各 benchmark 上的实测数据，调 `GET /ai-models/:id/results`。

---

## `/numerical-methods` — 数值方法

无过滤参数，`GET /numerical-methods` 返回全部数值方法列表。

### 预置数值方法 id 速查

| id | 方法名 | method_type | engine_id |
|---|---|---|---|
| `fdm` | Finite Difference Method | grid_based | `classical` ✅ |
| `fem` | Finite Element Method | mesh_based | — |
| `fvm` | Finite Volume Method | mesh_based | — |
| `spectral` | Spectral Methods | spectral_based | — |

> 没有 `rbf`（Radial Basis Function）节点 —— 不要尝试查询这个 id。`fdm` 的 `engine_id="classical"` 对应求解器服务的 py-pde 后端（实际能跑 FDM/谱方法等多种 grid_based / spectral_based 算法）。

### 响应格式（单个数值方法）

```json
{
  "id": "fem",
  "name": "Finite Element Method",
  "method_type": "mesh_based",
  "order": 2,
  "description": "...",
  "tags": ["classical", "mesh_based"],
  "engine_id": null
}
```

`engine_id` 字段缺省（null）时表示无可执行后端。

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
    "pdf_path": null,
    "tags": ["operator_learning", "fourier"]
  },
  "abstract": "We propose...",
  "notes": null
}
```

`pdf_path` 字段是 PDF 文件在本地文件系统的绝对路径，目前所有论文都为 `null`（未启用本地 PDF 存储）。

### `/papers/:id/profile` — 论文完整图谱

多跳查询，返回该论文提出/研究/引用的全部节点：

```json
{
  "paper": { /* 同上 Paper 节点字段 */ },
  "proposes": [
    { "label": "AIModel", "id": "fno", "name": "Fourier Neural Operator" }
  ],
  "studies": [
    { "id": "navier_stokes", "name": "...", "pde_type": "mixed" }
  ],
  "uses_datasets": [
    { "id": "navier_stokes_2d", "name": "...", "dimension": "2D" }
  ],
  "cites": [
    { "id": "1907.10322", "title": "...", "published_year": 2019, "arxiv_id": "1907.10322" }
  ],
  "cited_by": [
    { "id": "2402.03477", "title": "...", "published_year": 2024, "arxiv_id": "2402.03477" }
  ]
}
```

字段说明：
- `proposes` 元素的类型字段是 **`label`**（PascalCase Neo4j 标签），不是 `node_type`；可以是 `AIModel` 或 `NumericalMethod`。
- `cited_by` 是反向引用（哪些论文引用了本论文），由 `(:Paper)-[:CITES]->(本论文)` 反查得到。

---

## `/benchmarks` — 评测口径与排行榜

### 列出 / 获取 benchmark

```
GET /benchmarks
GET /benchmarks/:id
```

`Benchmark` 节点把一个 `Metric`（"L2 误差"）绑到一个 `Dataset`（"PDEBench NS-2D"）上，再加协议描述，构成可比较的评测单位。

### 预置 benchmark 速查

| id | 数据集 | 指标 | 协议简述 |
|---|---|---|---|
| `pdebench_burgers1d_rel_l2` | `burgers_1d` | `l2_error` | viscosity 0.01, 256 grid, t=1.0 |
| `pdebench_ns2d_rel_l2` | `navier_stokes_2d` | `l2_error` | Kolmogorov flow, 64×64, ν=1e-3, t=10s |
| `pdebench_darcy_rel_l2` | `darcy_flow` | `l2_error` | 421×421 grid, log-normal permeability, steady state |

`GET /benchmarks/:id` 返回：
```json
{
  "benchmark": {
    "id": "pdebench_ns2d_rel_l2",
    "name": "PDEBench NS-2D, Relative L2",
    "dataset_id": "navier_stokes_2d",
    "metric_id": "l2_error",
    "lower_is_better": true,
    "protocol": "Kolmogorov flow, 64x64, viscosity 1e-3, t=10s rollout",
    "tolerance": 0.05
  },
  "notes": null
}
```

`tolerance` 是相对容差（`(max - min) / |mean|`），用于多源校验时判断"verified vs disputed"。

### `/benchmarks/:id/leaderboard` — 排行榜（核心查询）

返回该评测下所有 BenchResult 按方法聚合后的排名：

```json
{
  "benchmark": { /* 同上 */ },
  "dataset_name": "Navier-Stokes 2D Dataset",
  "metric_name": "L2 Relative Error",
  "entries": [
    {
      "method_id": "fno",
      "method_label": "AIModel",
      "method_name": "Fourier Neural Operator",
      "best_value": 0.0382,
      "all_values": [0.0395, 0.0382],
      "n_independent_sources": 2,
      "n_results": 2,
      "confidence": "verified",
      "latest_recorded_at": "2026-05-18T07:09:42.548Z",
      "source_breakdown": { "paper_reported": 1, "self_run": 1 }
    },
    { "method_id": "fdm", "method_label": "NumericalMethod", "confidence": "disputed", ... },
    { "method_id": "pdeformer", "confidence": "single", ... }
  ]
}
```

**confidence 三态**（在查询时实时计算，永不存储）：
- `verified` — ≥2 个**独立来源**且数值差在 `benchmark.tolerance` 内
- `single` — 只有 1 个独立来源
- `disputed` — ≥2 个独立来源但数值差超过 tolerance

**独立来源**的定义：`(source_type, source_paper_id)` 二元组去重。两条都来自同一篇论文的 `paper_reported` 只算 1 个独立来源；两条 `self_run`（都没有 paper_id）合并成 1 个。

排序规则：按 `best_value` 升序（若 `lower_is_better=false` 则降序）；并列时 `verified > single > disputed`。

### `/ai-models/:id/results` 和 `/numerical-methods/:id/results`

返回某个具体方法在所有 benchmark 上的 BenchResult 列表（按 `recorded_at` 降序）：

```json
[
  {
    "id": "fno__pdebench_ns2d_rel_l2__paper__18b09742445b4a22",
    "method_id": "fno",
    "method_label": "AIModel",
    "benchmark_id": "pdebench_ns2d_rel_l2",
    "value": 0.0382,
    "source_type": "paper_reported",
    "source_paper_id": "2010.08895",
    "hardware": null,
    "code_ref": null,
    "recorded_at": "2026-05-18T07:09:23.682Z"
  }
]
```

`source_type` 合法值：`paper_reported` | `self_run` | `third_party_reproduction`。

---

## 写入 API（`/internal/`）

供知识录入使用，不用于普通查询：

| 端点 | 方法 | 功能 |
|---|---|---|
| `/internal/nodes` | POST | 新增或更新节点（MERGE 语义） |
| `/internal/nodes/:label/:id` | DELETE | 删除节点（DETACH，级联删除关系） |
| `/internal/relations` | POST | 新增或更新关系 |
| `/internal/relations` | DELETE | 删除关系 |
| `/internal/content` | POST | 单独写 SQLite 长文本（摘要/注释） |
| `/internal/results` | POST | 单步提交一条 BenchResult（自动连边、自动生成 id） |

### `POST /internal/nodes` —— 通用节点写入

请求体是**平铺**结构，`node_type` 与节点字段在同一层级（**没有** `data` 包装）：

```json
{
  "node_type": "paper",
  "id": "2301.12345",
  "title": "...",
  "authors": ["Wang, S."],
  "published_year": 2023,
  "arxiv_id": "2301.12345",
  "tags": ["pinns"],
  "abstract": "...",
  "notes": "..."
}
```

`node_type` 合法值：`equation` | `condition` | `theorem` | `numerical_method` | `ai_model` | `loss_function` | `metric` | `dataset` | `paper` | `benchmark` | `bench_result`。

特殊字段处理：
- 顶层的 `abstract` / `notes` 字段会被路由到 SQLite content 存储，不进 Neo4j。
- 写 `ai_model` / `numerical_method` 时可填 `engine_id` 字段把它桥接到求解器服务。
- 写 `benchmark` 时会自动建立 `(Benchmark)-[:ON_DATASET]->(Dataset)` 和 `(Benchmark)-[:USES_METRIC]->(Metric)` 边（Dataset 和 Metric 必须先存在）。
- 写 `bench_result` 时 `id` 字段可省略（服务端自动生成 `<method>__<benchmark>__<src>__<nanos>`），并自动建立 `OF_METHOD` / `ON_BENCHMARK` / `REPORTED_IN` 三条边。

返回：
```json
{ "status": "ok", "action": "upserted", "label": "Paper", "id": "2301.12345" }
```

### `POST /internal/results` —— 提交一次实测（推荐）

比走通用 `/internal/nodes` 路径快得多，单次调用就完成节点 + 三条关系边。

```json
{
  "method_id": "fno",
  "method_label": "AIModel",
  "benchmark_id": "pdebench_ns2d_rel_l2",
  "value": 0.0382,
  "source_type": "paper_reported",
  "source_paper_id": "2010.08895",
  "hardware": "1x A100 80G",
  "code_ref": "https://github.com/zongyi-li/...",
  "notes": "复现脚本 commit a1b2c3"
}
```

约束：
- `method_label` 必须是 `"AIModel"` 或 `"NumericalMethod"`（PascalCase）。
- `source_type` 是 `paper_reported` 或 `third_party_reproduction` 时**必须**提供 `source_paper_id`，否则 500。
- `recorded_at` 缺省时服务端写入当前时间。

### `POST /internal/relations` —— 通用关系写入

```json
{
  "from_id": "fno",
  "from_label": "AIModel",
  "to_id": "navier_stokes",
  "to_label": "Equation",
  "relation_type": "SOLVES",
  "properties": null
}
```

`from_label` / `to_label` 用 PascalCase Neo4j 标签字符串。`relation_type` 必须是合法的关系类型常量（详见 ingest skill 的关系参考表），传错会返回 400。
