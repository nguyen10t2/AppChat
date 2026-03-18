import { useEffect } from 'react'
import { toast } from 'sonner'
import { wsClient } from '@/lib/ws'
import { useAuthStore } from '@/stores/auth.store'
import { useChatStore } from '@/stores/chat.store'
import { usePresenceStore } from '@/stores/presence.store'

function normalizeMessage(raw: { _id?: string; id?: string } & Record<string, unknown>) {
  const id = typeof raw.id === 'string' ? raw.id : raw._id
  if (!id) return null

  return {
    id,
    conversation_id: String(raw.conversation_id ?? ''),
    sender_id: String(raw.sender_id ?? ''),
    reply_to_id: (raw.reply_to_id as string | null | undefined) ?? null,
    _type: String(raw._type ?? 'text') as 'text' | 'image' | 'video' | 'file' | 'system',
    content: (raw.content as string | null | undefined) ?? null,
    file_url: (raw.file_url as string | null | undefined) ?? null,
    is_edited: Boolean(raw.is_edited),
    deleted_at: (raw.deleted_at as string | null | undefined) ?? null,
    created_at: String(raw.created_at ?? new Date().toISOString()),
    updated_at: String(raw.updated_at ?? new Date().toISOString()),
  }
}

export function useWebSocketBridge() {
  const token = useAuthStore((state) => state.accessToken)
  const loadConversations = useChatStore((state) => state.loadConversations)

  useEffect(() => {
    if (!token) {
      wsClient.disconnect()
      return
    }

    wsClient.connect(token)

    const unsubscribe = wsClient.onMessage((message) => {
      const chatState = useChatStore.getState()
      const presenceState = usePresenceStore.getState()

      switch (message.type) {
        case 'new-message': {
          const normalized = normalizeMessage(message.message)
          if (normalized) {
            chatState.receiveMessage(normalized)
          }
          void loadConversations()
          break
        }
        case 'message-edited': {
          chatState.editMessageRealtime(
            message.conversation_id,
            message.message_id,
            message.new_content,
          )
          break
        }
        case 'message-deleted': {
          chatState.deleteMessageRealtime(message.conversation_id, message.message_id)
          break
        }
        case 'user-typing': {
          chatState.updateTyping(message.conversation_id, message.user_id, true)
          break
        }
        case 'user-stopped-typing': {
          chatState.updateTyping(message.conversation_id, message.user_id, false)
          break
        }
        case 'online-users': {
          presenceState.setOnlineUsers(message.user_ids)
          break
        }
        case 'user-online': {
          presenceState.markUserOnline(message.user_id)
          break
        }
        case 'user-offline': {
          presenceState.markUserOffline(message.user_id, message.last_seen)
          break
        }
        case 'error': {
          toast.error(message.message)
          break
        }
        default:
          break
      }
    })

    return () => {
      unsubscribe()
      wsClient.disconnect()
    }
  }, [loadConversations, token])
}
