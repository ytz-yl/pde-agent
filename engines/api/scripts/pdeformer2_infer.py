#!/usr/bin/env python3
"""
PDEformer-2 inference bridge.

Reads a JSON SolveRequest from stdin, runs PDEformer-2 inference,
writes a JSON result to stdout.

Expected to be run with the pdeformer2 conda environment:
    $HOME/miniconda3/envs/pdeformer2/bin/python pdeformer2_infer.py

Working directory must be the pdeformer-2 repo root so that `src` is
importable.

Request schema (PdeSpec fields used here):
  - equation       : str   — legacy single-equation DSL string
  - variables      : list  — variable names, e.g. ["u"] or ["u","v","p"]
  - equations      : list  — one equation string per constraint
  - initial_conditions : dict  — var_name → flat array | "zero" | "grf"
  - coef_fields    : dict  — field_name → flat n×n array
  - domains        : list  — [{name, sdf:[...], role}]
  - bcs            : list  — [{domain, vars:[...], bc_type, coef?}]
  - boundary_condition : str — legacy "periodic"/"dirichlet"/"neumann"
  - parameters     : dict  — scalar params (legacy)
  - history        : dict  — {
        file_id       : str (resolved to abs path by Rust layer),
        format        : str  "hdf5"|"npy"|"npz"|"pt"  (optional),
        dataset_key   : str  HDF5 dataset path or npz array name (optional),
        input_timesteps: [int, ...]  indices to use (optional, default all),
        variables     : [str, ...]  variable names per channel (optional),
    }
    When present, initial_condition / initial_conditions are ignored; the
    loaded tensor is used as the conditioning snapshot(s).

The Python script always returns:
  {"solution": [[[[...]]]], "variables": [...], "notes": [...]}
where solution has shape [n_t][n_x][n_y][n_vars].
"""

import json
import os
import sys
import traceback

# Ensure the pdeformer-2 repo root is on sys.path so `src` is importable.
_repo_root = os.environ.get(
    "PDEFORMER2_DIR",
    os.path.join(os.path.dirname(__file__), "../../../solvers/ml/pdeformer-2"),
)
_repo_root = os.path.realpath(_repo_root)
if _repo_root not in sys.path:
    sys.path.insert(0, _repo_root)
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

    x_vals = query_spec.get("x", [i / 31 for i in range(32)])
    y_vals = query_spec.get("y", [i / 31 for i in range(32)])
    t_vals = query_spec.get("t", [0.0, 0.25, 0.5, 0.75, 1.0])

    # ---------------------------------------------------------- import model
    try:
        from mindspore import context
        from src import load_config, get_model, PDENodesCollector, sample_grf
        from src.inference import inference_pde, x_fenc, y_fenc
    except ImportError as e:
        _error(f"Import error (is pdeformer2 env active?): {e}")
        return

    notes = []

    try:
        context.set_context(mode=0, device_target="CPU")
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
        # Check for history input first (data-driven / autoregressive path)
        history_spec = pde_spec.get("history")

        if history_spec:
            # ── History-conditioned path ────────────────────────────────────
            # Load historical snapshots from the uploaded tensor file.
            history_array, var_names, load_notes = _load_history_from_file(history_spec)
            notes.extend(load_notes)

            # Build a minimal PDE DAG conditioned on the loaded snapshots.
            # We set the initial condition from the last loaded timestep so
            # PDEformer-2 can do autoregressive prediction.
            pde = PDENodesCollector()
            var_map = {}
            if not var_names:
                var_names = ["u"]
            for vname in var_names:
                var_map[vname] = pde.new_uf()

            # history_array shape: [n_history, n_x, n_y, n_vars]
            # Use the last snapshot as the IC for each variable
            last_snap = history_array[-1]  # [n_x, n_y, n_vars]
            for vi, vname in enumerate(var_names):
                ic_arr = last_snap[:, :, vi].astype(np.float32)
                pde.set_ic(var_map[vname], ic_arr, x=x_fenc, y=y_fenc)

            # If there is still an equation string, parse it; otherwise use
            # a neutral transport equation as placeholder.
            equations_spec = pde_spec.get("equations", [])
            equation_str   = pde_spec.get("equation", "").strip()
            if equations_spec:
                for eq_str in equations_spec:
                    terms = _parse_equation_terms(eq_str, var_map, {}, {}, pde, notes)
                    if terms:
                        pde.sum_eq0(*terms)
            elif equation_str:
                params = pde_spec.get("parameters") or {}
                _build_single_equation(pde, var_map.get(var_names[0]), equation_str, params)
            else:
                notes.append("history mode: no equation provided, using u_t = 0 (identity transport)")
                u0 = var_map[var_names[0]]
                pde.sum_eq0(u0.dt)

        else:
            # Decide: multi-variable new-style OR legacy single-variable
            variables_spec = pde_spec.get("variables", [])
            equations_spec = pde_spec.get("equations", [])

            if variables_spec and equations_spec:
                # ── New multi-variable path ─────────────────────────────────────
                var_names, pde = _build_multivariable_pde(
                    pde_spec, variables_spec, equations_spec,
                    x_fenc, y_fenc, sample_grf, PDENodesCollector, notes
                )
            else:
                # ── Legacy single-variable path ─────────────────────────────────
                var_names = ["u"]
                pde = PDENodesCollector()
                u   = pde.new_uf()

                ic_flat = pde_spec.get("initial_condition")
                if ic_flat is not None:
                    ic = _flat_to_grid(ic_flat, x_fenc, y_fenc)
                    pde.set_ic(u, ic, x=x_fenc, y=y_fenc)
                else:
                    default_ic = np.sin(2 * np.pi * x_fenc) * np.cos(4 * np.pi * y_fenc)
                    pde.set_ic(u, default_ic, x=x_fenc, y=y_fenc)
                    notes.append("No initial_condition provided; using sin(2πx)cos(4πy)")

                equation  = pde_spec.get("equation", "")
                params    = pde_spec.get("parameters") or {}
                _build_single_equation(pde, u, equation, params)

    except Exception as e:
        _error(f"Failed to build PDE DAG: {e}\n{traceback.format_exc()}")
        return

    # ------------------------------------------------------- run inference
    try:
        pde_dag = pde.gen_dag(config)

        x_arr = np.array(x_vals, dtype=np.float64)
        y_arr = np.array(y_vals, dtype=np.float64)
        t_arr = np.array(t_vals, dtype=np.float64)

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

    result = snapshots.tolist()
    print(json.dumps({"solution": result, "variables": var_names, "notes": notes}))


# ---------------------------------------------------------------------------
# Multi-variable PDE builder
# ---------------------------------------------------------------------------

def _build_multivariable_pde(
    pde_spec, variables_spec, equations_spec,
    x_fenc, y_fenc, sample_grf, PDENodesCollector, notes
):
    """
    Build a PDENodesCollector from the new structured spec.

    Returns (var_names, pde).
    """
    pde = PDENodesCollector()

    # ── Create unknown field variables ──────────────────────────────────────
    var_map = {}   # name → pde field object
    for vname in variables_spec:
        var_map[vname] = pde.new_uf()

    # ── Coefficient fields ───────────────────────────────────────────────────
    coef_fields_spec = pde_spec.get("coef_fields", {})
    coef_map = {}  # name → pde coef field object
    for fname, fdata in coef_fields_spec.items():
        arr = _flat_to_grid(fdata, x_fenc, y_fenc)
        coef_map[fname] = pde.new_coef_field(arr, x=x_fenc, y=y_fenc)

    # ── Domains (SDF) ────────────────────────────────────────────────────────
    domains_spec = pde_spec.get("domains", [])
    domain_map = {}  # name → pde domain object
    for dspec in domains_spec:
        dname = dspec["name"]
        sdf_arr = _flat_to_grid(dspec["sdf"], x_fenc, y_fenc)
        domain_map[dname] = pde.new_domain(sdf_arr, x=x_fenc, y=y_fenc)

    # ── Initial conditions ───────────────────────────────────────────────────
    ic_spec = pde_spec.get("initial_conditions", {})
    for key, ic_val in ic_spec.items():
        # key is "u", "v", "u.dt", etc.
        if "." in key:
            vname, attr = key.split(".", 1)
            field_obj = var_map.get(vname)
            if field_obj is None:
                notes.append(f"Warning: unknown variable '{vname}' in initial_conditions key '{key}'")
                continue
            # e.g. attr = "dt" → field_obj.dt
            target = _resolve_attr(field_obj, attr)
        else:
            target = var_map.get(key)
            if target is None:
                notes.append(f"Warning: unknown variable '{key}' in initial_conditions")
                continue

        ic_array = _resolve_ic_value(ic_val, x_fenc, y_fenc, sample_grf)
        pde.set_ic(target, ic_array, x=x_fenc, y=y_fenc)

    # ── Equations ────────────────────────────────────────────────────────────
    for eq_str in equations_spec:
        terms = _parse_equation_terms(eq_str, var_map, coef_map, domain_map, pde, notes)
        if terms:
            pde.sum_eq0(*terms)
        else:
            notes.append(f"Warning: could not parse equation '{eq_str}', skipped")

    # ── Boundary conditions ──────────────────────────────────────────────────
    bcs_spec = pde_spec.get("bcs", [])
    for bc in bcs_spec:
        dname  = bc.get("domain")
        domain = domain_map.get(dname)
        if domain is None:
            notes.append(f"Warning: BC references unknown domain '{dname}', skipped")
            continue
        bc_vars = bc.get("vars", [])
        bc_type = bc.get("bc_type", "dirichlet")
        coef_val = bc.get("coef")

        resolved_terms = []
        for v_expr in bc_vars:
            term = _resolve_var_expr(v_expr, var_map, coef_map, domain_map, pde, notes)
            if term is not None:
                resolved_terms.append(term)

        if bc_type == "mur" and coef_val is not None:
            # Mur BC: bc_sum_eq0(boundary, [u.dt] + dn_sum_list(u, domain, coef=c))
            pde.bc_sum_eq0(domain, resolved_terms)
        else:
            pde.bc_sum_eq0(domain, *resolved_terms)

    return list(variables_spec), pde


# ---------------------------------------------------------------------------
# Equation string parser (mini-DSL)
# ---------------------------------------------------------------------------

def _parse_equation_terms(eq_str, var_map, coef_map, domain_map, pde, notes):
    """
    Parse an equation string into a list of PDENodesCollector term objects.

    Grammar (whitespace-insensitive):
      eq_str = term (+|- term)* = 0
      term   = coef_literal * var_expr
             | var_expr
             | coef_field_name
    var_expr = var_name (.dt | .dx | .dy | .dx.dx | .dy.dy | .dt.dt | .square)*
             | ( var_expr * var_expr ).dx  (product flux)
             | ( coef * var_expr ).dx      (scaled flux)

    This is intentionally a small DSL covering the notebook examples.
    The general approach: we normalize the string and dispatch known patterns.
    """
    import re

    eq = eq_str.strip()
    # Remove trailing "= 0" or "=0"
    eq = re.sub(r'\s*=\s*0\s*$', '', eq).strip()

    # Split into top-level terms separated by + or -
    # We use a simple tokenizer that respects nested parentheses.
    raw_terms = _split_top_level(eq)
    if not raw_terms:
        return []

    result = []
    for raw in raw_terms:
        term = _parse_single_term(raw.strip(), var_map, coef_map, pde, notes)
        if term is not None:
            result.append(term)
        else:
            notes.append(f"Could not parse term '{raw}' in equation '{eq_str}'")
            return []

    return result


def _split_top_level(s):
    """Split string s on top-level '+'/'-' term separators, preserving sign.

    A '+' or '-' is treated as a *separator* (start of a new term) only when
    ALL of the following hold:
      1. depth == 0  (not inside parentheses)
      2. i > 0       (not the very first character)
      3. The character immediately before it (ignoring spaces) is NOT one of
         '+', '-', '*', '/', '(', '[', 'e', 'E'
         — this prevents splitting on unary minus, scientific-notation
           exponents like "1e-3", or operators like "* -".
    """
    tokens = []
    depth  = 0
    start  = 0
    i      = 0
    while i < len(s):
        c = s[i]
        if c in ('(', '['):
            depth += 1
        elif c in (')', ']'):
            depth -= 1
        elif c in ('+', '-') and depth == 0 and i > 0:
            # Look back at the last non-space character
            prev = s[:i].rstrip()
            if prev and prev[-1] not in ('+', '-', '*', '/', '(', '[', 'e', 'E'):
                tokens.append(s[start:i])
                start = i  # keep the sign as part of the new token
        i += 1
    tokens.append(s[start:])
    return [t.strip() for t in tokens if t.strip()]


def _parse_single_term(s, var_map, coef_map, pde, notes):
    """
    Parse one term and return the corresponding PDENodesCollector object.

    Supported forms (after stripping leading sign):
      1.  numeric * var_attr_chain
      2.  var_attr_chain
      3.  coef_field_name
      4.  ( expr ).suffix          — flux / derivative of parenthesised expr
      5.  numeric * ( sum_expr )   — scalar multiple of a parenthesised sum
      6.  ( sum_expr )             — parenthesised sub-sum (no suffix)

    The leading sign (+/-) is accumulated into `sign` and applied to the
    final result.  Double signs like "+ -(...)" are resolved correctly:
    _parse_single_term receives the full token including the outer "+",
    strips it, then sees "-(...)" and strips that too.
    """
    import re

    # ── Strip all leading sign characters, accumulating into `sign` ──────────
    sign = 1.0
    s = s.strip()
    while s and s[0] in ('+', '-'):
        if s[0] == '-':
            sign *= -1.0
        s = s[1:].strip()

    if not s:
        return None

    # ── Form 4: ( expr ).suffix ───────────────────────────────────────────────
    m = re.match(r'^\((.+)\)((?:\.[a-z]+)+)$', s)
    if m:
        inner  = m.group(1).strip()
        suffix = m.group(2)
        inner_obj = _parse_inner_expr(inner, var_map, coef_map, pde, notes)
        if inner_obj is None:
            return None
        if sign != 1.0:
            inner_obj = sign * inner_obj
        return _apply_suffix(inner_obj, suffix)

    # ── Form 5: numeric * ( sum_expr ) ───────────────────────────────────────
    # Matches  "1e-3 * (u.dx.dx + u.dy.dy)"
    m = re.match(
        r'^([+-]?[0-9]*\.?[0-9]+(?:e[+-]?[0-9]+)?)\s*\*\s*\((.+)\)$',
        s, re.IGNORECASE
    )
    if m:
        coef_lit = float(m.group(1)) * sign
        inner    = m.group(2).strip()
        sub_terms = _parse_subsum(inner, var_map, coef_map, pde, notes)
        if sub_terms is None:
            return None
        # Apply coef_lit to each sub-term and sum — return as a list for
        # sum_eq0; but PDENodesCollector terms can be added with Python +.
        # We fold them with reduce.
        from functools import reduce
        import operator
        scaled = [coef_lit * t for t in sub_terms]
        return reduce(operator.add, scaled)

    # ── Form 6: ( sum_expr ) — bare parenthesised group ──────────────────────
    m = re.match(r'^\((.+)\)$', s)
    if m:
        inner = m.group(1).strip()
        sub_terms = _parse_subsum(inner, var_map, coef_map, pde, notes)
        if sub_terms is None:
            return None
        from functools import reduce
        import operator
        result = reduce(operator.add, sub_terms)
        return sign * result if sign != 1.0 else result

    # ── Form 1: numeric * var_attr_chain ─────────────────────────────────────
    m = re.match(r'^([+-]?[0-9]*\.?[0-9]+(?:e[+-]?[0-9]+)?)\s*\*\s*(.+)$', s, re.IGNORECASE)
    if m:
        coef_lit = float(m.group(1)) * sign
        rest     = m.group(2).strip()
        obj      = _resolve_var_attr_chain(rest, var_map, coef_map, pde, notes)
        if obj is None:
            return None
        return coef_lit * obj

    # ── Form 1b: var_attr_chain * var_attr_chain (product of two fields) ─────
    # Split on the LAST top-level '*' that separates two var expressions.
    # We scan for '*' at depth 0 that is not inside scientific notation.
    prod_idx = _find_top_level_star(s)
    if prod_idx is not None:
        left_s  = s[:prod_idx].strip()
        right_s = s[prod_idx + 1:].strip()
        left_obj  = _resolve_var_attr_chain(left_s,  var_map, coef_map, pde, notes)
        right_obj = _resolve_var_attr_chain(right_s, var_map, coef_map, pde, notes)
        if left_obj is not None and right_obj is not None:
            result = left_obj * right_obj
            return sign * result if sign != 1.0 else result

    # ── Form 2: var_attr_chain ────────────────────────────────────────────────
    obj = _resolve_var_attr_chain(s, var_map, coef_map, pde, notes)
    if obj is not None:
        return sign * obj if sign != 1.0 else obj

    # ── Form 3: coef_field_name ───────────────────────────────────────────────
    if s in coef_map:
        obj = coef_map[s]
        return sign * obj if sign != 1.0 else obj

    return None


def _find_top_level_star(s):
    """
    Find the index of the first '*' at depth 0 that is NOT part of a
    scientific-notation exponent (i.e. not preceded by 'e' or 'E').
    Returns the index, or None if not found.
    """
    depth = 0
    for i, c in enumerate(s):
        if c in ('(', '['):
            depth += 1
        elif c in (')', ']'):
            depth -= 1
        elif c == '*' and depth == 0:
            # Check it's not part of "e*" (shouldn't happen, but guard anyway)
            prev = s[:i].rstrip()
            if prev and prev[-1].lower() not in ('e',):
                return i
    return None


def _parse_subsum(s, var_map, coef_map, pde, notes):
    """
    Parse a sum/difference expression inside parentheses, e.g.
      "u.dx.dx + u.dy.dy"
    Returns a list of term objects, or None on failure.
    """
    raw_terms = _split_top_level(s)
    result = []
    for raw in raw_terms:
        t = _parse_single_term(raw.strip(), var_map, coef_map, pde, notes)
        if t is None:
            notes.append(f"Could not parse sub-term '{raw}' in '({s})'")
            return None
        result.append(t)
    return result if result else None


def _parse_inner_expr(s, var_map, coef_map, pde, notes):
    """
    Parse the inner expression of a flux term ( expr ), e.g.:
      u * v
      c * u
      u^2  or  u.square
      0.5 * u.square
    """
    import re
    s = s.strip()

    # numeric * var_attr
    m = re.match(r'^([+-]?[0-9]*\.?[0-9]+(?:e[+-]?[0-9]+)?)\s*\*\s*(.+)$', s, re.IGNORECASE)
    if m:
        coef_lit = float(m.group(1))
        rest = m.group(2).strip()
        obj  = _resolve_var_attr_chain(rest, var_map, coef_map, pde, notes)
        return None if obj is None else coef_lit * obj

    # var1 * var2
    parts = re.split(r'\s*\*\s*', s)
    if len(parts) == 2:
        a = _resolve_var_attr_chain(parts[0].strip(), var_map, coef_map, pde, notes)
        b = _resolve_var_attr_chain(parts[1].strip(), var_map, coef_map, pde, notes)
        if a is not None and b is not None:
            return a * b

    # u^2 → u.square
    if re.match(r'^(\w+)\^2$', s):
        vname = s[:-2]
        obj = var_map.get(vname)
        return None if obj is None else obj.square

    # single var_attr_chain
    return _resolve_var_attr_chain(s, var_map, coef_map, pde, notes)


def _resolve_var_attr_chain(s, var_map, coef_map, pde, notes):
    """
    Resolve "u", "u.dt", "u.dx.dx", "u.square", etc.
    Also handles known scalar coef names.
    """
    import re
    s = s.strip()

    # Try to split on first '.'
    parts = s.split('.')
    vname = parts[0]

    obj = var_map.get(vname)
    if obj is None:
        # Maybe it's a coef field
        if vname in coef_map:
            obj = coef_map[vname]
            if len(parts) == 1:
                return obj
        return None

    for attr in parts[1:]:
        obj = _apply_one_attr(obj, attr)
        if obj is None:
            notes.append(f"Unknown attribute '{attr}' on variable chain '{s}'")
            return None
    return obj


def _apply_one_attr(obj, attr):
    """Apply a single attribute name to an object."""
    attr = attr.lower()
    if attr == 'dt':
        return obj.dt
    elif attr == 'dx':
        return obj.dx
    elif attr == 'dy':
        return obj.dy
    elif attr == 'square':
        return obj.square
    else:
        return None


def _apply_suffix(obj, suffix):
    """Apply a dot-separated suffix string like '.dx' or '.dx.dx'."""
    for part in suffix.strip('.').split('.'):
        obj = _apply_one_attr(obj, part)
        if obj is None:
            return None
    return obj


def _resolve_var_expr(expr_str, var_map, coef_map, domain_map, pde, notes):
    """Resolve a BC variable expression (may be a var_attr_chain or a coef field)."""
    obj = _resolve_var_attr_chain(expr_str.strip(), var_map, coef_map, pde, notes)
    if obj is None and expr_str.strip() in coef_map:
        obj = coef_map[expr_str.strip()]
    return obj


def _resolve_attr(field_obj, attr_chain):
    """Resolve 'dt', 'dx', 'dt.dx', etc. on a field object."""
    obj = field_obj
    for attr in attr_chain.split('.'):
        obj = _apply_one_attr(obj, attr)
        if obj is None:
            return None
    return obj


# ---------------------------------------------------------------------------
# Legacy single-variable equation builder (backward compat)
# ---------------------------------------------------------------------------

def _build_single_equation(pde, u, equation: str, params: dict):
    """
    Translate a legacy equation string into PDENodesCollector calls.

    Supported patterns:
      - "u_t + (u^2)_x + (c*u)_y = 0"    nonlinear conservation law
      - "u_t - d*(u_xx + u_yy) = 0"       heat / diffusion
      - "u_t + c*u_x = 0"                 linear advection
      - anything else → fallback nonlinear conservation law
    """
    eq = equation.lower().replace(" ", "")
    c  = float(params.get("c", -0.3))
    d  = float(params.get("d", 0.01))

    if "u^2" in eq or "u²" in eq or "(u2)" in eq:
        pde.sum_eq0(pde.dt(u), pde.dx(pde.square(u)), pde.dy(c * u))
    elif "u_xx" in eq or "uxx" in eq or "laplacian" in eq:
        pde.sum_eq0(pde.dt(u), -d * pde.dx(pde.dx(u)), -d * pde.dy(pde.dy(u)))
    elif "c*u_x" in eq or "u_x" in eq:
        pde.sum_eq0(pde.dt(u), c * pde.dx(u), pde.dy(u))
    else:
        pde.sum_eq0(pde.dt(u), pde.dx(pde.square(u)), pde.dy(c * u))


# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

def _flat_to_grid(flat_data, x_fenc, y_fenc):
    """Convert a flat list to a 2-D numpy array, interpolating to 128×128 if needed."""
    from scipy.interpolate import RegularGridInterpolator
    arr = np.array(flat_data, dtype=np.float32)
    n = int(round(len(arr) ** 0.5))
    arr = arr.reshape(n, n)
    if n != 128:
        old_x = np.linspace(0, 1, n)
        interp = RegularGridInterpolator(
            (old_x, old_x), arr, method="linear",
            bounds_error=False, fill_value=None)
        arr = interp(np.stack([x_fenc, y_fenc], axis=-1))
    return arr


def _resolve_ic_value(ic_val, x_fenc, y_fenc, sample_grf):
    """Resolve an IcValue (array or token string) to a numpy array."""
    if isinstance(ic_val, list):
        return _flat_to_grid(ic_val, x_fenc, y_fenc)
    elif isinstance(ic_val, str):
        tok = ic_val.strip().lower()
        if tok == "zero":
            return np.zeros_like(x_fenc, dtype=np.float32)
        elif tok == "grf":
            return sample_grf()
        else:
            raise ValueError(f"Unknown IC token: '{ic_val}'")
    elif isinstance(ic_val, (int, float)):
        return np.full_like(x_fenc, float(ic_val), dtype=np.float32)
    else:
        raise ValueError(f"Unsupported IC value type: {type(ic_val)}")


# ---------------------------------------------------------------------------
# History tensor file loader
# ---------------------------------------------------------------------------

def _load_history_from_file(history_spec: dict):
    """
    Load historical snapshots from an uploaded tensor file.

    Parameters
    ----------
    history_spec : dict
        {
            file_id      : str  — resolved absolute path (set by Rust layer),
            format       : str  — "hdf5" | "npy" | "npz" | "pt"  (optional),
            dataset_key  : str  — HDF5 key or npz array name  (optional),
            input_timesteps : list[int]  — indices to select  (optional),
            variables    : list[str]    — channel names  (optional),
        }

    Returns
    -------
    array : np.ndarray, shape [n_history, n_x, n_y, n_vars]
    var_names : list[str]
    notes : list[str]
    """
    file_path   = history_spec.get("file_id", "")   # Rust sets this to abs path
    fmt         = (history_spec.get("format") or "").lower()
    dataset_key = history_spec.get("dataset_key")
    timesteps   = history_spec.get("input_timesteps")
    var_names   = list(history_spec.get("variables") or [])
    notes       = []

    if not file_path or not os.path.isfile(file_path):
        raise FileNotFoundError(f"History file not found: '{file_path}'")

    # Auto-detect format from extension when not provided.
    if not fmt:
        ext = os.path.splitext(file_path)[1].lstrip(".").lower()
        fmt = {"h5": "hdf5", "hdf5": "hdf5", "npy": "npy",
               "npz": "npz", "pt": "pt", "pth": "pt"}.get(ext, ext)
        notes.append(f"Inferred file format '{fmt}' from extension '.{ext}'")

    # ── Load the array ────────────────────────────────────────────────────────
    if fmt == "hdf5":
        try:
            import h5py
        except ImportError:
            raise ImportError("h5py is required to load HDF5 files. "
                              "Install with: pip install h5py")
        with h5py.File(file_path, "r") as f:
            key = dataset_key or _h5_auto_key(f)
            arr = f[key][()]   # load into memory as np.ndarray
            notes.append(f"Loaded HDF5 dataset '{key}' with shape {arr.shape}")

    elif fmt == "npy":
        arr = np.load(file_path, allow_pickle=False)
        notes.append(f"Loaded .npy array with shape {arr.shape}")

    elif fmt == "npz":
        archive = np.load(file_path, allow_pickle=False)
        if dataset_key:
            if dataset_key not in archive:
                raise KeyError(f"Key '{dataset_key}' not found in npz archive. "
                               f"Available keys: {list(archive.keys())}")
            arr = archive[dataset_key]
        else:
            keys = list(archive.keys())
            if len(keys) == 1:
                arr = archive[keys[0]]
                notes.append(f"Auto-selected npz array '{keys[0]}'")
            else:
                raise KeyError(f"npz archive has multiple arrays {keys}. "
                               f"Specify 'dataset_key'.")
        notes.append(f"Loaded npz array with shape {arr.shape}")

    elif fmt in ("pt", "pth"):
        try:
            import torch
        except ImportError:
            raise ImportError("PyTorch is required to load .pt files. "
                              "Install with: pip install torch")
        tensor = torch.load(file_path, map_location="cpu")
        # Accept Tensor, dict with single Tensor value, or list/tuple of Tensors.
        if isinstance(tensor, dict):
            if dataset_key and dataset_key in tensor:
                tensor = tensor[dataset_key]
            else:
                keys = [k for k, v in tensor.items()
                        if hasattr(v, "numpy")]
                if len(keys) == 1:
                    tensor = tensor[keys[0]]
                    notes.append(f"Auto-selected torch dict key '{keys[0]}'")
                else:
                    raise KeyError(f"Torch dict has multiple tensor keys {keys}. "
                                   f"Specify 'dataset_key'.")
        arr = tensor.numpy() if hasattr(tensor, "numpy") else np.array(tensor)
        notes.append(f"Loaded torch tensor with shape {arr.shape}")

    else:
        raise ValueError(f"Unsupported file format '{fmt}'. "
                         f"Supported: hdf5, npy, npz, pt")

    # ── Normalise to float32 ──────────────────────────────────────────────────
    arr = np.asarray(arr, dtype=np.float32)

    # ── Shape handling ────────────────────────────────────────────────────────
    # Expected: [n_t, n_x, n_y, n_vars]
    # Also accept:
    #   [n_t, n_x, n_y]      → n_vars = 1 (single variable)
    #   [n_x, n_y, n_vars]   → n_t = 1
    #   [n_x, n_y]           → n_t = 1, n_vars = 1
    if arr.ndim == 2:
        arr = arr[np.newaxis, :, :, np.newaxis]   # [1, n_x, n_y, 1]
    elif arr.ndim == 3:
        # Heuristic: if last dim is small (≤16) treat as n_vars channel
        if arr.shape[-1] <= 16 and arr.shape[-1] != arr.shape[-2]:
            arr = arr[np.newaxis]                  # [1, n_x, n_y, n_vars]
        else:
            arr = arr[:, :, :, np.newaxis]         # [n_t, n_x, n_y, 1]
    elif arr.ndim != 4:
        raise ValueError(f"Cannot interpret history tensor with {arr.ndim} dims "
                         f"(shape {arr.shape}). Expected 2-4 dimensional array.")

    n_t, n_x, n_y, n_vars = arr.shape
    notes.append(f"History tensor shape after normalisation: "
                 f"[n_t={n_t}, n_x={n_x}, n_y={n_y}, n_vars={n_vars}]")

    # ── Select time steps ─────────────────────────────────────────────────────
    if timesteps is not None:
        timesteps = list(timesteps)
        for idx in timesteps:
            if idx < 0 or idx >= n_t:
                raise IndexError(f"input_timestep {idx} out of range [0, {n_t})")
        arr = arr[timesteps]
        notes.append(f"Selected {len(timesteps)} timestep(s): {timesteps}")

    # ── Default variable names ────────────────────────────────────────────────
    if not var_names:
        if n_vars == 1:
            var_names = ["u"]
        else:
            var_names = [f"u{i}" for i in range(n_vars)]
        notes.append(f"Auto-assigned variable names: {var_names}")
    elif len(var_names) != n_vars:
        notes.append(
            f"Warning: {len(var_names)} variable name(s) provided but tensor has "
            f"{n_vars} channel(s). Truncating/padding to match."
        )
        if len(var_names) < n_vars:
            var_names += [f"u{i}" for i in range(len(var_names), n_vars)]
        else:
            var_names = var_names[:n_vars]

    return arr, var_names, notes


def _h5_auto_key(h5file):
    """
    Auto-select the first dataset in an HDF5 file (depth-first).
    Raises KeyError if no dataset is found.
    """
    import h5py

    def _find_first_dataset(item, path=""):
        if isinstance(item, h5py.Dataset):
            return path
        for k in item.keys():
            result = _find_first_dataset(item[k], f"{path}/{k}")
            if result is not None:
                return result
        return None

    key = _find_first_dataset(h5file)
    if key is None:
        raise KeyError("No dataset found in HDF5 file. Specify 'dataset_key'.")
    return key


# ---------------------------------------------------------------------------

def _error(msg: str):
    print(json.dumps({"error": msg}))


if __name__ == "__main__":
    main()
