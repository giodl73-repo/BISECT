/**
 * API hooks for run management using React Query.
 */
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import { apiClient } from './client'
import type {
  Run,
  RunDetail,
  RunListResponse,
  RunProgressResponse,
  StateConfigResponse,
  CreateRunRequest,
} from '../types/run'

// Query keys
export const runKeys = {
  all: ['runs'] as const,
  lists: () => [...runKeys.all, 'list'] as const,
  list: (filters?: { status?: string; year?: string; version?: string }) =>
    [...runKeys.lists(), filters] as const,
  details: () => [...runKeys.all, 'detail'] as const,
  detail: (id: number) => [...runKeys.details(), id] as const,
  progress: (id: number) => [...runKeys.all, 'progress', id] as const,
  stateConfig: (year: string) => ['states', 'config', year] as const,
}

// Hooks

/**
 * Fetch list of runs with optional filtering
 */
export function useRuns(params?: {
  status?: string
  year?: string
  version?: string
  limit?: number
  offset?: number
}) {
  return useQuery({
    queryKey: runKeys.list(params),
    queryFn: async () => {
      const { data } = await apiClient.get<RunListResponse>('/api/v1/runs', {
        params,
      })
      return data
    },
  })
}

/**
 * Fetch single run details
 */
export function useRun(id: number) {
  return useQuery({
    queryKey: runKeys.detail(id),
    queryFn: async () => {
      const { data } = await apiClient.get<RunDetail>(`/api/v1/runs/${id}`)
      return data
    },
    enabled: !!id,
  })
}

/**
 * Fetch run progress (for polling)
 */
export function useRunProgress(id: number, enabled = true) {
  return useQuery({
    queryKey: runKeys.progress(id),
    queryFn: async () => {
      const { data } = await apiClient.get<RunProgressResponse>(
        `/api/v1/runs/${id}/progress`
      )
      return data
    },
    enabled: enabled && !!id,
    refetchInterval: (data) => {
      // Poll every 2 seconds if running, otherwise stop
      return data?.status === 'running' ? 2000 : false
    },
  })
}

/**
 * Fetch state configuration for a year
 */
export function useStateConfig(year: string = '2020') {
  return useQuery({
    queryKey: runKeys.stateConfig(year),
    queryFn: async () => {
      const { data } = await apiClient.get<StateConfigResponse>(
        '/api/v1/runs/config/states',
        { params: { year } }
      )
      return data
    },
  })
}

/**
 * Create a new run
 */
export function useCreateRun() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (request: CreateRunRequest) => {
      const { data } = await apiClient.post<Run>('/api/v1/runs', request)
      return data
    },
    onSuccess: () => {
      // Invalidate run list to refetch
      queryClient.invalidateQueries({ queryKey: runKeys.lists() })
    },
  })
}

/**
 * Start a run
 */
export function useStartRun() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (runId: number) => {
      const { data } = await apiClient.post<Run>(
        `/api/v1/runs/${runId}/actions/start`
      )
      return data
    },
    onSuccess: (data) => {
      // Invalidate specific run and list queries
      queryClient.invalidateQueries({ queryKey: runKeys.detail(data.id) })
      queryClient.invalidateQueries({ queryKey: runKeys.lists() })
    },
  })
}

/**
 * Cancel a running run
 */
export function useCancelRun() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (runId: number) => {
      const { data } = await apiClient.post<Run>(
        `/api/v1/runs/${runId}/actions/cancel`
      )
      return data
    },
    onSuccess: (data) => {
      // Invalidate specific run and list queries
      queryClient.invalidateQueries({ queryKey: runKeys.detail(data.id) })
      queryClient.invalidateQueries({ queryKey: runKeys.lists() })
    },
  })
}

/**
 * Delete a run
 */
export function useDeleteRun() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: async (runId: number) => {
      await apiClient.delete(`/api/v1/runs/${runId}`)
    },
    onSuccess: () => {
      // Invalidate run list to refetch
      queryClient.invalidateQueries({ queryKey: runKeys.lists() })
    },
  })
}
