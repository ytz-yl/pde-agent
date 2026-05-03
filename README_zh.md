# PDE Agent 系统

为 AI Agent 提供偏微分方程（PDE）领域专业服务的基础设施。本项目不构建新的 Agent 框架，而是提供高质量的 PDE 领域能力，让任何 Agent 框架都能调用。

---

## 项目动机

现有的 Agent 框架（LangChain、AutoGen、CrewAI 等）已经足够成熟，问题不在于任务编排，而在于领域能力。当 Agent 面对 PDE 问题时，缺少的是：

- 结构化、持续更新的 PDE 理论与方法知识
- 覆盖多种场景的可靠数值求解器
- 清晰的方法选择指引与调用规范

本项目填补这一空白。

---

## 系统架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    Agent（外部框架）                      │
│          LangChain / AutoGen / CrewAI / 自定义           │
└────────────────────┬────────────────────────────────────┘
                     │  调用
         ┌───────────▼───────────┐
         │    PDE Agent 服务层    │
         │  ┌─────────────────┐  │
         │  │    知识库        │  │
         │  └─────────────────┘  │
         │  ┌─────────────────┐  │
         │  │   PDE 求解服务   │  │
         │  └─────────────────┘  │
         │  ┌─────────────────┐  │
         │  │    技能树        │  │
         │  └─────────────────┘  │
         └───────────────────────┘
                     │
         ┌───────────▼───────────┐
         │       前端界面         │
         └───────────────────────┘
```

系统由三个核心服务组件和一个前端层构成。

---

## 组件一：PDE 知识库

基于 Neo4j 构建的图结构知识库，将 PDE 理论、数值方法、AI 模型与研究文献组织为互联的知识图谱。

### 知识图谱模型

知识库以属性图表示 PDE 领域知识，包含九种节点类型和十五种关系类型：

**节点类型**：`Equation`（方程）、`Condition`（条件）、`Theorem`（定理）、`NumericalMethod`（数值方法）、`AIModel`（AI 模型）、`LossFunction`（损失函数）、`Metric`（评估指标）、`Dataset`（数据集）、`Paper`（论文）

**核心关系类型**：
- `SOLVES` — 连接求解器（AI 模型或数值方法）与其能处理的方程
- `HAS_CONDITION` — 连接方程与其边界/初始条件
- `TRAINED_BY` — 连接 AI 模型与其训练损失函数
- `EVALUATED_BY` / `TESTED_ON` — 连接 AI 模型与评估指标、基准数据集
- `PROPOSES` / `STUDIES` / `CITES` — 连接论文与其贡献的知识

### 存储设计

```
Neo4j（图数据库）
  └── 结构化字段：id、name、枚举类型、关系
      图遍历查询，Cypher 语言

SQLite（内容存储）
  └── 长文本：论文摘要、注释
      以 (node_id, node_type) 为主键
```

### 预置 Seed 数据

服务内置初始知识节点，覆盖：Heat / Wave / Poisson / Navier-Stokes / Burgers / Schrödinger / Allen-Cahn 方程；Dirichlet / Neumann / Periodic 边界条件；FDM / FEM / FVM / Spectral 数值方法；PINN / DeepONet / FNO / PDEformer / DeepXDE 模型；常用损失函数、评估指标和基准数据集——所有关系均已预先连接。

### API 接口

- **查询端点**：浏览方程、AI 模型、数值方法、论文；图关系遍历（方程的求解器、完整模型 Profile、论文引用图）
- **写入端点**（`/internal/`）：节点与关系的创建/更新/删除，供知识构建 Agent 调用
- **搜索**：对所有节点类型的名称进行全文搜索

---

## 组件二：PDE 求解服务

精选的先进 PDE 求解器库，以可调用服务形式对外暴露。Agent 无需自行实现求解器，直接调用即可。

### 求解器覆盖范围

| 类别 | 示例 |
|---|---|
| 经典数值方法 | FDM（显式/隐式）、FEM（线性/非线性）、FVM、谱方法 |
| 现代 ML 方法 | PINNs、DeepONet、傅里叶神经算子（FNO） |
| 混合方法 | 物理约束神经网络、自适应网格细化 + ML |
| 专用求解器 | Stokes / Navier-Stokes、薛定谔方程、Maxwell 方程、弹性方程等 |

### 服务接口设计

每个求解器采用统一接口封装：

```
输入：
  - PDE 描述（方程、求解域、边界/初始条件、参数）
  - 求解配置（方法、分辨率、精度容差、计算设备）

输出：
  - 解场（数值解或近似解析解）
  - 元数据（收敛信息、运行时间、误差估计）
  - 可视化就绪数据
```

求解器版本化管理，随领域发展持续引入新方法。

---

## 组件三：Agent Skill

层级化的技能规范，告诉 Agent **有哪些服务**、**何时使用**、**如何调用**。

位于 [`frontend/skills/pde-skill/`](./frontend/skills/pde-skill/)，包含以下文档：

- **[`SKILL.md`](./frontend/skills/pde-skill/SKILL.md)**：将用户意图映射到具体 API 端点，定义常见场景的推荐调用顺序
- **[`solve-api.md`](./frontend/skills/pde-skill/solve-api.md)**：`POST /solve` 请求体构造详解——方程字符串写法、初始条件格式、响应体解析
- **[`knowledge-api.md`](./frontend/skills/pde-skill/knowledge-api.md)**：知识库各端点详解——查询参数、过滤器用法、`/recommend` 的 `constraints` 关键词参考

Skill 文档也可通过前端界面的 **Skills** 标签页直接浏览和打包下载。

---

## 组件四：前端界面

单页 React + TypeScript 应用，让知识库和求解器服务对用户可见、可探索，支持**中英文切换**。

### 知识库界面

- 对论文和方法进行语义搜索与全文检索
- 按相关性评分排列的论文列表，含摘要预览、标签及原文链接
- 方法浏览器：列表、详情、相关方法、并排对比
- 基于 `POST /recommend` 的方法推荐表单

### 求解器界面

- 可用求解器目录，含支持的 PDE 类型说明
- 交互式表单，构建并提交 `POST /solve` 请求
- 结果可视化：各时间快照的解场热力图
- 元数据展示：所用求解器、耗时、后端信息

### Skills 界面

- 浏览 `frontend/skills/` 下所有 skill 包的文件树
- 支持 GitHub Flavored Markdown 的富文本预览
- 一键下载任意 skill 包为 `.zip` 压缩包

---

## 典型使用场景

### PDE 问题求解
Agent 收到问题：*"在不规则域上模拟带 Dirichlet 边界条件的二维热扩散"*。它查询技能树、选择 FEM、调用求解服务，返回带误差估计的解。

### PDE 研究辅助
Agent 被问及：*"高雷诺数下求解 Navier-Stokes 方程最有效的方法是什么？"* 它检索知识库中的最新论文，对比各方法，综合生成带引用的结构化回答。

### 方法基准测试
Agent 通过调用 `benchmark_methods` 对比 PINNs 与 FNO 在特定问题上的表现，结合求解服务的实验数据与知识库中的对比研究。

### 算法研究
Agent 通过知识检索（已有方法、已知失效模式）与求解基准测试（新方案的实证验证）相结合，对现有求解器提出改进方向。

---

## 项目目录结构

```
pde-agent/
├── knowledge_base/          # PDE 知识图谱服务（Rust）
│   └── src/
│       ├── store/           # Neo4j 节点/关系 CRUD、SQLite 内容库
│       ├── retrieval/       # 高层图遍历查询
│       └── api/             # axum HTTP 处理函数与路由
├── solvers/                 # PDE 求解器
│   ├── classical/           # FDM、FEM、FVM、谱方法
│   ├── ml/                  # PINNs、DeepONet、FNO
│   └── api/                 # 求解服务接口
├── frontend/                # 前端界面（React + TypeScript）
│   ├── skills/              # Agent skill 包
│   │   └── pde-skill/       # SKILL.md + 子指南
│   └── src/                 # 应用源码
├── start-neo4j.sh           # 启动 Neo4j 实例
├── start.sh                 # 一键启动脚本
└── README.md
└── README_zh.md
```

---

## 设计原则

- **服务优先，而非框架优先**：本项目提供服务而非 Agent 运行时，任何 Agent 框架均可通过 API 消费这些服务
- **结构化知识，而非原始检索**：论文不只是被索引，而是被分类、摘要、链接进连贯的知识结构
- **方法多样性**：经典数值方法与现代 ML 方法并重，不偏向任何单一范式
- **透明可查**：求解器界面展示具体实现，知识库展示原始论文，用户和 Agent 都能追溯来源
- **增量扩展**：新求解器和新论文持续添加，不破坏现有接口
