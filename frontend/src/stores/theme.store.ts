import { create } from 'zustand'
import { persist } from 'zustand/middleware'

type Theme = 'light' | 'dark'

type ThemeState = {
  theme: Theme
  setTheme: (theme: Theme) => void
  toggleTheme: () => void
  initTheme: () => void
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set, get) => ({
      theme: 'light',
      setTheme: (theme) => {
        set({ theme })
        updateThemeRoot(theme)
      },
      toggleTheme: () => {
        set((state) => {
          const next = state.theme === 'light' ? 'dark' : 'light'
          updateThemeRoot(next)
          return { theme: next }
        })
      },
      initTheme: () => {
        const current = get().theme
        updateThemeRoot(current)
      },
    }),
    {
      name: 'app-theme',
      onRehydrateStorage: () => (state) => {
        if (state) updateThemeRoot(state.theme)
      },
    }
  )
)

function updateThemeRoot(theme: Theme) {
  const root = window.document.documentElement
  root.classList.remove('light', 'dark')
  root.classList.add(theme)
}
