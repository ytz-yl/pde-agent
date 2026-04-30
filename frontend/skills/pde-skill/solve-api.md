# 求解器 API 调用技巧

端点：`POST /solve`（求解器服务，默认端口 3000）

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
    "supported_pde_types": ["general_2d", "conservation_law", ...],
    "available": true
  }
]
```

**技巧**：在调用 `/solve` 前先调用 `GET /solvers`，确认目标求解器的 `available` 字段为 `true`。

---

### `pde.equation`

方程字符串，使用标准数学符号：

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

### `pde.initial_condition`

扁平化的 128×128 网格数值数组（行优先，长度 128×128 = 16384），表示 t=0 时刻的解场。

```python
# 构造示例（Python）
import numpy as np
u0 = np.zeros((128, 128))
u0[40:88, 40:88] = 1.0          # 中央方块初始值
initial_condition = u0.flatten().tolist()
```

**注意**：
- 仅时间相关问题（含 `u_t`）需要提供此字段
- 空间域固定为 [0, 1] × [0, 1]，网格均匀分布
- 省略则由求解器使用默认初始条件（通常为零场）

---

### `pde.boundary_condition`

支持三种类型：

| 值 | 含义 |
|---|---|
| `"periodic"` | 周期性边界（适合守恒律、波动问题） |
| `"dirichlet"` | Dirichlet 边界（固定边界值，适合热方程、泊松方程） |
| `"neumann"` | Neumann 边界（固定法向导数，适合绝热边界等） |

省略时由求解器自行决定默认值。

---

### `pde.parameters`

方程中引用的自由标量参数，以 JSON 对象传入：

```json
"parameters": {
  "nu": 0.001,
  "alpha": 0.5
}
```

方程字符串中直接写数字也可以（如 `"u_t = 0.01*u_xx"`），`parameters` 主要用于需要动态配置参数的场景。

---

### `query.x` / `query.y`

指定返回解场的空间采样点坐标，值域均在 [0, 1]。

```python
# 32×32 均匀网格
import numpy as np
x = np.linspace(0, 1, 32).tolist()
y = np.linspace(0, 1, 32).tolist()
```

**技巧**：采样点不必与内部计算网格（128×128）对齐，求解器会自动插值。降低采样分辨率可显著减少响应体积。

---

### `query.t`

时间采样点列表，省略时默认为 `[0.0, 0.25, 0.5, 0.75, 1.0]`。

**注意**：时间值应在 [0, 1] 范围内，超出范围的行为由具体求解器决定。

---

## 响应体结构

```json
{
  "success": true,
  "data": {
    "solver_used": "pdeformer2",
    "solution": [[[[...]]]],
    "shape": { "n_t": 5, "n_x": 32, "n_y": 32, "n_vars": 1 },
    "metadata": {
      "wall_time_ms": 1234,
      "backend": "python-grpc",
      "notes": []
    }
  },
  "request_id": "...",
  "timestamp": "..."
}
```

### `data.solution` 的索引顺序

`solution[t_idx][x_idx][y_idx][var_idx]`

```python
# Python 读取示例
solution = response["data"]["solution"]
shape = response["data"]["shape"]

# 取 t=0 时刻的 u 场
u_t0 = [[solution[0][i][j][0] for j in range(shape["n_y"])]
         for i in range(shape["n_x"])]
```

---

## 常见错误处理

| HTTP 状态码 | 含义 | 处理建议 |
|---|---|---|
| 400 | 请求体格式错误 | 检查 JSON 结构，尤其是 `initial_condition` 长度 |
| 404 | 指定的求解器不存在 | 先调用 `GET /solvers` 确认可用 ID |
| 500 | 求解过程内部错误 | 检查 `error` 字段说明，可能是方程格式不支持 |

响应体中 `success: false` 时，`error` 字段包含具体原因。
