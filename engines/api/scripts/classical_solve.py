#!/usr/bin/env python3
"""
Classical PDE solver bridge (py-pde backend).

Reads a JSON SolveRequest from stdin, solves using py-pde,
writes a JSON result to stdout.

Run with the classical-pde conda environment:
    $HOME/miniconda3/envs/classical-pde/bin/python3 classical_solve.py
"""

import json
import sys
import traceback

import numpy as np


def main():
    raw = sys.stdin.read()
    try:
        req = json.loads(raw)
    except json.JSONDecodeError as e:
        _error(f"Failed to parse stdin JSON: {e}")
        return

    pde_spec   = req.get("pde", {})
    query_spec = req.get("query", {})
    options    = req.get("options") or {}

    equation   = pde_spec.get("equation", "diffusion").strip().lower()
    bc_type    = pde_spec.get("boundary_condition", "periodic").strip().lower()
    ic_flat    = pde_spec.get("initial_condition")
    params     = pde_spec.get("parameters") or {}
    history_spec = pde_spec.get("history")

    x_vals = query_spec.get("x", [i / 31 for i in range(32)])
    y_vals = query_spec.get("y", [i / 31 for i in range(32)])
    t_vals = query_spec.get("t", [0.0, 0.25, 0.5, 0.75, 1.0])

    # ---------------------------------------------------------- import py-pde
    try:
        import pde as pypde
    except ImportError as e:
        _error(f"py-pde not installed: {e}")
        return

    notes = []
    n_x = len(x_vals)
    n_y = len(y_vals)

    # ---------------------------------------------------------- build grid
    # py-pde works on UnitGrid; we solve at resolution max(n_x, n_y) then
    # interpolate to the query coordinates.
    grid_res = max(n_x, n_y, 32)
    try:
        bc = _build_bc(bc_type)
        grid = pypde.UnitGrid([grid_res, grid_res], periodic=bc == "periodic")
    except Exception as e:
        _error(f"Failed to build grid: {e}")
        return

    # ---------------------------------------------------------- initial condition
    try:
        if history_spec:
            # ── History path: load from uploaded tensor file ──────────────────
            arr, hist_var_names, hist_notes = _load_history_from_file(history_spec)
            notes.extend(hist_notes)
            # Use the last time step, first variable as 2D IC
            last_snap = arr[-1, :, :, 0]   # [n_x_hist, n_y_hist]
            n = last_snap.shape[0]
            # resize to grid_res if needed
            if n != grid_res:
                from scipy.ndimage import zoom
                last_snap = zoom(last_snap, grid_res / n, order=1)
            state = pypde.ScalarField(grid, data=last_snap.astype(np.float64))
            notes.append(
                f"Using last time-step of history file as initial condition "
                f"(variable: {hist_var_names[0] if hist_var_names else 'u'})"
            )
        elif ic_flat is not None:
            n = int(round(len(ic_flat) ** 0.5))
            ic_arr = np.array(ic_flat, dtype=np.float64).reshape(n, n)
            # resize to grid_res if needed
            if n != grid_res:
                from scipy.ndimage import zoom
                ic_arr = zoom(ic_arr, grid_res / n, order=1)
            state = pypde.ScalarField(grid, data=ic_arr)
        else:
            # default: sin(2πx)cos(4πy)
            state = pypde.ScalarField.from_expression(
                grid, "sin(2*pi*x) * cos(2*pi*y)"
            )
            notes.append("No initial_condition provided; using sin(2πx)cos(4πy)")
    except Exception as e:
        _error(f"Failed to build initial condition: {e}")
        return

    # ---------------------------------------------------------- build PDE
    try:
        eq, is_wave = _build_pde(equation, params, bc_type, grid, notes)
    except Exception as e:
        _error(f"Failed to build PDE for '{equation}': {e}\n{traceback.format_exc()}")
        return

    # ---------------------------------------------------------- solve at snapshots
    t_max = max(t_vals)
    dt = float(options.get("dt", 0.001))

    try:
        if is_wave:
            # WavePDE needs FieldCollection [u, u_dot]
            state_coll = pypde.FieldCollection([state, pypde.ScalarField(grid)])
            sol_snapshots = _solve_snapshots(eq, state_coll, t_vals, dt)
            # extract first component (u)
            solution = [sol_snapshots[i][0].data for i in range(len(t_vals))]
        else:
            sol_snapshots = _solve_snapshots(eq, state, t_vals, dt)
            solution = [sol_snapshots[i].data for i in range(len(t_vals))]
    except Exception as e:
        _error(f"Solver failed: {e}\n{traceback.format_exc()}")
        return

    # ---------------------------------------------------------- interpolate to query coords
    try:
        result = _interpolate_to_query(solution, grid_res, x_vals, y_vals)
    except Exception as e:
        _error(f"Interpolation failed: {e}")
        return

    print(json.dumps({"solution": result, "notes": notes}))


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _build_bc(bc_type: str):
    """Return py-pde-compatible boundary condition string."""
    mapping = {
        "periodic": "periodic",
        "dirichlet": "dirichlet",
        "neumann": "neumann",
    }
    return mapping.get(bc_type, "periodic")


def _build_pde(equation: str, params: dict, bc_type: str, grid, notes: list):
    """
    Dispatch equation string → py-pde PDE object.
    Returns (pde_obj, is_wave_pde).
    """
    import pde as pypde

    d = float(params.get("d", 0.1))
    c = float(params.get("c", 1.0))
    bc = _build_bc(bc_type)

    if "wave" in equation:
        return pypde.WavePDE(speed=c), True

    if any(k in equation for k in ("diffusion", "heat", "laplace(u)", "∇²u", "nabla")):
        return pypde.DiffusionPDE(diffusivity=d), False

    if "allen" in equation or "cahn" in equation:
        if "hilliard" in equation:
            return pypde.CahnHilliardPDE(), False
        return pypde.AllenCahnPDE(), False

    # Generic symbolic fallback — equation should be the RHS of ∂u/∂t = ...
    # e.g. "laplace(u) - u**3 + u"  or  "- c * d_dx(u)"
    # We also handle common shorthands
    rhs = equation
    for old, new in [
        ("u_t", ""), ("= 0", ""), ("∂u/∂t =", ""), ("d/dt u =", ""),
    ]:
        rhs = rhs.replace(old, new).strip(" =")

    notes.append(f"Using symbolic PDE: ∂u/∂t = {rhs}")
    return pypde.PDE({"u": rhs}), False


def _solve_snapshots(eq, state, t_vals, dt):
    """
    Solve the PDE and collect snapshots at the requested time values.
    Returns a list of field states, one per t in t_vals.
    """
    import pde as pypde

    t_sorted = sorted(set(t_vals))
    snapshots = {}

    current_state = state.copy()
    t_now = 0.0

    for t_target in t_sorted:
        if t_target <= t_now:
            snapshots[t_target] = current_state.copy()
            continue
        duration = t_target - t_now
        current_state = eq.solve(current_state, t_range=duration, dt=dt, tracker=None)
        t_now = t_target
        snapshots[t_target] = current_state.copy()

    return [snapshots[t] for t in t_vals]


def _interpolate_to_query(solution_arrays, grid_res, x_vals, y_vals):
    """
    Interpolate each snapshot from the uniform [grid_res × grid_res] grid
    to the requested (x_vals, y_vals) coordinates.

    Returns nested list [n_t][n_x][n_y][1].
    """
    from scipy.interpolate import RegularGridInterpolator

    grid_coords = np.linspace(0.0, 1.0, grid_res)
    result = []

    for snap in solution_arrays:
        interp = RegularGridInterpolator(
            (grid_coords, grid_coords), snap,
            method="linear", bounds_error=False, fill_value=None,
        )
        pts = np.array(
            [[x, y] for x in x_vals for y in y_vals], dtype=np.float64
        )
        vals = interp(pts).reshape(len(x_vals), len(y_vals))
        # shape [n_x][n_y][1]
        result.append([[[float(vals[ix, iy])] for iy in range(len(y_vals))]
                       for ix in range(len(x_vals))])

    return result  # [n_t][n_x][n_y][1]


def _error(msg: str):
    print(json.dumps({"error": msg}))


# ---------------------------------------------------------------------------
# History tensor file loader (shared logic — mirrored from pdeformer2_infer.py)
# ---------------------------------------------------------------------------

def _load_history_from_file(history_spec: dict):
    """
    Load historical snapshots from an uploaded tensor file.

    Returns (array [n_t, n_x, n_y, n_vars], var_names [str], notes [str]).
    file_id in history_spec has already been resolved to an absolute path by
    the Rust layer.
    """
    import os
    file_path   = history_spec.get("file_id", "")
    fmt         = (history_spec.get("format") or "").lower()
    dataset_key = history_spec.get("dataset_key")
    timesteps   = history_spec.get("input_timesteps")
    var_names   = list(history_spec.get("variables") or [])
    notes       = []

    if not file_path or not os.path.isfile(file_path):
        raise FileNotFoundError(f"History file not found: '{file_path}'")

    if not fmt:
        ext = os.path.splitext(file_path)[1].lstrip(".").lower()
        fmt = {"h5": "hdf5", "hdf5": "hdf5", "npy": "npy",
               "npz": "npz", "pt": "pt", "pth": "pt"}.get(ext, ext)
        notes.append(f"Inferred format '{fmt}' from extension '.{ext}'")

    if fmt == "hdf5":
        import h5py
        with h5py.File(file_path, "r") as f:
            key = dataset_key or _h5_auto_key(f)
            arr = f[key][()]
            notes.append(f"Loaded HDF5 dataset '{key}' shape={arr.shape}")
    elif fmt == "npy":
        arr = np.load(file_path, allow_pickle=False)
        notes.append(f"Loaded .npy shape={arr.shape}")
    elif fmt == "npz":
        archive = np.load(file_path, allow_pickle=False)
        key = dataset_key or (list(archive.keys())[0] if len(archive.keys()) == 1
                              else None)
        if key is None:
            raise KeyError(f"npz has multiple arrays {list(archive.keys())}; specify dataset_key")
        arr = archive[key]
        notes.append(f"Loaded npz array '{key}' shape={arr.shape}")
    elif fmt in ("pt", "pth"):
        import torch
        tensor = torch.load(file_path, map_location="cpu")
        if isinstance(tensor, dict):
            keys = [k for k, v in tensor.items() if hasattr(v, "numpy")]
            tensor = tensor[dataset_key] if (dataset_key and dataset_key in tensor) \
                     else tensor[keys[0]]
        arr = tensor.numpy() if hasattr(tensor, "numpy") else np.array(tensor)
        notes.append(f"Loaded torch tensor shape={arr.shape}")
    else:
        raise ValueError(f"Unsupported format '{fmt}'")

    arr = np.asarray(arr, dtype=np.float32)
    if arr.ndim == 2:
        arr = arr[np.newaxis, :, :, np.newaxis]
    elif arr.ndim == 3:
        if arr.shape[-1] <= 16 and arr.shape[-1] != arr.shape[-2]:
            arr = arr[np.newaxis]
        else:
            arr = arr[:, :, :, np.newaxis]
    elif arr.ndim != 4:
        raise ValueError(f"Cannot interpret {arr.ndim}-D history tensor")

    n_t, n_x, n_y, n_vars = arr.shape
    notes.append(f"History normalised: [n_t={n_t}, n_x={n_x}, n_y={n_y}, n_vars={n_vars}]")

    if timesteps is not None:
        arr = arr[list(timesteps)]
        notes.append(f"Selected timesteps {timesteps}")

    if not var_names:
        var_names = ["u"] if n_vars == 1 else [f"u{i}" for i in range(n_vars)]

    return arr, var_names, notes


def _h5_auto_key(h5file):
    import h5py
    def _find(item, path=""):
        if isinstance(item, h5py.Dataset):
            return path
        for k in item.keys():
            r = _find(item[k], f"{path}/{k}")
            if r is not None:
                return r
        return None
    key = _find(h5file)
    if key is None:
        raise KeyError("No dataset in HDF5 file; specify dataset_key")
    return key


if __name__ == "__main__":
    main()
