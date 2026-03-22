import { useEffect, useRef } from 'react'
import { useAuthStore } from '@/stores/auth.store'
import { useCallStore } from '@/stores/call.store'
import { useWebRTC } from '@/hooks/use-webrtc'
import { useCallSounds } from '@/hooks/use-call-sounds'
import { CallUI } from '@/components/call/call-ui'
import { IncomingCallModal } from '@/components/call/incoming-call-modal'
import { MiniCallWindow } from '@/components/call/mini-call-window'

export function CallLayer() {
  const {
    currentCall,
    incomingCall,
    lastSignaling,
    clearLastSignaling,
    isMuted,
    isVideoEnabled,
    localStream,
    remoteStream,
    isMinimized,
    setMinimized,
    endCall,
    cancelCall,
  } = useCallStore()

  const userId = useAuthStore((state) => state.user?.id)
  const offeredCallIdRef = useRef<string | null>(null)
  const prevCallStatusRef = useRef<string | null>(null)
  const prevHasCallRef = useRef(false)
  const localVideoRef = useRef<HTMLVideoElement | null>(null)
  const remoteVideoRef = useRef<HTMLVideoElement | null>(null)
  const remoteAudioRef = useRef<HTMLAudioElement | null>(null)
  const { startRingtone, stopRingtone, playAccepted, playEnded } = useCallSounds()

  const {
    createAndSendOffer,
    ensureLocalMedia,
    handleOffer,
    handleAnswer,
    handleIceCandidate,
    closePeer,
  } = useWebRTC()

  useEffect(() => {
    if (!currentCall) {
      offeredCallIdRef.current = null
      closePeer()
      return
    }

    void ensureLocalMedia(currentCall.call_type === 'video')
  }, [closePeer, currentCall, ensureLocalMedia])

  useEffect(() => {
    if (incomingCall) {
      startRingtone()
      return
    }

    stopRingtone()
  }, [incomingCall, startRingtone, stopRingtone])

  useEffect(() => {
    const currentStatus = currentCall?.status ?? null

    if (currentStatus === 'accepted' && prevCallStatusRef.current !== 'accepted') {
      playAccepted()
    }

    if (
      (currentStatus === 'ended' || currentStatus === 'rejected')
      && prevCallStatusRef.current !== currentStatus
    ) {
      playEnded()
    }

    if (prevHasCallRef.current && !currentCall) {
      playEnded()
    }

    prevCallStatusRef.current = currentStatus
    prevHasCallRef.current = Boolean(currentCall)
  }, [currentCall, playAccepted, playEnded])

  useEffect(() => {
    if (!currentCall || !userId) return
    if (currentCall.status !== 'accepted') return
    if (currentCall.initiator_id !== userId) return
    if (offeredCallIdRef.current === currentCall.id) return

    offeredCallIdRef.current = currentCall.id
    void createAndSendOffer(currentCall)
  }, [createAndSendOffer, currentCall, userId])

  useEffect(() => {
    if (!currentCall || !lastSignaling || !userId) return
    if (lastSignaling.call_id !== currentCall.id) return
    if (lastSignaling.sender_id === userId) {
      clearLastSignaling()
      return
    }

    if (lastSignaling.signaling_type === 'offer') {
      void handleOffer(currentCall, lastSignaling).finally(clearLastSignaling)
      return
    }

    if (lastSignaling.signaling_type === 'answer') {
      void handleAnswer(lastSignaling).finally(clearLastSignaling)
      return
    }

    void handleIceCandidate(lastSignaling).finally(clearLastSignaling)
  }, [
    clearLastSignaling,
    currentCall,
    handleAnswer,
    handleIceCandidate,
    handleOffer,
    lastSignaling,
    userId,
  ])

  useEffect(() => {
    if (!localStream) return
    localStream.getAudioTracks().forEach((track) => {
      track.enabled = !isMuted
    })
  }, [isMuted, localStream])

  useEffect(() => {
    if (!localStream) return
    localStream.getVideoTracks().forEach((track) => {
      track.enabled = isVideoEnabled
    })
  }, [isVideoEnabled, localStream])

  useEffect(() => {
    if (!localVideoRef.current || !localStream) return
    localVideoRef.current.srcObject = localStream
  }, [localStream])

  useEffect(() => {
    if (!remoteVideoRef.current || !remoteStream) return
    remoteVideoRef.current.srcObject = remoteStream
  }, [remoteStream])

  useEffect(() => {
    if (!remoteAudioRef.current || !remoteStream) return
    remoteAudioRef.current.srcObject = remoteStream

    remoteAudioRef.current.autoplay = true
    remoteAudioRef.current.muted = false
    remoteAudioRef.current.volume = 1

    void remoteAudioRef.current.play().catch(() => undefined)

    const tracks = remoteStream.getAudioTracks()
    const retryOnUnmute = () => {
      void remoteAudioRef.current?.play().catch(() => undefined)
    }

    tracks.forEach((track) => {
      track.addEventListener('unmute', retryOnUnmute)
    })

    return () => {
      tracks.forEach((track) => {
        track.removeEventListener('unmute', retryOnUnmute)
      })
    }
  }, [remoteStream])

  useEffect(() => {
    const isVideoCallActive = Boolean(currentCall && currentCall.call_type === 'video')
    if (!isVideoCallActive) return

    const orientation = screen.orientation as ScreenOrientation & {
      lock?: (
        orientation:
          | 'any'
          | 'natural'
          | 'landscape'
          | 'portrait'
          | 'portrait-primary'
          | 'portrait-secondary'
          | 'landscape-primary'
          | 'landscape-secondary',
      ) => Promise<void>
      unlock?: () => void
    }

    const canLock = typeof orientation?.lock === 'function'
    if (!canLock) return

    void orientation.lock?.('landscape').catch(() => undefined)

    return () => {
      if (typeof orientation?.unlock === 'function') {
        orientation.unlock()
      }
    }
  }, [currentCall])

  return (
    <>
      <IncomingCallModal />
      {currentCall && <audio ref={remoteAudioRef} autoPlay playsInline className="hidden" />}
      {currentCall && !isMinimized && (
        <CallUI
          localVideoRef={(node) => {
            localVideoRef.current = node
            if (node && localStream) {
              node.srcObject = localStream
            }
          }}
          remoteVideoRef={(node) => {
            remoteVideoRef.current = node
            if (node && remoteStream) {
              node.srcObject = remoteStream
            }
          }}
          onEndCall={() => {
            if (currentCall.status === 'initiated') {
              void cancelCall(currentCall.id)
              return
            }
            void endCall(currentCall.id)
          }}
          onMinimize={() => setMinimized(true)}
        />
      )}
      {currentCall && isMinimized && (
        <MiniCallWindow
          onRestore={() => setMinimized(false)}
          onEndCall={() => {
            if (currentCall.status === 'initiated') {
              void cancelCall(currentCall.id)
              return
            }
            void endCall(currentCall.id)
          }}
        />
      )}
    </>
  )
}
