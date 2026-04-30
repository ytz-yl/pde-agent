# PDE Agent Skill

本 skill 说明 AI Agent 在面对 PDE 相关任务时，应当调用哪些服务、何时调用、调用哪个端点。

系统由两个独立服务组成：

| 服务 | 默认端口 | 职责 |
|---|---|---|
| **知识库服务** (`knowledge-base`) | 3001 | 论文检索、方法查询、求解器推荐 |
| **求解器服务** (`solvers-api`) | 3000 | 实际执行 PDE 数值求解 |

---

## 何时调用哪个服务

### 知识库服务（knowledge-base）

以下场景优先调用知识库服务：

| 用户意图 | 对应端点 |
|---|---|
| "有哪些方法可以求解 X 类方程？" | `POST /recommend` |
| "FEM 和 FNO 哪个更适合这个问题？" | `GET /methods/compare?a=fem&b=fno` |
| "告诉我 FNO 的原理" | `GET /methods/{id}` |
| "最近有哪些关于 Navier-Stokes 的论文？" | `GET /papers/recent?domain=fluid_dynamics` |
| "搜索关于 PINNs 反问题的文献" | `GET /search?q=...` |
| "FEM 有哪些相关方法？" | `GET /methods/{id}/related` |
| "列出所有已知的经典数值方法" | `GET /methods?category=classical` |

### 求解器服务（solvers-api）

以下场景调用求解器服务：

| 用户意图 | 对应端点 |
|---|---|
| "帮我求解这个方程" | `POST /solve` |
| "有哪些可用的求解器？" | `GET /solvers` |
| 任何需要实际计算、返回数值解的请求 | `POST /solve` |

---

## 推荐的调用顺序

### 场景一：用户提出 PDE 问题需要求解

```
1. POST /recommend          ← 知识库：先询问推荐哪种求解器
2. GET  /methods/{id}       ← 知识库（可选）：获取推荐方法的详细信息
3. POST /solve              ← 求解器：提交求解任务
```

### 场景二：用户要做方法调研

```
1. GET /search?q=...              ← 知识库：语义检索相关论文
2. GET /methods?category=...      ← 知识库：列举相关方法
3. GET /methods/compare?a=&b=     ← 知识库：对比候选方法
```

### 场景三：用户询问某类方程的最佳实践

```
1. POST /recommend          ← 知识库：获取方法推荐
2. GET  /methods/{id}/related ← 知识库：了解相关方法生态
3. GET  /papers/recent?domain=... ← 知识库：补充最新文献证据
```

---

## 不应调用的情况

- **用户只是在做一般性数学推导**（无需查库或求解）：直接用 LLM 自身能力回答。
- **问题与 PDE 无关**：不调用任何本服务。
- **用户明确指定了求解器**：跳过 `/recommend`，直接调用 `POST /solve`，在请求体中设置 `solver` 字段。

---

## 子文档

详细的调用技巧请参阅：

- [`solve-api.md`](./solve-api.md) — `POST /solve` 请求体构造、初始条件格式、边界条件类型
- [`knowledge-api.md`](./knowledge-api.md) — 知识库各端点的查询参数、过滤器用法、搜索技巧
