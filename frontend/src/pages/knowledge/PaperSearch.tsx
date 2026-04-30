import { useState } from 'react'
import { Search } from 'lucide-react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { SearchHit } from '@/types'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { Select } from '@/components/ui/select'
import { Spinner } from '@/components/ui/spinner'
import { PaperCard } from './PaperCard'

const PDE_TYPES = ['navier_stokes', 'heat_equation', 'wave_equation', 'poisson', 'diffusion', 'hyperbolic', 'elliptic']
const METHODS = ['fdm', 'fem', 'fvm', 'spectral', 'pinns', 'deeponet', 'fno', 'pdeformer']
const DOMAINS = ['fluid_dynamics', 'elasticity', 'electromagnetics', 'heat_transfer', 'wave_propagation']

export function PaperSearch() {
  const { t } = useI18n()
  const [query, setQuery] = useState('')
  const [pdeType, setPdeType] = useState('')
  const [method, setMethod] = useState('')
  const [domain, setDomain] = useState('')
  const [results, setResults] = useState<SearchHit[] | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  async function handleSearch(e: React.FormEvent) {
    e.preventDefault()
    if (!query.trim()) return
    setLoading(true)
    setError(null)
    try {
      const hits = await knowledgeApi.search({
        q: query,
        pde_type: pdeType || undefined,
        method: method || undefined,
        domain: domain || undefined,
        limit: 20,
      })
      setResults(hits)
    } catch (err) {
      setError(err instanceof Error ? err.message : t.common.error)
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="space-y-6">
      <form onSubmit={handleSearch} className="space-y-3">
        <div className="flex gap-2">
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
        </div>
        <div className="flex gap-2 flex-wrap">
          <Select value={pdeType} onChange={e => setPdeType(e.target.value)} className="w-44">
            <option value="">{t.knowledge.search.allPdeTypes}</option>
            {PDE_TYPES.map(tp => (
              <option key={tp} value={tp}>{tp.replace(/_/g, ' ')}</option>
            ))}
          </Select>
          <Select value={method} onChange={e => setMethod(e.target.value)} className="w-36">
            <option value="">{t.knowledge.search.allMethods}</option>
            {METHODS.map(m => (
              <option key={m} value={m}>{m.toUpperCase()}</option>
            ))}
          </Select>
          <Select value={domain} onChange={e => setDomain(e.target.value)} className="w-44">
            <option value="">{t.knowledge.search.allDomains}</option>
            {DOMAINS.map(d => (
              <option key={d} value={d}>{d.replace(/_/g, ' ')}</option>
            ))}
          </Select>
        </div>
      </form>

      {error && (
        <div className="rounded-md bg-destructive/10 border border-destructive/20 text-destructive px-4 py-3 text-sm">
          {error}
        </div>
      )}

      {results !== null && (
        <div className="space-y-3">
          <p className="text-sm text-muted-foreground">{t.knowledge.search.results(results.length)}</p>
          {results.length === 0 ? (
            <p className="text-muted-foreground text-sm">{t.knowledge.search.noResults}</p>
          ) : (
            results.map(hit => <PaperCard key={hit.paper.id} hit={hit} />)
          )}
        </div>
      )}
    </div>
  )
}
