---
name: pde-skill
description: Use this skill whenever the user asks anything related to PDEs (partial differential equations) — including solving a PDE numerically, looking up solution methods or AI models for a specific equation, researching numerical methods like FEM/FDM/FNO/PINNs, querying the PDE knowledge base for papers or equation information, or comparing approaches for fluid dynamics, heat transfer, wave propagation, or other physics problems. Use this skill even if the user doesn't explicitly mention "PDE" but describes a physics simulation or asks which solver to use.
---

# PDE Agent Skill

本 skill 说明 AI Agent 在面对 PDE 相关任务时，应当调用哪些服务、何时调用、调用哪个端点。

系统由两个独立服务组成：

| 服务 | 默认端口 | 职责 |
|---|---|---|
| **知识库服务** (`knowledge-base`) | 3001 | 图结构知识查询：方程、AI 模型、数值方法、论文、评测榜单 |
| **求解器服务** (`engines`) | 3000 | 实际执行 PDE 数值求解，注册的 solver id：`pdeformer2` / `classical` |

> 仓库目录从 `solvers/` 重命名到 `engines/`，但 HTTP 接口和端口未变。本文沿用"求解器服务"称呼。

---

## 知识库的核心设计：以图为中心

知识库是一个 **Neo4j 图数据库**，围绕 11 种节点类型和图关系组织知识：

| 节点类型 | 端点前缀 | 含义 |
|---|---|---|
| `Equation` | `/equations` | PDE 方程（如热方程、Navier-Stokes） |
| `AIModel` | `/ai-models` | AI/ML 求解模型（如 FNO、PINNs、PDEformer） |
| `NumericalMethod` | `/numerical-methods` | 经典数值方法（如 FDM、FEM、FVM） |
| `Paper` | `/papers` | 研究论文 |
| `Benchmark` | `/benchmarks` | 评测口径（dataset × metric × 协议）|
| `BenchResult` | `/benchmarks/:id/leaderboard` 等 | 一次具体测量值，多源校验后聚合 |
| `Condition` | 通过 `/equations/:id/conditions` | 边界/初始条件 |
| `Dataset` | 通过 `/equations/:id/datasets` | 基准数据集 |
| `Metric` | 通过 `/benchmarks/:id` 引用 | 度量定义（"L2 误差是什么"） |
| `LossFunction` | 通过 `/ai-models/:id/profile` 引用 | 损失函数 |
| `Theorem` | 仅 `/internal/` 写入 | 数学定理（暂无公开查询路径） |

**查询的正确姿势是沿图关系遍历**，而不是全文搜索过滤。典型路径：
- 从方程出发 → 查哪些 AI 模型和数值方法能求解它（`/equations/:id/solvers`，按 `engine_id` 分两组）
- 从 AI 模型出发 → 查它能解哪些方程（`/ai-models/:id/equations`），或在哪些 benchmark 上跑过分（`/ai-models/:id/results`）
- 从方程/模型出发 → 查相关论文（`/equations/:id/papers`、`/ai-models/:id/papers`）
- 从 benchmark 出发 → 看实时排行榜（`/benchmarks/:id/leaderboard`）

### 关键概念：`engine_id` —— KB ↔ 求解器服务的桥梁

`AIModel` / `NumericalMethod` 节点上有一个可选字段 `engine_id`，**值就是求解器服务 `GET /solvers` 返回的 solver id**（目前 `pdeformer2` / `classical`）。

| KB 节点 | engine_id | 含义 |
|---|---|---|
| `pdeformer` (AIModel) | `pdeformer2` | 本地可调用，发到 `POST /solve` 用这个 id 当 `solver` |
| `fdm` (NumericalMethod) | `classical` | 本地可调用，对应 py-pde 后端 |
| `fno`、`pinn` 等 | 缺失 / null | 仅图谱里有节点，**没有可执行后端** |

`/equations/:id/solvers` 把方法分成 `executable` / `literature_only` 两组，agent 决定"调用还是引用"时的依据就是这个字段。

---

## 何时调用哪个服务

### 知识库服务（knowledge-base，端口 3001）

| 用户意图 | 推荐调用路径 |
|---|---|
| "有哪些方法可以求解热方程/NS 方程/…？" | 先 `GET /search?q=<方程名>` 找到方程 id，再 `GET /equations/:id/solvers`（注意分 executable / literature_only 两组）|
| "哪些方法本地能直接跑？" | `GET /equations/:id/solvers` 后只看 `executable` 组；或对任意方法节点检查 `engine_id` 是否非空 |
| "FNO 是什么？能解哪些方程？" | `GET /ai-models/fno` 获取模型详情（含 `engine_id`），`GET /ai-models/fno/equations` 获取所支持方程 |
| "FEM 这个数值方法的相关论文有哪些？" | `GET /numerical-methods/fem`，再 `GET /numerical-methods/fem/papers` |
| "搜索关于 PINNs 的信息" | `GET /search?q=PINNs`（跨全部节点类型做名称搜索；返回字段是 `label` 不是 `node_type`） |
| "这个方程有哪些边界条件类型？" | `GET /equations/:id/conditions` |
| "FNO 的完整研究背景？" | `GET /ai-models/fno/profile`（多跳：solves + trained_by + evaluated_by + tested_on）|
| "有哪些 AI 模型采用 operator_learning 范式？" | `GET /ai-models?training_type=operator_learning` |
| "列出所有双曲型 PDE" | `GET /equations?pde_type=hyperbolic` |
| "某篇论文提出了哪些方法/研究了哪些方程？" | `GET /papers/:id/profile` |
| "在 PDEBench NS-2D 上谁排第一？" | `GET /benchmarks` 找到 benchmark id，再 `GET /benchmarks/:id/leaderboard`（看 `entries[0]`，注意 confidence 状态）|
| "FNO 在哪些 benchmark 上有数据？" | `GET /ai-models/fno/results`（按时间倒序） |
| "有哪些标准评测可用？" | `GET /benchmarks` |

### 求解器服务（engines，端口 3000）

| 用户意图 | 对应端点 |
|---|---|
| "帮我求解这个方程" | `POST /solve` |
| "有哪些可用的求解器？" | `GET /solvers` |
| "上传我的张量数据作为模型输入" | `POST /files`（multipart，字段名必须是 `file`），拿 `file_id` 后填进 `pde.history.file_id` |
| 任何需要实际计算、返回数值解的请求 | `POST /solve` |

---

## 推荐的调用顺序

### 场景一：用户提出 PDE 问题需要求解

```
1. GET /search?q=<方程关键词>          ← 知识库：找到方程节点及其 id
2. GET /equations/:id/solvers          ← 知识库：查询该方程关联的 AI 模型和数值方法
3. GET /ai-models/:id（可选）           ← 知识库：了解推荐 AI 模型的详细信息
4. POST /solve                         ← 求解器：提交求解任务
```

### 场景二：用户询问某类方程能用什么方法求解

```
1. GET /equations?pde_type=<parabolic|elliptic|hyperbolic>  ← 按类型列出方程
2. GET /equations/:id/solvers                               ← 查该方程的求解器列表
3. GET /ai-models/:id/profile（可选）                        ← 获取 AI 模型全貌
```

### 场景三：用户做方法调研（AI 方法 vs 数值方法）

```
1. GET /search?q=<方法关键词>          ← 跨类型搜索（可能命中 AIModel 或 NumericalMethod）
2. GET /ai-models/:id                  ← AI 模型详情
   或 GET /numerical-methods/:id       ← 数值方法详情
3. GET /ai-models/:id/papers           ← 查该方法的来源论文
   或 GET /numerical-methods/:id/papers
```

### 场景四：用户想了解某篇论文的研究内容

```
1. GET /search?q=<论文标题关键词>      ← 找到论文 id
2. GET /papers/:id                     ← 获取论文基本信息 + 摘要
3. GET /papers/:id/profile             ← 完整图谱：该论文提出/研究了哪些方法/方程
```

### 场景五：用户想在某个 benchmark 上对比方法

```
1. GET /benchmarks                          ← 列出可用评测口径
2. GET /benchmarks/:id/leaderboard          ← 实时排行榜
3. （可选）GET /ai-models/:id/results       ← 看某个具体方法的所有实测记录
```

排行榜的 `confidence` 字段值得注意：
- `verified` ✅：≥2 个独立来源、数值在容差内 → 可信
- `single`：只有 1 个来源 → 仅供参考
- `disputed` ⚠️：≥2 个独立来源但分歧大 → 不要直接采信，可能不同协议或实现差异

如果要"提交一条新测量"（比如本地复现的结果），调 `POST /internal/results`，详见 `knowledge-api.md`。

### 场景六：用户要用历史快照作为输入跑求解

```
1. POST /files (multipart, field="file")    ← 上传 .h5/.npy/.pt 张量文件
2. （取响应里的 file_id）
3. POST /solve                              ← 请求体里 pde.history.file_id 引用上一步 id
```

注意 `history` 与 `initial_condition` 互斥（前者优先），且 `classical` 求解器只取第一个变量通道。

---

## 不应调用的情况

- **用户只是在做一般性数学推导**（无需查库或求解）：直接用 LLM 自身能力回答。
- **问题与 PDE 无关**：不调用任何本服务。
- **用户明确指定了求解器**：跳过知识库查询，直接调用 `POST /solve` 并在请求体中设置 `solver` 字段。

---

## 子文档

详细的调用技巧请参阅：

- [`knowledge-api.md`](./knowledge-api.md) — 知识库各端点的参数、响应格式、枚举值、种子数据 ID 速查
- [`solve-api.md`](./solve-api.md) — `POST /solve` 请求体构造、初始条件格式、边界条件类型
