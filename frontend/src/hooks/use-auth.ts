import { useAuthStore } from '@/stores/auth.store'
import { useShallow } from 'zustand/react/shallow'

export function useAuth() {
  return useAuthStore(
    useShallow((state) => ({
      user: state.user,
      accessToken: state.accessToken,
      isAuthenticated: state.isAuthenticated,
      signIn: state.signIn,
      signUp: state.signUp,
      signOut: state.signOut,
    }))
  )
}
