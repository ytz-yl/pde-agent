import { Link } from 'react-router-dom'
import { useI18n } from '@/i18n/context'
import {
  BookOpen,
  FlaskConical,
  BookMarked,
  ArrowRight,
  Database,
  Cpu,
  RefreshCw,
  CheckCircle2,
  Clock,
  Sparkles,
} from 'lucide-react'

// ── Shared primitives ─────────────────────────────────────────────────────────

function SectionTitle({ children }: { children: React.ReactNode }) {
  return (
    <h2 className="text-2xl font-semibold tracking-tight text-foreground">
      {children}
    </h2>
  )
}

function SectionSubtitle({ children }: { children: React.ReactNode }) {
  return (
    <p className="mt-2 text-muted-foreground text-sm leading-relaxed max-w-2xl">
      {children}
    </p>
  )
}

function Tag({ children }: { children: React.ReactNode }) {
  return (
    <span className="inline-block text-[10px] font-mono bg-muted text-muted-foreground rounded px-1.5 py-0.5">
      {children}
    </span>
  )
}

// ── Architecture card icons ───────────────────────────────────────────────────
const ARCH_ICONS = [Database, FlaskConical, BookMarked]

// ── Roadmap status styles ─────────────────────────────────────────────────────
const STATUS_CONFIG = {
  done:     { icon: CheckCircle2, ring: 'border-green-500',  dot: 'bg-green-500',  text: 'text-green-600 dark:text-green-400',   labelZh: '已完成', labelEn: 'Done'     },
  active:   { icon: Cpu,          ring: 'border-primary',    dot: 'bg-primary',    text: 'text-primary',                         labelZh: '进行中', labelEn: 'Active'   },
  optional: { icon: Sparkles,     ring: 'border-slate-400',  dot: 'bg-slate-400',  text: 'text-slate-500 dark:text-slate-400',   labelZh: '可选',   labelEn: 'Optional' },
  next:     { icon: Clock,        ring: 'border-amber-500',  dot: 'bg-amber-500',  text: 'text-amber-600 dark:text-amber-400',   labelZh: '计划中', labelEn: 'Planned'  },
} as const

// ── Flow diagram (Agent-driven loop) ─────────────────────────────────────────

function FlowDiagram({ isZh }: { isZh: boolean }) {
  const nodes = isZh
    ? ['PDE Agent', '场景识别 &\n数据生成', '专用模型\n训练', '知识库\n回流', '求解服务\n增强']
    : ['PDE Agent', 'Scenario ID &\nData Gen', 'Scenario Model\nTraining', 'Knowledge Base\nFeedback', 'Solver Service\nEnhancement']

  const colors = [
    'bg-primary text-primary-foreground',
    'bg-blue-500/10 border border-blue-500/30 text-blue-700 dark:text-blue-300',
    'bg-violet-500/10 border border-violet-500/30 text-violet-700 dark:text-violet-300',
    'bg-green-500/10 border border-green-500/30 text-green-700 dark:text-green-300',
    'bg-amber-500/10 border border-amber-500/30 text-amber-700 dark:text-amber-300',
  ]

  return (
    <div className="flex flex-wrap items-center justify-center gap-2 py-6">
      {nodes.map((node, i) => (
        <div key={i} className="flex items-center gap-2">
          <div className={`rounded-lg px-3 py-2 text-center text-xs font-medium leading-tight whitespace-pre-line ${colors[i]}`}>
            {node}
          </div>
          {i < nodes.length - 1 && (
            <ArrowRight className="h-4 w-4 text-muted-foreground shrink-0" />
          )}
        </div>
      ))}
      {/* Loop back arrow hint */}
      <div className="w-full flex justify-center mt-1">
        <span className="text-[10px] text-muted-foreground font-mono flex items-center gap-1">
          <RefreshCw className="h-3 w-3" />
          {isZh ? '持续闭环迭代' : 'Continuous closed-loop iteration'}
        </span>
      </div>
    </div>
  )
}

// ── Main component ────────────────────────────────────────────────────────────

export function HomePage() {
  const { t, locale } = useI18n()
  const h = t.home
  const isZh = locale === 'zh'

  return (
    <div className="space-y-20 pb-16">

      {/* ── Hero ── */}
      <section className="pt-10 pb-4 text-center space-y-5">
        <span className="inline-block text-xs font-medium tracking-widest uppercase text-primary border border-primary/30 bg-primary/5 rounded-full px-4 py-1">
          {h.hero.badge}
        </span>
        <h1 className="text-5xl font-bold tracking-tight text-foreground">
          {h.hero.title}
        </h1>
        <p className="text-base text-muted-foreground max-w-xl mx-auto leading-relaxed">
          {h.hero.subtitle}
        </p>
        <div className="flex flex-wrap justify-center gap-3 pt-2">
          <Link
            to="/knowledge"
            className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:opacity-90 transition-opacity"
          >
            <BookOpen className="h-4 w-4" />
            {h.hero.ctaKnowledge}
          </Link>
          <Link
            to="/solvers"
            className="inline-flex items-center gap-2 px-5 py-2.5 rounded-lg border border-border text-sm font-medium hover:bg-accent transition-colors"
          >
            <FlaskConical className="h-4 w-4" />
            {h.hero.ctaSolver}
          </Link>
        </div>
      </section>

      {/* ── Stats bar ── */}
      <section className="border-y border-border/60 py-6">
        <dl className="grid grid-cols-2 sm:grid-cols-4 divide-x divide-border/60">
          {h.stats.map((s, i) => (
            <div key={i} className="flex flex-col items-center px-6 py-2">
              <dt className="text-3xl font-bold text-foreground">{s.value}</dt>
              <dd className="text-xs text-muted-foreground text-center mt-1">{s.label}</dd>
            </div>
          ))}
        </dl>
      </section>

      {/* ── Architecture ── */}
      <section className="space-y-8">
        <div>
          <SectionTitle>{h.arch.title}</SectionTitle>
          <SectionSubtitle>{h.arch.subtitle}</SectionSubtitle>
        </div>

        {/* ASCII-style pipeline */}
        <div className="rounded-xl border border-border bg-muted/30 p-6 overflow-x-auto">
          <div className="flex flex-col sm:flex-row items-center gap-2 min-w-max mx-auto w-fit text-sm font-mono">
            {/* External Agent */}
            <div className="flex flex-col items-center">
              <div className="border-2 border-muted-foreground/40 rounded-lg px-5 py-3 text-center text-muted-foreground text-xs leading-tight">
                <div className="font-semibold text-foreground">{isZh ? '外部 Agent' : 'External Agent'}</div>
                <div className="text-[10px] mt-0.5">LangChain / AutoGen / …</div>
              </div>
            </div>

            <div className="flex items-center gap-1 text-muted-foreground">
              <div className="hidden sm:block">───</div>
              <ArrowRight className="h-4 w-4" />
            </div>

            {/* Service layer */}
            <div className="border-2 border-primary/40 rounded-xl px-4 py-3 space-y-2">
              <div className="text-[10px] text-center text-primary font-semibold uppercase tracking-widest mb-2">
                PDE Agent {isZh ? '服务层' : 'Service Layer'}
              </div>
              <div className="flex flex-col sm:flex-row gap-2">
                {[
                  { icon: Database,     label: isZh ? '知识库'     : 'Knowledge Base', color: 'border-blue-400/50  text-blue-600 dark:text-blue-300'    },
                  { icon: FlaskConical, label: isZh ? '求解服务'   : 'Solver Service', color: 'border-violet-400/50 text-violet-600 dark:text-violet-300' },
                  { icon: BookMarked,   label: isZh ? 'Skill 规范' : 'Skill Specs',    color: 'border-amber-400/50 text-amber-600 dark:text-amber-300'   },
                ].map(({ icon: Icon, label, color }) => (
                  <div key={label} className={`border rounded-lg px-3 py-2 text-xs text-center flex flex-col items-center gap-1 ${color}`}>
                    <Icon className="h-4 w-4" />
                    <span className="font-medium">{label}</span>
                  </div>
                ))}
              </div>
            </div>
          </div>
        </div>

        {/* Component cards — 3 columns for 3 services */}
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
          {h.arch.components.map((c, i) => {
            const Icon = ARCH_ICONS[i]
            return (
              <div key={i} className="rounded-xl border border-border bg-card p-5 space-y-3 hover:border-primary/40 transition-colors">
                <div className="flex items-center gap-2.5">
                  <div className="p-2 rounded-lg bg-primary/8 border border-primary/20">
                    <Icon className="h-4 w-4 text-primary" />
                  </div>
                  <h3 className="font-semibold text-sm text-foreground">{c.title}</h3>
                </div>
                <p className="text-xs text-muted-foreground leading-relaxed">{c.desc}</p>
                <div className="flex flex-wrap gap-1.5">
                  {c.tags.map(tag => <Tag key={tag}>{tag}</Tag>)}
                </div>
              </div>
            )
          })}
        </div>
      </section>

      {/* ── Vision ── */}
      <section className="rounded-2xl border border-primary/20 bg-primary/5 p-8 space-y-4">
        <div className="flex items-center gap-3">
          <RefreshCw className="h-6 w-6 text-primary shrink-0" />
          <SectionTitle>{h.vision.title}</SectionTitle>
        </div>
        <p className="text-sm text-muted-foreground leading-relaxed max-w-3xl">
          {h.vision.subtitle}
        </p>

        <div className="pt-2">
          <p className="text-xs text-muted-foreground font-medium uppercase tracking-widest mb-3">
            {h.flow.subtitle}
          </p>
          <FlowDiagram isZh={isZh} />
        </div>

        {/* Scenario tags */}
        <div className="pt-2 space-y-2">
          <p className="text-xs text-muted-foreground">
            {isZh ? '计划覆盖的 PDE 场景：' : 'PDE scenarios planned for coverage:'}
          </p>
          <div className="flex flex-wrap gap-1.5">
            {[
              'Burgers', 'Navier-Stokes', 'Shallow Water',
              'Heat Equation', 'Wave Equation', 'Advection-Diffusion',
              'Reaction-Diffusion', 'Poisson', 'Maxwell',
              'Schrödinger', 'Allen-Cahn', '…',
            ].map(s => <Tag key={s}>{s}</Tag>)}
          </div>
        </div>
      </section>

      {/* ── Roadmap ── */}
      <section className="space-y-8">
        <div>
          <SectionTitle>{h.roadmap.title}</SectionTitle>
          <SectionSubtitle>{h.roadmap.subtitle}</SectionSubtitle>
        </div>

        <div className="relative">
          {/* Vertical connector */}
          <div className="absolute left-[1.4rem] top-8 bottom-8 w-px bg-border hidden sm:block" />

          <div className="space-y-6">
            {h.roadmap.items.map((item, i) => {
              const cfg = STATUS_CONFIG[item.status]
              const StatusIcon = cfg.icon
              return (
                <div key={i} className="flex gap-4 items-start">
                  {/* Icon */}
                  <div className={`relative z-10 flex-shrink-0 w-11 h-11 rounded-full border-2 ${cfg.ring} bg-background flex items-center justify-center`}>
                    <StatusIcon className={`h-4 w-4 ${cfg.text}`} />
                  </div>

                  {/* Content */}
                  <div className={`flex-1 rounded-xl border p-4 space-y-1.5 ${
                    item.status === 'active'
                      ? 'border-primary/40 bg-primary/5'
                      : 'border-border bg-card'
                  }`}>
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="text-[10px] font-mono text-muted-foreground">{item.phase}</span>
                      <span className={`text-[10px] font-medium px-2 py-0.5 rounded-full border ${cfg.ring} ${cfg.text}`}>
                        {isZh ? cfg.labelZh : cfg.labelEn}
                      </span>
                      <h3 className="font-semibold text-sm text-foreground">{item.title}</h3>
                    </div>
                    <p className="text-xs text-muted-foreground leading-relaxed">{item.desc}</p>
                  </div>
                </div>
              )
            })}
          </div>
        </div>
      </section>

      {/* ── CTA footer ── */}
      <section className="text-center space-y-4 border-t border-border/60 pt-12">
        <p className="text-muted-foreground text-sm">
          {isZh
            ? '通过导航栏探索知识库、运行求解器或浏览 Skill 文档'
            : 'Explore the knowledge base, run solvers, or browse Skill docs from the navigation bar'}
        </p>
        <div className="flex flex-wrap justify-center gap-3">
          <Link to="/knowledge" className="inline-flex items-center gap-2 text-sm text-primary hover:underline">
            <BookOpen className="h-4 w-4" />
            {t.nav.knowledge}
            <ArrowRight className="h-3 w-3" />
          </Link>
          <Link to="/solvers" className="inline-flex items-center gap-2 text-sm text-primary hover:underline">
            <FlaskConical className="h-4 w-4" />
            {t.nav.solver}
            <ArrowRight className="h-3 w-3" />
          </Link>
          <Link to="/skills" className="inline-flex items-center gap-2 text-sm text-primary hover:underline">
            <BookMarked className="h-4 w-4" />
            {t.nav.skills}
            <ArrowRight className="h-3 w-3" />
          </Link>
        </div>
      </section>

    </div>
  )
}
