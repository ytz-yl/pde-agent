import type {
  SearchHit,
  Paper,
  Method,
  RelatedEntry,
  ComparisonReport,
  Recommendation,
  RecommendRequest,
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

const KB = '/api/knowledge'

export const knowledgeApi = {
  search(params: {
    q: string
    pde_type?: string
    method?: string
    domain?: string
    limit?: number
  }): Promise<SearchHit[]> {
    const sp = new URLSearchParams({ q: params.q })
    if (params.pde_type) sp.set('pde_type', params.pde_type)
    if (params.method) sp.set('method', params.method)
    if (params.domain) sp.set('domain', params.domain)
    if (params.limit) sp.set('limit', String(params.limit))
    return get<SearchHit[]>(`${KB}/search?${sp}`)
  },

  recentPapers(params?: { domain?: string; limit?: number }): Promise<Paper[]> {
    const sp = new URLSearchParams()
    if (params?.domain) sp.set('domain', params.domain)
    if (params?.limit) sp.set('limit', String(params.limit))
    return get<Paper[]>(`${KB}/papers/recent?${sp}`)
  },

  getPaper(id: string): Promise<Paper> {
    return get<Paper>(`${KB}/papers/${encodeURIComponent(id)}`)
  },

  listMethods(category?: string): Promise<Method[]> {
    const sp = new URLSearchParams()
    if (category) sp.set('category', category)
    return get<Method[]>(`${KB}/methods?${sp}`)
  },

  getMethod(id: string): Promise<Method> {
    return get<Method>(`${KB}/methods/${encodeURIComponent(id)}`)
  },

  relatedMethods(id: string): Promise<RelatedEntry[]> {
    return get<RelatedEntry[]>(`${KB}/methods/${encodeURIComponent(id)}/related`)
  },

  compareMethods(a: string, b: string): Promise<ComparisonReport> {
    return get<ComparisonReport>(`${KB}/methods/compare?a=${encodeURIComponent(a)}&b=${encodeURIComponent(b)}`)
  },

  recommend(req: RecommendRequest): Promise<Recommendation[]> {
    return post<Recommendation[]>(`${KB}/recommend`, req)
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
