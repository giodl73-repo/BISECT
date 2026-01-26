/**
 * Create new run page with form.
 */
import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { Button, LoadingSpinner } from '@common/ui'
import { useCreateRun, useStateConfig } from '../api/runs'

export function CreateRun() {
  const navigate = useNavigate()
  const createMutation = useCreateRun()
  const { data: stateConfig } = useStateConfig('2020')

  const [version, setVersion] = useState('v1')
  const [years, setYears] = useState<string[]>(['2020'])
  const [states, setStates] = useState<string[]>([])
  const [workers, setWorkers] = useState(4)
  const [partitionMode, setPartitionMode] = useState('edge-weighted')
  const [useAllStates, setUseAllStates] = useState(true)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    try {
      const run = await createMutation.mutateAsync({
        version,
        years,
        states: useAllStates ? undefined : states,
        workers,
        partition_mode: partitionMode,
      })

      // Navigate to the run detail page
      navigate(`/runs/${run.id}`)
    } catch (err: any) {
      alert(`Failed to create run: ${err.response?.data?.detail || err.message}`)
    }
  }

  const handleYearToggle = (year: string) => {
    setYears((prev) =>
      prev.includes(year)
        ? prev.filter((y) => y !== year)
        : [...prev, year].sort()
    )
  }

  const handleStateToggle = (stateCode: string) => {
    setStates((prev) =>
      prev.includes(stateCode)
        ? prev.filter((s) => s !== stateCode)
        : [...prev, stateCode].sort()
    )
  }

  return (
    <div className="max-w-3xl mx-auto">
      <div className="mb-6">
        <Link to="/" className="text-blue-600 hover:text-blue-800 text-sm">
          ← Back to runs
        </Link>
        <h1 className="text-3xl font-bold text-gray-900 mt-2">Create New Run</h1>
        <p className="text-gray-600 mt-1">
          Configure a new redistricting pipeline execution
        </p>
      </div>

      <form onSubmit={handleSubmit} className="bg-white shadow rounded-lg p-6 space-y-6">
        {/* Version */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Version
            <span className="text-red-500 ml-1">*</span>
          </label>
          <input
            type="text"
            value={version}
            onChange={(e) => setVersion(e.target.value)}
            required
            placeholder="e.g., v1, test, experiment-1"
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <p className="text-xs text-gray-500 mt-1">
            Unique identifier for this run (used in output paths)
          </p>
        </div>

        {/* Census Years */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Census Years
            <span className="text-red-500 ml-1">*</span>
          </label>
          <div className="flex gap-4">
            {['2000', '2010', '2020'].map((year) => (
              <label key={year} className="flex items-center">
                <input
                  type="checkbox"
                  checked={years.includes(year)}
                  onChange={() => handleYearToggle(year)}
                  className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                />
                <span className="ml-2 text-sm text-gray-700">{year}</span>
              </label>
            ))}
          </div>
          {years.length === 0 && (
            <p className="text-xs text-red-500 mt-1">
              At least one year is required
            </p>
          )}
        </div>

        {/* States */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            States
          </label>
          <div className="mb-2">
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={useAllStates}
                onChange={(e) => setUseAllStates(e.target.checked)}
                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
              />
              <span className="ml-2 text-sm text-gray-700">All 50 states</span>
            </label>
          </div>

          {!useAllStates && stateConfig && (
            <div className="border border-gray-300 rounded-md p-4 max-h-64 overflow-y-auto">
              <div className="grid grid-cols-2 md:grid-cols-3 gap-2">
                {stateConfig.states.map((state) => (
                  <label key={state.code} className="flex items-center">
                    <input
                      type="checkbox"
                      checked={states.includes(state.code)}
                      onChange={() => handleStateToggle(state.code)}
                      className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                    />
                    <span className="ml-2 text-sm text-gray-700">
                      {state.code}
                    </span>
                  </label>
                ))}
              </div>
              {!useAllStates && states.length === 0 && (
                <p className="text-xs text-red-500 mt-2">
                  Select at least one state or choose "All 50 states"
                </p>
              )}
            </div>
          )}
        </div>

        {/* Workers */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Worker Processes
          </label>
          <input
            type="number"
            min={1}
            max={12}
            value={workers}
            onChange={(e) => setWorkers(parseInt(e.target.value, 10))}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
          <p className="text-xs text-gray-500 mt-1">
            Number of parallel workers (1-12). More workers = faster execution but higher CPU usage.
          </p>
        </div>

        {/* Partition Mode */}
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Partition Mode
          </label>
          <select
            value={partitionMode}
            onChange={(e) => setPartitionMode(e.target.value)}
            className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            <option value="edge-weighted">Edge-weighted (default)</option>
            <option value="unweighted">Unweighted</option>
          </select>
          <p className="text-xs text-gray-500 mt-1">
            Edge-weighted mode produces more compact districts
          </p>
        </div>

        {/* Submit */}
        <div className="flex gap-3 pt-4 border-t">
          <Button
            type="submit"
            variant="primary"
            disabled={createMutation.isPending || years.length === 0 || (!useAllStates && states.length === 0)}
          >
            {createMutation.isPending ? (
              <span className="flex items-center gap-2">
                <LoadingSpinner size="sm" />
                Creating...
              </span>
            ) : (
              'Create Run'
            )}
          </Button>
          <Link to="/">
            <Button type="button" variant="secondary">
              Cancel
            </Button>
          </Link>
        </div>
      </form>
    </div>
  )
}
