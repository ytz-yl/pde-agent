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
  const { solution, shape, metadata } = result

  const currentSlice: number[][] = solution[tIdx]?.map(row => row.map(col => col[0])) ?? []
  const tValues = solution.map((_, i) => i)

  return (
    <div className="space-y-4">
      <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
        <div className="flex items-center gap-1.5">
          <Cpu className="h-4 w-4" /><span>{result.solver_used}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <Clock className="h-4 w-4" /><span>{metadata.wall_time_ms} ms</span>
        </div>
        <Badge variant="outline" className="font-mono text-xs">
          {shape.n_t}×{shape.n_x}×{shape.n_y}
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
                    className={`px-1.5 py-0.5 rounded transition-colors ${i === tIdx ? 'bg-primary text-primary-foreground' : 'hover:bg-accent'}`}>
                    t{i}
                  </button>
                ))}
              </div>
            </div>
          )}
          <div className="flex justify-center">
            <Heatmap data={currentSlice} width={360} height={360} colormap="viridis" label={`t = ${tIdx}`} />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
