import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'
import type {
  Message,
  SendDirectMessagePayload,
  SendGroupMessagePayload,
} from '@/types/chat'

export const messageService = {
  async sendDirect(payload: SendDirectMessagePayload): Promise<Message> {
    const response = await http.post('/messages/direct/', payload)
    return unwrapData<Message>(response)
  },

  async sendGroup(payload: SendGroupMessagePayload): Promise<Message> {
    const response = await http.post('/messages/group/', payload)
    return unwrapData<Message>(response)
  },

  async edit(messageId: string, content: string): Promise<Message> {
    const response = await http.patch(`/messages/${messageId}`, { content })
    return unwrapData<Message>(response)
  },

  async remove(messageId: string): Promise<void> {
    await http.delete(`/messages/${messageId}`)
  },
}
