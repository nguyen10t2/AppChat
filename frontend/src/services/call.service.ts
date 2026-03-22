import { unwrapData } from '@/lib/api'
import { http } from '@/lib/http'
import type { Call, CallHistoryPage, CallType } from '@/types/call'

export type InitiateCallPayload = {
  conversation_id: string
  call_type: CallType
}

export type InitiateCallResponse = {
  call_id: string
  status: string
}

export type RespondCallPayload = {
  accept: boolean
  reason?: string
}

export const callService = {
  async initiate(payload: InitiateCallPayload): Promise<InitiateCallResponse> {
    const response = await http.post('/calls', payload)
    return unwrapData<InitiateCallResponse>(response)
  },

  async respond(callId: string, payload: RespondCallPayload): Promise<void> {
    await http.post(`/calls/${callId}/respond`, payload)
  },

  async cancel(callId: string): Promise<void> {
    await http.post(`/calls/${callId}/cancel`)
  },

  async end(callId: string): Promise<void> {
    await http.post(`/calls/${callId}/end`)
  },

  async getHistory(params?: { limit?: number; cursor?: string | null }): Promise<CallHistoryPage> {
    const response = await http.get('/calls/history', {
      params: {
        limit: params?.limit ?? 20,
        cursor: params?.cursor ?? undefined,
      },
    })

    const payload = unwrapData<{ calls: Call[]; cursor?: string | null }>(response)
    return {
      calls: payload.calls,
      cursor: payload.cursor ?? null,
    }
  },
}
