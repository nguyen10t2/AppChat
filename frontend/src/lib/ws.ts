import { env } from '@/lib/env'
import type { ClientWsMessage, ServerWsMessage } from '@/types/websocket'

type MessageListener = (message: ServerWsMessage) => void

class RawWsClient {
  private socket: WebSocket | null = null
  private listeners = new Set<MessageListener>()
  private reconnectTimer: number | null = null
  private disconnectTimer: number | null = null
  private token: string | null = null
  private shouldReconnect = false

  connect(token: string) {
    if (this.disconnectTimer) {
      window.clearTimeout(this.disconnectTimer)
      this.disconnectTimer = null
    }

    this.token = token
    this.shouldReconnect = true

    if (
      this.socket &&
      (this.socket.readyState === WebSocket.OPEN || this.socket.readyState === WebSocket.CONNECTING)
    ) {
      return
    }

    this.open()
  }

  disconnect() {
    this.shouldReconnect = false

    if (this.reconnectTimer) {
      window.clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }

    if (this.disconnectTimer) {
      window.clearTimeout(this.disconnectTimer)
    }

    this.disconnectTimer = window.setTimeout(() => {
      if (this.socket) {
        this.socket.onclose = null
        this.socket.close()
      }
      this.socket = null
      this.disconnectTimer = null
    }, 250)
  }

  onMessage(listener: MessageListener) {
    this.listeners.add(listener)
    return () => this.listeners.delete(listener)
  }

  send(message: ClientWsMessage) {
    if (this.socket?.readyState !== WebSocket.OPEN) return
    this.socket.send(JSON.stringify(message))
  }

  private open() {
    if (!this.token) return

    if (this.socket) {
      this.socket.onclose = null
      this.socket.close()
    }

    const socket = new WebSocket(env.wsUrl)
    this.socket = socket

    socket.onopen = () => {
      this.send({ type: 'auth', token: this.token as string })
    }

    socket.onmessage = (event) => {
      try {
        const parsed = JSON.parse(String(event.data)) as ServerWsMessage
        this.listeners.forEach((listener) => listener(parsed))
      } catch {
        // ignore invalid payload
      }
    }

    socket.onclose = () => {
      if (!this.shouldReconnect) return
      this.reconnectTimer = window.setTimeout(() => this.open(), 1200)
    }
  }
}

export const wsClient = new RawWsClient()
