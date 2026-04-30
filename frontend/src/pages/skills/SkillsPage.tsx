import { useState, useEffect } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import { skillsApi } from '@/lib/api'
import { useI18n } from '@/i18n/context'
import { Button } from '@/components/ui/button'
import { Spinner } from '@/components/ui/spinner'
import { Badge } from '@/components/ui/badge'
import { Download, FileText, FolderOpen, ChevronRight } from 'lucide-react'
import { cn } from '@/lib/utils'

// ── File tree sidebar ─────────────────────────────────────────────────────────

interface FileSidebarProps {
  skills: string[]
  selectedPkg: string | null
  selectedFile: string | null
  files: string[]
  onSelectPkg: (pkg: string) => void
  onSelectFile: (file: string) => void
  onDownload: (pkg: string) => void
  downloading: string | null
}

function fileIcon(name: string) {
  return name.endsWith('.md') ? (
    <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
  ) : (
    <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
  )
}

function FileSidebar({
  skills, selectedPkg, selectedFile, files,
  onSelectPkg, onSelectFile, onDownload, downloading,
}: FileSidebarProps) {
  const { t } = useI18n()

  return (
    <aside className="w-56 shrink-0 border-r pr-4 space-y-1 min-h-[420px]">
      {skills.map(pkg => (
        <div key={pkg}>
          {/* Package row */}
          <button
            onClick={() => onSelectPkg(pkg)}
            className={cn(
              'w-full flex items-center gap-1.5 px-2 py-1.5 rounded-md text-sm font-medium transition-colors text-left',
              selectedPkg === pkg
                ? 'bg-accent text-accent-foreground'
                : 'text-foreground hover:bg-accent/50',
            )}
          >
            <FolderOpen className="h-4 w-4 shrink-0 text-primary" />
            <span className="flex-1 truncate">{pkg}</span>
            <ChevronRight
              className={cn('h-3.5 w-3.5 text-muted-foreground transition-transform', selectedPkg === pkg && 'rotate-90')}
            />
          </button>

          {/* File list (expanded when pkg selected) */}
          {selectedPkg === pkg && files.length > 0 && (
            <div className="ml-4 mt-0.5 space-y-0.5">
              {files.map(f => (
                <button
                  key={f}
                  onClick={() => onSelectFile(f)}
                  className={cn(
                    'w-full flex items-center gap-1.5 px-2 py-1 rounded text-xs transition-colors text-left',
                    selectedFile === f
                      ? 'bg-primary/10 text-primary font-medium'
                      : 'text-muted-foreground hover:text-foreground hover:bg-accent/40',
                  )}
                >
                  {fileIcon(f)}
                  <span className="truncate">{f}</span>
                </button>
              ))}
              {/* Download button */}
              <Button
                size="sm"
                variant="outline"
                className="w-full mt-2 text-xs h-7"
                disabled={downloading === pkg}
                onClick={() => onDownload(pkg)}
              >
                {downloading === pkg
                  ? <><Spinner size="sm" className="mr-1" />{t.skills.downloading}</>
                  : <><Download className="h-3 w-3 mr-1" />{t.skills.downloadButton(pkg)}</>}
              </Button>
            </div>
          )}
        </div>
      ))}
    </aside>
  )
}

// ── Markdown viewer ───────────────────────────────────────────────────────────

function MarkdownViewer({ content }: { content: string }) {
  return (
    <article
      className="prose prose-sm max-w-none"
      style={{
        // wire every prose colour token to our CSS variables so nothing is white-on-white
        ['--tw-prose-body' as string]:          'hsl(var(--foreground))',
        ['--tw-prose-headings' as string]:      'hsl(var(--foreground))',
        ['--tw-prose-lead' as string]:          'hsl(var(--muted-foreground))',
        ['--tw-prose-links' as string]:         'hsl(var(--primary))',
        ['--tw-prose-bold' as string]:          'hsl(var(--foreground))',
        ['--tw-prose-counters' as string]:      'hsl(var(--muted-foreground))',
        ['--tw-prose-bullets' as string]:       'hsl(var(--muted-foreground))',
        ['--tw-prose-hr' as string]:            'hsl(var(--border))',
        ['--tw-prose-quotes' as string]:        'hsl(var(--foreground))',
        ['--tw-prose-quote-borders' as string]: 'hsl(var(--primary))',
        ['--tw-prose-captions' as string]:      'hsl(var(--muted-foreground))',
        ['--tw-prose-kbd' as string]:           'hsl(var(--foreground))',
        ['--tw-prose-kbd-shadows' as string]:   'hsl(var(--foreground) / 0.1)',
        ['--tw-prose-code' as string]:          'hsl(var(--foreground))',
        ['--tw-prose-pre-code' as string]:      'hsl(var(--foreground))',
        ['--tw-prose-pre-bg' as string]:        'hsl(var(--muted))',
        ['--tw-prose-th-borders' as string]:    'hsl(var(--border))',
        ['--tw-prose-td-borders' as string]:    'hsl(var(--border))',
      }}
    >
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          // inline code: use muted background, keep foreground text
          code: ({ children, className, ...props }) => {
            const isBlock = className?.startsWith('language-')
            if (isBlock) {
              return (
                <code
                  className={className}
                  style={{ color: 'hsl(var(--foreground))' }}
                  {...props}
                >
                  {children}
                </code>
              )
            }
            return (
              <code
                style={{
                  background: 'hsl(var(--muted))',
                  color: 'hsl(var(--foreground))',
                  padding: '0.15em 0.35em',
                  borderRadius: '0.25rem',
                  fontSize: '0.8em',
                  fontFamily: 'monospace',
                }}
                {...props}
              >
                {children}
              </code>
            )
          },
          // pre block
          pre: ({ children, ...props }) => (
            <pre
              style={{
                background: 'hsl(var(--muted))',
                border: '1px solid hsl(var(--border))',
                borderRadius: '0.375rem',
                padding: '1rem',
                overflowX: 'auto',
              }}
              {...props}
            >
              {children}
            </pre>
          ),
          // table header cells
          th: ({ children, ...props }) => (
            <th
              style={{
                background: 'hsl(var(--muted))',
                color: 'hsl(var(--foreground))',
              }}
              {...props}
            >
              {children}
            </th>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </article>
  )
}

// ── Main page ─────────────────────────────────────────────────────────────────

export function SkillsPage() {
  const { t } = useI18n()
  const [skills, setSkills] = useState<string[]>([])
  const [loadingSkills, setLoadingSkills] = useState(true)

  const [selectedPkg, setSelectedPkg] = useState<string | null>(null)
  const [files, setFiles] = useState<string[]>([])

  const [selectedFile, setSelectedFile] = useState<string | null>(null)
  const [fileContent, setFileContent] = useState<string | null>(null)
  const [loadingFile, setLoadingFile] = useState(false)

  const [downloading, setDownloading] = useState<string | null>(null)

  // Load skill list on mount
  useEffect(() => {
    skillsApi.listSkills()
      .then(r => {
        setSkills(r.skills)
        // Auto-select first skill
        if (r.skills.length > 0) handleSelectPkg(r.skills[0])
      })
      .finally(() => setLoadingSkills(false))
  }, [])

  function handleSelectPkg(pkg: string) {
    setSelectedPkg(pkg)
    setSelectedFile(null)
    setFileContent(null)
    skillsApi.listFiles(pkg).then(r => {
      setFiles(r.files)
      // Auto-open SKILL.md if present, otherwise first file
      const auto = r.files.includes('SKILL.md') ? 'SKILL.md' : r.files[0]
      if (auto) handleSelectFile(pkg, auto)
    })
  }

  function handleSelectFile(pkg: string, file: string) {
    setSelectedFile(file)
    setLoadingFile(true)
    setFileContent(null)
    skillsApi.readFile(pkg, file)
      .then(setFileContent)
      .finally(() => setLoadingFile(false))
  }

  async function handleDownload(pkg: string) {
    setDownloading(pkg)
    try {
      const url = skillsApi.downloadUrl(pkg)
      const res = await fetch(url)
      if (!res.ok) throw new Error(`HTTP ${res.status}`)
      const blob = await res.blob()
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = `${pkg}.zip`
      a.click()
      URL.revokeObjectURL(a.href)
    } finally {
      setDownloading(null)
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-semibold">{t.skills.title}</h1>
        <p className="text-muted-foreground text-sm mt-1">{t.skills.subtitle}</p>
      </div>

      {loadingSkills ? (
        <div className="flex justify-center py-16"><Spinner size="lg" /></div>
      ) : skills.length === 0 ? (
        <p className="text-muted-foreground text-sm">{t.skills.noSkills}</p>
      ) : (
        <div className="flex gap-6">
          {/* Sidebar */}
          <FileSidebar
            skills={skills}
            selectedPkg={selectedPkg}
            selectedFile={selectedFile}
            files={files}
            onSelectPkg={handleSelectPkg}
            onSelectFile={f => selectedPkg && handleSelectFile(selectedPkg, f)}
            onDownload={handleDownload}
            downloading={downloading}
          />

          {/* Content area */}
          <div className="flex-1 min-w-0">
            {/* Breadcrumb */}
            {selectedPkg && selectedFile && (
              <div className="flex items-center gap-1.5 text-xs text-muted-foreground mb-4 font-mono">
                <span>{selectedPkg}</span>
                <ChevronRight className="h-3 w-3" />
                <Badge variant="outline" className="text-xs font-mono">{selectedFile}</Badge>
              </div>
            )}

            {loadingFile ? (
              <div className="flex justify-center py-16"><Spinner /></div>
            ) : fileContent !== null ? (
              <div className="border rounded-lg p-6 bg-card">
                <MarkdownViewer content={fileContent} />
              </div>
            ) : (
              <div className="flex items-center justify-center h-64 text-muted-foreground text-sm border rounded-lg border-dashed">
                {t.skills.selectFile}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  )
}
