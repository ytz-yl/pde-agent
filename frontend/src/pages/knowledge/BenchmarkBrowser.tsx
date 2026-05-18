import { useEffect, useState } from 'react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { Benchmark, BenchmarkLeaderboard, ResultConfidence } from '@/types'
import { Card, CardContent } from '@/components/ui/card'
import { Spinner } from '@/components/ui/spinner'
import { Award, CheckCircle2, AlertTriangle, HelpCircle } from 'lucide-react'

// ── Confidence badge ──────────────────────────────────────────────────────────

function ConfidenceBadge({ kind }: { kind: ResultConfidence }) {
  const { t } = useI18n()
  const styles: Record<ResultConfidence, { cls: string; icon: typeof CheckCircle2 }> = {
    verified: { cls: 'bg-green-50 text-green-700 border-green-200',  icon: CheckCircle2 },
    single:   { cls: 'bg-gray-50  text-gray-600  border-gray-200',   icon: HelpCircle    },
    disputed: { cls: 'bg-red-50   text-red-700   border-red-200',    icon: AlertTriangle },
  }
  const { cls, icon: Icon } = styles[kind]
  return (
    <span className={`inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs font-medium ${cls}`}>
      <Icon className="h-3 w-3" />
      {t.knowledge.benchmarks.confidence[kind]}
    </span>
  )
}

// ── Benchmark list (left column) ──────────────────────────────────────────────

function BenchmarkList({ benchmarks, selected, onSelect }: {
  benchmarks: Benchmark[]
  selected: string | null
  onSelect: (id: string) => void
}) {
  return (
    <div className="space-y-1.5">
      {benchmarks.map(b => (
        <button
          key={b.id}
          onClick={() => onSelect(b.id)}
          className={`w-full text-left rounded-md border px-3 py-2.5 text-sm transition-colors ${
            selected === b.id ? 'border-primary bg-primary/5' : 'border-border hover:bg-accent/50'
          }`}
        >
          <div className="flex items-center justify-between gap-2">
            <span className="font-medium truncate">{b.name}</span>
            <Award className="h-4 w-4 text-muted-foreground shrink-0" />
          </div>
          {b.protocol && (
            <p className="text-muted-foreground text-xs mt-1 line-clamp-2">{b.protocol}</p>
          )}
          <div className="flex flex-wrap gap-1 mt-1.5 text-xs text-muted-foreground">
            <span className="font-mono">{b.dataset_id}</span>
            <span>·</span>
            <span className="font-mono">{b.metric_id}</span>
          </div>
        </button>
      ))}
    </div>
  )
}

// ── Source breakdown chips ────────────────────────────────────────────────────

function SourceBreakdown({ breakdown }: { breakdown: Record<string, number> }) {
  const { t } = useI18n()
  const labelOf = (key: string): string => {
    const m = t.knowledge.benchmarks.sourceTypes
    if (key === 'paper_reported' || key === 'self_run' || key === 'third_party_reproduction') {
      return m[key]
    }
    return key
  }
  return (
    <div className="flex flex-wrap gap-1">
      {Object.entries(breakdown).map(([src, n]) => (
        <span key={src} className="rounded-full bg-muted px-2 py-0.5 text-xs">
          {labelOf(src)} <span className="font-mono">{n}</span>
        </span>
      ))}
    </div>
  )
}

// ── Leaderboard table (right column) ──────────────────────────────────────────

function LeaderboardView({ benchmarkId }: { benchmarkId: string }) {
  const { t } = useI18n()
  const [data, setData] = useState<BenchmarkLeaderboard | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    knowledgeApi.benchmarkLeaderboard(benchmarkId)
      .then(setData)
      .catch(e => setError(e instanceof Error ? e.message : String(e)))
      .finally(() => setLoading(false))
  }, [benchmarkId])

  if (loading) return <div className="flex justify-center py-16"><Spinner /></div>
  if (error)   return <p className="text-sm text-red-600">{error}</p>
  if (!data)   return null

  const { benchmark, dataset_name, metric_name, entries } = data
  const cols = t.knowledge.benchmarks.columns

  return (
    <div className="space-y-4">
      {/* Benchmark header */}
      <div className="space-y-1.5">
        <h2 className="text-xl font-semibold">{benchmark.name}</h2>
        <div className="flex flex-wrap gap-3 text-xs text-muted-foreground">
          {dataset_name && <span>Dataset: <span className="font-mono">{dataset_name}</span></span>}
          {metric_name && <span>Metric: <span className="font-mono">{metric_name}</span></span>}
          <span>{benchmark.lower_is_better ? t.knowledge.benchmarks.lowerIsBetter : t.knowledge.benchmarks.higherIsBetter}</span>
          {benchmark.tolerance != null && (
            <span>{t.knowledge.benchmarks.tolerance}: ±{(benchmark.tolerance * 100).toFixed(1)}%</span>
          )}
        </div>
        {benchmark.protocol && (
          <p className="text-sm text-muted-foreground leading-relaxed">
            <span className="font-medium">{t.knowledge.benchmarks.protocol}:</span> {benchmark.protocol}
          </p>
        )}
      </div>

      {/* Leaderboard table */}
      {entries.length === 0 ? (
        <p className="text-sm text-muted-foreground py-8 text-center">
          {t.knowledge.benchmarks.noEntries}
        </p>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead className="text-xs text-muted-foreground border-b">
              <tr>
                <th className="text-left font-medium py-2 pr-3 w-8">{cols.rank}</th>
                <th className="text-left font-medium py-2 pr-3">{cols.method}</th>
                <th className="text-right font-medium py-2 pr-3">{cols.bestValue}</th>
                <th className="text-right font-medium py-2 pr-3">{cols.sources}</th>
                <th className="text-left font-medium py-2 pr-3">{cols.confidence}</th>
                <th className="text-left font-medium py-2 pr-3">{t.knowledge.benchmarks.breakdownLabel}</th>
                <th className="text-right font-medium py-2">{cols.latest}</th>
              </tr>
            </thead>
            <tbody>
              {entries.map((e, i) => (
                <tr key={`${e.method_id}__${e.method_label}`} className="border-b last:border-0 hover:bg-accent/30">
                  <td className="py-2 pr-3 font-mono text-xs text-muted-foreground">{i + 1}</td>
                  <td className="py-2 pr-3">
                    <div className="font-medium">{e.method_name ?? e.method_id}</div>
                    <div className="text-xs text-muted-foreground font-mono">{e.method_label}</div>
                  </td>
                  <td className="py-2 pr-3 text-right font-mono">{formatValue(e.best_value)}</td>
                  <td className="py-2 pr-3 text-right font-mono">
                    {e.n_independent_sources}
                    {e.n_results !== e.n_independent_sources && (
                      <span className="text-xs text-muted-foreground"> /{e.n_results}</span>
                    )}
                  </td>
                  <td className="py-2 pr-3"><ConfidenceBadge kind={e.confidence} /></td>
                  <td className="py-2 pr-3"><SourceBreakdown breakdown={e.source_breakdown} /></td>
                  <td className="py-2 text-right text-xs text-muted-foreground font-mono">
                    {e.latest_recorded_at ? formatDate(e.latest_recorded_at) : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}

// ── Helpers ───────────────────────────────────────────────────────────────────

function formatValue(v: number): string {
  if (Math.abs(v) >= 0.01 && Math.abs(v) < 1000) return v.toPrecision(4)
  return v.toExponential(3)
}

function formatDate(iso: string): string {
  // Render YYYY-MM-DD HH:MM (local interpretation), keep tight.
  const d = new Date(iso)
  if (Number.isNaN(d.getTime())) return iso
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

// ── Main ──────────────────────────────────────────────────────────────────────

export function BenchmarkBrowser() {
  const { t } = useI18n()
  const [benchmarks, setBenchmarks] = useState<Benchmark[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<string | null>(null)

  useEffect(() => {
    knowledgeApi.listBenchmarks()
      .then(bs => {
        setBenchmarks(bs)
        if (bs.length > 0 && !selected) setSelected(bs[0].id)
      })
      .finally(() => setLoading(false))
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  if (loading) return <div className="flex justify-center py-16"><Spinner size="lg" /></div>

  if (benchmarks.length === 0) {
    return <p className="text-sm text-muted-foreground py-12 text-center">{t.knowledge.benchmarks.noBenchmarks}</p>
  }

  return (
    <div className="grid grid-cols-5 gap-6">
      <div className="col-span-2">
        <BenchmarkList benchmarks={benchmarks} selected={selected} onSelect={setSelected} />
      </div>
      <div className="col-span-3">
        {selected ? (
          <Card>
            <CardContent className="pt-6">
              <LeaderboardView benchmarkId={selected} />
            </CardContent>
          </Card>
        ) : (
          <div className="flex items-center justify-center h-48 text-muted-foreground text-sm">
            {t.knowledge.benchmarks.selectToView}
          </div>
        )}
      </div>
    </div>
  )
}
