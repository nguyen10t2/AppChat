import { CallControls } from '@/components/call/call-controls'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Button } from '@/components/ui/button'
import { useCallStore } from '@/stores/call.store'
import { Minimize2 } from 'lucide-react'

type CallUIProps = {
  localVideoRef: (node: HTMLVideoElement | null) => void
  remoteVideoRef: (node: HTMLVideoElement | null) => void
  remoteAudioRef: (node: HTMLAudioElement | null) => void
  onEndCall: () => void
  onMinimize: () => void
}

export function CallUI({ localVideoRef, remoteVideoRef, remoteAudioRef, onEndCall, onMinimize }: CallUIProps) {
  const {
    currentCall,
    localStream,
    remoteStream,
    isMuted,
    isVideoEnabled,
    toggleMute,
    toggleVideo,
  } = useCallStore()

  if (!currentCall) return null

  const isVideoCall = currentCall.call_type === 'video'

  const callStatusLabel =
    currentCall.status === 'initiated'
      ? 'Đang đổ chuông...'
      : currentCall.status === 'accepted'
        ? isVideoCall
          ? remoteStream
            ? 'Đã kết nối'
            : 'Đang kết nối media...'
          : 'Đã kết nối thoại'
        : 'Đang kết nối...'

  return (
    <div className="fixed inset-0 z-50 bg-black/65 md:grid md:place-items-center md:p-4 md:backdrop-blur-sm">
      <div className="relative flex h-full w-full flex-col overflow-hidden bg-card/95 md:h-[80vh] md:max-w-4xl md:rounded-3xl md:border md:border-border/70 md:shadow-2xl">
        <audio ref={remoteAudioRef} autoPlay playsInline />

        <div className="flex items-center justify-between px-4 py-3 md:px-5 md:py-4">
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold text-foreground">{currentCall.initiator_name}</p>
            <p className="text-xs text-muted-foreground">{callStatusLabel}</p>
          </div>
          <Button
            variant="outline"
            size="icon"
            className="h-8 w-8 rounded-full"
            onClick={onMinimize}
            title="Thu nhỏ cuộc gọi"
          >
            <Minimize2 className="h-4 w-4" />
          </Button>
        </div>

        <div className="relative flex-1 overflow-hidden">
          {isVideoCall && remoteStream ? (
            <video
              ref={remoteVideoRef}
              autoPlay
              playsInline
              className="h-full w-full bg-black object-cover"
            />
          ) : (
            <div className="grid h-full place-items-center bg-background">
              <div className="flex flex-col items-center gap-3">
                <Avatar size="lg" className="h-24 w-24">
                  <AvatarImage src={currentCall.initiator_avatar ?? undefined} alt={currentCall.initiator_name} />
                  <AvatarFallback>{currentCall.initiator_name.slice(0, 1).toUpperCase()}</AvatarFallback>
                </Avatar>
                <p className="text-base font-semibold text-foreground">{currentCall.initiator_name}</p>
                <p className="text-xs text-muted-foreground">{callStatusLabel}</p>
              </div>
            </div>
          )}

          {isVideoCall && localStream && (
            <video
              ref={localVideoRef}
              autoPlay
              muted
              playsInline
              className="absolute bottom-3 right-3 h-32 w-24 rounded-xl border border-border bg-black object-cover md:bottom-4 md:right-4 md:h-40 md:w-28"
            />
          )}
        </div>

        <div className="px-4 py-3 md:py-4">
          <CallControls
            isMuted={isMuted}
            isVideoEnabled={isVideoEnabled}
            isVideoCall={isVideoCall}
            onToggleMute={toggleMute}
            onToggleVideo={toggleVideo}
            onEndCall={onEndCall}
          />
        </div>
      </div>
    </div>
  )
}
