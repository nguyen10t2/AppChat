import { create } from 'zustand'
import { userService } from '@/services/user.service'

export type PresenceStatus = {
  isOnline: boolean
  lastSeen: string | null
}

type PresenceState = {
  byUserId: Record<string, PresenceStatus>
  fetchBatch: (userIds: string[]) => Promise<void>
  setOnlineUsers: (userIds: string[]) => void
  markUserOnline: (userId: string) => void
  markUserOffline: (userId: string, lastSeen: string | null) => void
}

export const usePresenceStore = create<PresenceState>((set) => ({
  byUserId: {},

  fetchBatch: async (userIds) => {
    const uniqueIds = Array.from(new Set(userIds)).filter(Boolean)
    if (uniqueIds.length === 0) return

    const data = await userService.getPresence(uniqueIds)
    set((state) => {
      const next = { ...state.byUserId }
      data.forEach((item) => {
        next[item.user_id] = {
          isOnline: item.is_online,
          lastSeen: item.last_seen,
        }
      })

      return { byUserId: next }
    })
  },

  setOnlineUsers: (userIds) => {
    const onlineSet = new Set(userIds)
    set((state) => {
      const next = { ...state.byUserId }
      Object.keys(next).forEach((userId) => {
        next[userId] = {
          ...next[userId],
          isOnline: onlineSet.has(userId),
        }
      })

      userIds.forEach((userId) => {
        if (!next[userId]) {
          next[userId] = { isOnline: true, lastSeen: null }
        }
      })

      return { byUserId: next }
    })
  },

  markUserOnline: (userId) => {
    set((state) => ({
      byUserId: {
        ...state.byUserId,
        [userId]: {
          isOnline: true,
          lastSeen: state.byUserId[userId]?.lastSeen ?? null,
        },
      },
    }))
  },

  markUserOffline: (userId, lastSeen) => {
    set((state) => ({
      byUserId: {
        ...state.byUserId,
        [userId]: {
          isOnline: false,
          lastSeen,
        },
      },
    }))
  },
}))
