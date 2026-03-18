import { useChatStore } from '@/stores/chat.store'
import { useShallow } from 'zustand/react/shallow'

export function useChat() {
  return useChatStore(
    useShallow((state) => ({
      conversations: state.conversations,
      activeConversationId: state.activeConversationId,
      messagesByConversation: state.messagesByConversation,
      typingUsers: state.typingUsers,
      loadingConversations: state.loadingConversations,
      loadConversations: state.loadConversations,
      openConversation: state.openConversation,
      sendMessage: state.sendMessage,
      markAsSeen: state.markAsSeen,
    }))
  )
}
