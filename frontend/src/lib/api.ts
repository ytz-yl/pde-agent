import type {
  Equation,
  EquationSolvers,
  EquationConditions,
  AIModel,
  AIModelProfile,
  NumericalMethod,
  Paper,
  PaperProfile,
  PaperRef,
  SearchHit,
  DatasetRef,
  SolverInfo,
  SolveRequest,
  SolveResponse,
  ApiResponse,
} from '@/types'

// ── Helpers ───────────────────────────────────────────────────────────────────

async function get<T>(url: string): Promise<T> {
  const res = await fetch(url)
  if (!res.ok) {
    const body = await res.json().catch(() => ({}))
    throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`)
  }
  return res.json() as Promise<T>
}

async function post<T>(url: string, body: unknown): Promise<T> {
  const res = await fetch(url, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const b = await res.json().catch(() => ({}))
    throw new Error((b as { error?: string }).error ?? `HTTP ${res.status}`)
  }
  return res.json() as Promise<T>
}

// ── Knowledge Base API ────────────────────────────────────────────────────────
// Maps to the Neo4j-backed knowledge_base Rust service (default: localhost:3001)

const KB = '/api/knowledge'

export const knowledgeApi = {
  // ── Health ──────────────────────────────────────────────────────────────────
  health(): Promise<{ status: string; service: string }> {
    return get(`${KB}/health`)
  },

  // ── Equations ───────────────────────────────────────────────────────────────
  listEquations(pdeType?: string): Promise<Equation[]> {
    const sp = new URLSearchParams()
    if (pdeType) sp.set('pde_type', pdeType)
    return get<Equation[]>(`${KB}/equations?${sp}`)
  },

  getEquation(id: string): Promise<Equation> {
    return get<Equation>(`${KB}/equations/${encodeURIComponent(id)}`)
  },

  /** Which AI models and numerical methods can solve this equation. */
  equationSolvers(id: string): Promise<EquationSolvers> {
    return get<EquationSolvers>(`${KB}/equations/${encodeURIComponent(id)}/solvers`)
  },

  /** Boundary / initial conditions associated with this equation. */
  equationConditions(id: string): Promise<EquationConditions> {
    return get<EquationConditions>(`${KB}/equations/${encodeURIComponent(id)}/conditions`)
  },

  /** Benchmark datasets based on this equation. */
  equationDatasets(id: string): Promise<DatasetRef[]> {
    return get<DatasetRef[]>(`${KB}/equations/${encodeURIComponent(id)}/datasets`)
  },

  /** Papers that study this equation. */
  equationPapers(id: string): Promise<PaperRef[]> {
    return get<PaperRef[]>(`${KB}/equations/${encodeURIComponent(id)}/papers`)
  },

  // ── AI Models ────────────────────────────────────────────────────────────────
  listAIModels(trainingType?: string): Promise<AIModel[]> {
    const sp = new URLSearchParams()
    if (trainingType) sp.set('training_type', trainingType)
    return get<AIModel[]>(`${KB}/ai-models?${sp}`)
  },

  getAIModel(id: string): Promise<AIModel> {
    return get<AIModel>(`${KB}/ai-models/${encodeURIComponent(id)}`)
  },

  /** Full profile: what it solves, loss functions, metrics, datasets. */
  aiModelProfile(id: string): Promise<AIModelProfile> {
    return get<AIModelProfile>(`${KB}/ai-models/${encodeURIComponent(id)}/profile`)
  },

  /** Equations this model can solve. */
  aiModelEquations(id: string): Promise<import('@/types').EquationRef[]> {
    return get(`${KB}/ai-models/${encodeURIComponent(id)}/equations`)
  },

  // ── Numerical Methods ────────────────────────────────────────────────────────
  listNumericalMethods(): Promise<NumericalMethod[]> {
    return get<NumericalMethod[]>(`${KB}/numerical-methods`)
  },

  getNumericalMethod(id: string): Promise<NumericalMethod> {
    return get<NumericalMethod>(`${KB}/numerical-methods/${encodeURIComponent(id)}`)
  },

  // ── Papers ───────────────────────────────────────────────────────────────────
  listPapers(year?: number): Promise<Paper[]> {
    const sp = new URLSearchParams()
    if (year) sp.set('year', String(year))
    return get<Paper[]>(`${KB}/papers?${sp}`)
  },

  /** Returns paper structural fields merged with abstract/notes from SQLite. */
  getPaper(id: string): Promise<{ paper: Paper; abstract?: string; notes?: string }> {
    return get(`${KB}/papers/${encodeURIComponent(id)}`)
  },

  /** Full paper profile: proposes, studies, datasets, citations. */
  paperProfile(id: string): Promise<PaperProfile> {
    return get<PaperProfile>(`${KB}/papers/${encodeURIComponent(id)}/profile`)
  },

  // ── Search ───────────────────────────────────────────────────────────────────
  /** Name-based search across all node types. */
  search(q: string): Promise<SearchHit[]> {
    return get<SearchHit[]>(`${KB}/search?q=${encodeURIComponent(q)}`)
  },
}

// ── Solver API ────────────────────────────────────────────────────────────────

const SV = '/api/solvers'

export const solverApi = {
  listSolvers(): Promise<ApiResponse<SolverInfo[]>> {
    return get<ApiResponse<SolverInfo[]>>(`${SV}/solvers`)
  },

  solve(req: SolveRequest): Promise<ApiResponse<SolveResponse>> {
    return post<ApiResponse<SolveResponse>>(`${SV}/solve`, req)
  },

  health(): Promise<{ status: string; service: string }> {
    return get(`${SV}/health`)
  },
}

// ── Skills API ────────────────────────────────────────────────────────────────

export const skillsApi = {
  listSkills(): Promise<{ skills: string[] }> {
    return get('/api/skills/list')
  },

  listFiles(pkg: string): Promise<{ files: string[] }> {
    return get(`/api/skills/${encodeURIComponent(pkg)}/files`)
  },

  readFile(pkg: string, filename: string): Promise<string> {
    return fetch(`/api/skills/${encodeURIComponent(pkg)}/file?f=${encodeURIComponent(filename)}`)
      .then(r => {
        if (!r.ok) throw new Error(`HTTP ${r.status}`)
        return r.text()
      })
  },

  downloadUrl(pkg: string): string {
    return `/api/skills/${encodeURIComponent(pkg)}/download`
  },
}
