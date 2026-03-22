import { useEffect, useMemo, useState } from 'react'
import { useSearchParams } from 'react-router-dom'
import { toast } from 'sonner'
import { cn } from '@/lib/utils'
import { ArrowLeftIcon, Info, CaretLeft, CaretRight } from '@phosphor-icons/react'
import { Plus } from 'lucide-react'
import { Phone, Video } from 'lucide-react'
import { ConversationList } from '@/components/chat/conversation-list'
import { MessagePane } from '@/components/chat/message-pane'
import { MessageComposer } from '@/components/chat/message-composer'
import { CreateGroupModal } from '@/components/chat/create-group-modal'
import { GroupInfoPanel } from '@/components/chat/group-info-panel'
import { useAuth } from '@/hooks/use-auth'
import { useChat } from '@/hooks/use-chat'
import { usePresenceStore } from '@/stores/presence.store'
import { useCallStore } from '@/stores/call.store'
import { fileUploadService } from '@/services/file-upload.service'
import { extractErrorMsg } from '@/lib/api'
import type { Message } from '@/types/chat'

export function ChatPage() {
  const [searchParams] = useSearchParams()
  const [replyTo, setReplyTo] = useState<Message | null>(null)
  // Mobile: true = đang xem chat (không xem list)
  const [mobileShowChat, setMobileShowChat] = useState(false)
  const [isCreateGroupOpen, setIsCreateGroupOpen] = useState(false)
  const [showInfo, setShowInfo] = useState(false)
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false)
  const conversationFromQuery = searchParams.get('conversation')

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
  const initiateCall = useCallStore((state) => state.initiateCall)

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
    if (!conversationFromQuery) return
    void openConversation(conversationFromQuery)
  }, [conversationFromQuery, openConversation])

  const activeConversation = useMemo(
    () =>
      conversations.find((item) => item.conversation_id === activeConversationId) ?? null,
    [activeConversationId, conversations],
  )

  const activeMessages = activeConversationId
    ? messagesByConversation[activeConversationId] ?? []
    : []
  const isMobileChatVisible = mobileShowChat || Boolean(conversationFromQuery)

  if (!user) return null

  const activeTitle = activeConversation
    ? activeConversation._type === 'group'
      ? activeConversation.group_info?.name ?? 'Nhóm'
      : activeConversation.participants.find((item) => item.user_id !== user.id)
          ?.display_name ?? 'Trò chuyện'
    : 'Chọn cuộc trò chuyện'

  const handleSelect = (conversationId: string) => {
    void openConversation(conversationId)
    void markAsSeen(conversationId)
    setMobileShowChat(true)
  }

  return (
    /* Full-height split layout, mobile stacks */
    <div className="flex h-full w-full overflow-hidden relative">
      {/* ── Conversation list panel ── */}
      <div
        className={cn(
          "flex h-full flex-col border-r border-border/60 bg-card/80 transition-all duration-300 ease-in-out relative group",
          isSidebarCollapsed ? "w-0 overflow-hidden md:w-0" : "w-full md:w-80",
          isMobileChatVisible ? 'hidden md:flex' : 'flex'
        )}
      >
        {/* Toggle Button for Desktop */}
        <button
          onClick={() => setIsSidebarCollapsed(!isSidebarCollapsed)}
          className={cn(
            "absolute -right-3 top-1/2 z-20 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full border border-border/60 bg-background text-muted-foreground shadow-sm transition-all hover:text-foreground md:flex",
            isSidebarCollapsed ? "hidden scale-0" : "opacity-0 group-hover:opacity-100"
          )}
          title={isSidebarCollapsed ? "Mở rộng" : "Thu gọn"}
        >
          <CaretLeft size={14} weight="bold" />
        </button>
        {/* Panel header */}
        <div className="flex items-center justify-between border-b border-border/60 px-4 py-3">
          <h2 className="text-sm font-semibold text-foreground">Tin nhắn</h2>
          <button
            onClick={() => setIsCreateGroupOpen(true)}
            className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted transition-colors"
            title="Tạo nhóm mới"
          >
            <Plus size={18} />
          </button>
        </div>

        <ConversationList
          conversations={conversations}
          activeConversationId={activeConversationId}
          myUserId={user.id}
          onlineMap={presenceMap}
          onSelectConversation={handleSelect}
        />
      </div>

      {/* Floating Toggle Button when Collapsed */}
      {isSidebarCollapsed && (
        <button
          onClick={() => setIsSidebarCollapsed(false)}
          className="absolute left-2 top-1/2 z-20 flex h-8 w-8 -translate-y-1/2 items-center justify-center rounded-xl border border-primary/20 bg-primary/10 text-primary shadow-sm transition-all hover:bg-primary/20 md:flex hidden"
          title="Mở rộng danh sách"
        >
          <CaretRight size={20} weight="bold" />
        </button>
      )}

      {/* ── Chat area ── */}
      <div
        className={cn(
          "flex h-full flex-1 flex-col bg-background transition-all duration-300 ease-in-out",
          isMobileChatVisible ? 'flex' : 'hidden md:flex'
        )}
      >
        {/* Chat header */}
        <div className="flex items-center gap-3 border-b border-border/60 px-4 py-3 bg-card/80">
          {/* Back button – mobile only */}
          <button
            onClick={() => setMobileShowChat(false)}
            className="md:hidden flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted transition-colors"
          >
            <ArrowLeftIcon size={18} />
          </button>

          <div className="min-w-0 flex-1">
            <p className="truncate text-sm font-semibold text-foreground">{activeTitle}</p>
            {activeConversation && (
              <p className="text-xs text-muted-foreground">
                {activeConversation._type === 'group' ? 'Nhóm' : 'Trò chuyện riêng'}
              </p>
            )}
          </div>

          {activeConversation?._type === 'direct' && activeConversationId && (
            <div className="flex items-center gap-1">
              <button
                onClick={() => {
                  void initiateCall(activeConversationId, 'audio').catch((error) => {
                    toast.error(extractErrorMsg(error))
                  })
                }}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted transition-colors"
                title="Gọi thoại"
              >
                <Phone size={18} />
              </button>
              <button
                onClick={() => {
                  void initiateCall(activeConversationId, 'video').catch((error) => {
                    toast.error(extractErrorMsg(error))
                  })
                }}
                className="flex h-8 w-8 items-center justify-center rounded-lg text-muted-foreground hover:bg-muted transition-colors"
                title="Gọi video"
              >
                <Video size={18} />
              </button>
            </div>
          )}

          {activeConversation?._type === 'group' && (
            <button
              onClick={() => setShowInfo(!showInfo)}
              className={cn(
                "h-8 w-8 flex items-center justify-center rounded-lg transition-colors",
                showInfo ? "bg-primary/10 text-primary" : "text-muted-foreground hover:bg-muted"
              )}
            >
              <Info size={20} />
            </button>
          )}
        </div>

        {/* Messages */}
        <div className="flex min-h-0 flex-1 flex-col">
          {activeConversationId ? (
            <>
              <MessagePane
                messages={activeMessages}
                myUserId={user.id}
                participants={activeConversation?.participants ?? []}
                onReply={(message) => setReplyTo(message)}
                typingUsers={
                  (typingUsers[activeConversationId] ?? []).filter(
                    (typingUserId) => typingUserId !== user.id,
                  )
                }
              />
              <MessageComposer
                disabled={loadingConversations}
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
            </>
          ) : (
            <div className="flex flex-1 flex-col items-center justify-center gap-3 text-muted-foreground">
              <div className="text-5xl">💬</div>
              <p className="text-sm">Chọn một cuộc trò chuyện để bắt đầu</p>
            </div>
          )}
        </div>
      </div>

      {activeConversation?._type === 'group' && showInfo && (
        <div className="w-80 h-full hidden lg:block shrink-0">
          <GroupInfoPanel 
            conversation={activeConversation} 
            onClose={() => setShowInfo(false)} 
          />
        </div>
      )}

      {/* Mobile Info Overlay */}
      {activeConversation?._type === 'group' && showInfo && (
        <div className="fixed inset-0 z-50 lg:hidden bg-background">
           <GroupInfoPanel 
            conversation={activeConversation} 
            onClose={() => setShowInfo(false)} 
          />
        </div>
      )}
      <CreateGroupModal
        open={isCreateGroupOpen}
        onOpenChange={setIsCreateGroupOpen}
      />
    </div>
  )
}
