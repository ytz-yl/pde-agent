import { useState } from 'react'
import type { SolveResponse } from '@/types'
import { useI18n } from '@/i18n/context'
import { Heatmap } from './Heatmap'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Clock, Cpu } from 'lucide-react'

export function SolveResult({ result }: { result: SolveResponse }) {
  const { t } = useI18n()
  const [tIdx, setTIdx] = useState(0)
  const [varIdx, setVarIdx] = useState(0)
  const { solution, shape, metadata } = result

  // variables array — fall back to ["u","v","w",...] if not present
  const varNames: string[] = (result as SolveResponse & { variables?: string[] }).variables
    ?? Array.from({ length: shape.n_vars }, (_, i) => String.fromCharCode(117 + i)) // u,v,w,...

  const currentSlice: number[][] =
    solution[tIdx]?.map(row => row.map(col => col[varIdx] ?? 0)) ?? []

  const tValues = solution.map((_, i) => i)

  return (
    <div className="space-y-4">
      {/* ── Meta bar ── */}
      <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
        <div className="flex items-center gap-1.5">
          <Cpu className="h-4 w-4" /><span>{result.solver_used}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Clock className="h-4 w-4" /><span>{metadata.wall_time_ms} ms</span>
        </div>
        <Badge variant="outline" className="font-mono text-xs">
          {shape.n_t}×{shape.n_x}×{shape.n_y}×{shape.n_vars}
        </Badge>
        {metadata.notes.map((n, i) => (
          <span key={i} className="text-xs bg-muted rounded px-2 py-0.5">{n}</span>
        ))}
      </div>

      <Card>
        <CardHeader className="pb-3">
          <CardTitle className="text-base">{t.solver.result.solutionField}</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* ── Variable selector (only shown when n_vars > 1) ── */}
          {shape.n_vars > 1 && (
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground font-medium">
                {t.solver.result.variable}
              </p>
              <div className="flex flex-wrap gap-1.5">
                {varNames.map((vn, i) => (
                  <button
                    key={i}
                    onClick={() => setVarIdx(i)}
                    className={`px-3 py-1 rounded-md text-sm font-mono transition-colors ${
                      i === varIdx
                        ? 'bg-primary text-primary-foreground'
                        : 'bg-muted hover:bg-accent'
                    }`}
                  >
                    {vn}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* ── Time slider ── */}
          {shape.n_t > 1 && (
            <div className="space-y-1">
              <p className="text-xs text-muted-foreground font-medium">
                {t.solver.result.timeStep(tIdx + 1, shape.n_t)}
              </p>
              <input
                type="range" min={0} max={shape.n_t - 1} value={tIdx}
                onChange={e => setTIdx(Number(e.target.value))}
                className="w-full h-1.5 accent-primary"
              />
              <div className="flex justify-between text-xs text-muted-foreground font-mono">
                {tValues.map(i => (
                  <button key={i} onClick={() => setTIdx(i)}
                    className={`px-1.5 py-0.5 rounded transition-colors ${
                      i === tIdx ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'
                    }`}>
                    t{i}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* ── Heatmap ── */}
          <div className="flex justify-center">
            <Heatmap
              data={currentSlice}
              width={360}
              height={360}
              colormap="viridis"
              label={`${varNames[varIdx] ?? 'u'} t=${tIdx}`}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
