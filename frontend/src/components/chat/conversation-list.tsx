import { Avatar, AvatarFallback } from '@/components/ui/avatar'
import { Badge } from '@/components/ui/badge'
import { ScrollArea } from '@/components/ui/scroll-area'
import type { Conversation } from '@/types/chat'
import { cn } from '@/lib/utils'
import { formatTime } from '@/lib/format'
import { OnlineDot } from '@/components/chat/online-dot'

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
    <ScrollArea className="h-[72vh]">
      <div className="space-y-1 pr-2">
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
                'flex w-full items-start gap-3 border px-3 py-2 text-left transition-colors',
                isActive
                  ? 'border-primary/40 bg-primary/10'
                  : 'border-transparent hover:border-border/60 hover:bg-muted/60',
              )}
            >
              <div className="relative mt-0.5">
                <Avatar className="h-8 w-8 rounded-none">
                  <AvatarFallback>{title.charAt(0).toUpperCase()}</AvatarFallback>
                </Avatar>
                {conversation._type === 'direct' ? (
                  <span className="absolute -right-1 -bottom-1">
                    <OnlineDot online={isOnline} />
                  </span>
                ) : null}
              </div>

              <div className="min-w-0 flex-1">
                <div className="flex items-center justify-between gap-2">
                  <p className="truncate text-sm font-medium text-foreground">{title}</p>
                  {conversation.last_message?.created_at ? (
                    <span className="text-[10px] text-muted-foreground">
                      {formatTime(conversation.last_message.created_at)}
                    </span>
                  ) : null}
                </div>

                <p className="truncate text-xs text-muted-foreground">
                  {conversation.last_message?.content ?? 'Chưa có tin nhắn'}
                </p>
              </div>

              {unread > 0 ? (
                <Badge className="rounded-none" variant="secondary">
                  {unread}
                </Badge>
              ) : null}
            </button>
          )
        })}
      </div>
    </ScrollArea>
  )
}
