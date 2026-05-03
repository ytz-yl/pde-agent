import { useState } from 'react'
import { Search } from 'lucide-react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { SearchHit } from '@/types'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Spinner } from '@/components/ui/spinner'

// Label → colour map for node types
const LABEL_COLOURS: Record<string, string> = {
  Equation: 'bg-blue-50 text-blue-700 border-blue-200',
  Condition: 'bg-teal-50 text-teal-700 border-teal-200',
  Theorem: 'bg-amber-50 text-amber-700 border-amber-200',
  NumericalMethod: 'bg-green-50 text-green-700 border-green-200',
  AIModel: 'bg-violet-50 text-violet-700 border-violet-200',
  LossFunction: 'bg-rose-50 text-rose-700 border-rose-200',
  Metric: 'bg-cyan-50 text-cyan-700 border-cyan-200',
  Dataset: 'bg-orange-50 text-orange-700 border-orange-200',
  Paper: 'bg-gray-50 text-gray-600 border-gray-200',
}

function NodeTypeBadge({ label }: { label: string }) {
  const cls = LABEL_COLOURS[label] ?? 'bg-muted text-muted-foreground border-border'
  return (
    <span className={`inline-flex items-center rounded-full border px-2 py-0.5 text-xs font-medium ${cls}`}>
      {label}
    </span>
  )
}

export function GraphSearch() {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const [results, setResults] = useState<SearchHit[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault()
    const q = query.trim()
    if (!q) return
    setLoading(true)
    setError(null)
    try {
      setResults(await knowledgeApi.search(q))
    } catch (err) {
      setError(err instanceof Error ? err.message : t.common.error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <form onSubmit={handleSearch} className="flex gap-2">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            className="pl-9"
            placeholder={t.knowledge.search.placeholder}
            value={query}
            onChange={e => setQuery(e.target.value)}
          />
        </div>
        <Button type="submit" disabled={loading || !query.trim()}>
          {loading ? <Spinner size="sm" /> : t.knowledge.search.button}
        </Button>
      </form>

      {error && (
        <div className="rounded-md bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 text-sm">
          {error}
        </div>
      )}

      {results !== null && (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">
            {t.knowledge.search.results(results.length)}
          </p>
          {results.length === 0 ? (
            <p className="text-sm text-muted-foreground">{t.knowledge.search.noResults}</p>
          ) : (
            <div className="space-y-2">
              {results.map(hit => (
                <div
                  key={`${hit.label}:${hit.id}`}
                  className="flex items-start justify-between gap-4 rounded-md border px-4 py-3"
                >
                  <div className="min-w-0 space-y-0.5">
                    <div className="flex items-center gap-2">
                      <span className="font-medium text-sm">{hit.name}</span>
                      <span className="font-mono text-xs text-muted-foreground">{hit.id}</span>
                    </div>
                    {hit.description && (
                      <p className="text-xs text-muted-foreground line-clamp-2 leading-relaxed">
                        {hit.description}
                      </p>
                    )}
                  </div>
                  <NodeTypeBadge label={hit.label} />
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  )
}
