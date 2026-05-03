import { useState, useEffect } from 'react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { Equation, EquationSolvers, EquationConditions, DatasetRef, PaperRef } from '@/types'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { ChevronRight, FlaskConical, BookOpen, Database, FileText } from 'lucide-react'

// ── PDE type badge ────────────────────────────────────────────────────────────

function PdeTypeBadge({ type }: { type: string }) {
  const variants: Record<string, string> = {
    parabolic: 'bg-blue-50 text-blue-700 border-blue-200',
    elliptic: 'bg-green-50 text-green-700 border-green-200',
    hyperbolic: 'bg-orange-50 text-orange-700 border-orange-200',
    mixed: 'bg-purple-50 text-purple-700 border-purple-200',
    other: 'bg-gray-50 text-gray-600 border-gray-200',
  }
  const cls = variants[type] ?? variants.other
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${cls}`}>
      {type}
    </span>
  )
}

// ── Equation list ─────────────────────────────────────────────────────────────

function EquationList({ equations, selected, onSelect }: {
  equations: Equation[]
  selected: string | null
  onSelect: (id: string) => void
}) {
  const [filter, setFilter] = useState('')
  const visible = filter ? equations.filter(e => e.pde_type === filter) : equations

  return (
    <div className="space-y-3">
      <Select value={filter} onChange={e => setFilter(e.target.value)} className="w-40">
        <option value="">All types</option>
        <option value="parabolic">Parabolic</option>
        <option value="elliptic">Elliptic</option>
        <option value="hyperbolic">Hyperbolic</option>
        <option value="mixed">Mixed</option>
      </Select>
      <div className="space-y-1.5">
        {visible.map(eq => (
          <button
            key={eq.id}
            onClick={() => onSelect(eq.id)}
            className={`w-full text-left rounded-md border px-3 py-2.5 text-sm transition-colors ${
              selected === eq.id ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
            }`}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium truncate">{eq.name}</span>
              <PdeTypeBadge type={eq.pde_type} />
            </div>
            {eq.description && (
              <p className="text-muted-foreground text-xs mt-1 line-clamp-2">{eq.description}</p>
            )}
          </button>
        ))}
      </div>
    </div>
  )
}

// ── Equation detail ───────────────────────────────────────────────────────────

function EquationDetail({ equationId }: { equationId: string }) {
  const [eq, setEq] = useState<Equation | null>(null)
  const [solvers, setSolvers] = useState<EquationSolvers | null>(null)
  const [conditions, setConditions] = useState<EquationConditions | null>(null)
  const [datasets, setDatasets] = useState<DatasetRef[]>([])
  const [papers, setPapers] = useState<PaperRef[]>([])
  const [loading, setLoading] = useState(true)
  const [activeSection, setActiveSection] = useState<'solvers' | 'conditions' | 'datasets' | 'papers'>('solvers')

  useEffect(() => {
    setLoading(true)
    Promise.all([
      knowledgeApi.getEquation(equationId),
      knowledgeApi.equationSolvers(equationId),
      knowledgeApi.equationConditions(equationId),
      knowledgeApi.equationDatasets(equationId),
      knowledgeApi.equationPapers(equationId),
    ]).then(([e, s, c, d, p]) => {
      setEq(e)
      setSolvers(s)
      setConditions(c)
      setDatasets(d)
      setPapers(p)
    }).finally(() => setLoading(false))
  }, [equationId])

  if (loading) return <div className="flex justify-center py-16"><Spinner /></div>
  if (!eq) return null

  const sections = [
    { key: 'solvers' as const, label: 'Solvers', icon: FlaskConical,
      count: (solvers?.ai_models.length ?? 0) + (solvers?.numerical_methods.length ?? 0) },
    { key: 'conditions' as const, label: 'Conditions', icon: ChevronRight,
      count: conditions?.conditions.length ?? 0 },
    { key: 'datasets' as const, label: 'Datasets', icon: Database, count: datasets.length },
    { key: 'papers' as const, label: 'Papers', icon: BookOpen, count: papers.length },
  ]

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="space-y-2">
        <div className="flex items-start justify-between gap-3">
          <h2 className="text-xl font-semibold">{eq.name}</h2>
          <PdeTypeBadge type={eq.pde_type} />
        </div>
        {eq.description && (
          <p className="text-sm text-muted-foreground leading-relaxed">{eq.description}</p>
        )}
        <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
          <span>Variables: <span className="font-mono">{eq.variables.join(', ')}</span></span>
          <span>{eq.time_dependent ? 'Time-dependent' : 'Steady-state'}</span>
          {eq.operator && <span>Operator: <span className="font-mono">{eq.operator}</span></span>}
        </div>
        {eq.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {eq.tags.map(tag => (
              <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>
            ))}
          </div>
        )}
      </div>

      {/* Section tabs */}
      <div className="flex gap-1 border-b">
        {sections.map(s => (
          <button
            key={s.key}
            onClick={() => setActiveSection(s.key)}
            className={`flex items-center gap-1.5 px-3 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
              activeSection === s.key
                ? 'border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            <s.icon className="h-3.5 w-3.5" />
            {s.label}
            {s.count > 0 && (
              <span className="rounded-full bg-muted px-1.5 py-0.5 text-xs font-normal">{s.count}</span>
            )}
          </button>
        ))}
      </div>

      {/* Solvers section */}
      {activeSection === 'solvers' && solvers && (
        <div className="space-y-4">
          {solvers.ai_models.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                AI / ML Models
              </h4>
              <div className="space-y-1.5">
                {solvers.ai_models.map(m => (
                  <div key={m.id} className="rounded-md border px-3 py-2 text-sm">
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{m.name}</span>
                      <span className="text-xs font-mono text-muted-foreground">{m.architecture}</span>
                    </div>
                    {m.description && (
                      <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{m.description}</p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
          {solvers.numerical_methods.length > 0 && (
            <div>
              <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">
                Numerical Methods
              </h4>
              <div className="space-y-1.5">
                {solvers.numerical_methods.map(m => (
                  <div key={m.id} className="rounded-md border px-3 py-2 text-sm">
                    <div className="flex items-center justify-between">
                      <span className="font-medium">{m.name}</span>
                      {m.order != null && m.order > 0 && (
                        <span className="text-xs text-muted-foreground">Order {m.order}</span>
                      )}
                    </div>
                    {m.description && (
                      <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">{m.description}</p>
                    )}
                  </div>
                ))}
              </div>
            </div>
          )}
          {solvers.ai_models.length === 0 && solvers.numerical_methods.length === 0 && (
            <p className="text-sm text-muted-foreground">No solvers linked yet.</p>
          )}
        </div>
      )}

      {/* Conditions section */}
      {activeSection === 'conditions' && conditions && (
        <div className="space-y-1.5">
          {conditions.conditions.length === 0 && (
            <p className="text-sm text-muted-foreground">No conditions linked yet.</p>
          )}
          {conditions.conditions.map(c => (
            <div key={c.id} className="rounded-md border px-3 py-2 text-sm">
              <div className="flex items-center justify-between gap-2">
                <span className="font-medium">{c.name}</span>
                <Badge variant="outline" className="text-xs">{c.condition_type}</Badge>
              </div>
              {c.form && <p className="font-mono text-xs mt-0.5 text-muted-foreground">{c.form}</p>}
            </div>
          ))}
        </div>
      )}

      {/* Datasets section */}
      {activeSection === 'datasets' && (
        <div className="space-y-1.5">
          {datasets.length === 0 && (
            <p className="text-sm text-muted-foreground">No datasets linked yet.</p>
          )}
          {datasets.map(d => (
            <div key={d.id} className="flex items-center justify-between rounded-md border px-3 py-2 text-sm">
              <span className="font-medium">{d.name}</span>
              {d.dimension && <span className="text-xs text-muted-foreground">{d.dimension}</span>}
            </div>
          ))}
        </div>
      )}

      {/* Papers section */}
      {activeSection === 'papers' && (
        <div className="space-y-1.5">
          {papers.length === 0 && (
            <p className="text-sm text-muted-foreground">No papers linked yet.</p>
          )}
          {papers.map(p => (
            <div key={p.id} className="rounded-md border px-3 py-2 text-sm">
              <div className="flex items-start justify-between gap-2">
                <span className="font-medium leading-snug">{p.title}</span>
                {p.published_year && (
                  <span className="text-xs text-muted-foreground shrink-0">{p.published_year}</span>
                )}
              </div>
              {p.arxiv_id && (
                <a
                  href={`https://arxiv.org/abs/${p.arxiv_id}`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-xs text-primary hover:underline mt-0.5 inline-flex items-center gap-1"
                >
                  <FileText className="h-3 w-3" />
                  arXiv:{p.arxiv_id}
                </a>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}

// ── Main ──────────────────────────────────────────────────────────────────────

export function EquationBrowser() {
  const { t } = useI18n()
  const [equations, setEquations] = useState<Equation[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<string | null>(null)

  useEffect(() => {
    knowledgeApi.listEquations().then(setEquations).finally(() => setLoading(false))
  }, [])

  if (loading) return <div className="flex justify-center py-16"><Spinner size="lg" /></div>

  return (
    <div className="grid grid-cols-5 gap-6">
      <div className="col-span-2">
        <EquationList equations={equations} selected={selected} onSelect={setSelected} />
      </div>
      <div className="col-span-3">
        {selected ? (
          <Card>
            <CardContent className="pt-6">
              <EquationDetail equationId={selected} />
            </CardContent>
          </Card>
        ) : (
          <div className="flex items-center justify-center h-48 text-muted-foreground text-sm">
            {t.knowledge.equations.selectToView}
          </div>
        )}
      </div>
    </div>
  )
}
