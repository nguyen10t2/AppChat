import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'
import type {
  AuthUser,
  SignInPayload,
  SignInResponse,
  SignUpPayload,
} from '@/types/auth'

export const authService = {
  async signUp(payload: SignUpPayload): Promise<{ id: string }> {
    const response = await http.post('/auth/signup', payload)
    return unwrapData<{ id: string }>(response)
  },

  async signIn(payload: SignInPayload): Promise<SignInResponse> {
    const response = await http.post('/auth/signin', payload)
    return unwrapData<SignInResponse>(response)
  },

  async refresh(): Promise<SignInResponse> {
    const response = await http.post('/auth/refresh')
    return unwrapData<SignInResponse>(response)
  },

  async signOut(): Promise<void> {
    await http.get('/auth/signout')
  },

  async profile(): Promise<AuthUser> {
    const response = await http.get('/users/profile')
    return unwrapData<AuthUser>(response)
  },
}
