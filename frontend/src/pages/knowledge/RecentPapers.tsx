import { useState, useEffect } from 'react'
import { knowledgeApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import type { Paper } from '@/types'
import { Spinner } from '@/components/ui/spinner'
import { Select } from '@/components/ui/select'
import { SimplePaperCard } from './PaperCard'

const DOMAINS = ['fluid_dynamics', 'elasticity', 'electromagnetics', 'heat_transfer', 'wave_propagation']

export function RecentPapers() {
  const { t } = useI18n()
  const [papers, setPapers] = useState<Paper[]>([])
  const [domain, setDomain] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    setLoading(true)
    setError(null)
    knowledgeApi.recentPapers({ domain: domain || undefined, limit: 20 })
      .then(setPapers)
      .catch(err => setError(err instanceof Error ? err.message : t.common.error))
      .finally(() => setLoading(false))
  }, [domain])

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <span className="text-sm text-muted-foreground">{t.knowledge.recent.filterLabel}</span>
        <Select value={domain} onChange={e => setDomain(e.target.value)} className="w-44">
          <option value="">{t.knowledge.recent.allDomains}</option>
          {DOMAINS.map(d => (
            <option key={d} value={d}>{d.replace(/_/g, ' ')}</option>
          ))}
        </Select>
      </div>
      {loading && <div className="flex justify-center py-8"><Spinner /></div>}
      {error && <p className="text-sm text-destructive">{error}</p>}
      {!loading && !error && papers.length === 0 && (
        <p className="text-muted-foreground text-sm">{t.knowledge.recent.noResults}</p>
      )}
      <div className="space-y-3">
        {papers.map(p => <SimplePaperCard key={p.id} paper={p} />)}
      </div>
    </div>
  )
}
