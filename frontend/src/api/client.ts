/**
 * Axios API client configuration.
 *
 * Provides a configured axios instance for making API requests.
 */
import axios from 'axios'

// Get API URL from environment or use default
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8002'

// Create axios instance with default configuration
export const apiClient = axios.create({
  baseURL: API_URL,
  timeout: 30000, // 30 second timeout
  headers: {
    'Content-Type': 'application/json',
  },
})

// Add request interceptor for logging (only in development)
if (import.meta.env.DEV) {
  apiClient.interceptors.request.use(
    (config) => {
      console.log(`[API] ${config.method?.toUpperCase()} ${config.url}`)
      return config
    },
    (error) => {
      console.error('[API] Request error:', error)
      return Promise.reject(error)
    }
  )

  apiClient.interceptors.response.use(
    (response) => {
      console.log(`[API] Response ${response.status} from ${response.config.url}`)
      return response
    },
    (error) => {
      console.error('[API] Response error:', error)
      return Promise.reject(error)
    }
  )
}
