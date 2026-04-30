import { useState } from 'react'
import { useI18n } from '@/i18n/context'
import { PaperSearch } from './PaperSearch'
import { RecentPapers } from './RecentPapers'
import { MethodBrowser } from './MethodBrowser'

type Tab = 'search' | 'recent' | 'methods'

export function KnowledgePage() {
  const { t } = useI18n()
  const [tab, setTab] = useState<Tab>('search')

  const TABS: { key: Tab; label: string }[] = [
    { key: 'search', label: t.knowledge.tabs.search },
    { key: 'recent', label: t.knowledge.tabs.recent },
    { key: 'methods', label: t.knowledge.tabs.methods },
  ]

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-semibold">{t.knowledge.title}</h1>
        <p className="text-muted-foreground text-sm mt-1">{t.knowledge.subtitle}</p>
      </div>

      <div className="flex gap-1 border-b">
        {TABS.map(tb => (
          <button
            key={tb.key}
            onClick={() => setTab(tb.key)}
            className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px transition-colors ${
              tab === tb.key
                ? 'border-primary text-foreground'
                : 'border-transparent text-muted-foreground hover:text-foreground'
            }`}
          >
            {tb.label}
          </button>
        ))}
      </div>

      {tab === 'search' && <PaperSearch />}
      {tab === 'recent' && <RecentPapers />}
      {tab === 'methods' && <MethodBrowser />}
    </div>
  )
}
