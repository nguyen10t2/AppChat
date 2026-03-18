import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'
import type { Conversation, CreateConversationPayload, MessagePage } from '@/types/chat'

export const conversationService = {
  async list(): Promise<Conversation[]> {
    const response = await http.get('/conversations')
    return unwrapData<Conversation[]>(response)
  },

  async create(payload: CreateConversationPayload): Promise<Conversation | null> {
    const response = await http.post('/conversations', payload)
    return unwrapData<Conversation | null>(response)
  },

  async getMessages(
    conversationId: string,
    params: { limit: number; cursor?: string | null },
  ): Promise<MessagePage> {
    const response = await http.get(`/conversations/${conversationId}/messages`, {
      params: {
        limit: params.limit,
        cursor: params.cursor ?? undefined,
      },
    })

    return unwrapData<MessagePage>(response)
  },

  async markAsSeen(conversationId: string): Promise<void> {
    await http.post(`/conversations/${conversationId}/mark-as-seen`)
  },
}
