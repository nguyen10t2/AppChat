import { ChatCircleDotsIcon, SignOutIcon, UsersIcon } from '@phosphor-icons/react'
import type { ReactNode } from 'react'
import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { useAuthStore } from '@/stores/auth.store'
import { useWebSocketBridge } from '@/hooks/use-websocket'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export function ProtectedLayout() {
  const navigate = useNavigate()
  const user = useAuthStore((state) => state.user)
  const signOut = useAuthStore((state) => state.signOut)

  useWebSocketBridge()

  const handleSignOut = async () => {
    await signOut()
    navigate('/login', { replace: true })
  }

  return (
    <div className="flex min-h-screen bg-gradient-to-b from-background to-muted/20">
      <aside className="w-64 border-r border-border/60 bg-card/70 p-4">
        <div className="mb-8">
          <h1 className="text-base font-semibold text-foreground">AppChat</h1>
          <p className="text-xs text-muted-foreground">{user?.display_name ?? 'Guest'}</p>
        </div>

        <nav className="space-y-2">
          <NavItem to="/chat" label="Tin nhắn" icon={<ChatCircleDotsIcon />} />
          <NavItem to="/friends" label="Bạn bè" icon={<UsersIcon />} />
        </nav>

        <Button className="mt-8 w-full" variant="outline" onClick={handleSignOut}>
          <SignOutIcon />
          Đăng xuất
        </Button>
      </aside>

      <main className="flex-1 p-4 md:p-6">
        <Outlet />
      </main>
    </div>
  )
}

function NavItem({ to, label, icon }: { to: string; label: string; icon: ReactNode }) {
  return (
    <NavLink
      to={to}
      className={({ isActive }) =>
        cn(
          'flex items-center gap-2 rounded-none px-3 py-2 text-sm transition-colors',
          isActive
            ? 'bg-primary text-primary-foreground'
            : 'text-muted-foreground hover:bg-muted hover:text-foreground',
        )
      }
    >
      {icon}
      <span>{label}</span>
    </NavLink>
  )
}
