import { useState } from 'react'
import {
  PaperPlaneRightIcon,
  PaperclipIcon,
  SmileyIcon,
  XIcon,
} from '@phosphor-icons/react'
import { Button } from '@/components/ui/button'
import { Textarea } from '@/components/ui/textarea'
import { EmojiPicker } from '@/components/chat/emoji-picker'
import type { Message } from '@/types/chat'

type Props = {
  disabled?: boolean
  replyTo: Message | null
  onCancelReply: () => void
  onSend: (payload: { content: string; file: File | null; replyTo: Message | null }) => Promise<void>
}

export function MessageComposer({ disabled, replyTo, onCancelReply, onSend }: Props) {
  const [message, setMessage] = useState('')
  const [showEmoji, setShowEmoji] = useState(false)
  const [file, setFile] = useState<File | null>(null)

  const submit = async () => {
    const content = message.trim()
    if ((!content && !file) || disabled) return

    // Xoá ngay lập tức input ở frontend để tạo cảm giác mượt mà (Optimistic UI)
    setMessage('')
    const currentFile = file
    setFile(null)
    onCancelReply()
    setShowEmoji(false)

    try {
      await onSend({ content, file: currentFile, replyTo })
    } catch {
      // Tuỳ chọn: Phục hồi lại tin nhắn nếu gửi lỗi văng exception
      setMessage(content)
      setFile(currentFile)
    }
  }

  return (
    <div className="space-y-2 border-t border-border/60 bg-card/50 p-3">
      {replyTo ? (
        <div className="flex items-center justify-between border border-border/60 bg-muted/40 px-2 py-1">
          <p className="truncate text-xs text-muted-foreground">
            Đang trả lời:{' '}
            {replyTo.content?.trim() || (replyTo.file_url ? 'Tệp đính kèm' : 'Tin nhắn')}
          </p>
          <Button variant="ghost" size="icon-xs" onClick={onCancelReply}>
            <XIcon />
          </Button>
        </div>
      ) : null}

      {file ? (
        <div className="flex items-center justify-between border border-border/60 bg-muted/40 px-2 py-1 text-xs">
          <span className="truncate">📎 {file.name}</span>
          <Button variant="ghost" size="icon-xs" onClick={() => setFile(null)}>
            <XIcon />
          </Button>
        </div>
      ) : null}

      {showEmoji ? (
        <EmojiPicker
          onSelect={(emoji) => {
            setMessage((prev) => `${prev}${emoji}`)
          }}
        />
      ) : null}

      <div className="flex items-end gap-2">
        <Textarea
          value={message}
          onChange={(event) => setMessage(event.target.value)}
          placeholder="Nhập tin nhắn..."
          className="min-h-[52px] resize-none"
          disabled={disabled}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && !event.shiftKey) {
              event.preventDefault()
              void submit()
            }
          }}
        />

        <Button
          type="button"
          variant="outline"
          size="icon"
          onClick={() => setShowEmoji((prev) => !prev)}
          disabled={disabled}
        >
          <SmileyIcon />
        </Button>

        <label>
          <input
            type="file"
            className="hidden"
            onChange={(event) => {
              const selected = event.target.files?.[0] ?? null
              setFile(selected)
            }}
            disabled={disabled}
          />
          <span className="inline-flex">
            <Button type="button" variant="outline" size="icon" asChild disabled={disabled}>
              <span>
                <PaperclipIcon />
              </span>
            </Button>
          </span>
        </label>

        <Button type="button" size="icon" onClick={() => void submit()} disabled={disabled}>
          <PaperPlaneRightIcon />
        </Button>
      </div>
    </div>
  )
}
