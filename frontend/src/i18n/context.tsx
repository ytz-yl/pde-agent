import { createContext, useContext, useState, type ReactNode } from 'react'
import { en, zh, type Locale, type Translations } from './translations'

interface I18nContextValue {
  locale: Locale
  t: Translations
  toggle: () => void
}

const I18nContext = createContext<I18nContextValue | null>(null)

export function I18nProvider({ children }: { children: ReactNode }) {
  const [locale, setLocale] = useState<Locale>('en')
  const t = locale === 'zh' ? zh : en

  function toggle() {
    setLocale(l => (l === 'en' ? 'zh' : 'en'))
  }

  return (
    <I18nContext.Provider value={{ locale, t, toggle }}>
      {children}
    </I18nContext.Provider>
  )
}

export function useI18n(): I18nContextValue {
  const ctx = useContext(I18nContext)
  if (!ctx) throw new Error('useI18n must be used inside I18nProvider')
  return ctx
}
