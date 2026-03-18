import axios, {
  AxiosError,
  type AxiosRequestConfig,
  type InternalAxiosRequestConfig,
} from 'axios'
import { env } from '@/lib/env'
import { useAuthStore } from '@/stores/auth.store'

type RetryRequestConfig = InternalAxiosRequestConfig & {
  _retry?: boolean
}

export const http = axios.create({
  baseURL: env.apiBaseUrl,
  withCredentials: true,
})

const refreshClient = axios.create({
  baseURL: env.apiBaseUrl,
  withCredentials: true,
})

let refreshPromise: Promise<string | null> | null = null

http.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  const token = useAuthStore.getState().accessToken

  if (token) {
    config.headers.set('Authorization', `Bearer ${token}`)
  }

  return config
})

async function performRefresh(): Promise<string | null> {
  try {
    const response = await refreshClient.post<{ data?: { access_token: string } }>('/auth/refresh')
    return response.data.data?.access_token ?? null
  } catch {
    return null
  }
}

export function forceRefresh(): Promise<string | null> {
  if (!refreshPromise) {
    refreshPromise = performRefresh().finally(() => {
      refreshPromise = null
    })
  }
  return refreshPromise
}

http.interceptors.response.use(
  (response) => response,
  async (error: AxiosError) => {
    const originalConfig = error.config as RetryRequestConfig | undefined
    const status = error.response?.status

    if (!originalConfig || status !== 401 || originalConfig._retry) {
      return Promise.reject(error)
    }

    originalConfig._retry = true

    const nextToken = await forceRefresh()

    if (!nextToken) {
      useAuthStore.getState().clearSession()
      return Promise.reject(error)
    }

    useAuthStore.getState().setAccessToken(nextToken)

    const nextConfig: AxiosRequestConfig = {
      ...originalConfig,
      headers: {
        ...originalConfig.headers,
        Authorization: `Bearer ${nextToken}`,
      },
    }

    return http.request(nextConfig)
  },
)
