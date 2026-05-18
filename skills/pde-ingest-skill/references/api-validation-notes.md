# PDE 知识库 API 验证笔记

验证日期: 2026-05-03
服务地址: http://localhost:3001

---

## 服务健康检查

```
GET /health  →  {"service":"pde-knowledge-base","status":"ok"}
```

注意：根路径 GET / 返回 404，不要用它判断服务存活。

---

## 必填字段陷阱

### Paper 节点

`authors` 是必填字段。缺少时返回：

```
HTTP 500
{"error":"invalid node body: missing field `authors`"}
```

完整最小可写 Paper 示例：

```json
{
  "node_type": "paper",
  "id": "2010.08895",
  "title": "Fourier Neural Operator for Parametric PDEs",
  "authors": ["Li, Z.", "Kovachki, N."],
  "published_year": 2021,
  "arxiv_id": "2010.08895",
  "tags": [],
  "abstract": "...",
  "notes": "..."
}
```

响应（成功）：
```json
{"action":"upserted","id":"2010.08895","label":"Paper","status":"ok"}
```

---

## 关系写入验证

```json
POST /internal/relations
{
  "from_id": "fno",
  "from_label": "AIModel",
  "to_id": "navier_stokes",
  "to_label": "Equation",
  "relation_type": "SOLVES"
}
```

响应（成功）：
```json
{"action":"upserted","from":"AIModel:fno","relation":"SOLVES","status":"ok","to":"Equation:navier_stokes"}
```

---

## 删除节点验证

```
DELETE /internal/nodes/Paper/__healthcheck__
→ {"deleted":true,"status":"ok"}
```

---

## 健康检查写入/清理流程

批量录入前，可用此序列验证服务可用性：

```bash
# 1. 写入测试节点
curl -s http://localhost:3001/internal/nodes \
  -X POST -H "Content-Type: application/json" \
  -d '{"node_type":"paper","id":"__healthcheck__","title":"healthcheck",
       "authors":["test"],"published_year":2026,"arxiv_id":"__healthcheck__",
       "tags":[],"abstract":"healthcheck","notes":"healthcheck"}'

# 2. 确认返回 {"action":"upserted","status":"ok"}

# 3. 清理
curl -s http://localhost:3001/internal/nodes/Paper/__healthcheck__ -X DELETE
```

---

## Cron 任务设置（每日 UTC 14:30 自动摄取）

本知识库已配置 cron job（ID: 177b30183328），每天 UTC 14:30 触发，
自动搜索 arXiv 最新 PDE 论文并录入图谱。
加载 skills: pde-ingest-skill + arxiv，可用工具: web + terminal。

查看日志:
```
hermes cron list
hermes cron log 177b30183328
```

---

## 节点类型完整清单（schema.rs 同步）

当前 API 支持以下 9 种 node_type：

| node_type 值（写入时用）| Neo4j Label | 用途 |
|---|---|---|
| `paper` | Paper | 论文 |
| `ai_model` | AIModel | AI/ML 模型 |
| `numerical_method` | NumericalMethod | 经典数值方法 |
| `equation` | Equation | PDE 方程 |
| `condition` | Condition | 边界/初始条件 |
| `theorem` | Theorem | 数学定理 |
| `loss_function` | LossFunction | 损失函数 |
| `metric` | Metric | 评估指标 |
| `dataset` | Dataset | 基准数据集 |

**注意**：`KnowledgeNode` 枚举中 AIModel 的 tag 是 `ai_model`（snake_case），
写入时 `node_type` 字段也必须用 `ai_model`，不能用 `aimodel`。

---

## /equations/:id/solvers 接口响应结构（已验证）

`GET /equations/:id/solvers` 返回：
```json
{
  "equation_id": "navier_stokes",
  "equation_name": "Navier-Stokes",
  "executable": {
    "ai_models": [...],
    "numerical_methods": [...]
  },
  "literature_only": {
    "ai_models": [...],
    "numerical_methods": [...]
  }
}
```

返回结构按 `engine_id` 字段分两组：
- **`executable`**：方法节点的 `engine_id` 非空，意味着 engines API（`localhost:3000/solvers`）注册了对应后端。
  这一组里的方法可以**直接发到 `POST /solve`** 进行计算。
- **`literature_only`**：方法存在于知识图谱里（论文写过、有节点），但 engines 没注册，**只能查询不能跑**。

每组下仍按 `ai_models` / `numerical_methods` 分类，保留了"solver = AIModel + NumericalMethod 的统一视图"语义。
两组里的方法节点本身仍带 `engine_id` 字段，agent 拿到后直接读这个字段决定是否调用 solver API。

---

## /equations/:id/datasets 接口（关系方向注意）

通过 `Dataset -[:BASED_ON]-> Equation` 方向查询，
写入时关系方向是 **Dataset → Equation**（from_label=Dataset，to_label=Equation）。
常见错误：误以为是 Equation → Dataset。

---

## /ai-models/:id/profile 完整字段

```json
{
  "model": { ...AIModel 节点字段... },
  "solves": [{ "id", "name", "pde_type" }],
  "trained_by": [{ "id", "name", "loss_type" }],
  "evaluated_by": [{ "id", "name", "metric_type" }],
  "tested_on": [{ "id", "name", "dimension" }]
}
```

---

## 扩展新模型类型的注意事项

当前 `TrainingType` 枚举支持：
`supervised` | `unsupervised` | `self_supervised` | `physics_informed` | `operator_learning`

如未来需要支持新的训练范式（如 `reinforcement_learning`、`diffusion_based`），
需要在 Rust 源码 `schema.rs` 的 `TrainingType` 枚举中添加新变体，重新编译服务。
API 对未知 training_type 会 fallback 到 `supervised`（见 FromStr 实现），
不会报错但会静默丢失分类信息——注意这个行为。

同理，`NumericalMethodType` 的未知值会 fallback 到 `other`。
