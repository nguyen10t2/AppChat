import { Mic, MicOff, PhoneOff, Video, VideoOff } from 'lucide-react'
import { Button } from '@/components/ui/button'

type CallControlsProps = {
  isMuted: boolean
  isVideoEnabled: boolean
  isVideoCall: boolean
  onToggleMute: () => void
  onToggleVideo: () => void
  onEndCall: () => void
}

export function CallControls({
  isMuted,
  isVideoEnabled,
  isVideoCall,
  onToggleMute,
  onToggleVideo,
  onEndCall,
}: CallControlsProps) {
  return (
    <div className="flex items-center justify-center gap-3">
      <Button
        variant={isMuted ? 'destructive' : 'outline'}
        size="icon"
        className="h-10 w-10 rounded-full"
        onClick={onToggleMute}
      >
        {isMuted ? <MicOff className="h-4 w-4" /> : <Mic className="h-4 w-4" />}
      </Button>

      {isVideoCall && (
        <Button
          variant={!isVideoEnabled ? 'destructive' : 'outline'}
          size="icon"
          className="h-10 w-10 rounded-full"
          onClick={onToggleVideo}
        >
          {isVideoEnabled ? <Video className="h-4 w-4" /> : <VideoOff className="h-4 w-4" />}
        </Button>
      )}

      <Button variant="destructive" size="icon" className="h-10 w-10 rounded-full" onClick={onEndCall}>
        <PhoneOff className="h-4 w-4" />
      </Button>
    </div>
  )
}
