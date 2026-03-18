export type AuthUser = {
  id: string
  username: string
  email: string
  display_name: string
  avatar_url: string | null
  bio: string | null
  phone: string | null
}

export type SignInPayload = {
  username: string
  password: string
}

export type SignUpPayload = {
  username: string
  email: string
  password: string
  display_name: string
}

export type SignInResponse = {
  access_token: string
}
