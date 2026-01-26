/**
 * Run detail page showing progress and controls.
 */
import { Link, useParams } from 'react-router-dom'
import { Button, LoadingSpinner, StatusIndicator } from '@common/ui'
import { useRun, useRunProgress, useStartRun, useCancelRun } from '../api/runs'
import type { RunStatus } from '../types/run'

const statusColors: Record<RunStatus, 'success' | 'error' | 'warning' | 'info'> = {
  pending: 'warning',
  running: 'info',
  completed: 'success',
  failed: 'error',
  cancelled: 'warning',
}

export function RunDetail() {
  const { id } = useParams<{ id: string }>()
  const runId = parseInt(id || '0', 10)

  const { data: run, isLoading, error } = useRun(runId)
  const { data: progress } = useRunProgress(runId, run?.status === 'running')
  const startMutation = useStartRun()
  const cancelMutation = useCancelRun()

  const handleStart = async () => {
    try {
      await startMutation.mutateAsync(runId)
    } catch (err: any) {
      alert(`Failed to start run: ${err.response?.data?.detail || err.message}`)
    }
  }

  const handleCancel = async () => {
    if (confirm('Are you sure you want to cancel this run?')) {
      try {
        await cancelMutation.mutateAsync(runId)
      } catch (err: any) {
        alert(`Failed to cancel run: ${err.response?.data?.detail || err.message}`)
      }
    }
  }

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return '-'
    return new Date(dateStr).toLocaleString()
  }

  const formatDuration = (seconds?: number) => {
    if (!seconds) return '-'
    const hours = Math.floor(seconds / 3600)
    const minutes = Math.floor((seconds % 3600) / 60)
    const secs = seconds % 60
    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`
    } else if (minutes > 0) {
      return `${minutes}m ${secs}s`
    } else {
      return `${secs}s`
    }
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  if (error || !run) {
    return (
      <div className="bg-red-50 border border-red-200 text-red-800 p-4 rounded-lg">
        <p className="font-medium">Failed to load run</p>
        <p className="text-sm mt-1">{error?.message || 'Run not found'}</p>
        <Link to="/" className="text-sm underline mt-2 inline-block">
          Back to runs
        </Link>
      </div>
    )
  }

  const currentProgress = progress || run.progress
  const overallProgress = currentProgress?.overall || 0

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex justify-between items-start">
        <div>
          <div className="flex items-center gap-2 mb-2">
            <Link to="/" className="text-blue-600 hover:text-blue-800 text-sm">
              ← Back to runs
            </Link>
          </div>
          <h1 className="text-3xl font-bold text-gray-900">Run #{run.id}</h1>
          <p className="text-gray-600 mt-1">Version: {run.version}</p>
        </div>

        <div className="flex gap-2">
          {run.status === 'pending' && (
            <Button
              variant="primary"
              onClick={handleStart}
              disabled={startMutation.isPending}
            >
              {startMutation.isPending ? 'Starting...' : 'Start Run'}
            </Button>
          )}
          {run.status === 'running' && (
            <Button
              variant="danger"
              onClick={handleCancel}
              disabled={cancelMutation.isPending}
            >
              {cancelMutation.isPending ? 'Cancelling...' : 'Cancel Run'}
            </Button>
          )}
        </div>
      </div>

      {/* Status card */}
      <div className="bg-white shadow rounded-lg p-6">
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          <div>
            <p className="text-sm text-gray-600 mb-1">Status</p>
            <div className="flex items-center gap-2">
              <StatusIndicator status={statusColors[run.status]} />
              <span className="text-lg font-semibold capitalize">{run.status}</span>
            </div>
          </div>

          <div>
            <p className="text-sm text-gray-600 mb-1">Progress</p>
            <p className="text-lg font-semibold">
              {(overallProgress * 100).toFixed(1)}%
            </p>
            {run.status === 'running' && (
              <div className="mt-2 w-full bg-gray-200 rounded-full h-2">
                <div
                  className="bg-blue-600 h-2 rounded-full transition-all duration-300"
                  style={{ width: `${overallProgress * 100}%` }}
                />
              </div>
            )}
          </div>

          <div>
            <p className="text-sm text-gray-600 mb-1">Created</p>
            <p className="text-sm font-medium">{formatDate(run.created_at)}</p>
          </div>

          <div>
            <p className="text-sm text-gray-600 mb-1">Duration</p>
            <p className="text-sm font-medium">{formatDuration(run.duration_seconds)}</p>
            {progress?.eta_seconds && progress.eta_seconds > 0 && (
              <p className="text-xs text-gray-500 mt-1">
                ETA: {formatDuration(progress.eta_seconds)}
              </p>
            )}
          </div>
        </div>

        {run.error_message && (
          <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-md">
            <p className="text-sm font-medium text-red-800 mb-1">Error</p>
            <p className="text-sm text-red-700">{run.error_message}</p>
          </div>
        )}
      </div>

      {/* Configuration */}
      <div className="bg-white shadow rounded-lg p-6">
        <h2 className="text-lg font-semibold text-gray-900 mb-4">Configuration</h2>
        <dl className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div>
            <dt className="text-sm text-gray-600">Years</dt>
            <dd className="text-sm font-medium text-gray-900 mt-1">
              {run.config.years.join(', ')}
            </dd>
          </div>
          <div>
            <dt className="text-sm text-gray-600">States</dt>
            <dd className="text-sm font-medium text-gray-900 mt-1">
              {run.config.states ? run.config.states.join(', ') : 'All 50 states'}
            </dd>
          </div>
          <div>
            <dt className="text-sm text-gray-600">Workers</dt>
            <dd className="text-sm font-medium text-gray-900 mt-1">
              {run.config.workers || 4}
            </dd>
          </div>
          <div>
            <dt className="text-sm text-gray-600">Partition Mode</dt>
            <dd className="text-sm font-medium text-gray-900 mt-1">
              {run.config.partition_mode || 'edge-weighted'}
            </dd>
          </div>
        </dl>
      </div>

      {/* Year details */}
      {run.year_details && run.year_details.length > 0 && (
        <div className="bg-white shadow rounded-lg overflow-hidden">
          <div className="px-6 py-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">Year Progress</h2>
          </div>
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Year
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  States
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase">
                  Stage
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {run.year_details.map((year) => (
                <tr key={year.id}>
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {year.year}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 capitalize">
                    {year.status}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {year.states_completed} / {year.states_total}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {year.current_stage || '-'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
