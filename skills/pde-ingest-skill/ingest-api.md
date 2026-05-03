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
  "tags": ["operator_learning", "fourier", "mesh_invariant"]
}
```

`training_type` 合法值：`supervised` | `unsupervised` | `self_supervised` | `physics_informed` | `operator_learning`

### 写入 NumericalMethod（数值方法）

```json
{
  "node_type": "numerical_method",
  "id": "fem",
  "name": "Finite Element Method",
  "method_type": "mesh_based",
  "order": 2,
  "description": "Variational formulation on unstructured meshes; handles complex geometry.",
  "tags": ["classical", "mesh_based", "variational"]
}
```

`method_type` 合法值：`grid_based` | `mesh_based` | `spectral_based` | `mesh_free` | `other`

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
| `REPORTS_METRIC` | Paper → Metric | 该论文报告了该指标 |
| `CITES` | Paper → Paper | 该论文引用了另一论文 |

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
