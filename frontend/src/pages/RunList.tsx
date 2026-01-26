/**
 * Run list page with filtering and pagination.
 */
import { useState } from 'react'
import { Link } from 'react-router-dom'
import { Button, LoadingSpinner, StatusIndicator } from '@common/ui'
import { useRuns, useDeleteRun } from '../api/runs'
import type { RunStatus } from '../types/run'

const statusColors: Record<RunStatus, 'success' | 'error' | 'warning' | 'info'> = {
  pending: 'warning',
  running: 'info',
  completed: 'success',
  failed: 'error',
  cancelled: 'warning',
}

export function RunList() {
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [yearFilter, setYearFilter] = useState<string>('')

  const { data, isLoading, error } = useRuns({
    status: statusFilter || undefined,
    year: yearFilter || undefined,
  })

  const deleteMutation = useDeleteRun()

  const handleDelete = async (id: number) => {
    if (confirm('Are you sure you want to delete this run?')) {
      try {
        await deleteMutation.mutateAsync(id)
      } catch (err) {
        alert('Failed to delete run')
      }
    }
  }

  const formatDate = (dateStr?: string) => {
    if (!dateStr) return '-'
    return new Date(dateStr).toLocaleString()
  }

  return (
    <div className="space-y-6">
      <div className="flex justify-between items-center">
        <h1 className="text-3xl font-bold text-gray-900">Pipeline Runs</h1>
        <Link to="/create">
          <Button variant="primary">New Run</Button>
        </Link>
      </div>

      {/* Filters */}
      <div className="bg-white shadow rounded-lg p-4">
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Status
            </label>
            <select
              value={statusFilter}
              onChange={(e) => setStatusFilter(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">All</option>
              <option value="pending">Pending</option>
              <option value="running">Running</option>
              <option value="completed">Completed</option>
              <option value="failed">Failed</option>
              <option value="cancelled">Cancelled</option>
            </select>
          </div>

          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Year
            </label>
            <select
              value={yearFilter}
              onChange={(e) => setYearFilter(e.target.value)}
              className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              <option value="">All</option>
              <option value="2020">2020</option>
              <option value="2010">2010</option>
              <option value="2000">2000</option>
            </select>
          </div>

          <div className="flex items-end">
            <Button
              variant="secondary"
              onClick={() => {
                setStatusFilter('')
                setYearFilter('')
              }}
            >
              Clear Filters
            </Button>
          </div>
        </div>
      </div>

      {/* Run list */}
      <div className="bg-white shadow rounded-lg overflow-hidden">
        {isLoading && (
          <div className="flex items-center justify-center py-12">
            <LoadingSpinner size="lg" />
          </div>
        )}

        {error && (
          <div className="p-4 bg-red-50 border border-red-200 text-red-800">
            Failed to load runs: {error.message}
          </div>
        )}

        {data && data.runs.length === 0 && (
          <div className="p-8 text-center text-gray-500">
            <p className="text-lg font-medium mb-2">No runs found</p>
            <p className="text-sm">Create a new run to get started</p>
          </div>
        )}

        {data && data.runs.length > 0 && (
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  ID
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Version
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Status
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Years
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Created
                </th>
                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {data.runs.map((run) => (
                <tr key={run.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4 whitespace-nowrap text-sm font-medium text-gray-900">
                    {run.id}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {run.version}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap">
                    <div className="flex items-center gap-2">
                      <StatusIndicator status={statusColors[run.status]} />
                      <span className="text-sm text-gray-900 capitalize">
                        {run.status}
                      </span>
                    </div>
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {run.config.years.join(', ')}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                    {formatDate(run.created_at)}
                  </td>
                  <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium space-x-2">
                    <Link
                      to={`/runs/${run.id}`}
                      className="text-blue-600 hover:text-blue-900"
                    >
                      View
                    </Link>
                    {run.status !== 'running' && (
                      <button
                        onClick={() => handleDelete(run.id)}
                        className="text-red-600 hover:text-red-900"
                        disabled={deleteMutation.isPending}
                      >
                        Delete
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      {data && data.total > 0 && (
        <div className="text-sm text-gray-500 text-center">
          Showing {data.runs.length} of {data.total} runs
        </div>
      )}
    </div>
  )
}
