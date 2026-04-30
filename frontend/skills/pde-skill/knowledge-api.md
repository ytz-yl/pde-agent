# 知识库 API 调用技巧

端点基础路径：知识库服务（默认端口 3001）

---

## 端点速览

| 端点 | 方法 | 功能 |
|---|---|---|
| `/search` | GET | 语义 + 全文混合检索论文 |
| `/papers/recent` | GET | 按领域获取最新论文 |
| `/papers/{id}` | GET | 获取单篇论文详情 |
| `/methods` | GET | 列出所有已知方法 |
| `/methods/{id}` | GET | 获取单个方法详情 |
| `/methods/{id}/related` | GET | 获取相关方法列表 |
| `/methods/compare` | GET | 对比两个方法 |
| `/recommend` | POST | 根据 PDE 类型推荐方法 |
| `/ingest/paper` | POST | 手动录入单篇论文 |
| `/ingest/fetch-arxiv` | POST | 触发 arXiv 批量抓取 |

---

## `/search` — 混合语义检索

```
GET /search?q=<query>&pde_type=&method=&domain=&limit=10&hybrid=true
```

### 参数说明

| 参数 | 类型 | 说明 |
|---|---|---|
| `q` | string（必填） | 自然语言查询，如 `"inverse problem with PINNs"` |
| `pde_type` | string（可选） | PDE 类型过滤，如 `"navier_stokes"`、`"heat_equation"` |
| `method` | string（可选） | 方法过滤，如 `"fem"`、`"fno"` |
| `domain` | string（可选） | 应用领域过滤，如 `"fluid_dynamics"`、`"elasticity"` |
| `limit` | int（默认 10） | 返回结果数量上限 |
| `hybrid` | bool（默认 true） | 启用混合检索（向量 0.7 + 全文 0.3 加权融合） |

### 技巧

- `hybrid=true` 时，向量相似度权重 0.7，全文匹配权重 0.3，适合大多数场景
- 同时指定多个过滤参数（如 `pde_type` + `method`）时，结果为交集过滤后再做语义排序
- `limit` 建议不超过 20，超大结果集意义有限且响应慢
- 查询词越具体效果越好，如 `"Fourier neural operator turbulence"` 优于 `"deep learning PDE"`

### 响应格式

```json
[
  {
    "score": 0.87,
    "paper": {
      "id": "2301.12345",
      "title": "...",
      "abstract_text": "...",
      "authors": ["..."],
      "published": "2023-01-20T00:00:00Z",
      "source_url": "https://arxiv.org/abs/...",
      "tags": [...]
    }
  }
]
```

---

## `/papers/recent` — 最新论文

```
GET /papers/recent?domain=<domain>&limit=10
```

| 参数 | 说明 |
|---|---|
| `domain` | 可选，按领域过滤，如 `"fluid_dynamics"` |
| `limit` | 返回数量，默认 10 |

按 `published` 时间降序排列。适合用于"最近有什么新进展"类问题。

---

## `/methods` — 方法列表

```
GET /methods?category=<category>
```

| `category` 值 | 含义 |
|---|---|
| `classical` | 经典数值方法（FDM、FEM、FVM、谱方法） |
| `ml` | 机器学习方法（PINNs、DeepONet、FNO） |
| `hybrid` | 混合方法 |
| 省略 | 返回所有方法 |

---

## `/methods/{id}` — 方法详情

常用方法 ID：

| ID | 方法名 |
|---|---|
| `fdm` | 有限差分法（FDM） |
| `fem` | 有限元法（FEM） |
| `fvm` | 有限体积法（FVM） |
| `spectral` | 谱方法 |
| `pinns` | 物理信息神经网络 |
| `deeponet` | Deep Operator Network |
| `fno` | 傅里叶神经算子 |
| `pdeformer` | PDEformer |

---

## `/methods/{id}/related` — 相关方法

```
GET /methods/fem/related
```

响应包含关联方法及其关系类型（如 `"generalization"`、`"alternative"`、`"extension"`）和权重。

**技巧**：用于向用户展示方法生态或在推荐后补充替代方案。

---

## `/methods/compare` — 方法对比

```
GET /methods/compare?a=fem&b=fno
```

返回 `ComparisonReport`，包含两个方法的详情、它们之间的关系边及自动生成的摘要文本。

**技巧**：当用户问"A 和 B 哪个更好"时，先调用此端点获取结构化对比，再结合 `/recommend` 的场景评分给出建议。

---

## `/recommend` — 方法推荐

```
POST /recommend
Content-Type: application/json

{
  "pde_type": "navier_stokes",
  "domain": "fluid_dynamics",
  "constraints": ["irregular_domain", "high_accuracy"],
  "top_k": 3
}
```

### 字段说明

| 字段 | 类型 | 说明 |
|---|---|---|
| `pde_type` | string（必填） | PDE 类型，见下方枚举 |
| `domain` | string（可选） | 应用领域提示 |
| `constraints` | string[]（可选） | 附加约束关键词，见下方枚举 |
| `top_k` | int（默认 3） | 返回推荐数量 |

### `pde_type` 常用值

`navier_stokes`、`fluid_dynamics`、`heat_equation`、`diffusion`、`wave_equation`、`hyperbolic`、`poisson`、`elliptic`

### `constraints` 常用值

| 值 | 语义 |
|---|---|
| `irregular_domain` | 不规则计算域（推高 FEM、PINNs） |
| `inverse_problem` | 反问题（推高 ML 方法） |
| `high_dimensional` | 高维问题（推高 ML 方法） |
| `parametric` | 参数化/多查询场景（推高 ML 方法） |
| `fast_inference` | 需要快速推断（推高 ML 方法） |
| `high_accuracy` | 高精度要求（推高经典方法） |
| `complex_geometry` | 复杂几何（推高 FEM） |
| `conservation_laws` | 守恒律问题（推高 FVM、FDM） |
| `guaranteed_convergence` | 需要收敛保证（推高经典方法） |

### 响应格式

```json
[
  {
    "method": { "id": "fem", "name": "Finite Element Method", ... },
    "reason": "FEM handles unstructured/irregular meshes natively; Classical methods are preferred for accuracy/convergence guarantees",
    "score": 0.7
  }
]
```

**技巧**：`reason` 字段可直接作为向用户解释推荐原因的文本。

---

## `/ingest/fetch-arxiv` — 触发 arXiv 抓取

```
POST /ingest/fetch-arxiv
Content-Type: application/json

{
  "query": "physics-informed neural networks PDE",
  "max_results": 25
}
```

这是一个耗时较长的操作（涉及网络请求 + LLM 标注）。通常用于知识库维护，不建议在用户交互的关键路径上同步调用。
