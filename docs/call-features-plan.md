# Kế Hoạch Xây Dựng Tính Năng Voice Call & Video Call
## AppChat Rewrite - Real-time Chat Application

---

## 📋 Tổng Quan

Tài liệu này cung cấp kế hoạch chi tiết để xây dựng tính năng **Voice Call (Gọi thoại)** và **Video Call (Gọi video)** cho AppChat, tương tự như Messenger và Zalo.

### Mục Tiêu
- Hỗ trợ gọi thoại 1-1 (Voice Call)
- Hỗ trợ gọi video 1-1 (Video Call)
- Chất lượng cuộc gọi tốt, độ trễ thấp (low latency)
- UI/UX thân thiện, tương đồng với Messenger/Zalo
- Tích hợp mượt mà với kiến trúc hiện có

---

## 🏗️ Kiến Trúc Tổng Thể

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend (React 19)                      │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Call UI        │  │  WebRTC Layer   │  │  Call Store     │ │
│  │  - Incoming     │  │  - RTCPeerConn  │  │  - Call State   │ │
│  │  - Ongoing      │  │  - MediaStream  │  │  - Signaling    │ │
│  │  - Controls     │  │  - ICE/STUN     │  │  - Local Stream │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ WebSocket (Signaling)
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Backend (Rust/Actix-Web)                    │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │  Call Service   │  │  WebSocket      │  │  Presence       │ │
│  │  - Initiate     │  │  Handler        │  │  Service        │ │
│  │  - Accept/End   │  │  - Signaling    │  │  - Online/Offline│ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┼────────────────────┐
         │                    │                    │
    ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
    │PostgreSQL│        │  Redis  │        │ STUN/TURN│
    │  - Calls │        │ - Cache │        │  Server  │
    │  - Parts │        │ - PubSub│        │  (Coturn)│
    └─────────┘         └─────────┘         └─────────┘
```

---

## 🗄️ Database Schema

### Bảng `calls`
Lưu trữ thông tin về các cuộc gọi

```sql
CREATE TYPE call_type AS ENUM ('audio', 'video');
CREATE TYPE call_status AS ENUM (
    'initiated',    -- Đã khởi tạo (chờ phản hồi)
    'accepted',     -- Đã chấp nhận
    'rejected',     -- Đã từ chối
    'ended',        -- Đã kết thúc
    'missed'        -- Bỏ lỡ
);

CREATE TABLE calls (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    initiator_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    _type call_type NOT NULL,
    status call_status NOT NULL DEFAULT 'initiated',
    started_at TIMESTAMP WITH TIME ZONE,
    ended_at TIMESTAMP WITH TIME ZONE,
    duration_seconds INTEGER,  -- Thời lượng cuộc gọi (giây)
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_calls_conversation ON calls(conversation_id);
CREATE INDEX idx_calls_initiator ON calls(initiator_id);
CREATE INDEX idx_calls_status ON calls(status);
CREATE INDEX idx_calls_created_at ON calls(created_at DESC);
```

### Bảng `call_participants`
Lưu trữ thông tin người tham gia cuộc gọi

```sql
CREATE TABLE call_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    call_id UUID NOT NULL REFERENCES calls(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE,
    left_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(call_id, user_id)
);

CREATE INDEX idx_call_participants_call ON call_participants(call_id);
CREATE INDEX idx_call_participants_user ON call_participants(user_id);
```

### Migration File: `0006_add_call_tables.sql`

---

## 🔌 WebSocket Signaling Protocol

### Message Types

Mở rộng enum `MessageType` trong `backend/src/modules/message/schema.rs`:

```rust
#[derive(Debug, PartialEq, Clone, Type, Serialize, Deserialize)]
#[sqlx(type_name = "message_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Text,
    Image,
    Video,
    File,
    System,
    CallRequest,       // Yêu cầu gọi
    CallAccept,        // Chấp nhận gọi
    CallReject,        // Từ chối gọi
    CallEnd,           // Kết thúc gọi
    CallCancel,        // Hủy gọi (người gọi)
    CallSignaling,     // WebRTC signaling (offer/answer/ice)
}
```

### Signaling Messages

#### 1. Call Request (Người gọi gửi)
```json
{
  "type": "call_request",
  "call_id": "uuid",
  "conversation_id": "uuid",
  "call_type": "audio" | "video",
  "initiator_id": "uuid",
  "initiator_name": "string",
  "initiator_avatar": "string | null"
}
```

#### 2. Call Accept (Người nhận gửi)
```json
{
  "type": "call_accept",
  "call_id": "uuid"
}
```

#### 3. Call Reject (Người nhận gửi)
```json
{
  "type": "call_reject",
  "call_id": "uuid",
  "reason": "busy" | "unavailable" | "declined"
}
```

#### 4. Call Cancel (Người gọi hủy)
```json
{
  "type": "call_cancel",
  "call_id": "uuid"
}
```

#### 5. Call End (Bất kỳ khi nào kết thúc)
```json
{
  "type": "call_end",
  "call_id": "uuid",
  "duration_seconds": 120,
  "ended_by": "uuid"
}
```

#### 6. WebRTC Signaling Messages

##### Offer (SDP Offer)
```json
{
  "type": "call_signaling",
  "call_id": "uuid",
  "signaling_type": "offer",
  "sdp": "v=0\r\no=- 123456 2 IN IP4 127.0.0.1\r\n...",
  "sender_id": "uuid"
}
```

##### Answer (SDP Answer)
```json
{
  "type": "call_signaling",
  "call_id": "uuid",
  "signaling_type": "answer",
  "sdp": "v=0\r\no=- 654321 2 IN IP4 127.0.0.1\r\n...",
  "sender_id": "uuid"
}
```

##### ICE Candidate
```json
{
  "type": "call_signaling",
  "call_id": "uuid",
  "signaling_type": "ice_candidate",
  "candidate": "candidate:1 1 UDP 2130706431 192.168.1.100 54400 typ host",
  "sdp_mid": "0",
  "sdp_mline_index": 0,
  "sender_id": "uuid"
}
```

---

## 🦀 Backend Implementation

### 1. Module Structure

```
backend/src/modules/call/
├── mod.rs                    # Module exports
├── model.rs                  # Request/Response models
├── schema.rs                 # Database entities
├── repository.rs             # Repository trait
├── repository_pg.rs          # PostgreSQL implementation
├── service.rs                # Business logic
├── handler.rs                # HTTP handlers
└── route.rs                  # Route configuration
```

### 2. Schema (`schema.rs`)

```rust
use serde::{Deserialize, Serialize};
use sqlx::prelude::{FromRow, Type};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "call_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CallType {
    Audio,
    Video,
}

#[derive(Debug, Clone, Type, Serialize, Deserialize, PartialEq)]
#[sqlx(type_name = "call_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Initiated,
    Accepted,
    Rejected,
    Ended,
    Missed,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CallEntity {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiator_id: Uuid,
    #[sqlx(rename = "type")]
    pub _type: CallType,
    pub status: CallStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_seconds: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CallParticipantEntity {
    pub id: Uuid,
    pub call_id: Uuid,
    pub user_id: Uuid,
    pub joined_at: Option<DateTime<Utc>>,
    pub left_at: Option<DateTime<Utc>>,
}
```

### 3. Models (`model.rs`)

```rust
use super::schema::{CallType, CallStatus};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct InitiateCallRequest {
    pub conversation_id: Uuid,
    pub call_type: CallType,
}

#[derive(Debug, Clone, Serialize)]
pub struct InitiateCallResponse {
    pub call_id: Uuid,
    pub status: CallStatus,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RespondCallRequest {
    pub call_id: Uuid,
    pub accept: bool,
    #[serde(default)]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EndCallRequest {
    pub call_id: Uuid,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallHistoryResponse {
    pub calls: Vec<CallWithDetails>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CallWithDetails {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub initiator_id: Uuid,
    pub initiator_name: String,
    pub initiator_avatar: Option<String>,
    pub call_type: CallType,
    pub status: CallStatus,
    pub duration_seconds: Option<i32>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
    pub created_at: String,
}
```

### 4. Service (`service.rs`)

```rust
use std::sync::Arc;
use uuid::Uuid;
use crate::modules::call::{
    schema::{CallEntity, CallType, CallStatus},
    model::*,
    repository::{CallRepository, CallParticipantRepository},
};
use crate::modules::websocket::server::WebSocketServer;

pub struct CallService {
    call_repo: Arc<dyn CallRepository>,
    participant_repo: Arc<dyn CallParticipantRepository>,
    ws_server: Arc<WebSocketServer>,
}

impl CallService {
    pub fn with_dependencies(
        call_repo: Arc<dyn CallRepository>,
        participant_repo: Arc<dyn CallParticipantRepository>,
        ws_server: Arc<WebSocketServer>,
    ) -> Self {
        Self {
            call_repo,
            participant_repo,
            ws_server,
        }
    }

    /// Khởi tạo cuộc gọi mới
    pub async fn initiate_call(
        &self,
        user_id: Uuid,
        request: InitiateCallRequest,
    ) -> Result<InitiateCallResponse, anyhow::Error> {
        // Tạo call record
        let call_id = self.call_repo.create_call(
            user_id,
            request.conversation_id,
            request.call_type,
        ).await?;

        // Gửi signaling message qua WebSocket
        // TODO: Implement

        Ok(InitiateCallResponse {
            call_id,
            status: CallStatus::Initiated,
        })
    }

    /// Phản hồi cuộc gọi (accept/reject)
    pub async fn respond_call(
        &self,
        user_id: Uuid,
        request: RespondCallRequest,
    ) -> Result<(), anyhow::Error> {
        if request.accept {
            // Update status to accepted
            self.call_repo.update_call_status(
                request.call_id,
                CallStatus::Accepted,
            ).await?;
            
            // Add participant
            self.participant_repo.add_participant(
                request.call_id,
                user_id,
            ).await?;

            // Send signaling message
            // TODO: Implement
        } else {
            // Update status to rejected
            self.call_repo.update_call_status(
                request.call_id,
                CallStatus::Rejected,
            ).await?;

            // Send reject message
            // TODO: Implement
        }

        Ok(())
    }

    /// Kết thúc cuộc gọi
    pub async fn end_call(
        &self,
        user_id: Uuid,
        request: EndCallRequest,
    ) -> Result<(), anyhow::Error> {
        // Calculate duration
        let duration = self.call_repo.calculate_duration(
            request.call_id,
        ).await?;

        // Update call
        self.call_repo.end_call(
            request.call_id,
            user_id,
            duration,
        ).await?;

        // Send end message
        // TODO: Implement

        Ok(())
    }

    /// Lấy lịch sử cuộc gọi
    pub async fn get_call_history(
        &self,
        user_id: Uuid,
        limit: i64,
        cursor: Option<DateTime<Utc>>,
    ) -> Result<CallHistoryResponse, anyhow::Error> {
        let calls = self.call_repo.get_user_calls(
            user_id,
            limit,
            cursor,
        ).await?;

        Ok(CallHistoryResponse {
            calls,
            cursor: None, // TODO: Implement cursor
        })
    }
}
```

### 5. Route (`route.rs`)

```rust
use actix_web::{web, Scope};

use super::handler::*;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/calls")
            .route("", web::post().to(initiate_call))
            .route("/{call_id}/respond", web::post().to(respond_call))
            .route("/{call_id}/end", web::post().to(end_call))
            .route("/history", web::get().to(get_call_history)),
    );
}
```

### 6. WebSocket Integration

Mở rộng `backend/src/modules/websocket/message.rs`:

```rust
#[derive(Debug, Deserialize)]
pub enum CallSignalingType {
    #[serde(rename = "offer")]
    Offer,
    #[serde(rename = "answer")]
    Answer,
    #[serde(rename = "ice_candidate")]
    IceCandidate,
}

#[derive(Debug, Deserialize)]
pub struct CallSignalingMessage {
    pub call_id: Uuid,
    pub signaling_type: CallSignalingType,
    pub sdp: Option<String>,
    pub candidate: Option<String>,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u16>,
    pub sender_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CallRequestMessage {
    pub call_id: Uuid,
    pub conversation_id: Uuid,
    pub call_type: String,  // "audio" or "video"
    pub initiator_id: Uuid,
    pub initiator_name: String,
    pub initiator_avatar: Option<String>,
}
```

---

## 🌐 Frontend Implementation

### 1. Component Structure

```
frontend/src/components/call/
├── incoming-call-modal.tsx      # Modal nhận cuộc gọi
├── call-ui.tsx                  # UI cuộc gọi chính
├── call-controls.tsx            # Controls (mute, camera, end)
├── mini-call-window.tsx         # Cửa sổ mini khi minimize
├── call-quality-indicator.tsx   # Hiển thị chất lượng
└── avatar-ring.tsx              # Avatar với ring animation
```

### 2. Types (`types/call.ts`)

```typescript
export type CallType = 'audio' | 'video';
export type CallStatus = 
  | 'initiated' 
  | 'accepted' 
  | 'rejected' 
  | 'ended' 
  | 'missed';

export type SignalingType = 'offer' | 'answer' | 'ice_candidate';

export interface Call {
  id: string;
  conversation_id: string;
  initiator_id: string;
  initiator_name: string;
  initiator_avatar: string | null;
  call_type: CallType;
  status: CallStatus;
  duration_seconds: number | null;
  started_at: string | null;
  ended_at: string | null;
  created_at: string;
}

export interface IncomingCall {
  call_id: string;
  conversation_id: string;
  call_type: CallType;
  initiator_id: string;
  initiator_name: string;
  initiator_avatar: string | null;
}

export interface WebRTCConfig {
  iceServers: RTCIceServer[];
}
```

### 3. Store (`stores/call.store.ts`)

```typescript
import { create } from 'zustand';
import { Call, CallType, IncomingCall, LocalMediaStream } from '@/types/call';

interface CallState {
  // Current call state
  currentCall: Call | null;
  incomingCall: IncomingCall | null;
  isCallActive: boolean;
  isCallIncoming: boolean;
  
  // Media streams
  localStream: MediaStream | null;
  remoteStream: MediaStream | null;
  
  // Call controls
  isMuted: boolean;
  isVideoEnabled: boolean;
  isScreenSharing: boolean;
  
  // Actions
  initiateCall: (conversationId: string, callType: CallType) => Promise<void>;
  acceptCall: (callId: string) => Promise<void>;
  rejectCall: (callId: string, reason?: string) => Promise<void>;
  endCall: (callId: string) => Promise<void>;
  cancelCall: (callId: string) => Promise<void>;
  
  toggleMute: () => void;
  toggleVideo: () => void;
  toggleScreenShare: () => void;
  
  setIncomingCall: (call: IncomingCall | null) => void;
  setCurrentCall: (call: Call | null) => void;
  setLocalStream: (stream: MediaStream | null) => void;
  setRemoteStream: (stream: MediaStream | null) => void;
  
  resetCallState: () => void;
}

export const useCallStore = create<CallState>((set, get) => ({
  // Initial state
  currentCall: null,
  incomingCall: null,
  isCallActive: false,
  isCallIncoming: false,
  
  localStream: null,
  remoteStream: null,
  
  isMuted: false,
  isVideoEnabled: true,
  isScreenSharing: false,
  
  // Actions
  initiateCall: async (conversationId, callType) => {
    // TODO: Implement
  },
  
  acceptCall: async (callId) => {
    // TODO: Implement
  },
  
  rejectCall: async (callId, reason) => {
    // TODO: Implement
  },
  
  endCall: async (callId) => {
    // TODO: Implement
  },
  
  cancelCall: async (callId) => {
    // TODO: Implement
  },
  
  toggleMute: () => set((state) => ({ isMuted: !state.isMuted })),
  toggleVideo: () => set((state) => ({ isVideoEnabled: !state.isVideoEnabled })),
  toggleScreenShare: () => set((state) => ({ isScreenSharing: !state.isScreenSharing })),
  
  setIncomingCall: (call) => set({ incomingCall: call, isCallIncoming: !!call }),
  setCurrentCall: (call) => set({ currentCall: call, isCallActive: !!call }),
  setLocalStream: (stream) => set({ localStream: stream }),
  setRemoteStream: (stream) => set({ remoteStream: stream }),
  
  resetCallState: () => set({
    currentCall: null,
    incomingCall: null,
    isCallActive: false,
    isCallIncoming: false,
    localStream: null,
    remoteStream: null,
    isMuted: false,
    isVideoEnabled: true,
    isScreenSharing: false,
  }),
}));
```

### 4. WebRTC Hook (`hooks/use-webrtc.ts`)

```typescript
import { useEffect, useRef, useCallback } from 'react';
import { useCallStore } from '@/stores/call.store';
import { WebRTCConfig } from '@/types/call';

const DEFAULT_RTC_CONFIG: WebRTCConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
    // TODO: Add TURN server
  ],
};

export function useWebRTC() {
  const peerConnectionRef = useRef<RTCPeerConnection | null>(null);
  const {
    localStream,
    remoteStream,
    setLocalStream,
    setRemoteStream,
    currentCall,
  } = useCallStore();

  // Initialize peer connection
  const initPeerConnection = useCallback(() => {
    if (peerConnectionRef.current) {
      return peerConnectionRef.current;
    }

    const pc = new RTCPeerConnection(DEFAULT_RTC_CONFIG);
    
    // Handle ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate && currentCall) {
        // Send ICE candidate via WebSocket
        // TODO: Implement
      }
    };
    
    // Handle remote stream
    pc.ontrack = (event) => {
      setRemoteStream(event.streams[0]);
    };
    
    // Handle connection state changes
    pc.onconnectionstatechange = () => {
      console.log('Connection state:', pc.connectionState);
    };
    
    peerConnectionRef.current = pc;
    return pc;
  }, [currentCall, setRemoteStream]);

  // Get local media stream
  const getLocalStream = useCallback(async (withVideo: boolean = true) => {
    try {
      const constraints: MediaStreamConstraints = {
        audio: true,
        video: withVideo,
      };
      
      const stream = await navigator.mediaDevices.getUserMedia(constraints);
      setLocalStream(stream);
      return stream;
    } catch (error) {
      console.error('Error getting local stream:', error);
      throw error;
    }
  }, [setLocalStream]);

  // Create offer
  const createOffer = useCallback(async () => {
    const pc = initPeerConnection();
    
    if (localStream) {
      localStream.getTracks().forEach(track => {
        pc.addTrack(track, localStream);
      });
    }
    
    const offer = await pc.createOffer();
    await pc.setLocalDescription(offer);
    
    return offer;
  }, [initPeerConnection, localStream]);

  // Create answer
  const createAnswer = useCallback(async (offer: RTCSessionDescriptionInit) => {
    const pc = initPeerConnection();
    
    await pc.setRemoteDescription(new RTCSessionDescription(offer));
    
    if (localStream) {
      localStream.getTracks().forEach(track => {
        pc.addTrack(track, localStream);
      });
    }
    
    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);
    
    return answer;
  }, [initPeerConnection, localStream]);

  // Set remote answer
  const setRemoteAnswer = useCallback(async (answer: RTCSessionDescriptionInit) => {
    const pc = peerConnectionRef.current;
    if (pc) {
      await pc.setRemoteDescription(new RTCSessionDescription(answer));
    }
  }, []);

  // Add ICE candidate
  const addIceCandidate = useCallback(async (candidate: RTCIceCandidateInit) => {
    const pc = peerConnectionRef.current;
    if (pc) {
      await pc.addIceCandidate(new RTCIceCandidate(candidate));
    }
  }, []);

  // Cleanup
  useEffect(() => {
    return () => {
      if (peerConnectionRef.current) {
        peerConnectionRef.current.close();
        peerConnectionRef.current = null;
      }
    };
  }, []);

  return {
    peerConnection: peerConnectionRef.current,
    getLocalStream,
    createOffer,
    createAnswer,
    setRemoteAnswer,
    addIceCandidate,
  };
}
```

### 5. Incoming Call Modal (`incoming-call-modal.tsx`)

```typescript
import { motion, AnimatePresence } from 'framer-motion';
import { Phone, Video, X } from 'lucide-react';
import { useCallStore } from '@/stores/call.store';
import { Button } from '@/components/ui/button';
import { Avatar } from '@/components/ui/avatar';
import { useTranslation } from 'react-i18next';

export function IncomingCallModal() {
  const { incomingCall, acceptCall, rejectCall } = useCallStore();
  const { t } = useTranslation();

  if (!incomingCall) return null;

  const isVideo = incomingCall.call_type === 'video';

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0, y: 50 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 50 }}
        className="fixed bottom-4 right-4 z-50"
      >
        <div className="bg-gradient-to-br from-purple-600 to-blue-600 rounded-2xl p-6 shadow-2xl w-80">
          {/* Avatar */}
          <div className="flex flex-col items-center mb-6">
            <div className="relative mb-4">
              <Avatar className="w-24 h-24 border-4 border-white/30">
                {incomingCall.initiator_avatar ? (
                  <img src={incomingCall.initiator_avatar} alt={incomingCall.initiator_name} />
                ) : (
                  <div className="w-full h-full bg-white/20 flex items-center justify-center">
                    <span className="text-3xl font-semibold text-white">
                      {incomingCall.initiator_name[0].toUpperCase()}
                    </span>
                  </div>
                )}
              </Avatar>
              <div className="absolute -bottom-2 left-1/2 transform -translate-x-1/2 bg-white/20 rounded-full px-3 py-1 flex items-center gap-1">
                {isVideo ? (
                  <Video className="w-4 h-4 text-white" />
                ) : (
                  <Phone className="w-4 h-4 text-white" />
                )}
                <span className="text-sm font-medium text-white">
                  {isVideo ? t('Video Call') : t('Voice Call')}
                </span>
              </div>
            </div>
            <h3 className="text-xl font-semibold text-white mb-1">
              {incomingCall.initiator_name}
            </h3>
            <p className="text-white/80 text-sm">
              {t('is calling you...')}
            </p>
          </div>

          {/* Action Buttons */}
          <div className="flex items-center justify-center gap-4">
            <Button
              size="lg"
              variant="destructive"
              className="rounded-full w-14 h-14 flex items-center justify-center"
              onClick={() => rejectCall(incomingCall.call_id)}
            >
              <X className="w-6 h-6" />
            </Button>
            <Button
              size="lg"
              className={`rounded-full w-14 h-14 flex items-center justify-center ${
                isVideo
                  ? 'bg-gradient-to-br from-green-500 to-emerald-600 hover:from-green-600 hover:to-emerald-700'
                  : 'bg-gradient-to-br from-blue-500 to-indigo-600 hover:from-blue-600 hover:to-indigo-700'
              }`}
              onClick={() => acceptCall(incomingCall.call_id)}
            >
              {isVideo ? (
                <Video className="w-6 h-6" />
              ) : (
                <Phone className="w-6 h-6" />
              )}
            </Button>
          </div>
        </div>
      </motion.div>
    </AnimatePresence>
  );
}
```

### 6. Call UI (`call-ui.tsx`)

```typescript
import { motion } from 'framer-motion';
import { Phone, Mic, MicOff, Video, VideoOff, Monitor, MonitorOff } from 'lucide-react';
import { useCallStore } from '@/stores/call.store';
import { CallControls } from './call-controls';
import { CallQualityIndicator } from './call-quality-indicator';

export function CallUI() {
  const {
    currentCall,
    localStream,
    remoteStream,
    isMuted,
    isVideoEnabled,
    isScreenSharing,
    toggleMute,
    toggleVideo,
    toggleScreenShare,
    endCall,
  } = useCallStore();

  if (!currentCall) return null;

  const isVideoCall = currentCall.call_type === 'video';

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      className="fixed inset-0 bg-black z-50 flex flex-col"
    >
      {/* Main Video Area */}
      <div className="flex-1 relative overflow-hidden">
        {/* Remote Stream (Full screen) */}
        {remoteStream && isVideoCall && (
          <video
            ref={(ref) => {
              if (ref && remoteStream) {
                ref.srcObject = remoteStream;
              }
            }}
            autoPlay
            playsInline
            className="w-full h-full object-cover"
          />
        )}

        {/* Local Stream (Picture-in-Picture) */}
        {localStream && isVideoCall && (
          <motion.div
            initial={{ scale: 0, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            className="absolute bottom-24 right-4 w-48 h-64 bg-gray-900 rounded-2xl overflow-hidden shadow-2xl border-2 border-white/20"
          >
            <video
              ref={(ref) => {
                if (ref && localStream) {
                  ref.srcObject = localStream;
                  ref.muted = true;
                }
              }}
              autoPlay
              playsInline
              className="w-full h-full object-cover"
            />
          </motion.div>
        )}

        {/* Audio Call - Avatar Display */}
        {!isVideoCall && (
          <div className="flex items-center justify-center h-full">
            <div className="text-center">
              <div className="w-48 h-48 mx-auto mb-6 rounded-full bg-gradient-to-br from-purple-500 to-blue-500 flex items-center justify-center shadow-2xl">
                <span className="text-6xl font-bold text-white">
                  {currentCall.initiator_name[0].toUpperCase()}
                </span>
              </div>
              <h2 className="text-3xl font-bold text-white mb-2">
                {currentCall.initiator_name}
              </h2>
              <p className="text-white/60 text-lg">
                {t(isVideoCall ? 'Video Call' : 'Voice Call')}
              </p>
            </div>
          </div>
        )}

        {/* Quality Indicator */}
        <div className="absolute top-4 left-4">
          <CallQualityIndicator />
        </div>
      </div>

      {/* Call Controls */}
      <div className="h-24 bg-gradient-to-t from-black/80 to-transparent flex items-center justify-center pb-6">
        <CallControls
          isMuted={isMuted}
          isVideoEnabled={isVideoEnabled}
          isScreenSharing={isScreenSharing}
          isVideoCall={isVideoCall}
          onToggleMute={toggleMute}
          onToggleVideo={toggleVideo}
          onToggleScreenShare={toggleScreenShare}
          onEndCall={() => endCall(currentCall.id)}
        />
      </div>
    </motion.div>
  );
}
```

### 7. Call Controls (`call-controls.tsx`)

```typescript
import { motion } from 'framer-motion';
import { Phone, PhoneOff, Mic, MicOff, Video, VideoOff, Monitor, MonitorOff } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface CallControlsProps {
  isMuted: boolean;
  isVideoEnabled: boolean;
  isScreenSharing: boolean;
  isVideoCall: boolean;
  onToggleMute: () => void;
  onToggleVideo: () => void;
  onToggleScreenShare: () => void;
  onEndCall: () => void;
}

export function CallControls({
  isMuted,
  isVideoEnabled,
  isScreenSharing,
  isVideoCall,
  onToggleMute,
  onToggleVideo,
  onToggleScreenShare,
  onEndCall,
}: CallControlsProps) {
  return (
    <div className="flex items-center gap-4">
      {/* Mute Toggle */}
      <motion.button
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={onToggleMute}
        className={`w-14 h-14 rounded-full flex items-center justify-center transition-colors ${
          isMuted
            ? 'bg-red-500 hover:bg-red-600'
            : 'bg-white/20 hover:bg-white/30 backdrop-blur-sm'
        }`}
      >
        {isMuted ? (
          <MicOff className="w-6 h-6 text-white" />
        ) : (
          <Mic className="w-6 h-6 text-white" />
        )}
      </motion.button>

      {/* Video Toggle (Only for video calls) */}
      {isVideoCall && (
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={onToggleVideo}
          className={`w-14 h-14 rounded-full flex items-center justify-center transition-colors ${
            !isVideoEnabled
              ? 'bg-red-500 hover:bg-red-600'
              : 'bg-white/20 hover:bg-white/30 backdrop-blur-sm'
          }`}
        >
          {isVideoEnabled ? (
            <Video className="w-6 h-6 text-white" />
          ) : (
            <VideoOff className="w-6 h-6 text-white" />
          )}
        </motion.button>
      )}

      {/* Screen Share Toggle (Only for video calls) */}
      {isVideoCall && (
        <motion.button
          whileHover={{ scale: 1.1 }}
          whileTap={{ scale: 0.9 }}
          onClick={onToggleScreenShare}
          className={`w-14 h-14 rounded-full flex items-center justify-center transition-colors ${
            isScreenSharing
              ? 'bg-green-500 hover:bg-green-600'
              : 'bg-white/20 hover:bg-white/30 backdrop-blur-sm'
          }`}
        >
          {isScreenSharing ? (
            <Monitor className="w-6 h-6 text-white" />
          ) : (
            <MonitorOff className="w-6 h-6 text-white" />
          )}
        </motion.button>
      )}

      {/* End Call */}
      <motion.button
        whileHover={{ scale: 1.1 }}
        whileTap={{ scale: 0.9 }}
        onClick={onEndCall}
        className="w-16 h-16 rounded-full bg-red-500 hover:bg-red-600 flex items-center justify-center shadow-lg"
      >
        <PhoneOff className="w-7 h-7 text-white" />
      </motion.button>
    </div>
  );
}
```

### 8. Call Service (`services/call.service.ts`)

```typescript
import { http } from '@/lib/http';
import { Call, CallType } from '@/types/call';

export interface InitiateCallRequest {
  conversation_id: string;
  call_type: CallType;
}

export interface InitiateCallResponse {
  call_id: string;
  status: string;
}

export interface RespondCallRequest {
  call_id: string;
  accept: boolean;
  reason?: string;
}

export interface EndCallRequest {
  call_id: string;
}

export class CallService {
  static async initiateCall(request: InitiateCallRequest): Promise<InitiateCallResponse> {
    const response = await http.post<InitiateCallResponse>('/calls', request);
    return response.data;
  }

  static async respondCall(request: RespondCallRequest): Promise<void> {
    await http.post(`/calls/${request.call_id}/respond`, request);
  }

  static async endCall(request: EndCallRequest): Promise<void> {
    await http.post(`/calls/${request.call_id}/end`, request);
  }

  static async cancelCall(callId: string): Promise<void> {
    await http.post(`/calls/${callId}/cancel`, {});
  }

  static async getCallHistory(limit: number = 20): Promise<Call[]> {
    const response = await http.get<{ calls: Call[] }>('/calls/history', {
      params: { limit },
    });
    return response.data.calls;
  }
}
```

---

## 🔧 STUN/TURN Server Setup

### Option 1: Google STUN (Free - Limited)
```javascript
const rtcConfig = {
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' },
  ],
};
```

### Option 2: Self-hosted TURN Server (Coturn)
Recommended for production environments.

#### Installation (Docker)
```yaml
# docker-compose.yml
services:
  turn-server:
    image: instrumentisto/coturn:latest
    ports:
      - "3478:3478/tcp"
      - "3478:3478/udp"
      - "5349:5349/tcp"
      - "5349:5349/udp"
      - "49152-65535:49152-65535/udp"
    environment:
      - TURN_PORT=3478
      - TURNS_PORT=5349
      - TURN_SERVER_PUBLIC_IP=your-public-ip
      - TURN_SERVER_REALM=your-domain.com
      - TURN_USER=username:password
      - TURN_SECRET=your-secret-key
    volumes:
      - ./turn-server.conf:/etc/coturn/turnserver.conf
```

#### Configuration (`turnserver.conf`)
```
listening-port=3478
tls-listening-port=5349
fingerprint
lt-cred-mech
user=username:password
realm=your-domain.com
external-ip=your-public-ip
```

---

## 📅 Implementation Phases

### Phase 1: Foundation (Week 1-2)
- [ ] Database migration for call tables
- [ ] Backend Call module implementation
- [ ] Basic API endpoints (initiate, respond, end)
- [ ] WebSocket signaling message structure
- [ ] Frontend types and store setup
- [ ] Basic UI components

### Phase 2: WebRTC Integration (Week 2-3)
- [ ] STUN/TURN server setup
- [ ] WebRTC hook implementation
- [ ] Peer connection management
- [ ] Local/remote stream handling
- [ ] ICE candidate exchange
- [ ] SDP offer/answer exchange

### Phase 3: UI Implementation (Week 3-4)
- [ ] Incoming call modal
- [ ] Call UI (audio & video)
- [ ] Call controls
- [ ] Mini call window
- [ ] Call quality indicator
- [ ] Sound notifications

### Phase 4: Integration & Testing (Week 4-5)
- [ ] Full flow testing (initiate → accept → call → end)
- [ ] Edge cases handling
- [ ] Error handling
- [ ] Performance optimization
- [ ] Cross-browser testing

### Phase 5: Polish & Features (Week 5-6)
- [ ] Screen sharing
- [ ] Call history
- [ ] Advanced UI animations
- [ ] Accessibility improvements
- [ ] Documentation
- [ ] Deployment

---

## 🧪 Testing Strategy

### Backend Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;
    
    #[sqlx::test]
    async fn test_create_call(pool: PgPool) {
        let repo = CallPgRepository::new(pool);
        let call_id = repo.create_call(
            user_id,
            conversation_id,
            CallType::Audio,
        ).await.unwrap();
        
        assert!(call_id != Uuid::nil());
    }
}
```

### Frontend Tests
```typescript
describe('CallStore', () => {
  it('should initiate call', async () => {
    const store = useCallStore.getState();
    await store.initiateCall('conv-123', 'audio');
    expect(store.currentCall).toBeDefined();
  });
});
```

### E2E Tests (Playwright)
```typescript
test('voice call flow', async ({ page }) => {
  // User A initiates call
  await page.goto('/chat');
  await page.click('[data-testid="call-button"]');
  
  // User B accepts call
  const userBPage = await context.newPage();
  await userBPage.goto('/chat');
  await userBPage.click('[data-testid="accept-call"]');
  
  // Verify call is active
  await expect(page.locator('[data-testid="call-ui"]')).toBeVisible();
});
```

---

## 🚀 Deployment Considerations

### Production Setup
1. **TURN Server**: Must be publicly accessible with proper DNS
2. **Certificate**: Valid SSL certificate for TURN/TLS
3. **Bandwidth**: Minimum 1 Mbps per concurrent call (video), 64 Kbps (audio)
4. **Scaling**: Consider MediaSoup for group calls (>2 participants)
5. **Monitoring**: Track call quality, duration, and failures

### Environment Variables
```env
# Backend
TURN_SERVER_URL=turn:turn.yourdomain.com:3478
TURN_USERNAME=username
TURN_PASSWORD=password
TURN_SECRET=your-secret-key

# Frontend
VITE_STUN_SERVERS=stun:stun.l.google.com:19302
VITE_TURN_SERVER_URL=turn:turn.yourdomain.com:3478
```

---

## 📚 Additional Resources

### WebRTC
- [WebRTC API Documentation](https://developer.mozilla.org/en-US/docs/Web/API/WebRTC_API)
- [WebRTC Samples](https://webrtc.github.io/samples/)
- [WebRTC for the Curious](https://webrtcforthecurious.com/)

### TURN Server
- [Coturn Documentation](https://github.com/coturn/coturn)
- [WebRTC TURN Server Setup](https://www.twilio.com/docs/stun-turn)

### Rust WebRTC
- [webrtc-rs](https://github.com/webrtc-rs/webrtc)
- [Rust WebRTC Examples](https://github.com/webrtc-rs/webrtc/tree/master/examples)

---

## 🎯 Success Metrics

- **Latency**: < 200ms end-to-end delay
- **Quality**: MOS (Mean Opinion Score) > 4.0
- **Reliability**: < 5% call failure rate
- **Performance**: < 3s time to establish connection
- **Browser Support**: Chrome, Firefox, Safari, Edge (latest versions)

---

## 📝 Notes

### Known Limitations
- Currently supports 1-1 calls only (not group calls)
- No recording functionality (yet)
- No call scheduling
- No voicemail

### Future Enhancements
- Group video calls (3+ participants)
- Call recording
- Voicemail integration
- Call scheduling
- Advanced effects (blur background, filters)
- Call analytics dashboard
- Integration with phone numbers (PSTN)

---

**Document Version**: 1.0  
**Last Updated**: March 2026  
**Author**: Development Team