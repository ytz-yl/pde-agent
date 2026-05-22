# 求解器 API 调用技巧

求解器服务（默认端口 3000；代码仓库目录已从 `solvers/` 重命名为 `engines/`，但接口和端口未变）有三个端点：

| 端点 | 方法 | 用途 |
|---|---|---|
| `/solvers` | GET | 列出可用求解器（`pdeformer2` / `classical`），见下文 |
| `/files` | POST | 上传张量文件（.npy/.npz/.h5/.pt），返回 `file_id` 供 `/solve` 引用 |
| `/solve` | POST | 提交求解任务，本文重点 |

**典型流程**：
- **不带历史输入**：直接 `POST /solve`（简单 PDE 用 IC 描述初值）。
- **带历史输入**（数据驱动 / 自回归模型）：先 `POST /files` 拿 `file_id` → 再在 `/solve` 请求体的 `pde.history.file_id` 引用它。

---

## 请求体结构

```json
{
  "solver": "pdeformer2",
  "pde": {
    "equation": "u_t + (u^2)_x + (-0.3*u)_y = 0",
    "initial_condition": [...],
    "boundary_condition": "periodic",
    "parameters": {}
  },
  "query": {
    "x": [0.0, 0.03125, ...],
    "y": [0.0, 0.03125, ...],
    "t": [0.0, 0.25, 0.5, 0.75, 1.0]
  },
  "options": {}
}
```

---

## 字段说明

### `solver`（可选）

指定求解器名称，省略时默认使用 `"pdeformer2"`。

可用求解器列表通过 `GET /solvers` 查询，返回格式：

```json
[
  {
    "id": "pdeformer2",
    "name": "PDEformer-2",
    "category": "machine_learning",
    "description": "...",
    "supported_pde_types": ["elliptic", "parabolic", "hyperbolic", "nonlinear_conservation_law", "reaction_diffusion", "navier_stokes"],
    "backend": "MindSpore / Python",
    "available": true
  },
  {
    "id": "classical",
    "name": "Classical FDM (py-pde)",
    "category": "classical",
    "description": "...",
    "supported_pde_types": ["diffusion", "heat", "wave", "allen_cahn", "cahn_hilliard", "custom_symbolic"],
    "backend": "py-pde / FDM / Python",
    "available": true
  }
]
```

**技巧**：在调用 `/solve` 前先调用 `GET /solvers`，确认目标求解器的 `available` 字段为 `true`。

当前注册的求解器：

| id | 类别 | 适用场景 |
|---|---|---|
| `pdeformer2` | machine_learning | 任意形式的 2D PDE，通用首选；支持 `history` 输入 |
| `classical` | classical | 热/波/Allen-Cahn/Cahn-Hilliard 及任意符号 PDE，需精确数值解 |

---

## `POST /files` —— 上传张量文件

数据驱动模型常常需要把过往时间步的解场作为输入（autoregressive rollout 起点）。流程是先把张量文件上传到求解器服务，拿到 `file_id`，再在 `/solve` 里引用。

### 请求

`Content-Type: multipart/form-data`，**字段名必须是 `file`**（拼错会返回 400）：

```bash
curl -X POST http://localhost:3000/files \
  -F "file=@history.h5"
```

支持的格式（按扩展名识别）：`.h5` / `.hdf5` / `.npy` / `.npz` / `.pt` / `.pth`。其它扩展名直接返回 400。

### 响应

```json
{
  "success": true,
  "data": {
    "file_id": "3f2a1b8e-7d4c-4a92-9f31-c0dd1e7e8f22",
    "filename": "history.h5",
    "format": "hdf5",
    "size_bytes": 204800,
    "path": "/tmp/pde-solver-uploads/3f2a1b8e-...h5"
  },
  "request_id": "...",
  "timestamp": "..."
}
```

把 `data.file_id` 拿出来，填到下文 `pde.history.file_id` 即可。

### 存储与生命周期

- 默认存到 `$SOLVER_UPLOAD_DIR`（缺省 `/tmp/pde-solver-uploads/`），保留原扩展名。
- 服务**没有显式 GC**，进程重启后只要文件还在磁盘上，旧 `file_id` 仍可用（基于扩展名扫描映射）。
- 不要假设 `file_id` 跨机器/跨实例共享；它只在生成它的那个求解器服务实例里有意义。

---

### `pde` 字段详解

`pde` 字段支持两种使用模式，向后兼容：

---

#### 模式一：单变量（传统写法）

适用于只有一个未知场变量的问题。

| 字段 | 类型 | 说明 |
|---|---|---|
| `equation` | string（必填） | 方程字符串，见下方写法说明 |
| `initial_condition` | float[] \| null | 扁平化 128×128 网格，行优先，长度 16384 |
| `boundary_condition` | string \| null | `"periodic"` \| `"dirichlet"` \| `"neumann"` |
| `parameters` | object \| null | 方程中的自由标量参数，如 `{"nu": 0.001}` |

---

#### 模式二：多变量 / 多方程（新增）

适用于联立方程组（如 SWE、NS 速度-压力分解等）。

| 字段 | 类型 | 说明 |
|---|---|---|
| `variables` | string[] | 未知场变量名列表，如 `["u","v","p"]` |
| `equations` | string[] | 约束方程列表，与 `variables` 对应 |
| `initial_conditions` | map | 变量名 → IC 数组 / `"zero"` / `"grf"` |
| `coef_fields` | map | 方程中引用的系数场，变量名 → 128×128 数组 |
| `domains` | SdfDomain[] | SDF 域定义，用于复杂几何边界 |
| `bcs` | BcSpec[] | 显式边界条件列表 |

**`initial_conditions` 的值类型（`IcValue`）：**
- 数组：`[0.1, 0.2, ...]` — 平坦的 n×n 网格值
- `"zero"` — 全零场（快捷方式）
- `"grf"` — 高斯随机场采样

**`SdfDomain` 结构：**
```json
{
  "name": "wall",
  "sdf": [...],
  "role": "boundary_dirichlet"
}
```
`role` 可选值：`"interior"` | `"boundary_dirichlet"` | `"boundary_neumann"` | `"boundary_mur"`

**`BcSpec` 结构：**
```json
{
  "domain": "wall",
  "vars": ["u"],
  "bc_type": "dirichlet",
  "coef": null
}
```
`bc_type` 可选值：`"dirichlet"` | `"neumann"` | `"mur"` | `"robin"`
`coef` 用于 Mur / Robin BC，如波速 c。

**多变量请求示例（简单）：**
```json
{
  "solver": "pdeformer2",
  "pde": {
    "equation": "",
    "variables": ["u", "v"],
    "equations": ["u_t = 0.01*(u_xx + u_yy)", "v_t = 0.01*(v_xx + v_yy)"],
    "initial_conditions": {
      "u": [0.0, ...],
      "v": "grf"
    }
  },
  "query": { "x": [...], "y": [...] }
}
```

> **向后兼容**：若 `variables` 和 `equations` 均为空，则走单变量路径，
> 使用 `equation` + `initial_condition` + `boundary_condition`。

---

#### 模式三：历史快照（数据驱动 / autoregressive 模型）

适用于把过往若干时间步作为输入、让模型预测后续帧的场景。先 `POST /files` 上传，再在 `pde.history` 引用：

```json
{
  "solver": "pdeformer2",
  "pde": {
    "equation": "",
    "history": {
      "file_id": "3f2a1b8e-7d4c-4a92-9f31-c0dd1e7e8f22",
      "format": "hdf5",
      "dataset_key": "/data/u",
      "input_timesteps": [0, 1, 2],
      "variables": ["u", "v"]
    }
  },
  "query": { "x": [...], "y": [...], "t": [...] }
}
```

`HistorySpec` 字段：

| 字段 | 类型 | 说明 |
|---|---|---|
| `file_id` | string（必填） | `POST /files` 返回的 id |
| `format` | string \| null | 文件格式（`hdf5` / `npy` / `npz` / `pt`），缺省时从扩展名推断 |
| `dataset_key` | string \| null | HDF5 dataset 路径或 npz 数组名；纯 `.npy` / `.pt` 单数组文件可省略 |
| `input_timesteps` | int[] \| null | 选取哪些时间步作为条件窗口，缺省用文件里全部 |
| `variables` | string[] | 张量最后一维对应的变量名列表（多通道时必填）；缺省 `["u"]` |

**重要约束**：
- **`history` 与 IC 互斥**：当 `history` 字段存在时，求解器**忽略** `initial_condition` / `initial_conditions`。两者不要同时填。
- `classical` 求解器虽然接受 `history`，但**只取张量第一个变量通道**（多通道数据会被裁掉），如需多变量请用 `pdeformer2`。
- 如果在线无关时间步采样（query.t 不重叠 history 区间），求解器自动外推；过远的预测精度会下降。

---

### `pde.equation` 方程字符串语法

| 写法 | 含义 |
|---|---|
| `u_t` | ∂u/∂t |
| `u_x`、`u_y` | ∂u/∂x、∂u/∂y |
| `(f(u))_x` | ∂f(u)/∂x（通量散度形式） |
| `u_xx` | ∂²u/∂x² |

示例：
- 热方程：`"u_t = 0.01 * (u_xx + u_yy)"`
- 守恒律：`"u_t + (u^2)_x + (-0.3*u)_y = 0"`
- Burgers 方程：`"u_t + u*u_x = 0.001*u_xx"`

---

### `pde.initial_condition`（单变量模式）

扁平化的 128×128 网格数值数组（行优先，长度 16384），表示 t=0 时刻的解场。

```python
import numpy as np
u0 = np.zeros((128, 128))
u0[40:88, 40:88] = 1.0
initial_condition = u0.flatten().tolist()
```

**注意**：
- 仅时间相关问题（含 `u_t`）需要提供此字段
- 空间域固定为 [0, 1] × [0, 1]，网格均匀分布
- 省略则由求解器使用默认初始条件（通常为零场）

---

### `pde.boundary_condition`（单变量模式）

| 值 | 含义 |
|---|---|
| `"periodic"` | 周期性边界（适合守恒律、波动问题） |
| `"dirichlet"` | Dirichlet 边界（固定边界值） |
| `"neumann"` | Neumann 边界（固定法向导数） |

省略时由求解器自行决定默认值。

---

### `query.x` / `query.y`

指定返回解场的空间采样点坐标，值域在 [0, 1]。

```python
import numpy as np
x = np.linspace(0, 1, 32).tolist()
y = np.linspace(0, 1, 32).tolist()
```

**技巧**：采样点不必与内部计算网格（128×128）对齐，求解器会自动插值。降低采样分辨率可显著减少响应体积。

---

### `query.t`

时间采样点列表，省略时默认为 `[0.0, 0.25, 0.5, 0.75, 1.0]`。

---

## 响应体结构

```json
{
  "success": true,
  "data": {
    "solver_used": "pdeformer2",
    "variables": ["u"],
    "solution": [[[[...]]]],
    "shape": { "n_t": 5, "n_x": 32, "n_y": 32, "n_vars": 1 },
    "metadata": {
      "wall_time_ms": 1234,
      "backend": "MindSpore 2.8 / CPU",
      "notes": []
    }
  },
  "request_id": "...",
  "timestamp": "..."
}
```

### 新增字段说明

- **`data.variables`**（新增）：变量名列表，与 `solution` 最后一维对应。
  单变量问题为 `["u"]`，多变量问题如 `["u", "v", "p"]`。
- **`data.shape.n_vars`**（新增）：变量数量，与 `variables` 长度一致。
- **`data.solution` 索引顺序**：`solution[t_idx][x_idx][y_idx][var_idx]`

```python
solution = response["data"]["solution"]
variables = response["data"]["variables"]
shape = response["data"]["shape"]

# 取 t=0 时刻的 u 场（单变量）
u_t0 = [[solution[0][i][j][0] for j in range(shape["n_y"])]
         for i in range(shape["n_x"])]

# 多变量：按变量名索引
var_idx = variables.index("v")
v_t1 = [[solution[1][i][j][var_idx] for j in range(shape["n_y"])]
          for i in range(shape["n_x"])]
```

---

## 常见错误处理

| 端点 | HTTP | 含义 | 处理建议 |
|---|---|---|---|
| `/solve` | 400 | 请求体格式错误 | 检查 JSON 结构，尤其是 `initial_condition` 长度（须 16384） |
| `/solve` | 404 | 指定的求解器不存在 | 先调用 `GET /solvers` 确认可用 id |
| `/solve` | 500 | 求解过程内部错误 | 检查 `error` 字段，可能是方程格式不支持，或 `history.file_id` 不存在 |
| `/files` | 400 | multipart 字段名错 | 字段名必须是 `file`，不能是 `upload` 或别的 |
| `/files` | 400 | 文件格式不支持 | 仅 `.h5/.hdf5/.npy/.npz/.pt/.pth` 被识别，其它扩展名拒收 |
| `/files` | 500 | 写盘失败 | 检查 `$SOLVER_UPLOAD_DIR` 是否可写、磁盘空间 |

响应体中 `success: false` 时，`error` 字段包含具体原因。
