export type UserSearchResult = {
  id: string
  username: string
  email: string
  display_name: string
  avatar_url: string | null
  bio: string | null
  phone: string | null
}

export type PresenceItem = {
  user_id: string
  is_online: boolean
  last_seen: string | null
}
