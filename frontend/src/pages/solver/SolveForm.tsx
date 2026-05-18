/**
 * SolveForm – case-card selection UI for PDEformer-2 examples.
 *
 * Each card corresponds to one of the notebook examples from
 * PDEformer_inference_CN.ipynb.  Selecting a card fills in the full
 * SolveRequest (multi-variable, domains, BCs, etc.) that is sent to the
 * Rust API, which in turn invokes the Python inference bridge.
 */

import { useState, useMemo } from 'react'
import { solverApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { SolveRequest, SolveResponse, PdeSpec } from '@/types'
import { Button } from '@/components/ui/button'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { SolveResult } from './SolveResult'
import { Play, CheckCircle2 } from 'lucide-react'

// ── Grid helpers ─────────────────────────────────────────────────────────────

function linspace(a: number, b: number, n: number): number[] {
  return Array.from({ length: n }, (_, i) => a + (i / (n - 1)) * (b - a))
}

/** Build a flat n×n array evaluated on a linspace grid over [0,1]². */
function makeGrid(n: number, fn: (x: number, y: number) => number): number[] {
  const xs = linspace(0, 1, n)
  const out: number[] = new Array(n * n)
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n; j++)
      out[i * n + j] = fn(xs[i], xs[j])
  return out
}

// ── Case definitions ─────────────────────────────────────────────────────────

export interface CaseDefinition {
  id: string
  titleZh: string
  titleEn: string
  /** LaTeX-like PDE formula for display */
  pdeFormulaZh: string
  pdeFormulaEn: string
  icDescZh: string
  icDescEn: string
  bcDescZh: string
  bcDescEn: string
  domainDescZh: string
  domainDescEn: string
  varsDisplay: string          // e.g. "u" or "u, v" or "h, u, v"
  /** Callback that builds the PdeSpec given a grid size n */
  buildPde: (n: number) => PdeSpec
  /** Default t snapshots */
  tSnaps: number[]
}

// We pre-build IC arrays at resolution 32 — the Python script will
// interpolate to 128 internally if needed.
const IC_N = 32

function makeConservationLawPde(_n: number): PdeSpec {
  return {
    equation: '',
    variables: ['u'],
    equations: ['u.dt + (u.square).dx + (-0.3 * u).dy = 0'],
    initial_conditions: {
      u: makeGrid(IC_N, (x, y) => Math.exp(-4 * (x - 0.5) ** 2 - 8 * (y - 0.5) ** 2)),
    },
    boundary_condition: 'periodic',
  }
}

function makeAdvectionDiffusionPde(_n: number): PdeSpec {
  const fField = makeGrid(IC_N, (x, y) =>
    Math.exp(-32 * (x - 0.5) ** 2 - 32 * (y - 0.5) ** 2)
  )
  return {
    equation: '',
    variables: ['u'],
    equations: ['u.dt + (0.5 * u).dx + f + -(3e-3 * (u.dx.dx + u.dy.dy)) = 0'],
    initial_conditions: {
      u: makeGrid(IC_N, (x, y) => Math.sin(2 * Math.PI * x) * Math.cos(4 * Math.PI * y)),
    },
    coef_fields: { f: fField },
    boundary_condition: 'periodic',
  }
}

function makeDampedWavePde(_n: number): PdeSpec {
  return {
    equation: '',
    variables: ['u'],
    equations: ['u.dt.dt + 0.3 * u.dt + -(u.dx.dx + u.dy.dy) = 0'],
    initial_conditions: {
      u: 'grf',
      'u.dt': 'zero',
    },
    boundary_condition: 'periodic',
  }
}

function makeBurgers2dPde(_n: number): PdeSpec {
  return {
    equation: '',
    variables: ['u', 'v'],
    equations: [
      'u.dt + (u.square).dx + (u * v).dy + -(1e-3 * (u.dx.dx + u.dy.dy)) = 0',
      'v.dt + (u * v).dx + (v.square).dy + -(1e-3 * (v.dx.dx + v.dy.dy)) = 0',
    ],
    initial_conditions: {
      u: 'grf',
      v: 'grf',
    },
    boundary_condition: 'periodic',
  }
}

function makeNSPde(_n: number): PdeSpec {
  return {
    equation: '',
    variables: ['u', 'v', 'p'],
    equations: [
      'u.dt + (u.square).dx + (u * v).dy + p.dx + -(1e-3 * (u.dx.dx + u.dy.dy)) = 0',
      'v.dt + (u * v).dx + (v.square).dy + p.dy + -(1e-3 * (v.dx.dx + v.dy.dy)) = 0',
      'u.dx + v.dy = 0',
    ],
    initial_conditions: {
      u: 'grf',
      v: 'grf',
    },
    boundary_condition: 'periodic',
  }
}

function makeMixedBcPde(_n: number): PdeSpec {
  const sdfL  = makeGrid(IC_N, (x, _y) => x)               // left boundary (x=0)
  const sdfR  = makeGrid(IC_N, (x, _y) => 1 - x)           // right boundary (x=1)
  const sdfDomain = makeGrid(IC_N, (x, _y) => Math.max(-x, x - 1))

  // g(y) = sin(2πy) evaluated at the right boundary
  const gField = makeGrid(IC_N, (_x, y) => Math.sin(2 * Math.PI * y))

  return {
    equation: '',
    variables: ['u'],
    equations: ['u.dt + (-0.2 * u).dx + (0.5 * u.square).dy + -(3e-3 * (u.dx.dx + u.dy.dy)) = 0'],
    initial_conditions: {
      u: makeGrid(IC_N, (x, y) => Math.sin(2 * Math.PI * x) * Math.cos(4 * Math.PI * y)),
    },
    coef_fields: { g: gField },
    domains: [
      { name: 'boundary_l', sdf: sdfL, role: 'boundary_dirichlet' },
      { name: 'boundary_r', sdf: sdfR, role: 'boundary_dirichlet' },
      { name: 'domain',     sdf: sdfDomain, role: 'interior' },
    ],
    bcs: [
      { domain: 'boundary_l', vars: ['u'],      bc_type: 'dirichlet' },
      { domain: 'boundary_r', vars: ['u', 'g'], bc_type: 'dirichlet' },
    ],
    boundary_condition: 'periodic', // top/bottom remain periodic
  }
}

function makeNeumannWavePde(_n: number): PdeSpec {
  const sdfDomain = makeGrid(IC_N, (x, _y) => Math.max(-x, x - 1))
  const sdfLR     = makeGrid(IC_N, (x, _y) => Math.min(x, 1 - x))

  return {
    equation: '',
    variables: ['u'],
    equations: ['u.dt.dt + -(2 * (u.dx.dx + u.dy.dy)) = 0'],
    initial_conditions: {
      u: makeGrid(IC_N, (x, y) => Math.sin(2 * Math.PI * x) * Math.cos(4 * Math.PI * y)),
      'u.dt': 'zero',
    },
    domains: [
      { name: 'domain',     sdf: sdfDomain, role: 'interior' },
      { name: 'boundary_lr', sdf: sdfLR,   role: 'boundary_neumann' },
    ],
    bcs: [
      { domain: 'boundary_lr', vars: ['u'], bc_type: 'neumann' },
    ],
    boundary_condition: 'periodic', // top/bottom periodic
  }
}

function makeMurWavePde(_n: number): PdeSpec {
  const x0 = 0.5, y0 = 0.6, r0 = 0.4, c = 0.7
  const diskSdf   = makeGrid(IC_N, (x, y) => Math.sqrt((x - x0) ** 2 + (y - y0) ** 2) - r0)
  const bdySdf    = makeGrid(IC_N, (x, y) => Math.abs(Math.sqrt((x - x0) ** 2 + (y - y0) ** 2) - r0))

  return {
    equation: '',
    variables: ['u'],
    equations: [`u.dt.dt + -(${c * c} * (u.dx.dx + u.dy.dy)) = 0`],
    initial_conditions: {
      u: makeGrid(IC_N, (x, y) => Math.sin(2 * Math.PI * x) * Math.cos(4 * Math.PI * y)),
      'u.dt': 'zero',
    },
    domains: [
      { name: 'disk_domain',  sdf: diskSdf, role: 'interior' },
      { name: 'mur_boundary', sdf: bdySdf,  role: 'boundary_mur' },
    ],
    bcs: [
      { domain: 'mur_boundary', vars: ['u.dt', 'u'], bc_type: 'mur', coef: c },
    ],
    boundary_condition: 'periodic',
  }
}

function makeShallowWaterPde(_n: number): PdeSpec {
  const g = 0.1
  return {
    equation: '',
    variables: ['h', 'u', 'v'],
    equations: [
      'h.dt + (h * u).dx + (h * v).dy = 0',
      `u.dt + u * u.dx + v * u.dy + ${g} * h.dx = 0`,
      `v.dt + u * v.dx + v * v.dy + ${g} * h.dy = 0`,
    ],
    initial_conditions: {
      h: makeGrid(IC_N, (x, y) => 1.5 + 0.5 * Math.sin(2 * Math.PI * x) * Math.sin(2 * Math.PI * y)),
      u: 'zero',
      v: 'zero',
    },
    boundary_condition: 'periodic',
  }
}

// ── Case list ────────────────────────────────────────────────────────────────

export const CASES: CaseDefinition[] = [
  {
    id: 'conservation_law',
    titleZh: '2D 非线性守恒律',
    titleEn: '2D Nonlinear Conservation Law',
    pdeFormulaZh: 'u_t + (u²)_x + (−0.3u)_y = 0',
    pdeFormulaEn: 'u_t + (u²)_x + (−0.3u)_y = 0',
    icDescZh: 'u(0,x,y) = exp(−4(x−½)² − 8(y−½)²)',
    icDescEn: 'u(0,x,y) = exp(−4(x−½)² − 8(y−½)²)',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²',
    domainDescEn: '[0,1]²',
    varsDisplay: 'u',
    buildPde: makeConservationLawPde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'advection_diffusion',
    titleZh: '有粘性对流扩散（含源项）',
    titleEn: 'Viscous Advection-Diffusion with Source',
    pdeFormulaZh: 'u_t + (½u)_x + f(x,y) − 3×10⁻³Δu = 0',
    pdeFormulaEn: 'u_t + (½u)_x + f(x,y) − 3×10⁻³Δu = 0',
    icDescZh: 'u(0,x,y) = sin(2πx)cos(4πy)',
    icDescEn: 'u(0,x,y) = sin(2πx)cos(4πy)',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²，源项 f = exp(−32(x−½)²−32(y−½)²)',
    domainDescEn: '[0,1]²，source f = exp(−32(x−½)²−32(y−½)²)',
    varsDisplay: 'u',
    buildPde: makeAdvectionDiffusionPde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'damped_wave',
    titleZh: '带阻尼波方程',
    titleEn: 'Damped Wave Equation',
    pdeFormulaZh: 'u_tt + 0.3u_t − Δu = 0',
    pdeFormulaEn: 'u_tt + 0.3u_t − Δu = 0',
    icDescZh: 'u(0) = GRF，u_t(0) = 0',
    icDescEn: 'u(0) = GRF, u_t(0) = 0',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²',
    domainDescEn: '[0,1]²',
    varsDisplay: 'u',
    buildPde: makeDampedWavePde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'burgers_2d',
    titleZh: '2D 有粘性 Burgers（双变量）',
    titleEn: '2D Viscous Burgers (Two-Variable)',
    pdeFormulaZh: 'u_t+(u²)_x+(uv)_y−10⁻³Δu=0，v_t+(uv)_x+(v²)_y−10⁻³Δv=0',
    pdeFormulaEn: 'u_t+(u²)_x+(uv)_y−10⁻³Δu=0, v_t+(uv)_x+(v²)_y−10⁻³Δv=0',
    icDescZh: 'u(0), v(0) 均为 GRF',
    icDescEn: 'u(0), v(0) both GRF',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²',
    domainDescEn: '[0,1]²',
    varsDisplay: 'u, v',
    buildPde: makeBurgers2dPde,
    tSnaps: [0, 0.25, 0.5, 0.75, 1.0],
  },
  {
    id: 'navier_stokes',
    titleZh: '2D 不可压 NS 方程',
    titleEn: '2D Incompressible Navier-Stokes',
    pdeFormulaZh: 'u_t+(u²)_x+(uv)_y+p_x−10⁻³Δu=0，v_t+(uv)_x+(v²)_y+p_y−10⁻³Δv=0，u_x+v_y=0',
    pdeFormulaEn: 'u_t+(u²)_x+(uv)_y+p_x−10⁻³Δu=0, v_t+(uv)_x+(v²)_y+p_y−10⁻³Δv=0, u_x+v_y=0',
    icDescZh: 'u(0), v(0) 均为 GRF',
    icDescEn: 'u(0), v(0) both GRF',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²',
    domainDescEn: '[0,1]²',
    varsDisplay: 'u, v, p',
    buildPde: makeNSPde,
    tSnaps: [0, 0.25, 0.5, 0.75, 1.0],
  },
  {
    id: 'mixed_bc',
    titleZh: '周期 + Dirichlet 混合边界对流扩散',
    titleEn: 'Periodic + Dirichlet Mixed BC Advection-Diffusion',
    pdeFormulaZh: 'u_t+(−0.2u)_x+(0.5u²)_y−3×10⁻³Δu=0',
    pdeFormulaEn: 'u_t+(−0.2u)_x+(0.5u²)_y−3×10⁻³Δu=0',
    icDescZh: 'u(0) = sin(2πx)cos(4πy)',
    icDescEn: 'u(0) = sin(2πx)cos(4πy)',
    bcDescZh: '上下周期，左 u|L=0，右 (u+g)|R=0，g=sin(2πy)',
    bcDescEn: 'Top/bottom periodic; left u|L=0; right (u+g)|R=0, g=sin(2πy)',
    domainDescZh: '[0,1]²（非方形端点）',
    domainDescEn: '[0,1]² (non-periodic x-edges)',
    varsDisplay: 'u',
    buildPde: makeMixedBcPde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'neumann_wave',
    titleZh: '周期 + Neumann 边界波方程',
    titleEn: 'Periodic + Neumann BC Wave Equation',
    pdeFormulaZh: 'u_tt − ∇·(2∇u) = 0',
    pdeFormulaEn: 'u_tt − ∇·(2∇u) = 0',
    icDescZh: 'u(0) = sin(2πx)cos(4πy)，u_t(0) = 0',
    icDescEn: 'u(0) = sin(2πx)cos(4πy), u_t(0) = 0',
    bcDescZh: '上下周期，左右 Neumann (∂u/∂n = 0)',
    bcDescEn: 'Top/bottom periodic; left/right Neumann (∂u/∂n = 0)',
    domainDescZh: '[0,1]²（x 方向非周期）',
    domainDescEn: '[0,1]² (non-periodic x-edges)',
    varsDisplay: 'u',
    buildPde: makeNeumannWavePde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'mur_wave',
    titleZh: '圆盘区域 Mur 吸收边界波方程',
    titleEn: 'Disk Domain Wave Equation with Mur ABC',
    pdeFormulaZh: 'u_tt − 0.49Δu = 0，吸收边界 u_t + 0.7∂ₙu = 0',
    pdeFormulaEn: 'u_tt − 0.49Δu = 0; Mur ABC u_t + 0.7∂ₙu = 0',
    icDescZh: 'u(0) = sin(2πx)cos(4πy)，u_t(0) = 0',
    icDescEn: 'u(0) = sin(2πx)cos(4πy), u_t(0) = 0',
    bcDescZh: 'Mur（吸收）边界',
    bcDescEn: 'Mur absorbing boundary',
    domainDescZh: '圆盘 (0.5, 0.6)，半径 0.4',
    domainDescEn: 'Disk at (0.5, 0.6), radius 0.4',
    varsDisplay: 'u',
    buildPde: makeMurWavePde,
    tSnaps: [0, 0.2, 0.4, 0.6, 0.8, 1.0],
  },
  {
    id: 'shallow_water',
    titleZh: '浅水波方程（三变量）',
    titleEn: 'Shallow Water Equations (Three-Variable)',
    pdeFormulaZh: 'h_t+(hu)_x+(hv)_y=0，u_t+uu_x+vu_y+0.1h_x=0，v_t+uv_x+vv_y+0.1h_y=0',
    pdeFormulaEn: 'h_t+(hu)_x+(hv)_y=0, u_t+uu_x+vu_y+0.1h_x=0, v_t+uv_x+vv_y+0.1h_y=0',
    icDescZh: 'h(0) = 1.5+0.5sin(2πx)sin(2πy)，u=v=0',
    icDescEn: 'h(0) = 1.5+0.5sin(2πx)sin(2πy), u=v=0',
    bcDescZh: '周期边界',
    bcDescEn: 'Periodic',
    domainDescZh: '[0,1]²',
    domainDescEn: '[0,1]²',
    varsDisplay: 'h, u, v',
    buildPde: makeShallowWaterPde,
    tSnaps: [0, 0.25, 0.5, 0.75, 1.0],
  },
]

// ── Case card component ───────────────────────────────────────────────────────

function CaseCard({
  case_: c,
  selected,
  onSelect,
  locale,
}: {
  case_: CaseDefinition
  selected: boolean
  onSelect: () => void
  locale: 'zh' | 'en'
}) {
  const isZh = locale === 'zh'
  return (
    <button
      onClick={onSelect}
      className={`text-left w-full rounded-lg border p-3 transition-all focus:outline-none focus-visible:ring-2 focus-visible:ring-primary ${
        selected
          ? 'border-primary bg-primary/5 shadow-sm'
          : 'border-border hover:border-primary/50 hover:bg-accent/40'
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <span className="text-sm font-medium leading-snug">
          {isZh ? c.titleZh : c.titleEn}
        </span>
        {selected && <CheckCircle2 className="h-4 w-4 text-primary shrink-0 mt-0.5" />}
      </div>
      <code className="mt-1 block text-[11px] text-muted-foreground font-mono leading-tight truncate">
        {isZh ? c.pdeFormulaZh : c.pdeFormulaEn}
      </code>
      <div className="mt-1.5 flex flex-wrap gap-1">
        <span className="text-[10px] bg-muted rounded px-1.5 py-0.5 font-mono text-muted-foreground">
          {c.varsDisplay}
        </span>
        <span className="text-[10px] bg-muted rounded px-1.5 py-0.5 text-muted-foreground">
          {isZh ? c.bcDescZh : c.bcDescEn}
        </span>
      </div>
    </button>
  )
}

// ── CaseDetail panel ──────────────────────────────────────────────────────────

function CaseDetail({ case_: c, locale }: { case_: CaseDefinition; locale: 'zh' | 'en' }) {
  const isZh = locale === 'zh'
  const rows: { label: string; value: string }[] = [
    { label: isZh ? '变量'     : 'Variables',          value: c.varsDisplay },
    { label: isZh ? 'PDE 形式' : 'PDE form',           value: isZh ? c.pdeFormulaZh : c.pdeFormulaEn },
    { label: isZh ? '初始条件' : 'Initial condition',  value: isZh ? c.icDescZh     : c.icDescEn },
    { label: isZh ? '边界条件' : 'Boundary condition', value: isZh ? c.bcDescZh     : c.bcDescEn },
    { label: isZh ? '定义域'   : 'Domain',             value: isZh ? c.domainDescZh : c.domainDescEn },
  ]
  return (
    <div className="rounded-lg border border-border bg-muted/40 p-4 space-y-2">
      {rows.map(r => (
        <div key={r.label} className="grid grid-cols-[7rem_1fr] gap-2 text-sm">
          <span className="text-muted-foreground font-medium shrink-0">{r.label}</span>
          <span className="font-mono text-xs break-all">{r.value}</span>
        </div>
      ))}
    </div>
  )
}

// ── Main component ────────────────────────────────────────────────────────────

export function SolveForm({ solverIds }: { solverIds: string[] }) {
  const { t, locale } = useI18n()
  const ct = t.solver.cases

  const [selectedCaseId, setSelectedCaseId] = useState<string>(CASES[0].id)
  const [solver, setSolver]   = useState(solverIds[0] ?? 'pdeformer2')
  const [resolution, setResolution] = useState(32)
  const [loading, setLoading] = useState(false)
  const [result, setResult]   = useState<SolveResponse | null>(null)
  const [error, setError]     = useState<string | null>(null)

  const selectedCase = useMemo(
    () => CASES.find(c => c.id === selectedCaseId) ?? CASES[0],
    [selectedCaseId]
  )

  async function handleRun() {
    setLoading(true); setError(null); setResult(null)
    const coords = linspace(0, 1, resolution)
    const pde = selectedCase.buildPde(resolution)
    const req: SolveRequest = {
      solver: solver || undefined,
      pde,
      query: { x: coords, y: coords, t: selectedCase.tSnaps },
    }
    try {
      const resp = await solverApi.solve(req)
      if (!resp.success || !resp.data) throw new Error(resp.error ?? t.common.error)
      setResult(resp.data)
    } catch (err) {
      setError(err instanceof Error ? err.message : t.common.error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      {/* ── Case selector grid ── */}
      <div className="space-y-2">
        <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
          {ct.selectCase}
        </p>
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
          {CASES.map(c => (
            <CaseCard
              key={c.id}
              case_={c}
              selected={c.id === selectedCaseId}
              onSelect={() => { setSelectedCaseId(c.id); setResult(null); setError(null) }}
              locale={locale}
            />
          ))}
        </div>
      </div>

      {/* ── Selected case detail ── */}
      <CaseDetail case_={selectedCase} locale={locale} />

      {/* ── Run controls ── */}
      <div className="flex flex-wrap items-end gap-4">
        <div className="space-y-1.5">
          <label className="text-sm font-medium">{ct.solver}</label>
          <Select value={solver} onChange={e => setSolver(e.target.value)}>
            {solverIds.map(id => <option key={id} value={id}>{id}</option>)}
          </Select>
        </div>

        <div className="space-y-1.5">
          <label className="text-sm font-medium">{ct.resolution}</label>
          <div className="flex items-center gap-2">
            <input
              type="number" min={8} max={64} value={resolution}
              onChange={e => setResolution(Number(e.target.value))}
              className="w-16 rounded-md border border-input bg-background px-2 py-1.5 text-sm"
            />
            <span className="text-xs text-muted-foreground">× {resolution} {ct.resolutionSuffix}</span>
          </div>
        </div>

        <Button onClick={handleRun} disabled={loading}>
          {loading
            ? <><Spinner size="sm" className="mr-2" />{ct.running}</>
            : <><Play className="h-4 w-4 mr-2" />{ct.runButton}</>
          }
        </Button>
      </div>

      {/* ── Error ── */}
      {error && (
        <div className="rounded-md bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 text-sm">
          {error}
        </div>
      )}

      {/* ── Result ── */}
      {result && <SolveResult result={result} />}
    </div>
  )
}
