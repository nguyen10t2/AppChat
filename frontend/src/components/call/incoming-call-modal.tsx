import { Phone, PhoneOff, Video } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { useCallStore } from '@/stores/call.store'

export function IncomingCallModal() {
  const { incomingCall, acceptCall, rejectCall } = useCallStore()

  if (!incomingCall) return null

  return (
    <div className="fixed bottom-3 left-1/2 z-50 w-[calc(100vw-1rem)] max-w-sm -translate-x-1/2 rounded-xl border border-border bg-card p-4 shadow-lg md:bottom-4 md:left-auto md:right-4 md:w-80 md:max-w-none md:translate-x-0">
      <div className="flex items-center gap-3">
        <Avatar size="lg">
          <AvatarImage src={incomingCall.initiator_avatar ?? undefined} alt={incomingCall.initiator_name} />
          <AvatarFallback>{incomingCall.initiator_name.slice(0, 1).toUpperCase()}</AvatarFallback>
        </Avatar>

        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-semibold text-foreground">{incomingCall.initiator_name}</p>
          <p className="text-xs text-muted-foreground">
            {incomingCall.call_type === 'video' ? 'Cuộc gọi video đến' : 'Cuộc gọi thoại đến'}
          </p>
        </div>

        {incomingCall.call_type === 'video' ? (
          <Video className="h-4 w-4 text-muted-foreground" />
        ) : (
          <Phone className="h-4 w-4 text-muted-foreground" />
        )}
      </div>

      <div className="mt-4 flex items-center justify-end gap-2">
        <Button
          variant="destructive"
          onClick={() => {
            void rejectCall(incomingCall.call_id, 'declined')
          }}
        >
          <PhoneOff className="h-4 w-4" />
          Từ chối
        </Button>
        <Button
          onClick={() => {
            void acceptCall(incomingCall.call_id)
          }}
        >
          <Phone className="h-4 w-4" />
          Nhận
        </Button>
      </div>
    </div>
  )
}
