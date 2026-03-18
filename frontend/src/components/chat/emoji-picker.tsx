import { Button } from '@/components/ui/button'

const EMOJIS = ['😀', '😂', '😍', '🥰', '😎', '🤝', '🔥', '🎉', '❤️', '👍', '👀', '🙏']

export function EmojiPicker({ onSelect }: { onSelect: (emoji: string) => void }) {
  return (
    <div className="grid grid-cols-6 gap-1 border border-border/60 bg-card p-2">
      {EMOJIS.map((emoji) => (
        <Button
          key={emoji}
          type="button"
          variant="ghost"
          size="icon-sm"
          onClick={() => onSelect(emoji)}
          className="text-base"
        >
          {emoji}
        </Button>
      ))}
    </div>
  )
}
