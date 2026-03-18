import { create } from 'zustand'
import { friendService } from '@/services/friend.service'
import { userService } from '@/services/user.service'
import type { Friend, FriendRequest } from '@/types/friend'
import type { UserSearchResult } from '@/types/user'

type FriendState = {
  friends: Friend[]
  requests: FriendRequest[]
  searchResult: UserSearchResult[]
  loading: boolean
  loadFriends: () => Promise<void>
  loadRequests: () => Promise<void>
  searchUsers: (keyword: string) => Promise<void>
  sendRequest: (recipientId: string, message?: string) => Promise<void>
  acceptRequest: (requestId: string) => Promise<void>
  declineRequest: (requestId: string) => Promise<void>
  removeFriend: (friendId: string) => Promise<void>
}

export const useFriendStore = create<FriendState>((set, get) => ({
  friends: [],
  requests: [],
  searchResult: [],
  loading: false,

  loadFriends: async () => {
    set({ loading: true })
    try {
      const data = await friendService.listFriends()
      set({ friends: data })
    } finally {
      set({ loading: false })
    }
  },

  loadRequests: async () => {
    const data = await friendService.listRequests()
    set({ requests: data })
  },

  searchUsers: async (keyword) => {
    const trimmed = keyword.trim()
    if (trimmed.length < 2) {
      set({ searchResult: [] })
      return
    }

    const data = await userService.search(trimmed, 10)
    set({ searchResult: data })
  },

  sendRequest: async (recipientId, message) => {
    await friendService.sendRequest({ recipient_id: recipientId, message })
    await get().loadRequests()
  },

  acceptRequest: async (requestId) => {
    await friendService.acceptRequest(requestId)
    await Promise.all([get().loadFriends(), get().loadRequests()])
  },

  declineRequest: async (requestId) => {
    await friendService.declineRequest(requestId)
    await get().loadRequests()
  },

  removeFriend: async (friendId) => {
    await friendService.removeFriend(friendId)
    await get().loadFriends()
  },
}))
