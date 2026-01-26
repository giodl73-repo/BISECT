/**
 * Districts page with map visualization and table.
 */
import { useState } from 'react'
import { useParams, Link } from 'react-router-dom'
import { Button, LoadingSpinner } from '@common/ui'
import { useRun } from '../api/runs'
import { useDistrictGeoJSON, useDistrictStats } from '../api/districts'
import { DistrictMap } from '../components/DistrictMap'
import type { ColorMetric } from '../types/district'

export function Districts() {
  const { id } = useParams<{ id: string }>()
  const runId = parseInt(id || '0', 10)

  const { data: run, isLoading: runLoading } = useRun(runId)
  const [selectedYear, setSelectedYear] = useState<string>('')
  const [colorMetric, setColorMetric] = useState<ColorMetric>('compactness')
  const [selectedDistrict, setSelectedDistrict] = useState<number | undefined>()

  // Auto-select first year when run loads
  const year = selectedYear || run?.config.years[0] || ''

  const {
    data: districts,
    isLoading: districtsLoading,
    error: districtsError,
  } = useDistrictGeoJSON(runId, year, run?.status === 'completed')

  const { data: stats } = useDistrictStats(runId, year, run?.status === 'completed')

  const handleYearChange = (newYear: string) => {
    setSelectedYear(newYear)
    setSelectedDistrict(undefined) // Reset selection
  }

  if (runLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  if (!run) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-800 p-4 rounded-lg">
        <p className="font-medium">Run not found</p>
        <Link to="/" className="text-sm underline mt-2 inline-block">
          Back to runs
        </Link>
      </div>
    )
  }

  if (run.status !== 'completed') {
    return (
      <div className="bg-yellow-50 border border-yellow-200 text-yellow-800 p-4 rounded-lg">
        <p className="font-medium">Districts not available</p>
        <p className="text-sm mt-1">
          This run must be completed before districts can be visualized.
        </p>
        <p className="text-sm mt-1">Current status: {run.status}</p>
        <Link to={`/runs/${runId}`} className="text-sm underline mt-2 inline-block">
          View run details
        </Link>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Link
              to={`/runs/${runId}`}
              className="text-blue-600 hover:text-blue-800 text-sm"
            >
              ← Back to run
            </Link>
          </div>
          <h1 className="text-3xl font-bold text-gray-900">
            District Visualization
          </h1>
          <p className="text-gray-600 mt-1">Run #{runId} - {run.version}</p>
        </div>
      </div>

      {/* Controls */}
      <div className="bg-white shadow rounded-lg p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          {/* Year selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Census Year
            </label>
            <select
              value={year}
              onChange={(e) => handleYearChange(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              {run.config.years.map((y) => (
                <option key={y} value={y}>
                  {y}
                </option>
              ))}
            </select>
          </div>

          {/* Metric selection */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Color By
            </label>
            <select
              value={colorMetric}
              onChange={(e) => setColorMetric(e.target.value as ColorMetric)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="compactness">Compactness</option>
              <option value="population">Population</option>
              <option value="partisan">Partisan Lean</option>
              <option value="demographic">Demographics</option>
            </select>
          </div>

          {/* Stats */}
          {stats && (
            <div className="flex items-end">
              <div className="text-sm text-gray-600">
                <p>
                  <strong>Total Districts:</strong> {stats.total_districts}
                </p>
                <p>
                  <strong>Avg Compactness:</strong>{' '}
                  {stats.avg_compactness.toFixed(3)}
                </p>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Map */}
      <div className="bg-white shadow rounded-lg p-4">
        {districtsLoading && (
          <div className="flex items-center justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        )}

        {districtsError && (
          <div className="bg-yellow-50 border border-yellow-200 text-yellow-800 p-4 rounded-md">
            <p className="font-medium">Districts not yet available</p>
            <p className="text-sm mt-1">
              District data is still being generated or the pipeline hasn't completed
              the visualization stage yet.
            </p>
          </div>
        )}

        {districts && (
          <DistrictMap
            districts={districts}
            colorMetric={colorMetric}
            selectedDistrict={selectedDistrict}
            onDistrictClick={setSelectedDistrict}
          />
        )}
      </div>

      {/* District table */}
      {districts && districts.features.length > 0 && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">
              District Statistics
            </h2>
          </div>
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                    District
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                    State
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                    Population
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                    Area (km²)
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase">
                    Compactness
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {districts.features
                  .sort(
                    (a, b) =>
                      a.properties.district_id - b.properties.district_id
                  )
                  .map((feature) => {
                    const d = feature.properties
                    const isSelected = selectedDistrict === d.district_id

                    return (
                      <tr
                        key={d.district_id}
                        className={`cursor-pointer hover:bg-gray-50 ${
                          isSelected ? 'bg-blue-50' : ''
                        }`}
                        onClick={() => setSelectedDistrict(d.district_id)}
                      >
                        <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                          {d.district_id}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                          {d.state}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 text-right">
                          {d.population.toLocaleString()}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 text-right">
                          {d.area_sq_km.toLocaleString(undefined, {
                            maximumFractionDigits: 0,
                          })}
                        </td>
                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 text-right">
                          {d.compactness_polsby_popper.toFixed(3)}
                        </td>
                      </tr>
                    )
                  })}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Legend */}
      {districts && (
        <div className="bg-white shadow rounded-lg p-4">
          <h3 className="text-sm font-semibold text-gray-900 mb-2">
            Color Legend - {colorMetric}
          </h3>
          {colorMetric === 'compactness' && (
            <div className="flex items-center gap-2 text-xs">
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-red-700" />
                <span>Low (&lt;0.1)</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-orange-600" />
                <span>0.1-0.2</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-yellow-600" />
                <span>0.2-0.3</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-lime-600" />
                <span>0.3-0.4</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-green-700" />
                <span>High (&gt;0.4)</span>
              </div>
            </div>
          )}
          {colorMetric === 'population' && (
            <div className="flex items-center gap-2 text-xs">
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-blue-100" />
                <span>&lt;600k</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-blue-300" />
                <span>600k-700k</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-blue-500" />
                <span>700k-800k</span>
              </div>
              <div className="flex items-center gap-1">
                <div className="w-4 h-4 bg-blue-700" />
                <span>&gt;800k</span>
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  )
}
