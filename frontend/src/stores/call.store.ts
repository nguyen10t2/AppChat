import { create } from 'zustand'
import { callService } from '@/services/call.service'
import type { Call, CallType, IncomingCall, WebRTCSignalPayload } from '@/types/call'
import { useAuthStore } from '@/stores/auth.store'

type CallState = {
  currentCall: Call | null
  incomingCall: IncomingCall | null
  isCallActive: boolean
  isCallIncoming: boolean
  localStream: MediaStream | null
  remoteStream: MediaStream | null
  isMuted: boolean
  isVideoEnabled: boolean
  isScreenSharing: boolean
  isMinimized: boolean
  lastSignaling: WebRTCSignalPayload | null

  initiateCall: (conversationId: string, callType: CallType) => Promise<void>
  acceptCall: (callId: string) => Promise<void>
  rejectCall: (callId: string, reason?: string) => Promise<void>
  endCall: (callId: string) => Promise<void>
  cancelCall: (callId: string) => Promise<void>

  toggleMute: () => void
  toggleVideo: () => void
  toggleScreenShare: () => void

  setIncomingCall: (call: IncomingCall | null) => void
  setCurrentCall: (call: Call | null) => void
  setLocalStream: (stream: MediaStream | null) => void
  setRemoteStream: (stream: MediaStream | null) => void
  setLastSignaling: (payload: WebRTCSignalPayload) => void
  clearLastSignaling: () => void
  setMinimized: (value: boolean) => void

  resetCallState: () => void
}

export const useCallStore = create<CallState>((set, get) => ({
  currentCall: null,
  incomingCall: null,
  isCallActive: false,
  isCallIncoming: false,
  localStream: null,
  remoteStream: null,
  isMuted: false,
  isVideoEnabled: true,
  isScreenSharing: false,
  isMinimized: false,
  lastSignaling: null,

  initiateCall: async (conversationId, callType) => {
    const me = useAuthStore.getState().user
    const res = await callService.initiate({
      conversation_id: conversationId,
      call_type: callType,
    })

    const now = new Date().toISOString()

    set({
      currentCall: {
        id: res.call_id,
        conversation_id: conversationId,
        initiator_id: me?.id ?? '',
        initiator_name: me?.display_name ?? me?.username ?? 'You',
        initiator_avatar: me?.avatar_url ?? null,
        call_type: callType,
        status: 'initiated',
        duration_seconds: null,
        started_at: null,
        ended_at: null,
        created_at: now,
      },
      isCallActive: true,
      incomingCall: null,
      isCallIncoming: false,
      isMinimized: false,
    })
  },

  acceptCall: async (callId) => {
    const incoming = get().incomingCall
    await callService.respond(callId, { accept: true })

    if (!incoming) return

    set({
      currentCall: {
        id: incoming.call_id,
        conversation_id: incoming.conversation_id,
        initiator_id: incoming.initiator_id,
        initiator_name: incoming.initiator_name,
        initiator_avatar: incoming.initiator_avatar,
        call_type: incoming.call_type,
        status: 'accepted',
        duration_seconds: null,
        started_at: new Date().toISOString(),
        ended_at: null,
        created_at: new Date().toISOString(),
      },
      isCallActive: true,
      incomingCall: null,
      isCallIncoming: false,
      isMinimized: false,
    })
  },

  rejectCall: async (callId, reason) => {
    await callService.respond(callId, { accept: false, reason })
    set({
      incomingCall: null,
      isCallIncoming: false,
    })
  },

  endCall: async (callId) => {
    await callService.end(callId)
    get().resetCallState()
  },

  cancelCall: async (callId) => {
    await callService.cancel(callId)
    get().resetCallState()
  },

  toggleMute: () => set((state) => ({ isMuted: !state.isMuted })),
  toggleVideo: () => set((state) => ({ isVideoEnabled: !state.isVideoEnabled })),
  toggleScreenShare: () => set((state) => ({ isScreenSharing: !state.isScreenSharing })),

  setIncomingCall: (call) =>
    set({
      incomingCall: call,
      isCallIncoming: Boolean(call),
    }),

  setCurrentCall: (call) =>
    set({
      currentCall: call,
      isCallActive: Boolean(call),
    }),

  setLocalStream: (stream) => set({ localStream: stream }),
  setRemoteStream: (stream) => set({ remoteStream: stream }),
  setLastSignaling: (payload) => set({ lastSignaling: payload }),
  clearLastSignaling: () => set({ lastSignaling: null }),
  setMinimized: (value) => set({ isMinimized: value }),

  resetCallState: () => {
    const local = get().localStream
    const remote = get().remoteStream

    local?.getTracks().forEach((track) => track.stop())
    remote?.getTracks().forEach((track) => track.stop())

    set({
      currentCall: null,
      incomingCall: null,
      isCallActive: false,
      isCallIncoming: false,
      localStream: null,
      remoteStream: null,
      isMuted: false,
      isVideoEnabled: true,
      isScreenSharing: false,
      isMinimized: false,
      lastSignaling: null,
    })
  },
}))
