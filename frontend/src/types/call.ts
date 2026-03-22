export type CallType = 'audio' | 'video'

export type CallStatus = 'initiated' | 'accepted' | 'rejected' | 'ended' | 'missed'

export type SignalingType = 'offer' | 'answer' | 'ice_candidate'

export type Call = {
  id: string
  conversation_id: string
  initiator_id: string
  initiator_name: string
  initiator_avatar: string | null
  call_type: CallType
  status: CallStatus
  duration_seconds: number | null
  started_at: string | null
  ended_at: string | null
  created_at: string
}

export type IncomingCall = {
  call_id: string
  conversation_id: string
  call_type: CallType
  initiator_id: string
  initiator_name: string
  initiator_avatar: string | null
}

export type CallHistoryPage = {
  calls: Call[]
  cursor: string | null
}

export type WebRTCSignalPayload = {
  call_id: string
  signaling_type: SignalingType
  sdp?: string
  candidate?: string
  sdp_mid?: string
  sdp_mline_index?: number
  sender_id: string
}
