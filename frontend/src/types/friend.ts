export type Friend = {
  id: string
  username: string
  display_name: string
  avatar_url: string | null
}

export type FriendRequestPayload = {
  recipient_id: string
  message?: string
}

export type IdOrInfo =
  | { Id: string }
  | { Info: Friend }

export type FriendRequest = {
  id: string
  from: IdOrInfo
  to: IdOrInfo
  message: string | null
  created_at: string
}
