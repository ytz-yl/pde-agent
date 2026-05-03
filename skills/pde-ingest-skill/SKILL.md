---
name: pde-ingest-skill
description: Use this skill when the user wants to add, update, or enrich knowledge in the PDE knowledge base — including ingesting a research paper (from arXiv, URL, or PDF), adding a new AI model or numerical method, recording relationships between methods and equations, or systematically surveying a PDE research area and writing structured findings into the graph. Use this skill whenever the task involves writing to the knowledge base, not just querying it.
---

# PDE Knowledge Ingestion Skill

本 skill 指导 agent 如何发现、分析、整合 PDE 领域知识，并通过写接口将其录入知识库图谱。

知识库是一个 Neo4j 图数据库，核心不是"存文章"，而是**建立结构化节点和它们之间的有向关系**。录入一篇论文的终点不是把 PDF 存进去，而是让图谱知道：这篇论文**提出了**哪个模型、**研究了**哪个方程、**使用了**哪个数据集。

详细写接口参数见 [`ingest-api.md`](./ingest-api.md)。

---

## 整体流程

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

**节点类型选择**
- 新的神经网络/ML 方法 → `AIModel`（`node_type: "ai_model"`）
- FDM/FEM/FVM 等经典方法 → `NumericalMethod`（`node_type: "numerical_method"`）
- 新发现的方程变体 → `Equation`
- 论文 → `Paper`

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
```

所有写操作均为 MERGE 语义（幂等），重复执行不会造成重复节点或关系。

---

## 第六步：验证

写入后通过读接口确认结果：

```
GET /search?q=<刚写入节点的名称>        ← 确认节点可被搜索到
GET /papers/:id/profile                 ← 确认论文的关系图谱完整
GET /ai-models/:id/equations            ← 确认 SOLVES 关系已建立
```

如果回读结果与预期不符，检查 id 是否正确、关系方向是否正确（见 `ingest-api.md` 关系方向表）。

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
