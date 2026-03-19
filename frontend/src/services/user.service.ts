import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'
import type { PresenceItem, UserSearchResult } from '@/types/user'

export const userService = {
  async search(q: string, limit = 10): Promise<UserSearchResult[]> {
    const response = await http.get('/users/search', { params: { q, limit } })
    return unwrapData<UserSearchResult[]>(response)
  },

  async getPresence(userIds: string[]): Promise<PresenceItem[]> {
    const response = await http.post('/users/presence', { user_ids: userIds })
    return unwrapData<PresenceItem[]>(response)
  },

  async updateProfile(
    userId: string,
    payload: {
      display_name?: string
      avatar_url?: string | null
      bio?: string | null
      phone?: string | null
    },
  ): Promise<void> {
    await http.patch(`/users/${userId}`, payload)
  },
}
