import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { I18nProvider } from './i18n/context'
import { Layout } from './components/Layout'
import { KnowledgePage } from './pages/knowledge/KnowledgePage'
import { SolverPage } from './pages/solver/SolverPage'
import { SkillsPage } from './pages/skills/SkillsPage'

export default function App() {
  return (
    <I18nProvider>
      <BrowserRouter>
        <Routes>
          <Route element={<Layout />}>
            <Route index element={<Navigate to="/skills" replace />} />
            <Route path="/knowledge" element={<KnowledgePage />} />
            <Route path="/solvers" element={<SolverPage />} />
            <Route path="/skills" element={<SkillsPage />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </I18nProvider>
  )
}
