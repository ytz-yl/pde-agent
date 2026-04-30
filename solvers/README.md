# Solvers — 架构文档

本文档说明 `solvers/` 目录的整体结构、各子模块的职责，以及 `api/`、`classical/`、`ml/` 三者之间的关系。

---

## 目录结构

```
solvers/
├── api/                        # Rust HTTP 服务（统一入口）
│   ├── src/
│   │   ├── main.rs             # axum 路由注册、启动
│   │   ├── models/             # 请求/响应类型定义（SolveRequest, SolveResponse …）
│   │   ├── error/              # ApiError + IntoResponse
│   │   ├── routes/
│   │   │   ├── health.rs       # GET /health
│   │   │   ├── solvers.rs      # GET /solvers
│   │   │   └── solve.rs        # POST /solve
│   │   └── solvers/
│   │       ├── mod.rs          # Solver trait + SolverRegistry
│   │       ├── pdeformer2.rs   # ML backend: PDEformer-2
│   │       └── classical.rs    # Classical backend: py-pde
│   └── scripts/
│       ├── pdeformer2_infer.py # Python bridge — PDEformer-2 推理
│       └── classical_solve.py  # Python bridge — py-pde 求解
│
├── classical/                  # 传统数值方法（git submodule）
│   └── py-pde/                 # FDM/谱方法库（zwicker-group/py-pde）
│
└── ml/                         # 机器学习方法（git submodule）
    └── pdeformer-2/            # 2D PDE 基础模型（mindspore-ai/pdeformer-2）
```

---

## 三层关系

```
  外部调用者（Agent / 用户 / 前端）
         │
         │  HTTP JSON
         ▼
  ┌──────────────────────────────────────────┐
  │            solvers/api/                  │
  │   Rust · axum · 异步 · 统一接口           │
  │                                          │
  │  POST /solve                             │
  │    └─ SolverRegistry.get(solver_id)      │
  │         │                                │
  │    ┌────┴─────────────────────┐          │
  │    │                          │          │
  │  pdeformer2.rs            classical.rs   │
  │  (ML backend)         (Classical backend)│
  │    │                          │          │
  │    │ spawn subprocess         │          │
  └────┼──────────────────────────┼──────────┘
       │                          │
       ▼                          ▼
  scripts/                   scripts/
  pdeformer2_infer.py        classical_solve.py
       │                          │
       ▼                          ▼
  ml/pdeformer-2/src/        classical/py-pde/
  (MindSpore, 基础模型)       (FDM / 谱方法)
```

### 设计原则

**api/** 是唯一的 HTTP 入口，对外暴露统一接口。它本身不做数值计算，只负责：
- 路由分发（根据 `solver` 字段选择 backend）
- 参数校验与序列化
- 并发管理（tokio async）
- 错误处理与日志

**classical/** 和 **ml/** 是纯粹的计算资产，以 git submodule 形式管理：
- 版本锁定：submodule commit 固定，保证计算结果可复现
- 独立演进：可分别升级到新版本，不影响 API 层
- 不对外暴露：所有访问都经过 api/ 层的 bridge 脚本

**Bridge 脚本**（`api/scripts/`）是连接层，每个 backend 对应一个 Python 脚本：
- 从 `stdin` 读取 JSON 格式的 `SolveRequest`
- 导入对应的计算库，执行求解
- 将结果以 JSON 写入 `stdout`
- Rust 进程通过 `tokio::process::Command` spawn 管理其生命周期

---

## HTTP API 参考

### `GET /health`

```json
{
  "success": true,
  "data": {
    "status": "ok",
    "version": "0.1.0",
    "solvers_available": ["pdeformer2", "classical"]
  }
}
```

### `GET /solvers`

返回所有注册 solver 的元数据列表（id、名称、分类、支持的 PDE 类型、backend）。

### `POST /solve`

**请求体：**

```json
{
  "solver": "classical",
  "pde": {
    "equation": "u_t = d * laplace(u)",
    "boundary_condition": "periodic",
    "parameters": { "d": 0.1 }
  },
  "query": {
    "x": [0.0, 0.03125, ..., 1.0],
    "y": [0.0, 0.03125, ..., 1.0],
    "t": [0.0, 0.25, 0.5, 0.75, 1.0]
  }
}
```

| 字段 | 类型 | 说明 |
|---|---|---|
| `solver` | string（可选） | backend id，默认 `"pdeformer2"` |
| `pde.equation` | string | 方程描述（支持格式见各 backend 说明） |
| `pde.initial_condition` | float[] （可选） | 128×128 flat row-major 初始场 |
| `pde.boundary_condition` | string（可选） | `"periodic"` \| `"dirichlet"` \| `"neumann"` |
| `pde.parameters` | object（可选） | 方程中的标量参数 |
| `query.x` | float[] | 求解的 x 坐标列表 |
| `query.y` | float[] | 求解的 y 坐标列表 |
| `query.t` | float[]（可选） | 求解的时间快照，默认 `[0, 0.25, 0.5, 0.75, 1.0]` |

**响应体：**

```json
{
  "success": true,
  "data": {
    "solver_used": "classical",
    "solution": [[[[...]]]],
    "shape": { "n_t": 5, "n_x": 32, "n_y": 32, "n_vars": 1 },
    "metadata": {
      "wall_time_ms": 320,
      "backend": "py-pde 0.42 / FDM",
      "notes": []
    }
  },
  "request_id": "...",
  "timestamp": "..."
}
```

`solution` 的 shape 为 `[n_t][n_x][n_y][n_vars]`。

---

## Backend 说明

### `pdeformer2`（ml/pdeformer-2）

| 属性 | 值 |
|---|---|
| 类别 | Machine Learning |
| 框架 | MindSpore 2.8 |
| 参数量 | 82.65M（L）/ 71.07M（M）/ 27.75M（S） |
| 支持 PDE 类型 | 任意符号表达式可描述的 2D PDE |
| 边界条件 | 周期 / Dirichlet / Neumann / 混合 |
| 求解坐标 | 任意时空坐标（网格无关） |
| 硬件要求 | CPU（推理）/ Ascend NPU（训练） |
| checkpoint | `solvers/ml/pdeformer-2/results/PDEformer2-L/model-L.ckpt` |
| bridge 脚本 | `api/scripts/pdeformer2_infer.py` |

**方程格式（DSL）：**

| 关键词 | 识别的方程 |
|---|---|
| 含 `u^2` | 非线性守恒律 `u_t + (u²)_x + (c·u)_y = 0` |
| 含 `u_xx` / `laplacian` | 热扩散 `u_t - d·∇²u = 0` |
| 含 `u_x` | 线性对流 `u_t + c·u_x + u_y = 0` |
| 其他 | 回退至非线性守恒律（默认） |

---

### `classical`（classical/py-pde）

| 属性 | 值 |
|---|---|
| 类别 | Classical Numerical |
| 框架 | py-pde（FDM + 谱方法）|
| 数值方法 | 有限差分（显式/隐式）、谱方法 |
| 支持 PDE 类型 | 扩散、波动、Allen-Cahn、Cahn-Hilliard、自定义 |
| 边界条件 | 周期 / Dirichlet / Neumann |
| 求解坐标 | 均匀笛卡尔网格，结果插值到查询点 |
| 硬件要求 | CPU（numba JIT） |
| bridge 脚本 | `api/scripts/classical_solve.py` |

**方程格式（py-pde 表达式）：**

| `equation` 关键词 | 识别方式 | py-pde 类 |
|---|---|---|
| `diffusion` / `heat` / `laplace` | 字符串包含 | `DiffusionPDE(diffusivity=d)` |
| `wave` | 字符串包含 | `WavePDE(speed=c)` |
| `allen-cahn` / `allencahn` | 字符串包含 | `AllenCahnPDE(...)` |
| 其他 | 作为符号表达式 | `PDE({'u': equation})` |

---

## 扩展指南

### 添加新的 classical solver

1. 在 `solvers/classical/` 添加新的 git submodule
2. 在 `api/scripts/` 新建对应 bridge 脚本（读 stdin JSON，写 stdout JSON）
3. 在 `api/src/solvers/` 新建 `xxx.rs`，实现 `Solver` trait
4. 在 `api/src/solvers/mod.rs` 的 `SolverRegistry::new()` 中注册

### 添加新的 ML solver

同上，ml backend 通常需要额外处理 checkpoint 路径（`PDEFORMER2_DIR`、`PYTHON` 等环境变量）。

### 环境变量

| 变量 | 默认值 | 说明 |
|---|---|---|
| `LISTEN_ADDR` | `0.0.0.0:8080` | API 监听地址 |
| `PDEFORMER2_DIR` | `../ml/pdeformer-2`（相对于 Cargo.toml）| pdeformer-2 仓库根目录 |
| `PDEFORMER2_PYTHON` | `$HOME/miniconda3/envs/pdeformer2/bin/python` | pdeformer2 环境 Python |
| `CLASSICAL_PYTHON` | `$HOME/miniconda3/envs/pdeformer2/bin/python` | classical 环境 Python（默认复用 pdeformer2 env）|
| `RUST_LOG` | `info` | 日志级别 |
