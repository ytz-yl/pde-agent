#!/usr/bin/env python3
"""
PDEformer-2 inference bridge.

Reads a JSON SolveRequest from stdin, runs PDEformer-2 inference,
writes a JSON result to stdout.

Expected to be run with the pdeformer2 conda environment:
    $HOME/miniconda3/envs/pdeformer2/bin/python pdeformer2_infer.py

Working directory must be the pdeformer-2 repo root so that `src` is
importable.
"""

import json
import os
import sys
import traceback

# Ensure the pdeformer-2 repo root is on sys.path so `src` is importable.
# The Rust bridge sets cwd to the pdeformer-2 directory; if it is not, we
# also accept PDEFORMER2_DIR env var.
_repo_root = os.environ.get(
    "PDEFORMER2_DIR",
    os.path.join(os.path.dirname(__file__), "../../../solvers/ml/pdeformer-2"),
)
_repo_root = os.path.realpath(_repo_root)
if _repo_root not in sys.path:
    sys.path.insert(0, _repo_root)
# Also honour the current working directory (set by Rust bridge)
_cwd = os.getcwd()
if _cwd not in sys.path:
    sys.path.insert(0, _cwd)

import numpy as np


def main():
    # ------------------------------------------------------------------ input
    raw = sys.stdin.read()
    try:
        req = json.loads(raw)
    except json.JSONDecodeError as e:
        _error(f"Failed to parse stdin JSON: {e}")
        return

    pde_spec   = req.get("pde", {})
    query_spec = req.get("query", {})
    options    = req.get("options") or {}

    equation   = pde_spec.get("equation", "")
    bc         = pde_spec.get("boundary_condition", "periodic")
    ic_flat    = pde_spec.get("initial_condition")   # list[float] | None
    params     = pde_spec.get("parameters") or {}

    x_vals = query_spec.get("x", [i / 31 for i in range(32)])
    y_vals = query_spec.get("y", [i / 31 for i in range(32)])
    t_vals = query_spec.get("t", [0.0, 0.25, 0.5, 0.75, 1.0])

    # ---------------------------------------------------------- import model
    try:
        from mindspore import context
        from src import load_config, get_model, PDENodesCollector
        from src.inference import inference_pde, x_fenc, y_fenc
    except ImportError as e:
        _error(f"Import error (is pdeformer2 env active?): {e}")
        return

    notes = []

    try:
        context.set_context(mode=0, device_target="CPU")  # PYNATIVE_MODE=0
    except Exception as e:
        notes.append(f"context warning: {e}")

    config_path = options.get("config", "results/model-L.yaml")
    try:
        config = load_config(config_path)
        model  = get_model(config)
    except Exception as e:
        _error(f"Failed to load model from {config_path}: {e}\n{traceback.format_exc()}")
        return

    # ------------------------------------------------------- build PDE DAG
    try:
        pde = PDENodesCollector()
        u   = pde.new_uf()

        # Initial condition
        if ic_flat is not None:
            n = int(round(len(ic_flat) ** 0.5))
            ic = np.array(ic_flat, dtype=np.float32).reshape(n, n)
            # interpolate to 128x128 if necessary
            if n != 128:
                from scipy.interpolate import RegularGridInterpolator
                old_x = np.linspace(0, 1, n)
                interp = RegularGridInterpolator(
                    (old_x, old_x), ic, method="linear",
                    bounds_error=False, fill_value=None)
                ic = interp((x_fenc, y_fenc))
            pde.set_ic(u, ic, x=x_fenc, y=y_fenc)
        else:
            # default: sin(2πx)cos(4πy)
            default_ic = np.sin(2 * np.pi * x_fenc) * np.cos(4 * np.pi * y_fenc)
            pde.set_ic(u, default_ic, x=x_fenc, y=y_fenc)
            notes.append("No initial_condition provided; using sin(2πx)cos(4πy)")

        # Build equation from spec string via a simple DSL interpreter
        _build_equation(pde, u, equation, params)

    except Exception as e:
        _error(f"Failed to build PDE DAG for '{equation}': {e}\n{traceback.format_exc()}")
        return

    # ------------------------------------------------------- run inference
    try:
        pde_dag = pde.gen_dag(config)

        x_arr = np.array(x_vals, dtype=np.float64)
        y_arr = np.array(y_vals, dtype=np.float64)
        t_arr = np.array(t_vals, dtype=np.float64)

        # Build coordinate grid [n_t, n_x, n_y, 4]
        coord = np.stack(
            np.broadcast_arrays(
                np.reshape(t_arr, (-1, 1, 1)),
                np.reshape(x_arr, (1, -1, 1)),
                np.reshape(y_arr, (1, 1, -1)),
                0.0,
            ),
            axis=-1,
        ).astype(np.float32)

        snapshots = inference_pde(model, pde_dag, coord)
        # snapshots: [n_t, n_x, n_y, n_vars]

    except Exception as e:
        _error(f"Inference failed: {e}\n{traceback.format_exc()}")
        return

    # -------------------------------------------------------- format output
    # Convert to nested Python lists: [n_t][n_x][n_y][n_vars]
    result = snapshots.tolist()

    print(json.dumps({"solution": result, "notes": notes}))


# ---------------------------------------------------------------------------
# Equation DSL
# ---------------------------------------------------------------------------

def _build_equation(pde, u, equation: str, params: dict):
    """
    Translate the equation string into PDENodesCollector calls.

    Supported patterns (case-insensitive):
      - "u_t + (u^2)_x + (c*u)_y = 0"   nonlinear conservation law
      - "u_t - d*(u_xx + u_yy) = 0"      heat / diffusion
      - "u_t + c*u_x + u_y = 0"          linear advection
      - anything else → fallback to default nonlinear conservation law

    Scalar parameters can be supplied via the `params` dict, e.g.
    {"c": 0.5, "d": 0.01}.
    """
    eq = equation.lower().replace(" ", "")
    c  = float(params.get("c", -0.3))
    d  = float(params.get("d", 0.01))

    if "u^2" in eq or "u²" in eq or "(u2)" in eq:
        # nonlinear conservation law: u_t + (u^2)_x + (c*u)_y = 0
        pde.sum_eq0(pde.dt(u), pde.dx(pde.square(u)), pde.dy(c * u))
    elif "u_xx" in eq or "uxx" in eq or "laplacian" in eq:
        # diffusion/heat: u_t - d*(u_xx + u_yy) = 0
        pde.sum_eq0(pde.dt(u), -d * pde.dx(pde.dx(u)), -d * pde.dy(pde.dy(u)))
    elif "c*u_x" in eq or "u_x" in eq:
        # linear advection: u_t + c*u_x + u_y = 0
        pde.sum_eq0(pde.dt(u), c * pde.dx(u), pde.dy(u))
    else:
        # Fallback: nonlinear conservation law (same as README example)
        pde.sum_eq0(pde.dt(u), pde.dx(pde.square(u)), pde.dy(c * u))


# ---------------------------------------------------------------------------

def _error(msg: str):
    print(json.dumps({"error": msg}))


if __name__ == "__main__":
    main()
