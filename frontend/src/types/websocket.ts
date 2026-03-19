import type { Conversation, Message } from '@/types/chat'

export type ClientWsMessage =
  | { type: 'auth'; token: string }
  | { type: 'send_message'; conversation_id: string; content: string }
  | { type: 'join_conversation'; conversation_id: string }
  | { type: 'leave_conversation'; conversation_id: string }
  | { type: 'typing_start'; conversation_id: string }
  | { type: 'typing_stop'; conversation_id: string }
  | { type: 'ping' }

type WsMessageLike = Message & {
  _id?: string
}

type NewMessagePayload = {
  type: 'new-message'
  message: WsMessageLike
  conversation: {
    _id: string
    last_message: {
      _id: string
      content: string | null
      created_at: string
      sender: {
        _id: string
        display_name: string
        avatar_url: string | null
      }
    }
    last_message_at: string
  }
  unread_counts: Record<string, number>
}

export type ServerWsMessage =
  | { type: 'auth-success'; user_id: string }
  | { type: 'auth-failed'; reason: string }
  | NewMessagePayload
  | {
      type: 'message-edited'
      conversation_id: string
      message_id: string
      new_content: string
    }
  | { type: 'message-deleted'; conversation_id: string; message_id: string }
  | { type: 'user-typing'; conversation_id: string; user_id: string }
  | { type: 'user-stopped-typing'; conversation_id: string; user_id: string }
  | { type: 'online-users'; user_ids: string[] }
  | { type: 'user-online'; user_id: string }
  | { type: 'user-offline'; user_id: string; last_seen: string | null }
  | { type: 'pong' }
  | { type: 'error'; message: string }
  | { type: 'new-group'; conversation: Conversation | null }
  | {
      type: 'group-updated'
      conversation_id: string
      name?: string
      avatar_url?: string | null
    }
  | {
      type: 'member-added'
      conversation_id: string
      user_id: string
      display_name: string
      avatar_url: string | null
    }
  | { type: 'member-removed'; conversation_id: string; user_id: string }
