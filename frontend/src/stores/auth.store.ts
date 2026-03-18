import { create } from 'zustand'
import { authService } from '@/services/auth.service'
import { forceRefresh } from '@/lib/http'
import type { AuthUser, SignInPayload, SignUpPayload } from '@/types/auth'

type AuthState = {
  accessToken: string | null
  user: AuthUser | null
  isAuthenticated: boolean
  isBootstrapping: boolean
  setAccessToken: (token: string | null) => void
  clearSession: () => void
  bootstrap: () => Promise<void>
  signIn: (payload: SignInPayload) => Promise<void>
  signUp: (payload: SignUpPayload) => Promise<void>
  signOut: () => Promise<void>
}

export const useAuthStore = create<AuthState>()(
  (set, get) => ({
      accessToken: null,
      user: null,
      isAuthenticated: false,
      isBootstrapping: true,

      setAccessToken: (token) =>
        set({
          accessToken: token,
          isAuthenticated: Boolean(token),
        }),

      clearSession: () =>
        set({
          accessToken: null,
          user: null,
          isAuthenticated: false,
        }),

      bootstrap: async () => {
        set({ isBootstrapping: true })

        try {
          let token = get().accessToken

          if (!token) {
            token = await forceRefresh()
            if (token) {
              set({ accessToken: token, isAuthenticated: true })
            }
          }

          if (token) {
            const profile = await authService.profile()
            set({
              user: profile,
              isAuthenticated: true,
            })
          } else {
            set({ isAuthenticated: false, user: null })
          }
        } finally {
          set({ isBootstrapping: false })
        }
      },

      signIn: async (payload) => {
        const result = await authService.signIn(payload)
        set({ accessToken: result.access_token, isAuthenticated: true })

        const profile = await authService.profile()
        set({ user: profile, isAuthenticated: true })
      },

      signUp: async (payload) => {
        await authService.signUp(payload)
      },

      signOut: async () => {
        await authService.signOut().catch(() => null)
        get().clearSession()
      },
  }),
)
