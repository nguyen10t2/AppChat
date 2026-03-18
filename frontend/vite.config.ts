import { defineConfig } from 'vite'
import react, { reactCompilerPreset } from '@vitejs/plugin-react'
import babel from '@rolldown/plugin-babel'
import path from 'path'
import taiwindcss from '@tailwindcss/vite'

// https://vite.dev/config/
export default defineConfig({
  plugins: [
    react(),
    taiwindcss(),
    babel({ presets: [reactCompilerPreset()] })
  ],
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id.includes('node_modules')) return

          if (id.includes('/react/') || id.includes('/react-dom/')) {
            return 'vendor-react'
          }

          if (id.includes('/react-router/') || id.includes('/react-router-dom/')) {
            return 'vendor-router'
          }

          if (id.includes('/zustand/') || id.includes('/axios/')) {
            return 'vendor-state-net'
          }

          if (
            id.includes('/radix-ui/') ||
            id.includes('/@phosphor-icons/') ||
            id.includes('/lucide-react/') ||
            id.includes('/sonner/')
          ) {
            return 'vendor-ui'
          }

          if (
            id.includes('/react-hook-form/') ||
            id.includes('/@hookform/resolvers/') ||
            id.includes('/zod/')
          ) {
            return 'vendor-forms'
          }

          return 'vendor-misc'
        },
      },
    },
  },
  resolve: {
    alias: {
      "@": path.resolve(__dirname, './src')
    }
  }
})
