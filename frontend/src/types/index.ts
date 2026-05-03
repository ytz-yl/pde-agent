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

/** All solvers for a given equation. */
export interface EquationSolvers {
  equation_id: string
  equation_name: string
  ai_models: AIModel[]
  numerical_methods: NumericalMethod[]
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
