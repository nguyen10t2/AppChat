import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'
import type { Friend, FriendRequest, FriendRequestPayload } from '@/types/friend'

export const friendService = {
  async listFriends(): Promise<Friend[]> {
    const response = await http.get('/friends/')
    return unwrapData<Friend[]>(response)
  },

  async listRequests(): Promise<FriendRequest[]> {
    const response = await http.get('/friends/requests')
    return unwrapData<FriendRequest[]>(response)
  },

  async sendRequest(payload: FriendRequestPayload): Promise<void> {
    await http.post('/friends/requests', payload)
  },

  async acceptRequest(requestId: string): Promise<void> {
    await http.post(`/friends/requests/${requestId}/accept`)
  },

  async declineRequest(requestId: string): Promise<void> {
    await http.post(`/friends/requests/${requestId}/decline`)
  },

  async removeFriend(friendId: string): Promise<void> {
    await http.delete(`/friends/${friendId}`)
  },
}
