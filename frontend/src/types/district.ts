/**
 * Type definitions for districts and visualizations.
 */

export interface District {
  district_id: number
  state: string
  population: number
  area_sq_km: number
  perimeter_km: number
  compactness_polsby_popper: number
  compactness_reock: number
  partisan_lean?: number
  demographic_stats?: Record<string, number>
}

export interface DistrictGeoJSON {
  type: 'Feature'
  properties: District
  geometry: {
    type: 'Polygon' | 'MultiPolygon'
    coordinates: number[][][] | number[][][][]
  }
}

export interface DistrictCollection {
  type: 'FeatureCollection'
  features: DistrictGeoJSON[]
}

export type ColorMetric =
  | 'compactness'
  | 'partisan'
  | 'demographic'
  | 'population'

export interface DistrictStats {
  total_districts: number
  avg_compactness: number
  avg_population: number
  total_population: number
  total_area: number
}
