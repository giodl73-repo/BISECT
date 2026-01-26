/**
 * API hooks for district data and visualizations.
 */
import { useQuery } from '@tanstack/react-query'
import { apiClient } from './client'
import type { DistrictCollection, DistrictStats } from '../types/district'

// Query keys
export const districtKeys = {
  all: ['districts'] as const,
  geojson: (runId: number, year: string) =>
    [...districtKeys.all, 'geojson', runId, year] as const,
  stats: (runId: number, year: string) =>
    [...districtKeys.all, 'stats', runId, year] as const,
}

/**
 * Fetch district GeoJSON for a run and year
 */
export function useDistrictGeoJSON(runId: number, year: string, enabled = true) {
  return useQuery({
    queryKey: districtKeys.geojson(runId, year),
    queryFn: async () => {
      const { data } = await apiClient.get<DistrictCollection>(
        `/api/v1/runs/${runId}/districts/${year}/geojson`
      )
      return data
    },
    enabled: enabled && !!runId && !!year,
    staleTime: 5 * 60 * 1000, // Districts don't change often, cache for 5 minutes
  })
}

/**
 * Fetch district statistics for a run and year
 */
export function useDistrictStats(runId: number, year: string, enabled = true) {
  return useQuery({
    queryKey: districtKeys.stats(runId, year),
    queryFn: async () => {
      const { data } = await apiClient.get<DistrictStats>(
        `/api/v1/runs/${runId}/districts/${year}/stats`
      )
      return data
    },
    enabled: enabled && !!runId && !!year,
  })
}
