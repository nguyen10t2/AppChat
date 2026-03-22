import { useMemo, useEffect, useRef } from 'react'
import { formatTime } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { Message, Participant } from '@/types/chat'
import { resolveAssetUrl } from '@/lib/url'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'

type Props = {
  messages: Message[]
  myUserId: string
  participants: Participant[]
  typingUsers: string[]
  onReply: (message: Message) => void
}

export function MessagePane({ messages, myUserId, participants, typingUsers, onReply }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, typingUsers])

  const messageById = useMemo(
    () => new Map(messages.map((message) => [message.id, message])),
    [messages],
  )

  const participantMap = useMemo(
    () => new Map(participants.map((p) => [p.user_id, p])),
    [participants],
  )

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-y-auto bg-background/50 p-3">
      <div className="space-y-2 flex flex-col">
        {messages.map((message) => {
          const isMine = message.sender_id === myUserId
          const replyMessage = message.reply_to_id ? messageById.get(message.reply_to_id) : null
          const replySnippet = replyMessage?.content?.trim()
            ? replyMessage.content.replace(/\n+/g, ' ').slice(0, 90)
            : replyMessage?.file_url
              ? 'Tệp đính kèm'
              : 'Tin nhắn'
          const fileUrl = message.file_url ? resolveAssetUrl(message.file_url) : null
          const text = message.content?.trim() ?? ''

          const sender = participantMap.get(message.sender_id)

          return (
            <div
              key={message.id}
              className={cn('flex gap-2', isMine ? 'justify-end' : 'justify-start')}
            >
              {!isMine && (
                <div className="shrink-0 self-end mb-1">
                  <Avatar className="h-7 w-7 border bg-muted">
                    <AvatarImage src={sender?.avatar_url || ''} />
                    <AvatarFallback className="text-[10px] uppercase">
                      {sender?.display_name?.charAt(0) ?? '?'}
                    </AvatarFallback>
                  </Avatar>
                </div>
              )}
              <div className={cn("max-w-[75%] flex flex-col min-w-0", isMine ? "items-end" : "items-start")}>
                {!isMine && sender && (
                   <span className="text-[10px] text-muted-foreground px-1 mb-0.5 truncate max-w-full">
                     {sender.display_name}
                   </span>
                )}
                <div
                className={cn(
                  'w-fit max-w-full rounded-xl px-3 py-2 text-xs transition-opacity',
                  isMine ? 'chat-bubble-sent' : 'chat-bubble-received',
                  message.is_sending && 'opacity-60',
                )}
                onClick={() => onReply(message)}
              >
                {replyMessage ? (
                  <p className="mb-1 border-l-2 border-current/40 pl-2 text-[11px] opacity-80">
                    ↪ {replySnippet}
                  </p>
                ) : null}

                {fileUrl ? (
                  /\.(jpeg|jpg|gif|png|webp|bmp)($|\?)/i.test(fileUrl) ? (
                    <div className="mb-1 overflow-hidden rounded-lg border border-border/50 max-w-[240px]">
                      <a href={fileUrl} target="_blank" rel="noreferrer">
                        <img 
                          src={fileUrl} 
                          alt="Đính kèm" 
                          className="w-full h-auto object-cover max-h-[300px]" 
                          loading="lazy" 
                        />
                      </a>
                    </div>
                  ) : (
                    <a
                      href={fileUrl}
                      target="_blank"
                      rel="noreferrer"
                      className="mb-1 block underline underline-offset-4"
                    >
                      📎 Tệp đính kèm
                    </a>
                  )
                ) : null}

                {text ? (
                  <p className="break-words whitespace-pre-wrap">{text}</p>
                ) : null}

                <p className="mt-1 flex items-center gap-1 text-[10px] opacity-70">
                  {message.is_sending ? 'Đang gửi...' : formatTime(message.created_at)}
                  {message.is_edited && !message.is_sending ? ' • đã sửa' : ''}
                </p>
              </div>
            </div>
            </div>
          )
        })}

        {typingUsers.length > 0 ? (
          <div className="text-xs text-muted-foreground p-1 animate-pulse">Đang nhập...</div>
        ) : null}
        
        <div ref={bottomRef} className="h-1" />
      </div>
    </div>
  )
}
