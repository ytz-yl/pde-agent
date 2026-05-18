# 知识库写接口参考

写接口基础路径：`/internal/`（知识库服务，默认端口 3001）

所有写操作均使用 **MERGE 语义**，即"存在则更新，不存在则创建"，可以安全地重复调用。

---

## 接口总览

| 端点 | 方法 | 功能 |
|---|---|---|
| `/internal/nodes` | POST | 新增或更新任意节点 |
| `/internal/nodes/:label/:id` | DELETE | 删除节点（级联删除所有关联关系） |
| `/internal/relations` | POST | 新增或更新关系 |
| `/internal/relations` | DELETE | 删除特定关系 |
| `/internal/content` | POST | 仅更新节点的长文本（abstract/notes），不改动 Neo4j |

---

## POST `/internal/nodes` — 写入节点

请求体是一个 JSON 对象，必须包含 `node_type` 字段标识节点类型。

### 写入 Paper（论文）

`abstract` 和 `notes` 字段是可选的，写在请求体顶层，会被自动路由到 SQLite 存储。

```json
{
  "node_type": "paper",
  "id": "2010.08895",
  "title": "Fourier Neural Operator for Parametric Partial Differential Equations",
  "authors": ["Li, Z.", "Kovachki, N.", "Azizzadenesheli, K."],
  "published_year": 2021,
  "arxiv_id": "2010.08895",
  "doi": null,
  "pdf_path": null,
  "tags": ["operator_learning", "fourier"],
  "abstract": "We propose the Fourier neural operator that learns mappings between function spaces...",
  "notes": "Introduces FNO, benchmarked on Navier-Stokes 2D and Darcy Flow."
}
```

### 写入 AIModel（AI 模型）

```json
{
  "node_type": "ai_model",
  "id": "fno",
  "name": "Fourier Neural Operator",
  "architecture": "FNO",
  "input_vars": ["x", "y", "t"],
  "output_vars": ["u"],
  "training_type": "operator_learning",
  "description": "Learns solution operators in Fourier space, achieving mesh-invariant inference.",
  "paper_ref": "2010.08895",
  "tags": ["operator_learning", "fourier", "mesh_invariant"],
  "engine_id": null
}
```

`training_type` 合法值：`supervised` | `unsupervised` | `self_supervised` | `physics_informed` | `operator_learning`

> **`engine_id` 字段**：指向 engines API 注册的 solver id（运行 `GET http://localhost:3000/solvers` 查看；目前是 `pdeformer2` 和 `classical`）。**写入时给值，意味着声明这个方法本地可调用**；留空（或省略）= 仅文献存在、不可执行。预置的 `pdeformer` 节点的 `engine_id="pdeformer2"`，已可直接调用。

### 写入 NumericalMethod（数值方法）

```json
{
  "node_type": "numerical_method",
  "id": "fem",
  "name": "Finite Element Method",
  "method_type": "mesh_based",
  "order": 2,
  "description": "Variational formulation on unstructured meshes; handles complex geometry.",
  "tags": ["classical", "mesh_based", "variational"],
  "engine_id": null
}
```

`method_type` 合法值：`grid_based` | `mesh_based` | `spectral_based` | `mesh_free` | `other`

> **`engine_id` 字段**：同 AIModel，指向 engines API 的 solver id。预置的 `fdm` 节点 `engine_id="classical"`，对应 `engines/classical` 后端（py-pde 实现的 FDM/谱方法）。

### 写入 Equation（方程）

仅在需要录入知识库中尚未存在的新方程时使用（预置方程无需重新创建）。

```json
{
  "node_type": "equation",
  "id": "cahn_hilliard",
  "name": "Cahn-Hilliard Equation",
  "pde_type": "parabolic",
  "variables": ["t", "x", "y"],
  "time_dependent": true,
  "operator": "bilaplacian",
  "description": "Phase-field model for phase separation dynamics.",
  "tags": ["phase_field", "diffuse_interface"]
}
```

`pde_type` 合法值：`parabolic` | `elliptic` | `hyperbolic` | `mixed` | `other`

### 写入 Dataset（数据集）

```json
{
  "node_type": "dataset",
  "id": "ns2d_dataset",
  "name": "Navier-Stokes 2D Benchmark",
  "dimension": "2D",
  "num_samples": 1000,
  "description": "2D turbulent flow dataset generated with pseudo-spectral solver.",
  "url": "https://github.com/..."
}
```

### 写入 LossFunction（损失函数）

```json
{
  "node_type": "loss_function",
  "id": "pde_residual_loss",
  "name": "PDE Residual Loss",
  "loss_type": "physics",
  "formulation": "L = ||N[u](x,t)||^2 where N is the differential operator",
  "description": "Penalises violation of the PDE at collocation points."
}
```

`loss_type` 合法值：`physics` | `data_driven` | `boundary` | `combined` | `other`

### 写入 Metric（评估指标）

```json
{
  "node_type": "metric",
  "id": "l2_relative_error",
  "name": "L2 Relative Error",
  "metric_type": "accuracy",
  "unit": "dimensionless",
  "description": "||u_pred - u_true||_2 / ||u_true||_2"
}
```

`metric_type` 合法值：`accuracy` | `efficiency` | `stability` | `generalisation` | `other`

> **Metric 现在的角色**：只是"度量词表"，定义"L2 误差"是什么。具体在某个数据集上的某次测量值，请用下面的 BenchResult 节点。

### 写入 Benchmark（评测口径）

`Benchmark` 把一个 `Metric` 绑到一个 `Dataset` 上，再加上协议描述，构成一个可比较的评测单位。

```json
{
  "node_type": "benchmark",
  "id": "pdebench_ns2d_rel_l2",
  "name": "PDEBench NS-2D, Relative L2",
  "dataset_id": "navier_stokes_2d",
  "metric_id": "l2_error",
  "lower_is_better": true,
  "protocol": "Kolmogorov flow, 64x64, viscosity 1e-3, t=10s rollout",
  "tolerance": 0.05
}
```

- `dataset_id` 和 `metric_id` 必须指向已存在的 Dataset / Metric 节点。
- 写入时**自动**建立 `(Benchmark)-[:ON_DATASET]->(Dataset)` 和 `(Benchmark)-[:USES_METRIC]->(Metric)` 关系，不要重复手写。
- `tolerance` 是相对容差（`(max-min)/|mean|`），用于多源校验时判断"verified vs disputed"。缺省 0.05。
- 长版本协议（完整复现说明）放 `notes` 字段，会被路由到 SQLite。

### 写入 BenchResult（一次具体测量）

每条 BenchResult 表示**一个方法在一个 Benchmark 上一次具体的实测值**，永远 append，从不覆盖。

```json
{
  "node_type": "bench_result",
  "method_id": "fno",
  "method_label": "AIModel",
  "benchmark_id": "pdebench_ns2d_rel_l2",
  "value": 0.012,
  "source_type": "paper_reported",
  "source_paper_id": "2010.08895",
  "hardware": "1x A100 40G",
  "code_ref": "https://github.com/zongyi-li/fourier_neural_operator"
}
```

- `id` 字段省略时**自动生成**（`{method}__{benchmark}__{src}__{nanos}`），返回值会带上。
- `source_type` 合法值：`paper_reported` | `self_run` | `third_party_reproduction`。
- `paper_reported` / `third_party_reproduction` **必须**带 `source_paper_id`，否则返回 500（校验失败）。
- `method_label` 必须是 `"AIModel"` 或 `"NumericalMethod"`。
- 写入时**自动**建立三条边：`OF_METHOD`、`ON_BENCHMARK`、`REPORTED_IN`（仅当 `source_paper_id` 非空），不要再手动 POST 关系。
- 推荐用下面的 `POST /internal/results` 简化端点，入参更直观，单次调用即可。

---

## POST `/internal/relations` — 写入关系

关系体现了图谱的核心价值，每条关系都有明确的方向。

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

### 关系类型完整参考表

| relation_type | 方向（from → to） | 语义 |
|---|---|---|
| `SOLVES` | AIModel / NumericalMethod → Equation | 该方法能求解该方程 |
| `REQUIRES` | 任意 → 任意 | 该节点依赖另一节点 |
| `HAS_CONDITION` | Equation → Condition | 该方程关联该边界/初始条件 |
| `APPLIES_TO` | 方法 → Equation | 该方法适用于该方程（弱于 SOLVES） |
| `TRAINED_BY` | AIModel → LossFunction | 该模型使用该损失函数训练 |
| `EVALUATED_BY` | AIModel → Metric | 该模型用该指标评估 |
| `REPRESENTS` | LossFunction → Equation | 该损失函数编码了该 PDE 的物理约束 |
| `BASED_ON` | Dataset → Equation | 该数据集基于该方程生成 |
| `TESTED_ON` | AIModel → Dataset | 该模型在该数据集上测试 |
| `VARIANT_OF` | AIModel / NumericalMethod → 同类型 | 该方法是另一方法的变体 |
| `PROPOSES` | Paper → AIModel / NumericalMethod | 该论文提出了该方法 |
| `STUDIES` | Paper → Equation | 该论文研究了该方程 |
| `USES_DATASET` | Paper → Dataset | 该论文使用了该数据集 |
| `REPORTS_METRIC` | Paper → Metric | 该论文报告了哪些指标**类型**（不含数值；具体数值用 BenchResult） |
| `CITES` | Paper → Paper | 该论文引用了另一论文 |
| `ON_DATASET` | Benchmark → Dataset | 评测在哪个数据集上做（Benchmark 写入时自动建立） |
| `USES_METRIC` | Benchmark → Metric | 评测使用哪个指标（Benchmark 写入时自动建立） |
| `OF_METHOD` | BenchResult → AIModel / NumericalMethod | 这次测量是哪个方法（BenchResult 写入时自动建立） |
| `ON_BENCHMARK` | BenchResult → Benchmark | 这次测量针对哪个评测（BenchResult 写入时自动建立） |
| `REPORTED_IN` | BenchResult → Paper | 这次测量来自哪篇论文（自动，仅当 source_paper_id 非空） |

**注意**：`relation_type` 必须使用上表中的大写字符串，传入其他值会收到 400 错误并返回合法值列表。

### 常用关系示例

```json
// FNO 求解 Navier-Stokes
{ "from_id": "fno", "from_label": "AIModel",
  "to_id": "navier_stokes", "to_label": "Equation", "relation_type": "SOLVES" }

// 论文提出 FNO
{ "from_id": "2010.08895", "from_label": "Paper",
  "to_id": "fno", "to_label": "AIModel", "relation_type": "PROPOSES" }

// 论文研究 Navier-Stokes
{ "from_id": "2010.08895", "from_label": "Paper",
  "to_id": "navier_stokes", "to_label": "Equation", "relation_type": "STUDIES" }

// FNO 在 NS 数据集上测试
{ "from_id": "fno", "from_label": "AIModel",
  "to_id": "ns2d_dataset", "to_label": "Dataset", "relation_type": "TESTED_ON" }

// FNO 用 PDE 残差损失训练
{ "from_id": "fno", "from_label": "AIModel",
  "to_id": "pde_residual_loss", "to_label": "LossFunction", "relation_type": "TRAINED_BY" }
```

---

## POST `/internal/results` — 单步提交一次测量（推荐）

提交一条 BenchResult，所有关联边（OF_METHOD / ON_BENCHMARK / REPORTED_IN）一次写完，比走通用 `/internal/nodes` + 三次 `/internal/relations` 快得多。

```json
POST /internal/results
{
  "method_id": "fno",
  "method_label": "AIModel",
  "benchmark_id": "pdebench_ns2d_rel_l2",
  "value": 0.012,
  "source_type": "paper_reported",
  "source_paper_id": "2010.08895",
  "hardware": "1x A100 40G",
  "code_ref": "https://github.com/zongyi-li/fourier_neural_operator",
  "notes": "复现脚本：scripts/run_fno_ns2d.sh，commit a1b2c3"
}
```

返回：

```json
{ "status": "ok", "action": "upserted", "label": "BenchResult",
  "id": "fno__pdebench_ns2d_rel_l2__paper__1850e3f0..." }
```

字段约束：
- `source_type` ∈ {`paper_reported`, `self_run`, `third_party_reproduction`}
- `paper_reported` / `third_party_reproduction` 必须带 `source_paper_id`
- `method_label` 必须是 `"AIModel"` 或 `"NumericalMethod"`
- `recorded_at` 缺省时服务端写入当前时间
- `notes` 字段路由到 SQLite，记录复现信息

录入完后用 `GET /benchmarks/<benchmark_id>/leaderboard` 查看实时榜单和 confidence 状态（`single` / `verified` / `disputed`）。

---

## POST `/internal/content` — 仅更新长文本

当节点已存在、只想补充或修改摘要/注释时，用此接口，不触碰 Neo4j。

```json
{
  "node_id": "2010.08895",
  "node_type": "Paper",
  "abstract_text": "We propose the Fourier neural operator...",
  "notes": "Key result: 1000x speedup over traditional solvers on NS-2D."
}
```

---

## DELETE `/internal/nodes/:label/:id` — 删除节点

删除节点会**级联删除**所有关联关系，谨慎使用。

```
DELETE /internal/nodes/AIModel/fno
DELETE /internal/nodes/Paper/2010.08895
```

---

## DELETE `/internal/relations` — 删除关系

只删除关系，不删除节点。

```json
{
  "from_label": "AIModel",
  "from_id": "fno",
  "to_label": "Equation",
  "to_id": "navier_stokes",
  "relation_type": "SOLVES"
}
```

---

## 预置节点 id 速查（可直接建立关系，无需重新创建）

### 方程
| id | 名称 |
|---|---|
| `heat_equation` | Heat Equation |
| `wave_equation` | Wave Equation |
| `poisson_equation` | Poisson Equation |
| `navier_stokes` | Navier-Stokes |
| `burgers_equation` | Burgers Equation |
| `schrodinger_equation` | Schrödinger Equation |
| `allen_cahn` | Allen-Cahn Equation |

### AI 模型
| id | 名称 |
|---|---|
| `pinns` | Physics-Informed Neural Networks |
| `deeponet` | Deep Operator Network |
| `fno` | Fourier Neural Operator |
| `pdeformer` | PDEformer |
| `deepxde` | DeepXDE |

### 数值方法
| id | 名称 |
|---|---|
| `fdm` | Finite Difference Method |
| `fem` | Finite Element Method |
| `fvm` | Finite Volume Method |
| `spectral` | Spectral Method |

### 数据集
| id | 名称 |
|---|---|
| `burgers1d_dataset` | Burgers 1D |
| `ns2d_dataset` | Navier-Stokes 2D |
| `heat2d_dataset` | Heat 2D |
| `darcy_flow_dataset` | Darcy Flow |

### 损失函数
| id | 名称 |
|---|---|
| `pde_residual_loss` | PDE Residual Loss |
| `boundary_condition_loss` | Boundary Condition Loss |
| `data_mse_loss` | Data MSE Loss |
| `combined_pinn_loss` | Combined PINN Loss |

### 评估指标
| id | 名称 |
|---|---|
| `l2_relative_error` | L2 Relative Error |
| `linf_error` | L-infinity Error |
| `mse` | Mean Squared Error |
| `inference_time` | Inference Time |

### 评测口径（Benchmark）
| id | 数据集 | 指标 | 协议简述 |
|---|---|---|---|
| `pdebench_burgers1d_rel_l2` | `burgers_1d` | `l2_error` | PDEBench split, viscosity 0.01, 256 grid points, t=1.0 |
| `pdebench_ns2d_rel_l2` | `navier_stokes_2d` | `l2_error` | Kolmogorov flow, 64x64, viscosity 1e-3, t=10s |
| `pdebench_darcy_rel_l2` | `darcy_flow` | `l2_error` | 421x421 grid, log-normal permeability, steady state |

> 录入新的 Benchmark 时：先确认 dataset_id 和 metric_id 已存在；标准评测优先复用上面三个，避免分裂同一个评测的样本量。
