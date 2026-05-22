/**
 * Unit tests for frontend/src/lib/api.ts
 *
 * Strategy: intercept global `fetch` with vi.stubGlobal so we can test the
 * API client behaviour without a running server.
 *
 * Covered:
 *   - knowledgeApi: URL construction, query-string params, error propagation
 *   - solverApi: POST body serialisation, error extraction
 *   - skillsApi: downloadUrl helper (pure function — no fetch needed)
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { knowledgeApi, solverApi, skillsApi } from '@/lib/api'
import type { SolveRequest } from '@/types'

// ── Fetch mock helpers ────────────────────────────────────────────────────────

function mockFetchOk(data: unknown) {
  return vi.fn().mockResolvedValue({
    ok: true,
    json: () => Promise.resolve(data),
    text: () => Promise.resolve(String(data)),
  })
}

function mockFetchError(status: number, errorMsg: string) {
  return vi.fn().mockResolvedValue({
    ok: false,
    status,
    json: () => Promise.resolve({ error: errorMsg }),
  })
}

function mockFetchBadJson(status: number) {
  return vi.fn().mockResolvedValue({
    ok: false,
    status,
    json: () => Promise.reject(new Error('bad json')),
  })
}

// ── knowledgeApi tests ────────────────────────────────────────────────────────

describe('knowledgeApi', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('listEquations calls the correct URL', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.listEquations()
    expect(fetchMock).toHaveBeenCalledWith('/api/knowledge/equations?')
  })

  it('listEquations appends pde_type query param', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.listEquations('parabolic')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('pde_type=parabolic')
  })

  it('getEquation encodes the id in the URL', async () => {
    const fetchMock = mockFetchOk({ id: 'navier stokes' })
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.getEquation('navier stokes')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('navier%20stokes')
  })

  it('throws an Error with the server error message on non-ok response', async () => {
    const fetchMock = mockFetchError(404, 'equation not found')
    vi.stubGlobal('fetch', fetchMock)
    await expect(knowledgeApi.getEquation('missing')).rejects.toThrow(
      'equation not found'
    )
  })

  it('falls back to HTTP status code message when body has no error field', async () => {
    const fetchMock = mockFetchBadJson(500)
    vi.stubGlobal('fetch', fetchMock)
    await expect(knowledgeApi.listEquations()).rejects.toThrow('HTTP 500')
  })

  it('listAIModels passes training_type param', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.listAIModels('physics_informed')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('training_type=physics_informed')
  })

  it('search encodes special characters in query', async () => {
    const fetchMock = mockFetchOk([])
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.search('Navier & Stokes')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain(encodeURIComponent('Navier & Stokes'))
  })

  it('equationSolvers calls correct sub-path', async () => {
    const fetchMock = mockFetchOk({ ai_models: [], numerical_methods: [] })
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.equationSolvers('heat_eq')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('/equations/heat_eq/solvers')
  })

  it('benchmarkLeaderboard calls correct sub-path', async () => {
    const fetchMock = mockFetchOk({ benchmark_id: 'b1', entries: [] })
    vi.stubGlobal('fetch', fetchMock)
    await knowledgeApi.benchmarkLeaderboard('pdebench_ns2d')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('/benchmarks/pdebench_ns2d/leaderboard')
  })
})

// ── solverApi tests ───────────────────────────────────────────────────────────

describe('solverApi', () => {
  afterEach(() => vi.unstubAllGlobals())

  const minimalReq: SolveRequest = {
    solver: 'classical',
    pde: {
      equation: 'u_t = 0.1 * laplace(u)',
      variables: [],
      equations: [],
      initial_conditions: {},
      coef_fields: {},
      domains: [],
      bcs: [],
    },
    query: { x: [0.0, 1.0], y: [0.0, 1.0], t: [0.0, 1.0] },
  }

  it('solve POSTs to /api/solvers/solve', async () => {
    const fetchMock = mockFetchOk({ success: true, data: {} })
    vi.stubGlobal('fetch', fetchMock)
    await solverApi.solve(minimalReq)
    const [url, opts] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect(url).toBe('/api/solvers/solve')
    expect(opts.method).toBe('POST')
  })

  it('solve sends Content-Type: application/json', async () => {
    const fetchMock = mockFetchOk({ success: true, data: {} })
    vi.stubGlobal('fetch', fetchMock)
    await solverApi.solve(minimalReq)
    const [, opts] = fetchMock.mock.calls[0] as [string, RequestInit]
    expect((opts.headers as Record<string, string>)['Content-Type']).toBe(
      'application/json'
    )
  })

  it('solve serialises the request body as JSON', async () => {
    const fetchMock = mockFetchOk({ success: true, data: {} })
    vi.stubGlobal('fetch', fetchMock)
    await solverApi.solve(minimalReq)
    const [, opts] = fetchMock.mock.calls[0] as [string, RequestInit]
    const parsed = JSON.parse(opts.body as string)
    expect(parsed.solver).toBe('classical')
    expect(parsed.pde.equation).toBe('u_t = 0.1 * laplace(u)')
  })

  it('listSolvers calls /api/solvers/solvers', async () => {
    const fetchMock = mockFetchOk({ success: true, data: [] })
    vi.stubGlobal('fetch', fetchMock)
    await solverApi.listSolvers()
    expect(fetchMock).toHaveBeenCalledWith('/api/solvers/solvers')
  })

  it('propagates error message from solver response body', async () => {
    const fetchMock = mockFetchError(408, 'Solver timed out after 300s')
    vi.stubGlobal('fetch', fetchMock)
    await expect(solverApi.solve(minimalReq)).rejects.toThrow(
      'Solver timed out after 300s'
    )
  })
})

// ── skillsApi tests ───────────────────────────────────────────────────────────

describe('skillsApi', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('downloadUrl returns the correct URL', () => {
    const url = skillsApi.downloadUrl('pde-ingest-skill')
    expect(url).toBe('/api/skills/pde-ingest-skill/download')
  })

  it('downloadUrl encodes special characters in package name', () => {
    const url = skillsApi.downloadUrl('my skill/v2')
    expect(url).toContain(encodeURIComponent('my skill/v2'))
  })

  it('listSkills calls the correct URL', async () => {
    const fetchMock = mockFetchOk({ skills: ['pde-ingest-skill'] })
    vi.stubGlobal('fetch', fetchMock)
    const result = await skillsApi.listSkills()
    expect(result.skills).toContain('pde-ingest-skill')
    expect(fetchMock).toHaveBeenCalledWith('/api/skills/list')
  })

  it('readFile calls the correct URL with encoded filename', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      text: () => Promise.resolve('# SKILL'),
    })
    vi.stubGlobal('fetch', fetchMock)
    const text = await skillsApi.readFile('pde-ingest-skill', 'SKILL.md')
    expect(text).toBe('# SKILL')
    const url = fetchMock.mock.calls[0][0] as string
    expect(url).toContain('f=SKILL.md')
  })

  it('readFile rejects on non-ok response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: false, status: 404 })
    vi.stubGlobal('fetch', fetchMock)
    await expect(
      skillsApi.readFile('missing-pkg', 'SKILL.md')
    ).rejects.toThrow('HTTP 404')
  })
})
