import {
  ChatCircleDotsIcon,
  GearSixIcon,
  SignOutIcon,
  User,
  UsersIcon,
  XIcon,
} from '@phosphor-icons/react'
import { useEffect, useState } from 'react'
import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth.store'
import { useThemeStore } from '@/stores/theme.store'
import { useWebSocketBridge } from '@/hooks/use-websocket'
import { cn } from '@/lib/utils'

import { SettingsPanel } from '@/components/layout/settings-panel'
import { CallLayer } from '@/components/call/call-layer'

export function ProtectedLayout() {
  const navigate = useNavigate()
  const user = useAuthStore((state) => state.user)
  const signOut = useAuthStore((state) => state.signOut)
  const initTheme = useThemeStore((state) => state.initTheme)
  const [showProfile, setShowProfile] = useState(false)
  const [isSettingsOpen, setIsSettingsOpen] = useState(false)

  useWebSocketBridge()

  useEffect(() => {
    initTheme()
  }, [initTheme])

  const handleSignOut = async () => {
    await signOut()
    navigate('/login', { replace: true })
  }

  const initials = user?.display_name
    ? user.display_name
      .split(' ')
      .map((w) => w[0])
      .join('')
      .slice(0, 2)
      .toUpperCase()
    : '?'

  return (
    <div className="flex h-screen overflow-hidden bg-background">
      {/* ── Narrow icon nav (Zalo-style left rail) ── */}
      <nav className="flex h-full w-14 flex-col items-center border-r border-border/60 bg-card/80 py-3 shrink-0">
        {/* App logo / brand */}
        <NavLink
          to="/chat"
          title="Trang chủ"
          className="mb-6 mt-1 flex h-10 w-10 items-center justify-center rounded-xl transition-all hover:scale-110 active:scale-95 overflow-hidden ring-2 ring-transparent hover:ring-primary/20"
        >
          <img src="/zula.png" alt="App Logo" className="w-full h-full object-contain" />
        </NavLink>

        {/* Nav icons */}
        <div className="flex flex-1 flex-col items-center gap-1">
          <NavIconItem
            to="/chat"
            label="Tin nhắn"
            icon={<ChatCircleDotsIcon size={22} />}
          />
          <NavIconItem
            to="/friends"
            label="Bạn bè"
            icon={<UsersIcon size={22} />}
          />
        </div>

        {/* Bottom: settings + avatar */}
        <div className="flex flex-col items-center gap-2 mt-2">
          <button
            title="Cài đặt"
            onClick={() => setIsSettingsOpen(true)}
            className="flex h-9 w-9 items-center justify-center rounded-xl text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          >
            <GearSixIcon size={20} />
          </button>

          {/* Avatar → profile popup */}
          <div className="relative">
            <button
              title={user?.display_name ?? 'Hồ sơ'}
              onClick={() => setShowProfile((v) => !v)}
              className="flex h-9 w-9 items-center justify-center rounded-full bg-primary/20 text-primary text-xs font-semibold hover:ring-2 hover:ring-primary/40 transition-all overflow-hidden"
            >
              {user?.avatar_url ? (
                <img src={user.avatar_url} alt="" className="h-full w-full object-cover" />
              ) : initials}
            </button>

            {showProfile && (
              <ProfilePopup
                displayName={user?.display_name ?? 'Guest'}
                username={user?.username ?? ''}
                initials={initials}
                avatarUrl={user?.avatar_url}
                onSignOut={handleSignOut}
                onOpenSettings={() => {
                  setIsSettingsOpen(true)
                  setShowProfile(false)
                }}
                onClose={() => setShowProfile(false)}
              />
            )}
          </div>
        </div>
      </nav>

      {/* ── Main content ── */}
      <main className="flex min-w-0 flex-1 overflow-hidden">
        <Outlet />
      </main>

      <SettingsPanel
        open={isSettingsOpen}
        onOpenChange={setIsSettingsOpen}
      />
      <CallLayer />
    </div>
  )
}

/* ── Helpers ── */

function NavIconItem({
  to,
  label,
  icon,
}: {
  to: string
  label: string
  icon: React.ReactNode
}) {
  return (
    <NavLink
      to={to}
      title={label}
      className={({ isActive }) =>
        cn(
          'relative flex h-10 w-10 items-center justify-center rounded-xl transition-colors',
          isActive
            ? 'bg-primary text-primary-foreground shadow-md'
            : 'text-muted-foreground hover:bg-muted hover:text-foreground',
        )
      }
    >
      {icon}
    </NavLink>
  )
}

function ProfilePopup({
  displayName,
  username,
  initials,
  avatarUrl,
  onSignOut,
  onOpenSettings,
  onClose,
}: {
  displayName: string
  username: string
  initials: string
  avatarUrl?: string | null
  onSignOut: () => void
  onOpenSettings: () => void
  onClose: () => void
}) {
  return (
    <>
      {/* Backdrop */}
      <div className="fixed inset-0 z-40" onClick={onClose} />

      <div className="absolute bottom-0 left-14 z-50 w-64 rounded-2xl border border-border/60 bg-card shadow-xl p-4 flex flex-col gap-3 animate-in slide-in-from-bottom-2 duration-200">
        <button
          onClick={onClose}
          className="absolute top-3 right-3 text-muted-foreground hover:text-foreground"
        >
          <XIcon size={16} />
        </button>

        {/* User info */}
        <div className="flex items-center gap-3">
          <div className="flex h-12 w-12 shrink-0 items-center justify-center rounded-full bg-primary/20 text-primary font-semibold text-base overflow-hidden">
            {avatarUrl ? (
              <img src={avatarUrl} alt="" className="h-full w-full object-cover" />
            ) : initials}
          </div>
          <div className="min-w-0">
            <p className="truncate text-sm font-semibold text-foreground">{displayName}</p>
            <p className="truncate text-xs text-muted-foreground">@{username}</p>
          </div>
        </div>

        <div className="h-px bg-border/60" />

        <button
          onClick={onOpenSettings}
          className="flex items-center gap-2 rounded-xl px-3 py-2 text-sm text-foreground hover:bg-muted transition-colors"
        >
          <User size={16} />
          Hồ sơ & cài đặt
        </button>

        <button
          onClick={onSignOut}
          className="flex items-center gap-2 rounded-xl px-3 py-2 text-sm text-destructive hover:bg-destructive/10 transition-colors"
        >
          <SignOutIcon size={16} />
          Đăng xuất
        </button>
      </div>
    </>
  )
}
