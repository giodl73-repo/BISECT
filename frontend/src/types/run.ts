/**
 * Type definitions for pipeline runs.
 */

export type RunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface Run {
  id: number
  version: string
  status: RunStatus
  config: RunConfig
  progress?: RunProgress
  created_at: string
  started_at?: string
  completed_at?: string
  error_message?: string
  output_path?: string
}

export interface RunConfig {
  years: string[]
  states?: string[]
  workers?: number
  dpi?: number
  partition_mode?: string
  version: string
}

export interface RunProgress {
  overall?: number
  years?: Record<string, YearProgress>
}

export interface YearProgress {
  status: string
  states_completed: number
  states_total: number
  current_stage?: string
}

export interface RunDetail extends Run {
  year_details?: YearDetail[]
  duration_seconds?: number
}

export interface YearDetail {
  id: number
  run_id: number
  year: string
  status: string
  states_completed: number
  states_total: number
  current_stage?: string
  started_at?: string
  completed_at?: string
  error_message?: string
}

export interface RunListResponse {
  runs: Run[]
  total: number
  limit: number
  offset: number
}

export interface RunProgressResponse {
  run_id: number
  status: RunStatus
  overall_progress: number
  years: Record<string, {
    status: string
    states_completed: number
    states_total: number
    current_stage?: string
  }>
  eta_seconds?: number
}

export interface StateInfo {
  code: string
  name: string
  districts: number
  fips: string
}

export interface StateConfigResponse {
  year: string
  states: StateInfo[]
}

export interface CreateRunRequest {
  version: string
  years: string[]
  states?: string[]
  workers?: number
  dpi?: number
  partition_mode?: string
}
