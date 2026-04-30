import { useState } from 'react'
import { solverApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { SolveRequest, SolveResponse } from '@/types'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Label } from '@/components/ui/label'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { SolveResult } from './SolveResult'
import { Play } from 'lucide-react'

const EXAMPLE_PRESETS = [
  { label: 'Heat equation (2D)', equation: 'u_t = 0.01 * (u_xx + u_yy)', bc: 'dirichlet' },
  { label: 'Conservation law',   equation: 'u_t + (u^2)_x + (-0.3*u)_y = 0',   bc: 'periodic' },
  { label: "Burgers' equation",  equation: 'u_t + u*u_x = 0.001*u_xx',          bc: 'periodic' },
]

function makeDefaultIc(n = 128): number[] {
  const arr: number[] = new Array(n * n).fill(0)
  const cx = Math.floor(n / 2), r = Math.floor(n / 6)
  for (let i = 0; i < n; i++)
    for (let j = 0; j < n; j++)
      if (Math.sqrt((i - cx) ** 2 + (j - cx) ** 2) < r) arr[i * n + j] = 1.0
  return arr
}

function linspace(a: number, b: number, n: number): number[] {
  return Array.from({ length: n }, (_, i) => a + (i / (n - 1)) * (b - a))
}

export function SolveForm({ solverIds }: { solverIds: string[] }) {
  const { t } = useI18n()
  const ft = t.solver.form
  const [solver, setSolver] = useState(solverIds[0] ?? 'pdeformer2')
  const [equation, setEquation] = useState(EXAMPLE_PRESETS[0].equation)
  const [bc, setBc] = useState(EXAMPLE_PRESETS[0].bc)
  const [resolution, setResolution] = useState(32)
  const [useIc, setUseIc] = useState(false)
  const [loading, setLoading] = useState(false)
  const [result, setResult] = useState<SolveResponse | null>(null)
  const [error, setError] = useState<string | null>(null)

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    if (!equation.trim()) return
    setLoading(true); setError(null); setResult(null)
    const coords = linspace(0, 1, resolution)
    const req: SolveRequest = {
      solver: solver || undefined,
      pde: { equation, boundary_condition: bc || undefined, initial_condition: useIc ? makeDefaultIc() : undefined },
      query: { x: coords, y: coords, t: [0, 0.25, 0.5, 0.75, 1.0] },
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
      <div className="space-y-1.5">
        <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide">{ft.presets}</p>
        <div className="flex flex-wrap gap-2">
          {EXAMPLE_PRESETS.map(p => (
            <button
              key={p.label}
              onClick={() => { setEquation(p.equation); setBc(p.bc) }}
              className="text-xs border border-border rounded-md px-3 py-1.5 hover:bg-accent transition-colors"
            >
              {p.label}
            </button>
          ))}
        </div>
      </div>

      <form onSubmit={handleSubmit} className="space-y-4">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-1.5">
            <Label>{ft.solver}</Label>
            <Select value={solver} onChange={e => setSolver(e.target.value)}>
              {solverIds.map(id => <option key={id} value={id}>{id}</option>)}
            </Select>
          </div>
          <div className="space-y-1.5">
            <Label>{ft.bc}</Label>
            <Select value={bc} onChange={e => setBc(e.target.value)}>
              <option value="">{ft.bcOptions.unspecified}</option>
              <option value="periodic">{ft.bcOptions.periodic}</option>
              <option value="dirichlet">{ft.bcOptions.dirichlet}</option>
              <option value="neumann">{ft.bcOptions.neumann}</option>
            </Select>
          </div>
        </div>

        <div className="space-y-1.5">
          <Label>{ft.equation}</Label>
          <Textarea
            rows={2}
            placeholder={ft.equationPlaceholder}
            value={equation}
            onChange={e => setEquation(e.target.value)}
          />
          <p className="text-xs text-muted-foreground">
            {ft.equationHint}{' '}
            <code className="font-mono bg-muted rounded px-1">u_t</code>,{' '}
            <code className="font-mono bg-muted rounded px-1">u_x</code>,{' '}
            <code className="font-mono bg-muted rounded px-1">u_xx</code>{' '}
            {ft.equationHint2}
          </p>
        </div>

        <div className="flex items-center gap-6 flex-wrap">
          <div className="space-y-1.5">
            <Label>{ft.resolution}</Label>
            <div className="flex items-center gap-2">
              <Input type="number" min={8} max={128} value={resolution}
                onChange={e => setResolution(Number(e.target.value))} className="w-20" />
              <span className="text-xs text-muted-foreground">× {resolution} {ft.resolutionSuffix}</span>
            </div>
          </div>
          <label className="flex items-center gap-2 text-sm cursor-pointer select-none">
            <input type="checkbox" checked={useIc} onChange={e => setUseIc(e.target.checked)} className="accent-primary" />
            {ft.sendIc}
          </label>
        </div>

        <Button type="submit" disabled={loading || !equation.trim()}>
          {loading ? <Spinner size="sm" className="mr-2" /> : <Play className="h-4 w-4 mr-2" />}
          {ft.runButton}
        </Button>
      </form>

      {error && (
        <div className="rounded-md bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 text-sm">
          {error}
        </div>
      )}
      {result && <SolveResult result={result} />}
    </div>
  )
}
