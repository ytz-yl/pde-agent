// ── Knowledge Base types ──────────────────────────────────────────────────────

export interface PaperTag {
  pde_type?: string
  method?: string
  domain?: string
  benchmark?: string
}

export interface Paper {
  id: string
  title: string
  abstract_text?: string
  authors: string[]
  published?: string
  source_url?: string
  pdf_url?: string
  tags: PaperTag[]
  created_at: string
  updated_at: string
}

export interface SearchHit {
  score: number
  paper: Paper
}

export interface Method {
  id: string
  name: string
  category: 'classical' | 'ml' | 'hybrid'
  description?: string
  tags: string[]
}

export interface RelatedEntry {
  relation: string
  weight: number
  method: Method
}

export interface ComparisonReport {
  method_a: Method
  method_b: Method
  relations: Array<{ kind: string; weight: number }>
  summary: string
}

export interface Recommendation {
  method: Method
  reason: string
  score: number
}

export interface RecommendRequest {
  pde_type: string
  domain?: string
  constraints: string[]
  top_k: number
}

// ── Solver types ──────────────────────────────────────────────────────────────

export type SolverCategory = 'classical' | 'machine_learning' | 'hybrid'

export interface SolverInfo {
  id: string
  name: string
  category: SolverCategory
  description: string
  supported_pde_types: string[]
  backend: string
  available: boolean
}

export interface PdeSpec {
  equation: string
  initial_condition?: number[]
  boundary_condition?: string
  parameters?: Record<string, number>
}

export interface QuerySpec {
  x: number[]
  y: number[]
  t?: number[]
}

export interface SolveRequest {
  solver?: string
  pde: PdeSpec
  query: QuerySpec
  options?: Record<string, unknown>
}

export interface SolutionShape {
  n_t: number
  n_x: number
  n_y: number
  n_vars: number
}

export interface SolveMetadata {
  wall_time_ms: number
  backend: string
  notes: string[]
}

export interface SolveResponse {
  solver_used: string
  solution: number[][][][]
  shape: SolutionShape
  metadata: SolveMetadata
}

export interface ApiResponse<T> {
  success: boolean
  data?: T
  error?: string
  request_id: string
  timestamp: string
}
