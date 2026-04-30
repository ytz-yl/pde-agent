import { useState, useEffect } from 'react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { Method, RelatedEntry, ComparisonReport, Recommendation } from '@/types'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { ArrowRight, GitCompare, Lightbulb } from 'lucide-react'

// ── Category badge ────────────────────────────────────────────────────────────

function CategoryBadge({ cat }: { cat: Method['category'] }) {
  const { t } = useI18n()
  const labels = t.knowledge.methods.categoryLabels
  const variants: Record<Method['category'], 'default' | 'secondary' | 'outline'> = {
    classical: 'default', ml: 'secondary', hybrid: 'outline',
  }
  return <Badge variant={variants[cat]}>{labels[cat]}</Badge>
}

// ── Method list ───────────────────────────────────────────────────────────────

function MethodList({ methods, selected, onSelect }: {
  methods: Method[]
  selected: string | null
  onSelect: (id: string) => void
}) {
  const { t } = useI18n()
  const [filter, setFilter] = useState('')
  const visible = filter ? methods.filter(m => m.category === filter) : methods

  return (
    <div className="space-y-3">
      <Select value={filter} onChange={e => setFilter(e.target.value)} className="w-40">
        <option value="">{t.knowledge.methods.allCategories}</option>
        <option value="classical">{t.knowledge.methods.categoryLabels.classical}</option>
        <option value="ml">{t.knowledge.methods.categoryLabels.ml}</option>
        <option value="hybrid">{t.knowledge.methods.categoryLabels.hybrid}</option>
      </Select>
      <div className="space-y-2">
        {visible.map(m => (
          <button
            key={m.id}
            onClick={() => onSelect(m.id)}
            className={`w-full text-left rounded-md border px-3 py-2.5 text-sm transition-colors ${
              selected === m.id ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
            }`}
          >
            <div className="flex items-center justify-between">
              <span className="font-medium">{m.name}</span>
              <CategoryBadge cat={m.category} />
            </div>
            {m.description && (
              <p className="text-muted-foreground text-xs mt-1 line-clamp-2">{m.description}</p>
            )}
          </button>
        ))}
      </div>
    </div>
  )
}

// ── Method detail ─────────────────────────────────────────────────────────────

function MethodDetail({ methodId }: { methodId: string }) {
  const { t } = useI18n()
  const [method, setMethod] = useState<Method | null>(null)
  const [related, setRelated] = useState<RelatedEntry[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    Promise.all([knowledgeApi.getMethod(methodId), knowledgeApi.relatedMethods(methodId)])
      .then(([m, r]) => { setMethod(m); setRelated(r) })
      .finally(() => setLoading(false))
  }, [methodId])

  if (loading) return <div className="flex justify-center py-8"><Spinner /></div>
  if (!method) return null

  return (
    <div className="space-y-4">
      <div className="flex items-start justify-between">
        <div>
          <h2 className="text-xl font-semibold">{method.name}</h2>
          <p className="text-sm text-muted-foreground font-mono">{method.id}</p>
        </div>
        <CategoryBadge cat={method.category} />
      </div>
      {method.description && (
        <p className="text-sm text-muted-foreground leading-relaxed">{method.description}</p>
      )}
      {method.tags.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {method.tags.map(tag => <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>)}
        </div>
      )}
      {related.length > 0 && (
        <div>
          <h3 className="text-sm font-medium mb-2">{t.knowledge.methods.relatedMethods}</h3>
          <div className="space-y-1.5">
            {related.map(r => (
              <div key={r.method.id} className="flex items-center gap-2 text-sm">
                <ArrowRight className="h-3 w-3 text-muted-foreground shrink-0" />
                <span className="font-medium">{r.method.name}</span>
                <span className="text-muted-foreground text-xs">({r.relation})</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  )
}

// ── Compare panel ─────────────────────────────────────────────────────────────

function ComparePanel({ methods }: { methods: Method[] }) {
  const { t } = useI18n()
  const ct = t.knowledge.methods.compare
  const [a, setA] = useState('')
  const [b, setB] = useState('')
  const [report, setReport] = useState<ComparisonReport | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleCompare() {
    if (!a || !b || a === b) return
    setLoading(true); setError(null)
    try { setReport(await knowledgeApi.compareMethods(a, b)) }
    catch (err) { setError(err instanceof Error ? err.message : t.common.error) }
    finally { setLoading(false) }
  }

  return (
    <div className="space-y-4">
      <div className="flex gap-2 items-end">
        <div className="flex-1 space-y-1">
          <Label>{ct.methodA}</Label>
          <Select value={a} onChange={e => setA(e.target.value)}>
            <option value="">Select…</option>
            {methods.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
          </Select>
        </div>
        <span className="text-muted-foreground pb-2">{ct.vs}</span>
        <div className="flex-1 space-y-1">
          <Label>{ct.methodB}</Label>
          <Select value={b} onChange={e => setB(e.target.value)}>
            <option value="">Select…</option>
            {methods.map(m => <option key={m.id} value={m.id}>{m.name}</option>)}
          </Select>
        </div>
        <Button onClick={handleCompare} disabled={!a || !b || a === b || loading} size="sm">
          {loading ? <Spinner size="sm" /> : <><GitCompare className="h-4 w-4 mr-1" />{ct.button}</>}
        </Button>
      </div>
      {error && <p className="text-sm text-destructive">{error}</p>}
      {report && (
        <Card>
          <CardContent className="pt-4 space-y-3">
            <div className="grid grid-cols-2 gap-4">
              {[report.method_a, report.method_b].map((m, i) => (
                <div key={i}>
                  <p className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
                    {i === 0 ? ct.methodA : ct.methodB}
                  </p>
                  <p className="font-semibold">{m.name}</p>
                  <CategoryBadge cat={m.category} />
                </div>
              ))}
            </div>
            <p className="text-sm text-muted-foreground border-t pt-3">{report.summary}</p>
            {report.relations.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {report.relations.map((r, i) => (
                  <Badge key={i} variant="outline" className="text-xs">{r.kind}</Badge>
                ))}
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}

// ── Recommend panel ───────────────────────────────────────────────────────────

const CONSTRAINT_OPTIONS = [
  'irregular_domain', 'inverse_problem', 'high_dimensional', 'parametric',
  'fast_inference', 'high_accuracy', 'complex_geometry', 'conservation_laws',
  'guaranteed_convergence',
]
const PDE_TYPES = [
  'navier_stokes', 'fluid_dynamics', 'heat_equation', 'diffusion',
  'wave_equation', 'hyperbolic', 'poisson', 'elliptic',
]

function RecommendPanel() {
  const { t } = useI18n()
  const rt = t.knowledge.methods.recommend
  const [pdeType, setPdeType] = useState('')
  const [domain, setDomain] = useState('')
  const [constraints, setConstraints] = useState<string[]>([])
  const [recs, setRecs] = useState<Recommendation[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  function toggleConstraint(c: string) {
    setConstraints(prev => prev.includes(c) ? prev.filter(x => x !== c) : [...prev, c])
  }

  async function handleRecommend() {
    if (!pdeType) return
    setLoading(true); setError(null)
    try {
      setRecs(await knowledgeApi.recommend({ pde_type: pdeType, domain: domain || undefined, constraints, top_k: 3 }))
    } catch (err) {
      setError(err instanceof Error ? err.message : t.common.error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-3">
        <div className="space-y-1">
          <Label>{rt.pdeType}</Label>
          <Select value={pdeType} onChange={e => setPdeType(e.target.value)}>
            <option value="">{rt.selectPde}</option>
            {PDE_TYPES.map(tp => <option key={tp} value={tp}>{tp.replace(/_/g, ' ')}</option>)}
          </Select>
        </div>
        <div className="space-y-1">
          <Label>{rt.domain}</Label>
          <Input placeholder={rt.domainPlaceholder} value={domain} onChange={e => setDomain(e.target.value)} />
        </div>
      </div>
      <div>
        <Label className="block mb-2">{rt.constraints}</Label>
        <div className="flex flex-wrap gap-1.5">
          {CONSTRAINT_OPTIONS.map(c => (
            <button
              key={c}
              onClick={() => toggleConstraint(c)}
              className={`rounded-full px-2.5 py-1 text-xs font-medium border transition-colors ${
                constraints.includes(c)
                  ? 'bg-primary text-primary-foreground border-primary'
                  : 'border-border text-muted-foreground hover:border-primary/50'
              }`}
            >
              {c.replace(/_/g, ' ')}
            </button>
          ))}
        </div>
      </div>
      <Button onClick={handleRecommend} disabled={!pdeType || loading}>
        {loading ? <Spinner size="sm" className="mr-2" /> : <Lightbulb className="h-4 w-4 mr-2" />}
        {rt.button}
      </Button>
      {error && <p className="text-sm text-destructive">{error}</p>}
      {recs && (
        <div className="space-y-3">
          {recs.map((rec, i) => (
            <Card key={rec.method.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">
                    <span className="text-muted-foreground mr-2">#{i + 1}</span>
                    {rec.method.name}
                  </CardTitle>
                  <div className="flex items-center gap-2">
                    <CategoryBadge cat={rec.method.category} />
                    <span className="text-xs font-mono text-muted-foreground">
                      {rt.score} {rec.score.toFixed(2)}
                    </span>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">{rec.reason}</p>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}

// ── Main ──────────────────────────────────────────────────────────────────────

type Tab = 'browse' | 'compare' | 'recommend'

export function MethodBrowser() {
  const { t } = useI18n()
  const [methods, setMethods] = useState<Method[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<string | null>(null)
  const [tab, setTab] = useState<Tab>('browse')

  useEffect(() => {
    knowledgeApi.listMethods().then(setMethods).finally(() => setLoading(false))
  }, [])

  const tabs: { key: Tab; label: string }[] = [
    { key: 'browse', label: t.knowledge.methods.tabs.browse },
    { key: 'compare', label: t.knowledge.methods.tabs.compare },
    { key: 'recommend', label: t.knowledge.methods.tabs.recommend },
  ]

  if (loading) return <div className="flex justify-center py-16"><Spinner size="lg" /></div>

  return (
    <div className="space-y-4">
      <div className="flex gap-1 border-b">
        {tabs.map(tb => (
          <button
            key={tb.key}
            onClick={() => setTab(tb.key)}
            className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
              tab === tb.key ? 'border-primary text-foreground' : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {tb.label}
          </button>
        ))}
      </div>

      {tab === 'browse' && (
        <div className="grid grid-cols-5 gap-6">
          <div className="col-span-2">
            <MethodList methods={methods} selected={selected} onSelect={setSelected} />
          </div>
          <div className="col-span-3">
            {selected ? (
              <Card><CardContent className="pt-6"><MethodDetail methodId={selected} /></CardContent></Card>
            ) : (
              <div className="flex items-center justify-center h-48 text-muted-foreground text-sm">
                {t.knowledge.methods.selectToView}
              </div>
            )}
          </div>
        </div>
      )}
      {tab === 'compare' && <ComparePanel methods={methods} />}
      {tab === 'recommend' && <RecommendPanel />}
    </div>
  )
}
