// ── Knowledge Base types (Neo4j graph backend) ────────────────────────────────

// ── Enums ─────────────────────────────────────────────────────────────────────

export type PdeType = 'parabolic' | 'elliptic' | 'hyperbolic' | 'mixed' | 'other'
export type ConditionType = 'boundary' | 'initial' | 'domain' | 'regularity' | 'other'
export type NumericalMethodType = 'grid_based' | 'mesh_based' | 'spectral_based' | 'mesh_free' | 'other'
export type TrainingType = 'supervised' | 'unsupervised' | 'self_supervised' | 'physics_informed' | 'operator_learning'
export type LossType = 'physics' | 'data_driven' | 'boundary' | 'combined' | 'other'
export type MetricType = 'accuracy' | 'efficiency' | 'stability' | 'generalisation' | 'other'

// ── Node types ────────────────────────────────────────────────────────────────

export interface Equation {
  id: string
  name: string
  pde_type: PdeType
  variables: string[]
  time_dependent: boolean
  operator?: string
  description?: string
  tags: string[]
}

export interface Condition {
  id: string
  name: string
  condition_type: ConditionType
  form?: string
  description?: string
}

export interface Theorem {
  id: string
  name: string
  result: string
  confidence: number
  description?: string
  source?: string
}

export interface NumericalMethod {
  id: string
  name: string
  method_type: NumericalMethodType
  order?: number
  description?: string
  tags: string[]
  /** Solver id at engines API `GET /solvers`. Set → callable locally. */
  engine_id?: string
}

export interface AIModel {
  id: string
  name: string
  architecture: string
  input_vars: string[]
  output_vars: string[]
  training_type: TrainingType
  description?: string
  paper_ref?: string
  tags: string[]
  /** Solver id at engines API `GET /solvers`. Set → callable locally. */
  engine_id?: string
}

export interface LossFunction {
  id: string
  name: string
  loss_type: LossType
  formulation?: string
  description?: string
}

export interface Metric {
  id: string
  name: string
  metric_type: MetricType
  unit?: string
  description?: string
}

export interface Dataset {
  id: string
  name: string
  dimension?: string
  num_samples?: number
  description?: string
  url?: string
}

export interface Paper {
  id: string
  title: string
  authors: string[]
  published_year?: number
  arxiv_id?: string
  doi?: string
  pdf_path?: string
  tags: string[]
}

// ── Benchmark / BenchResult ───────────────────────────────────────────────────

export type SourceType = 'paper_reported' | 'self_run' | 'third_party_reproduction'
export type ResultConfidence = 'verified' | 'single' | 'disputed'

export interface Benchmark {
  id: string
  name: string
  dataset_id: string
  metric_id: string
  lower_is_better: boolean
  protocol?: string
  tolerance?: number
}

export interface BenchResult {
  id?: string
  method_id: string
  method_label: 'AIModel' | 'NumericalMethod'
  benchmark_id: string
  value: number
  source_type: SourceType
  source_paper_id?: string
  hardware?: string
  code_ref?: string
  recorded_at?: string
}

export interface LeaderboardEntry {
  method_id: string
  method_label: 'AIModel' | 'NumericalMethod'
  method_name?: string
  best_value: number
  all_values: number[]
  n_independent_sources: number
  n_results: number
  confidence: ResultConfidence
  latest_recorded_at?: string
  source_breakdown: Record<string, number>
}

export interface BenchmarkLeaderboard {
  benchmark: Benchmark
  dataset_name?: string
  metric_name?: string
  entries: LeaderboardEntry[]
}

// ── Compound query results ────────────────────────────────────────────────────

/** Lightweight reference to an equation (used in traversal results). */
export interface EquationRef {
  id: string
  name: string
  pde_type: string
}

export interface LossFunctionRef {
  id: string
  name: string
  loss_type: string
}

export interface MetricRef {
  id: string
  name: string
  metric_type: string
}

export interface DatasetRef {
  id: string
  name: string
  dimension?: string
}

export interface PaperRef {
  id: string
  title: string
  published_year?: number
  arxiv_id?: string
}

/** One group of solvers (used by EquationSolvers below). */
export interface SolverGroup {
  ai_models: AIModel[]
  numerical_methods: NumericalMethod[]
}

/** All solvers for a given equation, split by whether they are callable
 *  through the engines API. The split is based on `engine_id` on each method. */
export interface EquationSolvers {
  equation_id: string
  equation_name: string
  executable: SolverGroup
  literature_only: SolverGroup
}

/** Full profile of an AI model. */
export interface AIModelProfile {
  model: AIModel
  solves: EquationRef[]
  trained_by: LossFunctionRef[]
  evaluated_by: MetricRef[]
  tested_on: DatasetRef[]
}

/** Conditions associated with an equation. */
export interface EquationConditions {
  equation_id: string
  conditions: Condition[]
}

/** Full profile of a paper. */
export interface PaperProfile {
  paper: Paper
  proposes: Array<{ label: string; id: string; name: string }>
  studies: EquationRef[]
  uses_datasets: DatasetRef[]
  cites: PaperRef[]
  cited_by: PaperRef[]
}

/** Search hit across all node types. */
export interface SearchHit {
  label: string
  id: string
  name: string
  description?: string
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

// ── IC value: flat array of numbers OR keyword token ─────────────────────────
export type IcValue = number[] | 'zero' | 'grf'

// ── SDF domain for non-periodic / complex-geometry BCs ───────────────────────
export type SdfRole = 'interior' | 'boundary_dirichlet' | 'boundary_neumann' | 'boundary_mur'

export interface SdfDomain {
  name: string
  sdf: number[]     // flat n×n signed-distance-function values
  role: SdfRole
}

// ── Boundary condition spec ───────────────────────────────────────────────────
export interface BcSpec {
  domain: string
  vars: string[]    // variable / expression names summed to zero
  bc_type: 'dirichlet' | 'neumann' | 'mur' | 'robin'
  coef?: number
}

// ── Full PDE specification ────────────────────────────────────────────────────
export interface PdeSpec {
  // Legacy single-variable fields (backward-compatible)
  equation: string
  initial_condition?: number[]
  boundary_condition?: string
  parameters?: Record<string, number>

  // Multi-variable / multi-equation extensions
  variables?: string[]
  equations?: string[]
  initial_conditions?: Record<string, IcValue>
  coef_fields?: Record<string, number[]>
  domains?: SdfDomain[]
  bcs?: BcSpec[]
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
  variables: string[]
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
