import { create } from 'zustand'
import { wsClient } from '@/lib/ws'
import { conversationService } from '@/services/conversation.service'
import { messageService } from '@/services/message.service'
import type { Conversation, Message } from '@/types/chat'
import { useAuthStore } from '@/stores/auth.store'

type ChatState = {
  conversations: Conversation[]
  activeConversationId: string | null
  messagesByConversation: Record<string, Message[]>
  loadingConversations: boolean
  typingUsers: Record<string, string[]>
  loadConversations: () => Promise<void>
  openConversation: (conversationId: string) => Promise<void>
  refreshConversationMessages: (conversationId: string) => Promise<void>
  sendMessage: (payload: {
    content?: string
    fileUrl?: string
    replyToId?: string
    myUserId: string
  }) => Promise<void>
  markAsSeen: (conversationId: string) => Promise<void>
  receiveMessage: (message: Message) => void
  updateTyping: (conversationId: string, userId: string, isTyping: boolean) => void
  editMessageRealtime: (conversationId: string, messageId: string, newContent: string) => void
  deleteMessageRealtime: (conversationId: string, messageId: string) => void
  setActiveConversationId: (id: string | null) => void
  updateConversation: (conversationId: string, partial: Partial<Conversation>) => void
  updateConversationLastMessage: (payload: {
    conversationId: string
    message: Message
    unreadCounts?: Record<string, number>
  }) => void
  addConversation: (conversation: Conversation) => void
  addParticipant: (conversationId: string, participant: Conversation['participants'][number]) => void
  removeParticipant: (conversationId: string, userId: string) => void
  removeConversation: (conversationId: string) => void
}

function sortByRecent(a: Conversation, b: Conversation) {
  const ad = a.last_message?.created_at ?? a.updated_at
  const bd = b.last_message?.created_at ?? b.updated_at
  return new Date(bd).getTime() - new Date(ad).getTime()
}

function findDirectRecipient(conversation: Conversation, myUserId: string): string | null {
  if (conversation._type !== 'direct') return null
  const recipient = conversation.participants.find((item) => item.user_id !== myUserId)
  return recipient?.user_id ?? null
}

function sortAndReplace(conversations: Conversation[], updated: Conversation): Conversation[] {
  return conversations
    .map((item) => (item.conversation_id === updated.conversation_id ? updated : item))
    .sort(sortByRecent)
}

export const useChatStore = create<ChatState>((set, get) => ({
  conversations: [],
  activeConversationId: null,
  messagesByConversation: {},
  loadingConversations: false,
  typingUsers: {},

  loadConversations: async () => {
    set({ loadingConversations: true })
    try {
      const data = await conversationService.list()
      set({ conversations: [...data].sort(sortByRecent) })
    } finally {
      set({ loadingConversations: false })
    }
  },

  openConversation: async (conversationId) => {
    const alreadyLoaded = Boolean(get().messagesByConversation[conversationId])
    set({ activeConversationId: conversationId })

    // Tham gia room websocket để nhận sự kiện "Typing"
    wsClient.send({ type: 'join_conversation', conversation_id: conversationId })

    if (alreadyLoaded) return

    const page = await conversationService.getMessages(conversationId, {
      limit: 30,
      cursor: null,
    })

    set((state) => ({
      messagesByConversation: {
        ...state.messagesByConversation,
        [conversationId]: page.messages,
      },
    }))
  },

  refreshConversationMessages: async (conversationId) => {
    const page = await conversationService.getMessages(conversationId, {
      limit: 30,
      cursor: null,
    })

    set((state) => ({
      messagesByConversation: {
        ...state.messagesByConversation,
        [conversationId]: page.messages,
      },
    }))
  },

  sendMessage: async ({ content, fileUrl, replyToId, myUserId }) => {
    const normalized = content?.trim()
    const hasText = Boolean(normalized)
    const hasFile = Boolean(fileUrl)
    if (!hasText && !hasFile) return

    const activeConversationId = get().activeConversationId
    if (!activeConversationId) return

    const conversation = get().conversations.find(
      (item) => item.conversation_id === activeConversationId,
    )

    if (!conversation) return

    const tempId = `temp-${Date.now()}`
    const tempMessage: Message = {
      id: tempId,
      conversation_id: activeConversationId,
      sender_id: myUserId,
      reply_to_id: replyToId ?? null,
      _type: hasFile ? 'file' : 'text',
      content: normalized ?? null,
      file_url: fileUrl ?? null,
      is_edited: false,
      deleted_at: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      is_sending: true,
    }

    set((state) => ({
      messagesByConversation: {
        ...state.messagesByConversation,
        [activeConversationId]: [
          ...(state.messagesByConversation[activeConversationId] ?? []),
          tempMessage,
        ],
      },
    }))

    try {
      let realMsg: Message

      if (conversation._type === 'group') {
        realMsg = await messageService.sendGroup({
          conversation_id: activeConversationId,
          content: normalized,
          type: hasFile ? 'file' : 'text',
          file_url: fileUrl,
          reply_to_id: replyToId,
        })
      } else {
        const recipientId = findDirectRecipient(conversation, myUserId)
        if (!recipientId) throw new Error('Không tìm thấy người nhận')

        realMsg = await messageService.sendDirect({
          conversation_id: activeConversationId,
          recipient_id: recipientId,
          content: normalized,
          type: hasFile ? 'file' : 'text',
          file_url: fileUrl,
          reply_to_id: replyToId,
        })
      }

      set((state) => {
        const prev = state.messagesByConversation[activeConversationId] ?? []
        
        // Kiểm tra xem WebSocket có chạy về nhanh hơn cả HTTP Response không?
        // Nếu có, WebSocket đã thêm message thật vào cuối mảng rồi.
        const arrivedViaWs = prev.some((m) => m.id === realMsg.id)

        return {
          messagesByConversation: {
            ...state.messagesByConversation,
            [activeConversationId]: arrivedViaWs
              ? prev.filter((m) => m.id !== tempId) // Thu hồi temp message vì WS đã render
              : prev.map((m) => (m.id === tempId ? realMsg : m)), // Bình thường thì thay thế
          },
        }
      })
    } catch (e) {
      set((state) => {
        const prev = state.messagesByConversation[activeConversationId] ?? []
        return {
          messagesByConversation: {
            ...state.messagesByConversation,
            [activeConversationId]: prev.filter((m) => m.id !== tempId),
          },
        }
      })
      throw e
    }
  },

  markAsSeen: async (conversationId) => {
    await conversationService.markAsSeen(conversationId)
    const myUserId = useAuthStore.getState().user?.id
    if (!myUserId) return

    set((state) => ({
      conversations: state.conversations.map((conversation) =>
        conversation.conversation_id !== conversationId
          ? conversation
          : {
              ...conversation,
              participants: conversation.participants.map((participant) =>
                participant.user_id === myUserId
                  ? { ...participant, unread_count: 0 }
                  : participant,
              ),
            },
      ),
    }))
  },

  receiveMessage: (message) => {
    set((state) => {
      const conversationId = message.conversation_id
      const previous = state.messagesByConversation[conversationId] ?? []
      const exists = previous.some((item) => item.id === message.id)

      return {
        messagesByConversation: {
          ...state.messagesByConversation,
          [conversationId]: exists ? previous : [...previous, message],
        },
      }
    })
  },

  updateTyping: (conversationId, userId, isTyping) => {
    set((state) => {
      const current = new Set(state.typingUsers[conversationId] ?? [])
      if (isTyping) {
        current.add(userId)
      } else {
        current.delete(userId)
      }

      return {
        typingUsers: {
          ...state.typingUsers,
          [conversationId]: Array.from(current),
        },
      }
    })
  },

  editMessageRealtime: (conversationId, messageId, newContent) => {
    set((state) => {
      const messages = state.messagesByConversation[conversationId] ?? []
      return {
        messagesByConversation: {
          ...state.messagesByConversation,
          [conversationId]: messages.map((item) =>
            item.id === messageId
              ? {
                  ...item,
                  content: newContent,
                  is_edited: true,
                }
              : item,
          ),
        },
      }
    })
  },

  deleteMessageRealtime: (conversationId, messageId) => {
    set((state) => {
      const messages = state.messagesByConversation[conversationId] ?? []
      return {
        messagesByConversation: {
          ...state.messagesByConversation,
          [conversationId]: messages.map((item) =>
            item.id === messageId
              ? {
                  ...item,
                  content: 'Tin nhắn đã bị xóa',
                  deleted_at: new Date().toISOString(),
                }
              : item,
          ),
        },
      }
    })
  },

  setActiveConversationId: (id) => set({ activeConversationId: id }),

  updateConversation: (conversationId, partial) => {
    set((state) => ({
      conversations: state.conversations.map((conversation) =>
        conversation.conversation_id === conversationId
          ? { ...conversation, ...partial }
          : conversation,
      ),
    }))
  },

  updateConversationLastMessage: ({ conversationId, message, unreadCounts }) => {
    set((state) => {
      const updated = state.conversations.find(
        (item) => item.conversation_id === conversationId,
      )
      if (!updated) return state

      const nextConversation: Conversation = {
        ...updated,
        last_message: {
          content: message.content,
          sender_id: message.sender_id,
          created_at: message.created_at,
        },
        updated_at: message.created_at,
        participants: updated.participants.map((participant) => {
          const unread = unreadCounts?.[participant.user_id]
          return unread === undefined
            ? participant
            : { ...participant, unread_count: unread }
        }),
      }

      return {
        conversations: sortAndReplace(state.conversations, nextConversation),
      }
    })
  },

  addConversation: (conversation) => {
    set((state) => {
      if (state.conversations.some((item) => item.conversation_id === conversation.conversation_id)) {
        return {
          conversations: sortAndReplace(state.conversations, conversation),
        }
      }

      return {
        conversations: [...state.conversations, conversation].sort(sortByRecent),
      }
    })
  },

  addParticipant: (conversationId, participant) => {
    set((state) => ({
      conversations: state.conversations.map((conversation) => {
        if (conversation.conversation_id !== conversationId) return conversation
        if (conversation.participants.some((item) => item.user_id === participant.user_id)) {
          return conversation
        }

        return {
          ...conversation,
          participants: [...conversation.participants, participant],
        }
      }),
    }))
  },

  removeParticipant: (conversationId, userId) => {
    set((state) => ({
      conversations: state.conversations.map((conversation) =>
        conversation.conversation_id !== conversationId
          ? conversation
          : {
              ...conversation,
              participants: conversation.participants.filter(
                (participant) => participant.user_id !== userId,
              ),
            },
      ),
    }))
  },

  removeConversation: (conversationId) => {
    set((state) => ({
      conversations: state.conversations.filter(
        (conversation) => conversation.conversation_id !== conversationId,
      ),
      activeConversationId:
        state.activeConversationId === conversationId ? null : state.activeConversationId,
      messagesByConversation: Object.fromEntries(
        Object.entries(state.messagesByConversation).filter(
          ([key]) => key !== conversationId,
        ),
      ),
    }))
  },
}))
