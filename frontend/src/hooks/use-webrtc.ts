import { useCallback, useEffect, useRef } from 'react'
import { wsClient } from '@/lib/ws'
import { useAuthStore } from '@/stores/auth.store'
import { useCallStore } from '@/stores/call.store'
import type { Call, WebRTCSignalPayload } from '@/types/call'

function buildRtcConfig(): RTCConfiguration {
  const iceServers: RTCIceServer[] = [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ]

  const turnUrlsRaw = import.meta.env.VITE_TURN_URLS as string | undefined
  const turnUsername = import.meta.env.VITE_TURN_USERNAME as string | undefined
  const turnCredential = import.meta.env.VITE_TURN_CREDENTIAL as string | undefined

  const turnUrls = turnUrlsRaw
    ?.split(',')
    .map((value) => value.trim())
    .filter(Boolean)

  if (turnUrls && turnUrls.length > 0) {
    iceServers.push({
      urls: turnUrls,
      username: turnUsername,
      credential: turnCredential,
    })
  }

  return { iceServers }
}

export function useWebRTC() {
  const peerRef = useRef<RTCPeerConnection | null>(null)
  const {
    localStream,
    setLocalStream,
    setRemoteStream,
    currentCall,
  } = useCallStore()
  const userId = useAuthStore((state) => state.user?.id)

  const sendSignal = useCallback(
    (callId: string, payload: Omit<WebRTCSignalPayload, 'call_id' | 'sender_id'>) => {
      if (!userId) return
      wsClient.send({
        type: 'call_signaling',
        call_id: callId,
        signaling_type: payload.signaling_type,
        sdp: payload.sdp,
        candidate: payload.candidate,
        sdp_mid: payload.sdp_mid,
        sdp_mline_index: payload.sdp_mline_index,
        sender_id: userId,
      })
    },
    [userId],
  )

  const ensurePeer = useCallback(
    (call: Call) => {
      if (peerRef.current) return peerRef.current

      const pc = new RTCPeerConnection(buildRtcConfig())

      pc.onconnectionstatechange = () => {
        console.debug('[webrtc] connectionState:', pc.connectionState)
      }

      pc.oniceconnectionstatechange = () => {
        console.debug('[webrtc] iceConnectionState:', pc.iceConnectionState)
      }

      pc.onicegatheringstatechange = () => {
        console.debug('[webrtc] iceGatheringState:', pc.iceGatheringState)
      }

      pc.onicecandidateerror = (event) => {
        console.warn('[webrtc] icecandidateerror', {
          url: event.url,
          errorCode: event.errorCode,
          errorText: event.errorText,
        })
      }

      pc.onicecandidate = (event) => {
        if (!event.candidate) return
        console.debug('[webrtc] local ICE candidate generated')
        sendSignal(call.id, {
          signaling_type: 'ice_candidate',
          candidate: event.candidate.candidate,
          sdp_mid: event.candidate.sdpMid ?? undefined,
          sdp_mline_index: event.candidate.sdpMLineIndex ?? undefined,
        })
      }

      pc.ontrack = (event) => {
        const [remote] = event.streams
        if (remote) {
          setRemoteStream(remote)
        }
      }

      peerRef.current = pc
      return pc
    },
    [sendSignal, setRemoteStream],
  )

  const ensureLocalMedia = useCallback(
    async (withVideo: boolean) => {
      if (localStream) return localStream

      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true,
        },
        video: withVideo
          ? {
              facingMode: 'user',
            }
          : false,
      })

      setLocalStream(stream)
      return stream
    },
    [localStream, setLocalStream],
  )

  const ensureTracksAdded = useCallback(
    (pc: RTCPeerConnection, stream: MediaStream) => {
      stream.getTracks().forEach((track) => {
        const exists = pc.getSenders().some((sender) => sender.track?.id === track.id)
        if (!exists) {
          pc.addTrack(track, stream)
        }
      })
    },
    [],
  )

  const createAndSendOffer = useCallback(
    async (call: Call) => {
      const stream = await ensureLocalMedia(call.call_type === 'video')
      const pc = ensurePeer(call)
      ensureTracksAdded(pc, stream)

      const offer = await pc.createOffer()
      await pc.setLocalDescription(offer)

      sendSignal(call.id, {
        signaling_type: 'offer',
        sdp: offer.sdp ?? undefined,
      })
    },
    [ensureLocalMedia, ensurePeer, ensureTracksAdded, sendSignal],
  )

  const handleOffer = useCallback(
    async (call: Call, signal: WebRTCSignalPayload) => {
      if (!signal.sdp) return

      const stream = await ensureLocalMedia(call.call_type === 'video')
      const pc = ensurePeer(call)
      ensureTracksAdded(pc, stream)

      await pc.setRemoteDescription({ type: 'offer', sdp: signal.sdp })
      const answer = await pc.createAnswer()
      await pc.setLocalDescription(answer)

      sendSignal(call.id, {
        signaling_type: 'answer',
        sdp: answer.sdp ?? undefined,
      })
    },
    [ensureLocalMedia, ensurePeer, ensureTracksAdded, sendSignal],
  )

  const handleAnswer = useCallback(async (signal: WebRTCSignalPayload) => {
    if (!signal.sdp || !peerRef.current) return
    console.debug('[webrtc] received answer')
    await peerRef.current.setRemoteDescription({ type: 'answer', sdp: signal.sdp })
  }, [])

  const handleIceCandidate = useCallback(async (signal: WebRTCSignalPayload) => {
    if (!signal.candidate || !peerRef.current) return
    console.debug('[webrtc] received remote ICE candidate')

    await peerRef.current.addIceCandidate({
      candidate: signal.candidate,
      sdpMid: signal.sdp_mid,
      sdpMLineIndex: signal.sdp_mline_index,
    })
  }, [])

  const closePeer = useCallback(() => {
    if (!peerRef.current) return
    peerRef.current.close()
    peerRef.current = null
    setRemoteStream(null)
  }, [setRemoteStream])

  useEffect(
    () => () => {
      closePeer()
    },
    [closePeer],
  )

  return {
    createAndSendOffer,
    ensureLocalMedia,
    handleOffer,
    handleAnswer,
    handleIceCandidate,
    closePeer,
    currentCall,
  }
}
