import { Phone, PhoneOff, Maximize2, Video } from 'lucide-react'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { useCallStore } from '@/stores/call.store'

type MiniCallWindowProps = {
  onRestore: () => void
  onEndCall: () => void
}

export function MiniCallWindow({ onRestore, onEndCall }: MiniCallWindowProps) {
  const { currentCall } = useCallStore()

  if (!currentCall) return null

  const statusLabel =
    currentCall.status === 'initiated'
      ? 'Đang đổ chuông...'
      : currentCall.status === 'accepted'
        ? 'Đang trong cuộc gọi'
        : 'Đang kết nối...'

  return (
    <div className="fixed bottom-3 left-1/2 z-50 w-[calc(100vw-1rem)] max-w-sm -translate-x-1/2 rounded-2xl border border-border/70 bg-card/95 p-3 shadow-2xl md:bottom-4 md:left-auto md:right-4 md:w-48 md:max-w-none md:translate-x-0 md:rounded-3xl">
      <div className="flex items-center justify-between">
        <p className="truncate text-xs font-medium text-foreground">{currentCall.initiator_name}</p>
        <Button variant="ghost" size="icon" className="h-6 w-6 rounded-full" onClick={onRestore} title="Mở to cuộc gọi">
          <Maximize2 className="h-3.5 w-3.5" />
        </Button>
      </div>

      <div className="mt-3 flex flex-row items-center gap-3 md:mt-5 md:flex-col md:gap-2">
        <Avatar size="lg" className="h-12 w-12">
          <AvatarImage src={currentCall.initiator_avatar ?? undefined} alt={currentCall.initiator_name} />
          <AvatarFallback>{currentCall.initiator_name.slice(0, 1).toUpperCase()}</AvatarFallback>
        </Avatar>
        <div className="min-w-0 flex-1 md:flex-none md:text-center">
          <p className="truncate text-[11px] text-muted-foreground">{statusLabel}</p>
        </div>
        <div className="hidden md:block">
          {currentCall.call_type === 'video' ? (
            <Video className="h-3.5 w-3.5 text-muted-foreground" />
          ) : (
            <Phone className="h-3.5 w-3.5 text-muted-foreground" />
          )}
        </div>
      </div>

      <div className="mt-3 flex items-center justify-end gap-2 md:mt-4 md:justify-center">
        <Button variant="outline" size="icon" className="h-8 w-8 rounded-full" onClick={onRestore} title="Mở to cuộc gọi">
          <Maximize2 className="h-4 w-4" />
        </Button>
        <Button variant="destructive" size="icon" className="h-8 w-8 rounded-full" onClick={onEndCall} title="Kết thúc cuộc gọi">
          <PhoneOff className="h-4 w-4" />
        </Button>
      </div>
    </div>
  )
}
