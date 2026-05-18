---
name: pde-ingest-skill
description: Use this skill when the user wants to add, update, or enrich knowledge in the PDE knowledge base — including ingesting a research paper (from arXiv, URL, or PDF), adding a new AI model or numerical method, recording relationships between methods and equations, or systematically surveying a PDE research area and writing structured findings into the graph. Use this skill whenever the task involves writing to the knowledge base, not just querying it.
---

# PDE Knowledge Ingestion Skill

本 skill 指导 agent 如何发现、分析、整合 PDE 领域知识，并通过写接口将其录入知识库图谱。

知识库是一个 Neo4j 图数据库，核心不是"存文章"，而是**建立结构化节点和它们之间的有向关系**。录入一篇论文的终点不是把 PDF 存进去，而是让图谱知道：这篇论文**提出了**哪个模型、**研究了**哪个方程、**使用了**哪个数据集。

详细写接口参数见 [`ingest-api.md`](./ingest-api.md)。
API 实测验证笔记（必填字段、健康检查流程、cron 配置）见 [`references/api-validation-notes.md`](./references/api-validation-notes.md)。
Solver API 张量文件上传（`POST /files` + `history` 字段）见 [`references/solver-api-history.md`](./references/solver-api-history.md)。
上传 + 求解请求模板见 [`templates/solve-with-history.json`](./templates/solve-with-history.json)。

---

## Solver API 服务（http://localhost:8080）

pde-agent 项目包含两个独立服务，注意区分：

| 服务 | 端口 | 用途 |
|---|---|---|
| 知识库 API（Rust + Neo4j） | 3001 | 图谱读写（本 skill 主要覆盖范围） |
| 求解器 API（Rust + Python） | 8080 | PDE 数值/ML 求解 |

**求解器 API 关键端点：**
- `GET  /health` — 健康检查
- `GET  /solvers` — 列出可用求解器（`pdeformer2` / `classical`）
- `POST /files` — 上传张量文件（.npy/.npz/.h5/.pt），返回 `file_id`
- `POST /solve` — 执行求解，支持 `pde.history.file_id` 引用上传文件

**求解器 API 数据流：**
```
客户端 JSON → Rust (axum) → JSON via stdin → Python 脚本 → JSON via stdout → 客户端
```

`engines/api/scripts/pdeformer2_infer.py` 和 `classical_solve.py` 是两个独立桥接脚本，
分别使用 `pdeformer2` 和 `classical-pde` conda 环境。

### KB ↔ engines 桥梁：`engine_id` 字段

知识库的 `AIModel` / `NumericalMethod` 节点有一个可选字段 `engine_id`，**值就是上面 `/solvers` 返回的 solver id**：

| KB 节点 | engine_id | 含义 |
|---|---|---|
| `pdeformer` (AIModel) | `pdeformer2` | 本地可调用，发到 `POST /solve` 的 `solver_id="pdeformer2"` |
| `fdm` (NumericalMethod) | `classical` | 本地可调用，对应 py-pde 后端 |
| 其他文献录入的方法 | （留空） | 仅图谱中存在，无法直接执行 |

录入新方法时，如果对应的 solver 已经在 engines 注册，就把这个 id 填进 `engine_id`；agent 看到非空 `engine_id` 就知道可以直接调用 solver API 跑这个方法。

---

## 知识库 API 整体流程

```
发现来源（arXiv / 代码仓库 / 用户提供）
      ↓
信息提取（读论文 / 读 README / 读代码）
      ↓
映射到图谱结构（节点 + 关系）
      ↓
查重（先搜索，避免重复写入）
      ↓
写入（POST /internal/nodes → POST /internal/relations）
      ↓
验证（回读确认写入成功）
```

---

## 第一步：发现与获取来源

根据用户意图选择来源类型：

| 来源 | 处理方式 |
|---|---|
| arXiv ID（如 `2010.08895`） | 通过 `https://arxiv.org/abs/<id>` 获取标题/作者/摘要 |
| arXiv 搜索词 | 访问 `https://arxiv.org/search/?query=<keywords>&searchtype=all` |
| GitHub 仓库 | 读 README、论文引用、模型架构描述 |
| 用户提供的文本/PDF | 直接从内容中提取结构化信息 |
| 关键词调研 | 先搜索知识库 `GET /search?q=<keywords>` 了解现有覆盖范围，再决定补充哪些 |

---

## 第二步：信息提取

从来源中提取以下信息，后续将映射到图谱节点：

**论文信息**
- 标题、作者列表、发表年份、arXiv ID / DOI
- 摘要（完整文本，存入 SQLite）
- 核心贡献：提出了什么新方法或新结果？

**方法信息**（如果论文提出了新模型或新算法）
- 方法名称和简称（如 FNO、PINNs）
- 类型：AI 模型还是经典数值方法？
- AI 模型：架构（MLP/CNN/Transformer/FNO）、训练范式（physics_informed / operator_learning / supervised 等）
- 数值方法：类型（grid_based / mesh_based / spectral_based / mesh_free）、精度阶数

**关系信息**（最重要的部分）
- 该方法能求解哪些 PDE？→ `SOLVES`
- 该方法用了哪种损失函数？→ `TRAINED_BY`
- 该方法在哪些数据集上评测？→ `TESTED_ON`
- 该论文研究了哪些方程？→ `STUDIES`
- 该论文引用了哪些已知论文？→ `CITES`

---

## 第三步：映射到图谱结构

在写入前，先将提取的信息翻译成图谱语言。思路如下：

**节点 id 命名规范**
- 方程：用下划线小写，如 `heat_equation`、`navier_stokes`
- AI 模型：用简称小写，如 `fno`、`pinns`、`deeponet`
- 数值方法：如 `fdm`、`fem`、`fvm`
- 论文：优先用 arXiv ID，如 `2010.08895`；无 arXiv 则用 `doi_<doi-slug>` 或 `paper_<slug>`
- 数据集：如 `ns2d_dataset`、`burgers1d_dataset`

**节点类型选择**（共 11 种，全部支持写入）

| 用途 | node_type 值 | Neo4j Label |
|---|---|---|
| 论文 | `paper` | Paper |
| AI/ML 模型 | `ai_model` | AIModel |
| 经典数值方法（FDM/FEM/FVM 等） | `numerical_method` | NumericalMethod |
| PDE 方程 | `equation` | Equation |
| 边界/初始条件 | `condition` | Condition |
| 数学定理 | `theorem` | Theorem |
| 损失函数 | `loss_function` | LossFunction |
| 评估指标（词表） | `metric` | Metric |
| 基准数据集 | `dataset` | Dataset |
| 评测口径（metric+dataset+协议） | `benchmark` | Benchmark |
| 一次实测值（method×benchmark→value） | `bench_result` | BenchResult |

写入时 `node_type` 用上表左列小写值。`AIModel` 对应 `"ai_model"`，不能用 `"aimodel"`。

> **Metric vs Benchmark vs BenchResult 三层关系**：Metric 是抽象定义（"L2 误差是什么"），Benchmark 把 Metric 绑到具体 Dataset 和协议上（"PDEBench NS-2D 上的 L2"），BenchResult 是一次具体测量（"FNO 在那个 Benchmark 上跑出 0.012"）。同一个 (method, benchmark) 可以有多条 BenchResult，**永远 append 不覆盖**，多源一致即被标为 `verified`。

---

## 第四步：查重

写入前必须先查重，避免创建重复节点：

```
GET /search?q=<方法名或论文标题关键词>
```

- 如果搜索结果中已有同名节点，直接用已有 id 建立关系，**不重新创建节点**
- 如果搜索结果为空或名称不匹配，才创建新节点
- 已有的预置种子节点（`heat_equation`、`fno`、`fdm` 等）直接使用，无需重建

---

## 第五步：写入

写入顺序很重要：**先写节点，后写关系**。关系引用节点 id，节点必须存在才能建立关系。

### 典型论文录入序列

```
1. POST /internal/nodes          ← 写入 Paper 节点（含摘要）
2. POST /internal/nodes          ← 写入论文提出的 AIModel 或 NumericalMethod（若为新方法）
3. POST /internal/nodes          ← 写入论文使用的 Dataset（若为新数据集）
4. POST /internal/relations      ← Paper PROPOSES AIModel
5. POST /internal/relations      ← Paper STUDIES Equation（使用已有方程 id）
6. POST /internal/relations      ← AIModel SOLVES Equation
7. POST /internal/relations      ← AIModel TESTED_ON Dataset
8. POST /internal/relations      ← Paper CITES 已知论文（如有）
9. POST /internal/results        ← 论文报告了 benchmark 数值时，每个数值发一次（可选）
```

第 9 步必要前提：目标 Benchmark 已存在（用 `GET /benchmarks` 检查；不存在则先 `POST /internal/nodes` 建一个 `node_type:"benchmark"`）。每条 BenchResult 必须带 `source_type` 和——若来自论文——`source_paper_id`，否则会被服务端拒收。

所有写操作均为 MERGE 语义（幂等），重复执行不会造成重复节点或关系。BenchResult 由于 id 自动生成，重复 POST 同一组 (method, benchmark, value) 会创建多条节点；这是有意为之，独立的多次测量正是验证的依据。如确需去重，调用方需自己持有上次返回的 id 改用 `/internal/nodes` 显式写入。

---

## 第六步：验证

写入后通过读接口确认结果：

```
GET /search?q=<刚写入节点的名称>          ← 确认节点可被搜索到
GET /papers/:id/profile                   ← 确认论文的关系图谱完整
GET /ai-models/:id/equations              ← 确认 SOLVES 关系已建立
GET /benchmarks/:id/leaderboard           ← 确认 BenchResult 已挂上并参与排名
GET /ai-models/:id/results                ← 查看一个方法所有 benchmark 的实测记录
```

如果回读结果与预期不符，检查 id 是否正确、关系方向是否正确（见 `ingest-api.md` 关系方向表）。

leaderboard 的 `confidence` 字段会随数据增加自动调整：
- `single` — 只有 1 个独立来源
- `verified` — ≥2 个独立来源，且数值在 benchmark 的 `tolerance` 内（默认相对 5%）
- `disputed` — ≥2 个独立来源但数值差异超出 tolerance（需要人工排查）

独立来源 = 不同的 (`source_type`, `source_paper_id`) 二元组。两条都来自同一论文的 `paper_reported` 只算 1 个独立来源。

---

## 已知 Solver API 陷阱

- **`POST /files` multipart 字段名必须是 `file`**，否则返回 400 "must contain a field named 'file'"。

- **`file_id` 没有持久化**：服务器重启后 `/tmp/pde-solver-uploads/` 目录内文件依然存在，
  但重启后 `file_path_for_id()` 靠扫描目录找文件，只要文件还在就能继续使用。
  若改了 `SOLVER_UPLOAD_DIR`，旧 file_id 对应的文件路径会失效。

- **`history` 和 `initial_condition` 不能同时生效**：当 `pde.history` 存在时，
  Python 脚本会忽略 `initial_condition` / `initial_conditions`，以 history 最后帧为准。

- **classical solver 只取第一个变量通道**：多变量 history 文件（n_vars > 1）传给
  classical 求解器时，只有 `arr[-1, :, :, 0]` 被用作 IC，其余通道被丢弃，
  `notes` 字段不会提示这一点（待改进）。

- **`cargo build` 后 linter 误报"Rust 2015"错误**：这是 patch 工具的 linter 假报错，
  实际项目 `Cargo.toml` 已声明 `edition = "2021"`，`cargo build` 能正常通过，忽略即可。

---

## 已知 知识库 API 陷阱

- **Paper 节点的 `authors` 字段是必填项**。缺少时服务器返回 500（而非 400），
  错误信息为 `{"error":"invalid node body: missing field \`authors\`"}`。
  写入 Paper 时务必包含 `"authors": ["作者1", "作者2"]`，哪怕只有一个元素。

- **写接口前置健康检查**：在批量录入之前，可以用一个完整的 Paper 测试节点
  验证服务可用性，写入成功后立即用 `DELETE /internal/nodes/Paper/<id>` 清除。

- **服务健康端点**：`GET /health` 返回 `{"service":"pde-knowledge-base","status":"ok"}`，
  可用于快速检查服务是否在线（根路径 `/` 返回 404，不要用它做健康检查）。

- **`/equations/:id/solvers` 按 `engine_id` 分两组返回**：响应结构为
  `{ equation_id, equation_name, executable: { ai_models, numerical_methods }, literature_only: { ai_models, numerical_methods } }`。
  `executable` 组里的方法本地可调用（engines API 已注册对应 solver_id），
  `literature_only` 组只在图谱里有节点。agent 决定"直接跑还是只引用"时，**从 `executable` 取**。
  详见 `references/api-validation-notes.md` 的 solvers 接口节。

- **Dataset → Equation 关系方向**：写入"数据集基于某方程生成"时，
  关系方向是 `Dataset -[BASED_ON]-> Equation`（from=Dataset, to=Equation），
  不是反向。查询 `/equations/:id/datasets` 也遵循此方向。

- **未知枚举值的静默 fallback**：`training_type` 传入未知值会 fallback 到 `supervised`，
  `method_type` 传入未知值会 fallback 到 `other`，API 不报错但分类信息会丢失。
  如需支持新训练范式（如 `reinforcement_learning`），需在 Rust 源码 `schema.rs`
  的 `TrainingType` 枚举中添加新变体并重新编译服务。

---

## 常见场景

### 场景一：录入一篇 arXiv 论文

```
1. 读取 https://arxiv.org/abs/<id> 获取元数据
2. 判断论文提出了什么方法（AIModel / NumericalMethod）
3. 搜索知识库确认方法和方程节点是否已存在
4. POST /internal/nodes 写入 Paper（含 abstract 字段）
5. 如有新方法，POST /internal/nodes 写入方法节点
6. POST /internal/relations 建立所有关系
7. GET /papers/:id/profile 验证
```

### 场景二：系统调研某个 PDE 研究方向

```
1. GET /search?q=<主题> 了解知识库现有覆盖
2. 搜索 arXiv 找到 5-10 篇代表性论文
3. 对每篇论文走"录入论文"流程
4. 完成后 GET /equations/:id/solvers 查看该方向的方法全貌
```

### 场景三：补充已有方法的信息

```
1. GET /ai-models/:id 查看现有字段
2. POST /internal/nodes 用完整数据重新 upsert（MERGE 不会丢失已有关系）
3. 如需补充论文关系，POST /internal/relations 追加即可
```

### 场景四：把论文里的 benchmark 数值录入并对比

录入一个新方法在标准 benchmark 上的数值，并和已有方法对比排名：

```
1. GET /benchmarks                                ← 看目标 benchmark 是否已存在（如 pdebench_ns2d_rel_l2）
2. 若不存在：POST /internal/nodes 建一个 benchmark
   （需先确认 dataset_id 和 metric_id 对应的节点已存在）
3. POST /internal/results                         ← 提交一条 BenchResult（source_type=paper_reported）
4. GET /benchmarks/<bench_id>/leaderboard         ← 查看实时排名和 confidence
5. 若想自己复现验证，再 POST /internal/results 一条 source_type=self_run
   （value 在 tolerance 内 → confidence 升级为 verified；超出则 disputed）
```

**何时需要新建 benchmark**：当协议（分辨率、时间步、训练/测试切分）与现有 benchmark 不一致时；否则**优先复用**预置 id，避免数据被切碎到无法横向比较。
