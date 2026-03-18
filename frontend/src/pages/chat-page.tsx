import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { toast } from 'sonner'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { ConversationList } from '@/components/chat/conversation-list'
import { MessagePane } from '@/components/chat/message-pane'
import { MessageComposer } from '@/components/chat/message-composer'
import { useAuth } from '@/hooks/use-auth'
import { useChat } from '@/hooks/use-chat'
import { usePresenceStore } from '@/stores/presence.store'
import { fileUploadService } from '@/services/file-upload.service'
import { extractErrorMsg } from '@/lib/api'
import type { Message } from '@/types/chat'

export function ChatPage() {
  const [searchParams] = useSearchParams()
  const [replyTo, setReplyTo] = useState<Message | null>(null)
  const { user } = useAuth()
  const {
    conversations,
    activeConversationId,
    messagesByConversation,
    typingUsers,
    loadingConversations,
    loadConversations,
    openConversation,
    sendMessage,
    markAsSeen,
  } = useChat()

  const presenceMap = usePresenceStore((state) => state.byUserId)
  const fetchPresence = usePresenceStore((state) => state.fetchBatch)

  useEffect(() => {
    void loadConversations()
  }, [loadConversations])

  useEffect(() => {
    const ids = conversations.flatMap((conversation) =>
      conversation.participants.map((participant) => participant.user_id),
    )
    void fetchPresence(ids)
  }, [conversations, fetchPresence])

  useEffect(() => {
    const conversationId = searchParams.get('conversation')
    if (!conversationId) return
    void openConversation(conversationId)
  }, [openConversation, searchParams])

  const activeConversation = useMemo(
    () =>
      conversations.find((item) => item.conversation_id === activeConversationId) ?? null,
    [activeConversationId, conversations],
  )

  const activeMessages = activeConversationId
    ? messagesByConversation[activeConversationId] ?? []
    : []

  if (!user) return null

  return (
    <div className="grid gap-4 lg:grid-cols-[360px_minmax(0,1fr)]">
      <Card className="glass-strong">
        <CardHeader>
          <CardTitle>Cuộc trò chuyện</CardTitle>
        </CardHeader>
        <CardContent>
          <ConversationList
            conversations={conversations}
            activeConversationId={activeConversationId}
            myUserId={user.id}
            onlineMap={presenceMap}
            onSelectConversation={(conversationId) => {
              void openConversation(conversationId)
              void markAsSeen(conversationId)
            }}
          />
        </CardContent>
      </Card>

      <Card className="glass-strong">
        <CardHeader className="border-b border-border/60">
          <CardTitle>
            {activeConversation
              ? activeConversation._type === 'group'
                ? activeConversation.group_info?.name ?? 'Nhóm'
                : activeConversation.participants.find((item) => item.user_id !== user.id)
                    ?.display_name ?? 'Trò chuyện'
              : 'Chọn cuộc trò chuyện'}
          </CardTitle>
        </CardHeader>

        <CardContent className="p-0">
          <MessagePane
            messages={activeMessages}
            myUserId={user.id}
            onReply={(message) => setReplyTo(message)}
            typingUsers={
              activeConversationId
                ? (typingUsers[activeConversationId] ?? []).filter(
                    (typingUserId) => typingUserId !== user.id,
                  )
                : []
            }
          />

          <MessageComposer
            disabled={!activeConversationId || loadingConversations}
            replyTo={replyTo}
            onCancelReply={() => setReplyTo(null)}
            onSend={async ({ content, file, replyTo: targetMessage }) => {
              try {
                let fileUrl: string | undefined

                if (file) {
                  const uploaded = await fileUploadService.upload(file)
                  fileUrl = uploaded.url
                }

                await sendMessage({
                  content,
                  fileUrl,
                  replyToId: targetMessage?.id,
                  myUserId: user.id,
                })
                setReplyTo(null)
              } catch (error) {
                toast.error(extractErrorMsg(error))
              }
            }}
          />
        </CardContent>
      </Card>
    </div>
  )
}
