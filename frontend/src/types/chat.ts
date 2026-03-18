export type ConversationType = 'direct' | 'group'

export type GroupInfo = {
  name: string
  created_by: string
  avatar_url: string | null
}

export type Participant = {
  user_id: string
  display_name: string
  avatar_url: string | null
  unread_count: number
  joined_at: string
}

export type LastMessage = {
  content: string | null
  sender_id: string
  created_at: string
}

export type Conversation = {
  conversation_id: string
  _type: ConversationType
  group_info: GroupInfo | null
  last_message: LastMessage | null
  participants: Participant[]
  created_at: string
  updated_at: string
}

export type MessageType = 'text' | 'image' | 'video' | 'file' | 'system'

export type Message = {
  id: string
  conversation_id: string
  sender_id: string
  reply_to_id: string | null
  _type: MessageType
  content: string | null
  file_url: string | null
  is_edited: boolean
  deleted_at: string | null
  created_at: string
  updated_at: string
  is_sending?: boolean
  is_error?: boolean
}

export type MessagePage = {
  messages: Message[]
  cursor: string | null
}

export type CreateConversationPayload = {
  type: ConversationType
  name: string
  member_ids: string[]
}

export type SendDirectMessagePayload = {
  conversation_id?: string
  recipient_id?: string
  content?: string
  type?: MessageType
  file_url?: string
  reply_to_id?: string
}

export type SendGroupMessagePayload = {
  conversation_id: string
  content?: string
  type?: MessageType
  file_url?: string
  reply_to_id?: string
}
