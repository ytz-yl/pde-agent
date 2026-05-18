# Solver API — POST /files + history 张量输入 (方案 B)

实现日期: 2026-05-13  
服务地址: http://localhost:8080  
文件存储目录: `/tmp/pde-solver-uploads`（可通过 `SOLVER_UPLOAD_DIR` 覆盖）

---

## 新增接口概览

| 方法 | 路径 | 说明 |
|---|---|---|
| POST | `/files` | 上传张量文件，返回 `file_id` |
| POST | `/solve` | 同原来，`pde.history.file_id` 字段新增 |

---

## POST /files

**请求格式：** `multipart/form-data`，字段名必须是 `file`

**支持格式：** `.npy` / `.npz` / `.h5` / `.hdf5` / `.pt` / `.pth`

**成功响应：**
```json
{
  "success": true,
  "data": {
    "file_id": "a2e4127a-82b3-475c-95de-977c1557dbcf",
    "filename": "history.npy",
    "format": "npy",
    "size_bytes": 20608,
    "path": "/tmp/pde-solver-uploads/a2e4127a-82b3-475c-95de-977c1557dbcf.npy"
  }
}
```

**错误（不支持的格式）：**
```json
{
  "success": false,
  "error": "Invalid request: Unsupported file format for 'data.csv'. Allowed: .h5, .hdf5, .npy, .npz, .pt, .pth"
}
```

**curl 示例：**
```bash
curl -X POST http://localhost:8080/files \
  -F "file=@/path/to/snapshots.npy"
```

---

## HistorySpec 字段（加入 pde.history）

```json
{
  "file_id": "<POST /files 返回的 file_id>",
  "format": "hdf5",         // 可选，从文件扩展名自动推断
  "dataset_key": "/data/u", // 可选，HDF5 路径 or npz array 名
  "input_timesteps": [0,1,2], // 可选，选取哪些时间步索引（默认全选）
  "variables": ["u", "v"]   // 可选，通道名（默认 ["u"] 或 ["u0","u1",...]）
}
```

当 `history` 存在时，`initial_condition` / `initial_conditions` 被忽略。

**错误（file_id 不存在）：**
```json
{
  "success": false,
  "error": "File not found: No uploaded file with id 'xxx'. Upload it first via POST /files."
}
```

---

## 完整调用示例

```bash
# 1. 上传文件
FILE_ID=$(curl -s -X POST http://localhost:8080/files \
  -F "file=@snapshots.npy" | python3 -c "import sys,json; print(json.load(sys.stdin)['data']['file_id'])")

# 2. 用 history 求解
curl -X POST http://localhost:8080/solve \
  -H "Content-Type: application/json" \
  -d "{
    \"solver\": \"classical\",
    \"pde\": {
      \"equation\": \"diffusion\",
      \"boundary_condition\": \"periodic\",
      \"parameters\": {\"d\": 0.05},
      \"history\": {
        \"file_id\": \"$FILE_ID\",
        \"input_timesteps\": [4],
        \"variables\": [\"u\"]
      }
    },
    \"query\": {
      \"x\": [0.0, 0.25, 0.5, 0.75, 1.0],
      \"y\": [0.0, 0.25, 0.5, 0.75, 1.0],
      \"t\": [0.0, 0.1, 0.2]
    }
  }"
```

---

## 关键实现细节

### Rust 层：file_id → 绝对路径注入

`src/routes/solve.rs` 在将请求转发给 Python 脚本之前，会将 `history.file_id`
（UUID 字符串）解析成服务器本地绝对路径，并原地覆盖 `file_id` 字段。

Python 脚本收到的 `file_id` 因此已经是可以直接 `open()` 的路径，
不需要知道上传目录在哪里。

```rust
// src/routes/solve.rs 关键逻辑
if let Some(ref mut history) = req.pde.history {
    let path = file_path_for_id(&history.file_id)
        .ok_or_else(|| ApiError::FileNotFound(...))?;
    history.file_id = path.to_string_lossy().to_string();  // 覆盖为绝对路径
    // 从扩展名自动推断 format（若调用方未指定）
    if history.format.is_none() { ... }
}
```

`file_path_for_id()` 实现在 `src/routes/files.rs`，扫描上传目录中文件名 stem 匹配的文件。

### Python 层：张量形状标准化

`_load_history_from_file()` 接受任意 2-4 维张量，统一规范化为 `[n_t, n_x, n_y, n_vars]`：

| 输入维度 | 处理方式 |
|---|---|
| 2D `[n_x, n_y]` | → `[1, n_x, n_y, 1]` |
| 3D，最后维 ≤16 且不等于前一维 | → `[1, n_x, n_y, n_vars]`（推断为通道维） |
| 3D，其他 | → `[n_t, n_x, n_y, 1]`（推断为时间维） |
| 4D | 直接使用 |

### pdeformer2 history 模式

当有 `history` 时，`pdeformer2_infer.py` 取最后一帧（`arr[-1]`）作为各变量的 IC，
然后按正常 PDEformer-2 推理流程走。如果 `pde.equations` 也有值，会同时构建 PDE DAG；
否则用 `u_t = 0` 作为占位（identity transport）。

### classical solver history 模式

`classical_solve.py` 取 `arr[-1, :, :, 0]`（最后帧第一个通道）作为 py-pde 的标量初始条件，
其余通道当前被忽略（classical 求解器目前只支持单变量）。

---

## 响应中的 notes 字段（诊断用）

调用成功后 `metadata.notes` 会包含完整的文件加载链路，例如：
```json
[
  "Loaded .npy shape=(5, 32, 32, 1)",
  "History normalised: [n_t=5, n_x=32, n_y=32, n_vars=1]",
  "Selected timesteps [4]",
  "Using last time-step of history file as initial condition (variable: u)"
]
```

---

## 文件清理

目前没有自动清理机制。上传目录会持续增长。如需手动清理：
```bash
rm /tmp/pde-solver-uploads/*.npy
rm /tmp/pde-solver-uploads/*.npz
```

未来改进方向：在 `/solve` 完成后异步删除单次使用的文件，或加 TTL 清理 cron。
