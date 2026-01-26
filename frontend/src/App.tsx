import { useEffect, useState } from 'react'
import { Button, LoadingSpinner, StatusIndicator } from '@common/ui'
import { apiClient } from './api/client'

interface HealthStatus {
  status: string
  database: string
  timestamp: string
}

interface VersionInfo {
  version: string
  api_version: string
  wave: number
}

function App() {
  const [health, setHealth] = useState<HealthStatus | null>(null)
  const [version, setVersion] = useState<VersionInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const fetchStatus = async () => {
    setLoading(true)
    setError(null)
    try {
      const [healthRes, versionRes] = await Promise.all([
        apiClient.get<HealthStatus>('/health'),
        apiClient.get<VersionInfo>('/version'),
      ])
      setHealth(healthRes.data)
      setVersion(versionRes.data)
    } catch (err) {
      setError('Failed to connect to API')
      console.error('API connection error:', err)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchStatus()
  }, [])

  return (
    <div className="min-h-screen bg-gray-100 py-12 px-4 sm:px-6 lg:px-8">
      <div className="max-w-3xl mx-auto">
        <div className="bg-white shadow rounded-lg p-6">
          <h1 className="text-3xl font-bold text-gray-900 mb-4">
            Apportionment Dashboard
          </h1>
          <p className="text-sm text-gray-600 mb-6">
            Congressional Redistricting via Algorithmic METIS Bisection
          </p>

          {loading && (
            <div className="flex items-center justify-center py-8">
              <LoadingSpinner size="lg" />
            </div>
          )}

          {error && (
            <div className="bg-red-50 border border-red-200 text-red-800 px-4 py-3 rounded mb-4">
              {error}
            </div>
          )}

          {!loading && health && version && (
            <div className="space-y-4">
              <div className="border-l-4 border-green-500 bg-green-50 p-4">
                <h2 className="text-lg font-semibold text-gray-900 mb-2">
                  System Status
                </h2>
                <div className="space-y-2 text-sm text-gray-600">
                  <div className="flex items-center gap-2">
                    <span className="font-medium">API Status:</span>
                    <StatusIndicator
                      status={health.status === 'healthy' ? 'success' : 'error'}
                    />
                    <span>{health.status}</span>
                  </div>
                  <div className="flex items-center gap-2">
                    <span className="font-medium">Database:</span>
                    <StatusIndicator
                      status={health.database === 'connected' ? 'success' : 'error'}
                    />
                    <span>{health.database}</span>
                  </div>
                  <p>
                    <span className="font-medium">Version:</span> {version.version}
                  </p>
                  <p>
                    <span className="font-medium">API Version:</span> {version.api_version}
                  </p>
                  <p>
                    <span className="font-medium">Wave:</span> {version.wave}
                  </p>
                </div>
              </div>

              <div className="flex gap-2">
                <Button variant="primary" onClick={fetchStatus}>
                  Refresh Status
                </Button>
                <Button variant="secondary" onClick={() => window.open('/docs', '_blank')}>
                  View API Docs
                </Button>
              </div>

              <div className="text-sm text-gray-500 border-t pt-4">
                <p className="font-semibold mb-1">Enhancement 60 - Project Setup & Infrastructure</p>
                <p>Using shared components from App Manager (@common/ui)</p>
                <p className="mt-2">Full dashboard features will be added in Enhancements 63 and 64.</p>
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

export default App
