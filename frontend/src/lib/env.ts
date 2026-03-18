const API_BASE_URL = import.meta.env.VITE_API_BASE_URL ?? 'http://localhost:8080/api'
const WS_URL = import.meta.env.VITE_WS_URL ?? 'ws://localhost:8080/ws'

export const env = {
  apiBaseUrl: API_BASE_URL,
  wsUrl: WS_URL,
}
