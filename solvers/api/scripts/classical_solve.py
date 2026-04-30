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
        if ic_flat is not None:
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


if __name__ == "__main__":
    main()
