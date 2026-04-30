import { useState, useEffect } from 'react'
import { solverApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { SolverInfo } from '@/types'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Spinner } from '@/components/ui/spinner'
import { SolveForm } from './SolveForm'
import { CheckCircle2, XCircle } from 'lucide-react'

function SolverCard({ solver }: { solver: SolverInfo }) {
  const { t } = useI18n()
  const labels = t.solver.catalog.categoryLabels
  const variants: Record<SolverInfo['category'], 'default' | 'secondary' | 'outline'> = {
    classical: 'default', machine_learning: 'secondary', hybrid: 'outline',
  }
  return (
    <Card className={solver.available ? '' : 'opacity-60'}>
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <div>
            <CardTitle className="text-base flex items-center gap-1.5">
              {solver.available
                ? <CheckCircle2 className="h-4 w-4 text-green-500 shrink-0" />
                : <XCircle className="h-4 w-4 text-muted-foreground shrink-0" />}
              {solver.name}
            </CardTitle>
            <CardDescription className="font-mono text-xs">{solver.id}</CardDescription>
          </div>
          <Badge variant={variants[solver.category]}>{labels[solver.category]}</Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-2">
        {solver.description && <p className="text-sm text-muted-foreground">{solver.description}</p>}
        {solver.supported_pde_types.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {solver.supported_pde_types.map(tp => (
              <Badge key={tp} variant="outline" className="text-xs">{tp.replace(/_/g, ' ')}</Badge>
            ))}
          </div>
        )}
        <p className="text-xs text-muted-foreground font-mono">
          {t.solver.catalog.backend}: {solver.backend}
        </p>
      </CardContent>
    </Card>
  )
}

type Tab = 'catalog' | 'solve'

export function SolverPage() {
  const { t } = useI18n()
  const [solvers, setSolvers] = useState<SolverInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [tab, setTab] = useState<Tab>('catalog')

  useEffect(() => {
    solverApi.listSolvers()
      .then(resp => {
        if (resp.success && resp.data) setSolvers(resp.data)
        else setError(resp.error ?? t.common.error)
      })
      .catch(err => setError(err instanceof Error ? err.message : t.common.error))
      .finally(() => setLoading(false))
  }, [])

  const availableIds = solvers.filter(s => s.available).map(s => s.id)
  const tabs: { key: Tab; label: string }[] = [
    { key: 'catalog', label: t.solver.tabs.catalog },
    { key: 'solve',   label: t.solver.tabs.solve },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold">{t.solver.title}</h1>
        <p className="text-muted-foreground text-sm mt-1">{t.solver.subtitle}</p>
      </div>

      <div className="flex gap-1 border-b">
        {tabs.map(tb => (
          <button key={tb.key} onClick={() => setTab(tb.key)}
            className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
              tab === tb.key ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}>
            {tb.label}
          </button>
        ))}
      </div>

      {loading && <div className="flex justify-center py-16"><Spinner size="lg" /></div>}
      {error && <p className="text-sm text-destructive">{error}</p>}

      {!loading && !error && tab === 'catalog' && (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {solvers.map(s => <SolverCard key={s.id} solver={s} />)}
          {solvers.length === 0 && (
            <p className="text-muted-foreground text-sm col-span-full">{t.solver.catalog.noSolvers}</p>
          )}
        </div>
      )}

      {!loading && !error && tab === 'solve' && (
        <SolveForm solverIds={availableIds.length > 0 ? availableIds : ['pdeformer2']} />
      )}
    </div>
  )
}
