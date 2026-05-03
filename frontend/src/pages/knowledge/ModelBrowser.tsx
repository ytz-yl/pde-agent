import { useState, useEffect } from 'react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { AIModel, AIModelProfile, NumericalMethod, EquationRef } from '@/types'
import { Card, CardContent } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { FlaskConical, Layers } from 'lucide-react'

// ── Training type badge ───────────────────────────────────────────────────────

function TrainingBadge({ type }: { type: string }) {
  const variants: Record<string, string> = {
    physics_informed: 'bg-blue-50 text-blue-700 border-blue-200',
    operator_learning: 'bg-violet-50 text-violet-700 border-violet-200',
    supervised: 'bg-green-50 text-green-700 border-green-200',
    unsupervised: 'bg-yellow-50 text-yellow-700 border-yellow-200',
    self_supervised: 'bg-orange-50 text-orange-700 border-orange-200',
  }
  const cls = variants[type] ?? 'bg-gray-50 text-gray-600 border-gray-200'
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${cls}`}>
      {type.replace(/_/g, ' ')}
    </span>
  )
}

// ── AI Model list ─────────────────────────────────────────────────────────────

function AIModelList({ models, selected, onSelect }: {
  models: AIModel[]
  selected: string | null
  onSelect: (id: string) => void
}) {
  const [filter, setFilter] = useState('')
  const visible = filter ? models.filter(m => m.training_type === filter) : models

  return (
    <div className="space-y-3">
      <Select value={filter} onChange={e => setFilter(e.target.value)} className="w-48">
        <option value="">All training types</option>
        <option value="physics_informed">Physics-informed</option>
        <option value="operator_learning">Operator learning</option>
        <option value="supervised">Supervised</option>
      </Select>
      <div className="space-y-1.5">
        {visible.map(m => (
          <button
            key={m.id}
            onClick={() => onSelect(m.id)}
            className={`w-full text-left rounded-md border px-3 py-2.5 text-sm transition-colors ${
              selected === m.id ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
            }`}
          >
            <div className="flex items-center justify-between gap-2">
              <span className="font-medium truncate">{m.name}</span>
              <TrainingBadge type={m.training_type} />
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

// ── AI Model detail ───────────────────────────────────────────────────────────

function AIModelDetail({ modelId }: { modelId: string }) {
  const [profile, setProfile] = useState<AIModelProfile | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    knowledgeApi.aiModelProfile(modelId).then(setProfile).finally(() => setLoading(false))
  }, [modelId])

  if (loading) return <div className="flex justify-center py-16"><Spinner /></div>
  if (!profile) return null

  const { model } = profile

  return (
    <div className="space-y-5">
      {/* Header */}
      <div className="space-y-2">
        <div className="flex items-start justify-between gap-3">
          <div>
            <h2 className="text-xl font-semibold">{model.name}</h2>
            <p className="text-xs text-muted-foreground font-mono">{model.architecture}</p>
          </div>
          <TrainingBadge type={model.training_type} />
        </div>
        {model.description && (
          <p className="text-sm text-muted-foreground leading-relaxed">{model.description}</p>
        )}
        {model.paper_ref && (
          <p className="text-xs text-muted-foreground">Reference: {model.paper_ref}</p>
        )}
        {model.tags.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {model.tags.map(tag => (
              <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>
            ))}
          </div>
        )}
      </div>

      {/* Solves */}
      {profile.solves.length > 0 && (
        <Section title="Solves" icon={FlaskConical}>
          <div className="flex flex-wrap gap-1.5">
            {profile.solves.map(eq => <EquationChip key={eq.id} eq={eq} />)}
          </div>
        </Section>
      )}

      {/* Training */}
      {profile.trained_by.length > 0 && (
        <Section title="Loss Functions">
          <div className="space-y-1">
            {profile.trained_by.map(l => (
              <div key={l.id} className="flex items-center justify-between text-sm">
                <span>{l.name}</span>
                <Badge variant="outline" className="text-xs">{l.loss_type}</Badge>
              </div>
            ))}
          </div>
        </Section>
      )}

      {/* Metrics */}
      {profile.evaluated_by.length > 0 && (
        <Section title="Evaluation Metrics">
          <div className="flex flex-wrap gap-1.5">
            {profile.evaluated_by.map(m => (
              <span key={m.id} className="rounded-full bg-muted px-2.5 py-1 text-xs">{m.name}</span>
            ))}
          </div>
        </Section>
      )}

      {/* Datasets */}
      {profile.tested_on.length > 0 && (
        <Section title="Tested On">
          <div className="space-y-1">
            {profile.tested_on.map(d => (
              <div key={d.id} className="flex items-center justify-between text-sm">
                <span>{d.name}</span>
                {d.dimension && <span className="text-xs text-muted-foreground">{d.dimension}</span>}
              </div>
            ))}
          </div>
        </Section>
      )}
    </div>
  )
}

// ── Numerical Method detail ───────────────────────────────────────────────────

function NumericalMethodDetail({ method }: { method: NumericalMethod }) {
  const [equations, setEquations] = useState<EquationRef[]>([])

  useEffect(() => {
    knowledgeApi.aiModelEquations(method.id).catch(() => setEquations([]))
    // numerical-methods/:id/equations not yet in routes; we silently skip
  }, [method.id])

  return (
    <div className="space-y-4">
      <div>
        <h2 className="text-xl font-semibold">{method.name}</h2>
        <p className="text-xs text-muted-foreground font-mono">{method.method_type.replace(/_/g, ' ')}</p>
      </div>
      {method.description && (
        <p className="text-sm text-muted-foreground leading-relaxed">{method.description}</p>
      )}
      {method.order != null && method.order > 0 && (
        <p className="text-sm">Convergence order: <span className="font-mono font-semibold">{method.order}</span></p>
      )}
      {method.tags.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {method.tags.map(tag => <Badge key={tag} variant="outline" className="text-xs">{tag}</Badge>)}
        </div>
      )}
      {equations.length > 0 && (
        <Section title="Solves">
          <div className="flex flex-wrap gap-1.5">
            {equations.map(eq => <EquationChip key={eq.id} eq={eq} />)}
          </div>
        </Section>
      )}
    </div>
  )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function Section({ title, icon: Icon, children }: {
  title: string
  icon?: React.ComponentType<{ className?: string }>
  children: React.ReactNode
}) {
  return (
    <div>
      <h4 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2 flex items-center gap-1.5">
        {Icon && <Icon className="h-3.5 w-3.5" />}
        {title}
      </h4>
      {children}
    </div>
  )
}

function EquationChip({ eq }: { eq: EquationRef }) {
  const typeColors: Record<string, string> = {
    parabolic: 'bg-blue-50 text-blue-700',
    elliptic: 'bg-green-50 text-green-700',
    hyperbolic: 'bg-orange-50 text-orange-700',
    mixed: 'bg-purple-50 text-purple-700',
  }
  const cls = typeColors[eq.pde_type] ?? 'bg-muted text-muted-foreground'
  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-medium ${cls}`}>
      {eq.name}
    </span>
  )
}

// ── View selector ─────────────────────────────────────────────────────────────

type ModelView = 'ai' | 'numerical'

// ── Main ──────────────────────────────────────────────────────────────────────

export function ModelBrowser() {
  const { t } = useI18n()
  const [view, setView] = useState<ModelView>('ai')
  const [aiModels, setAIModels] = useState<AIModel[]>([])
  const [numMethods, setNumMethods] = useState<NumericalMethod[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedAI, setSelectedAI] = useState<string | null>(null)
  const [selectedNM, setSelectedNM] = useState<NumericalMethod | null>(null)

  useEffect(() => {
    Promise.all([
      knowledgeApi.listAIModels(),
      knowledgeApi.listNumericalMethods(),
    ]).then(([ai, nm]) => {
      setAIModels(ai)
      setNumMethods(nm)
    }).finally(() => setLoading(false))
  }, [])

  if (loading) return <div className="flex justify-center py-16"><Spinner size="lg" /></div>

  const viewTabs: { key: ModelView; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
    { key: 'ai', label: 'AI / ML Models', icon: Layers },
    { key: 'numerical', label: 'Numerical Methods', icon: FlaskConical },
  ]

  return (
    <div className="space-y-4">
      <div className="flex gap-1 border-b">
        {viewTabs.map(vt => (
          <button
            key={vt.key}
            onClick={() => setView(vt.key)}
            className={`flex items-center gap-1.5 px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
              view === vt.key
                ? 'border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            <vt.icon className="h-4 w-4" />
            {vt.label}
          </button>
        ))}
      </div>

      {view === 'ai' && (
        <div className="grid grid-cols-5 gap-6">
          <div className="col-span-2">
            <AIModelList models={aiModels} selected={selectedAI} onSelect={setSelectedAI} />
          </div>
          <div className="col-span-3">
            {selectedAI ? (
              <Card>
                <CardContent className="pt-6">
                  <AIModelDetail modelId={selectedAI} />
                </CardContent>
              </Card>
            ) : (
              <div className="flex items-center justify-center h-48 text-muted-foreground text-sm">
                {t.knowledge.models.selectToView}
              </div>
            )}
          </div>
        </div>
      )}

      {view === 'numerical' && (
        <div className="grid grid-cols-5 gap-6">
          <div className="col-span-2">
            <div className="space-y-1.5">
              {numMethods.map(m => (
                <button
                  key={m.id}
                  onClick={() => setSelectedNM(m)}
                  className={`w-full text-left rounded-md border px-3 py-2.5 text-sm transition-colors ${
                    selectedNM?.id === m.id ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
                  }`}
                >
                  <div className="flex items-center justify-between gap-2">
                    <span className="font-medium">{m.name}</span>
                    <span className="text-xs text-muted-foreground">{m.method_type.replace(/_/g, ' ')}</span>
                  </div>
                  {m.description && (
                    <p className="text-muted-foreground text-xs mt-1 line-clamp-2">{m.description}</p>
                  )}
                </button>
              ))}
            </div>
          </div>
          <div className="col-span-3">
            {selectedNM ? (
              <Card>
                <CardContent className="pt-6">
                  <NumericalMethodDetail method={selectedNM} />
                </CardContent>
              </Card>
            ) : (
              <div className="flex items-center justify-center h-48 text-muted-foreground text-sm">
                {t.knowledge.models.selectToView}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
