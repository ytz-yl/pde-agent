import { useState } from 'react'
import { useI18n } from '@/i18n/context'
import { EquationBrowser } from './EquationBrowser'
import { ModelBrowser } from './ModelBrowser'
import { GraphSearch } from './GraphSearch'

type Tab = 'equations' | 'models' | 'search'

export function KnowledgePage() {
  const { t } = useI18n()
  const [tab, setTab] = useState<Tab>('equations')

  const TABS: { key: Tab; label: string }[] = [
    { key: 'equations', label: t.knowledge.tabs.equations },
    { key: 'models', label: t.knowledge.tabs.models },
    { key: 'search', label: t.knowledge.tabs.search },
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

      {tab === 'equations' && <EquationBrowser />}
      {tab === 'models' && <ModelBrowser />}
      {tab === 'search' && <GraphSearch />}
    </div>
  )
}
