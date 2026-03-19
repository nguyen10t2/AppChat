import { Badge } from '@/components/ui/badge'
import type { Conversation } from '@/types/chat'
import { cn } from '@/lib/utils'
import { formatTime } from '@/lib/format'
import { OnlineDot } from '@/components/chat/online-dot'
import { GroupAvatar } from '@/components/chat/group-avatar'

type Props = {
  conversations: Conversation[]
  activeConversationId: string | null
  myUserId: string
  onlineMap: Record<string, { isOnline: boolean }>
  onSelectConversation: (conversationId: string) => void
}

function conversationName(conversation: Conversation, myUserId: string) {
  if (conversation._type === 'group') {
    return conversation.group_info?.name || 'Nhóm'
  }

  const counterpart = conversation.participants.find((item) => item.user_id !== myUserId)
  return counterpart?.display_name ?? 'Người dùng'
}

function unreadOfMe(conversation: Conversation, myUserId: string) {
  return (
    conversation.participants.find((item) => item.user_id === myUserId)?.unread_count ?? 0
  )
}

export function ConversationList(props: Props) {
  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto">
      <div className="space-y-1 p-2">
        {props.conversations.length === 0 && (
          <div className="flex flex-col items-center gap-2 py-12 text-muted-foreground">
            <span className="text-3xl">💬</span>
            <p className="text-xs">Chưa có cuộc trò chuyện nào</p>
          </div>
        )}
        {props.conversations.map((conversation) => {
          const isActive = props.activeConversationId === conversation.conversation_id
          const title = conversationName(conversation, props.myUserId)
          const unread = unreadOfMe(conversation, props.myUserId)
          const counterpart = conversation.participants.find(
            (item) => item.user_id !== props.myUserId,
          )
          const isOnline = counterpart
            ? props.onlineMap[counterpart.user_id]?.isOnline ?? false
            : false

          return (
            <button
              key={conversation.conversation_id}
              onClick={() => props.onSelectConversation(conversation.conversation_id)}
              className={cn(
                'flex w-full items-center gap-3 rounded-xl px-3 py-2 text-left transition-colors',
                isActive
                  ? 'bg-primary/10 text-foreground'
                  : 'hover:bg-muted/60',
              )}
            >
                <GroupAvatar
                  avatarUrl={
                    conversation._type === 'group'
                      ? conversation.group_info?.avatar_url
                      : counterpart?.avatar_url
                  }
                  participants={
                    conversation._type === 'group'
                      ? conversation.participants
                      : counterpart
                      ? [counterpart]
                      : []
                  }
                  size="md"
                />
                {conversation._type === 'direct' ? (
                  <span className="absolute -right-0.5 -bottom-0.5">
                    <OnlineDot online={isOnline} />
                  </span>
                ) : null}

              <div className="min-w-0 flex-1">
                <div className="flex items-center justify-between gap-2">
                  <p className="truncate text-sm font-medium text-foreground">{title}</p>
                  {conversation.last_message?.created_at ? (
                    <span className="shrink-0 text-[10px] text-muted-foreground">
                      {formatTime(conversation.last_message.created_at)}
                    </span>
                  ) : null}
                </div>

                <p className="truncate text-xs text-muted-foreground">
                  {conversation.last_message?.content ?? 'Chưa có tin nhắn'}
                </p>
              </div>

              {unread > 0 ? (
                <Badge className="shrink-0 rounded-full text-[10px] h-5 min-w-5 px-1.5" variant="destructive">
                  {unread}
                </Badge>
              ) : null}
            </button>
          )
        })}
      </div>
    </div>
  )
}
