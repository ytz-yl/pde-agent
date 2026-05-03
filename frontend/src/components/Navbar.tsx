import { NavLink } from 'react-router-dom'
import { cn } from '@/lib/utils'
import { BookOpen, Cpu, FlaskConical, BookMarked, Languages } from 'lucide-react'
import { useI18n } from '@/i18n/context'

export function Navbar() {
  const { t, toggle, locale } = useI18n()

  const links = [
    { to: '/skills', label: t.nav.skills, icon: BookMarked },
    { to: '/knowledge', label: t.nav.knowledge, icon: BookOpen },
    { to: '/solvers', label: t.nav.solver, icon: FlaskConical },
  ]

  return (
    <header className="border-b bg-background sticky top-0 z-10">
      <div className="max-w-7xl mx-auto px-4 h-14 flex items-center gap-6">
        {/* Logo */}
        <div className="flex items-center gap-2 font-semibold text-foreground mr-4 shrink-0">
          <Cpu className="h-5 w-5 text-primary" />
          PDE Agent
        </div>

        {/* Nav links */}
        <nav className="flex items-center gap-1 flex-1">
          {links.map(({ to, label, icon: Icon }) => (
            <NavLink
              key={to}
              to={to}
              className={({ isActive }) =>
                cn(
                  'flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm transition-colors',
                  isActive
                    ? 'bg-accent text-accent-foreground font-medium'
                    : 'text-muted-foreground hover:text-foreground hover:bg-accent/50',
                )
              }
            >
              <Icon className="h-4 w-4" />
              {label}
            </NavLink>
          ))}
        </nav>

        {/* Language toggle */}
        <button
          onClick={toggle}
          title={t.lang.toggle}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-muted-foreground hover:text-foreground hover:bg-accent/50 transition-colors shrink-0"
        >
          <Languages className="h-4 w-4" />
          <span className="font-medium">{locale === 'en' ? '中文' : 'EN'}</span>
        </button>
      </div>
    </header>
  )
}
