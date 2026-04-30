import { useState } from 'react'
import type { SearchHit } from '@/types'
import { useI18n } from '@/i18n/context'
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card'
import { Badge } from '@/components/ui/badge'
import { ExternalLink, ChevronDown, ChevronUp } from 'lucide-react'

interface PaperCardProps {
  hit: SearchHit
}

export function PaperCard({ hit }: PaperCardProps) {
  const { paper, score } = hit
  const { t } = useI18n()
  const [expanded, setExpanded] = useState(false)

  const tags = paper.tags ?? []
  const allTagValues = tags.flatMap(tag =>
    Object.values(tag).filter((v): v is string => Boolean(v)),
  )

  const published = paper.published
    ? new Date(paper.published).toLocaleDateString('en-US', { year: 'numeric', month: 'short' })
    : null

  return (
    <Card>
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-3">
          <div className="flex-1 min-w-0">
            <CardTitle className="text-base leading-snug">{paper.title}</CardTitle>
            {paper.authors.length > 0 && (
              <CardDescription className="mt-1 text-xs">
                {paper.authors.slice(0, 3).join(', ')}
                {paper.authors.length > 3 ? ` ${t.knowledge.paper.etAl}` : ''}
                {published && <span className="ml-2 text-muted-foreground/60">{published}</span>}
              </CardDescription>
            )}
          </div>
          {score > 0 && (
            <span className="text-xs font-mono text-muted-foreground shrink-0 mt-0.5">
              {(score * 100).toFixed(0)}%
            </span>
          )}
        </div>
      </CardHeader>
      <CardContent className="space-y-3">
        {allTagValues.length > 0 && (
          <div className="flex flex-wrap gap-1">
            {allTagValues.map(v => (
              <Badge key={v} variant="secondary" className="text-xs">{v.replace(/_/g, ' ')}</Badge>
            ))}
          </div>
        )}
        {paper.abstract_text && (
          <div>
            <p className={`text-sm text-muted-foreground leading-relaxed ${expanded ? '' : 'line-clamp-3'}`}>
              {paper.abstract_text}
            </p>
            {paper.abstract_text.length > 200 && (
              <button
                onClick={() => setExpanded(e => !e)}
                className="text-xs text-primary mt-1 flex items-center gap-0.5 hover:underline"
              >
                {expanded
                  ? <><ChevronUp className="h-3 w-3" />{t.knowledge.paper.showLess}</>
                  : <><ChevronDown className="h-3 w-3" />{t.knowledge.paper.showMore}</>}
              </button>
            )}
          </div>
        )}
        {paper.source_url && (
          <a
            href={paper.source_url}
            target="_blank"
            rel="noopener noreferrer"
            className="inline-flex items-center gap-1 text-xs text-primary hover:underline"
          >
            <ExternalLink className="h-3 w-3" />
            {t.knowledge.paper.viewPaper}
          </a>
        )}
      </CardContent>
    </Card>
  )
}

export function SimplePaperCard({ paper }: { paper: import('@/types').Paper }) {
  return <PaperCard hit={{ score: 0, paper }} />
}
