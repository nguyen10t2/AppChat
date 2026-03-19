import { useMemo, useEffect, useRef } from 'react'
import { formatTime } from '@/lib/format'
import { cn } from '@/lib/utils'
import type { Message } from '@/types/chat'
import { resolveAssetUrl } from '@/lib/url'

type Props = {
  messages: Message[]
  myUserId: string
  typingUsers: string[]
  onReply: (message: Message) => void
}

export function MessagePane({ messages, myUserId, typingUsers, onReply }: Props) {
  const bottomRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages, typingUsers])

  const messageById = useMemo(
    () => new Map(messages.map((message) => [message.id, message])),
    [messages],
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

          return (
            <div
              key={message.id}
              className={cn('flex', isMine ? 'justify-end' : 'justify-start')}
            >
              <div
                className={cn(
                  'max-w-[75%] rounded-xl px-3 py-2 text-xs transition-opacity',
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
                  <a
                    href={fileUrl}
                    target="_blank"
                    rel="noreferrer"
                    className="mb-1 block underline underline-offset-4"
                  >
                    📎 Tệp đính kèm
                  </a>
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
