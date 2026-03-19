import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { MagnifyingGlassIcon, UserPlusIcon } from '@phosphor-icons/react'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useFriends } from '@/hooks/use-friends'
import { useAuthStore } from '@/stores/auth.store'
import { conversationService } from '@/services/conversation.service'
import type { FriendRequest } from '@/types/friend'
import { cn } from '@/lib/utils'

/** Lấy ID của user trong một request */
function getRequestUserId(req: FriendRequest, side: 'from' | 'to'): string {
  const obj = req[side]
  if ('Id' in obj) return obj.Id
  return obj.Info.id
}

/** Lấy display name từ IdOrInfo */
function getDisplayName(req: FriendRequest, side: 'from' | 'to'): string {
  const obj = req[side]
  if ('Info' in obj) return obj.Info.display_name
  return `ID: ${obj.Id.slice(0, 8)}…`
}

type Tab = 'contacts' | 'received' | 'sent' | 'search'

export function FriendManager() {
  const navigate = useNavigate()
  const currentUser = useAuthStore((state) => state.user)
  const {
    friends,
    requests,
    searchResult,
    searchUsers,
    sendRequest,
    acceptRequest,
    declineRequest,
    removeFriend,
  } = useFriends()

  const [tab, setTab] = useState<Tab>('contacts')
  const [keyword, setKeyword] = useState('')

  // Phân loại lời mời
  const receivedRequests = requests.filter(
    (req) => getRequestUserId(req, 'to') === currentUser?.id,
  )
  const sentRequests = requests.filter(
    (req) => getRequestUserId(req, 'from') === currentUser?.id,
  )

  const tabs: { id: Tab; label: string; count?: number }[] = [
    { id: 'contacts', label: 'Bạn bè', count: friends.length },
    { id: 'received', label: 'Lời mời', count: receivedRequests.length },
    { id: 'sent', label: 'Đã gửi', count: sentRequests.length },
    { id: 'search', label: 'Tìm kiếm' },
  ]

  return (
    <div className="flex h-full w-full overflow-hidden">
      {/* ── Left: Tab list (sidebar-style on desktop, top tabs on mobile) ── */}
      <div className="flex h-full w-full flex-col md:w-72 md:border-r border-border/60 bg-card/80 md:shrink-0">
        {/* Header */}
        <div className="border-b border-border/60 px-4 py-3">
          <h2 className="text-sm font-semibold text-foreground">Danh bạ</h2>
        </div>

        {/* Search bar (always visible) */}
        <div className="px-3 py-2 border-b border-border/60">
          <div className="relative">
            <MagnifyingGlassIcon
              size={15}
              className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
            />
            <Input
              className="pl-8 h-8 text-sm rounded-xl bg-muted/60 border-0"
              placeholder="Tìm tên, username..."
              value={keyword}
              onChange={(e) => {
                const v = e.target.value
                setKeyword(v)
                if (v.trim()) {
                  setTab('search')
                  void searchUsers(v)
                } else {
                  setTab('contacts')
                }
              }}
            />
          </div>
        </div>

        {/* Tabs */}
        <div className="flex gap-1 overflow-x-auto scrollbar-none px-3 py-2 border-b border-border/60">
          {tabs.map((t) => (
            <button
              key={t.id}
              onClick={() => setTab(t.id)}
              className={cn(
                'flex-1 shrink-0 whitespace-nowrap rounded-lg px-2 py-1 text-[11px] font-medium transition-colors',
                tab === t.id
                  ? 'bg-primary text-primary-foreground'
                  : 'text-muted-foreground hover:bg-muted hover:text-foreground',
              )}
            >
              {t.label}
              {t.count !== undefined && t.count > 0 ? (
                <span className="ml-1 inline-flex h-4 min-w-4 items-center justify-center rounded-full bg-white/20 px-1 text-[10px]">
                  {t.count}
                </span>
              ) : null}
            </button>
          ))}
        </div>

        {/* Content */}
        <ScrollArea className="flex-1">
          <div className="space-y-1 p-2">
            {/* ── Bạn bè ── */}
            {tab === 'contacts' &&
              (friends.length === 0 ? (
                <EmptyState icon="👥" text="Chưa có bạn bè nào" />
              ) : (
                friends.map((friend) => (
                  <div
                    key={friend.id}
                    className="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-muted/60 transition-colors"
                  >
                    <Avatar name={friend.display_name} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-foreground">
                        {friend.display_name}
                      </p>
                      <p className="truncate text-xs text-muted-foreground">@{friend.username}</p>
                    </div>
                    <div className="flex gap-1">
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 rounded-lg text-xs px-2"
                        onClick={async () => {
                          const conversation = await conversationService.create({
                            type: 'direct',
                            name: '',
                            member_ids: [friend.id],
                          })
                          if (conversation?.conversation_id) {
                            navigate(`/chat?conversation=${conversation.conversation_id}`)
                          }
                        }}
                      >
                        Nhắn tin
                      </Button>
                      <Button
                        size="sm"
                        variant="ghost"
                        className="h-7 rounded-lg text-xs px-2 text-destructive hover:text-destructive"
                        onClick={async () => {
                          await removeFriend(friend.id)
                          toast.success('Đã hủy kết bạn')
                        }}
                      >
                        Xóa
                      </Button>
                    </div>
                  </div>
                ))
              ))}

            {/* ── Lời mời nhận được ── */}
            {tab === 'received' &&
              (receivedRequests.length === 0 ? (
                <EmptyState icon="📭" text="Không có lời mời kết bạn nào" />
              ) : (
                receivedRequests.map((req) => (
                  <div
                    key={req.id}
                    className="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-muted/60 transition-colors"
                  >
                    <Avatar name={getDisplayName(req, 'from')} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-foreground">
                        {getDisplayName(req, 'from')}
                      </p>
                      {req.message && (
                        <p className="truncate text-xs text-muted-foreground">{req.message}</p>
                      )}
                    </div>
                    <div className="flex gap-1">
                      <Button
                        size="sm"
                        className="h-7 rounded-lg text-xs px-2"
                        onClick={async () => {
                          await acceptRequest(req.id)
                          toast.success('Đã chấp nhận lời mời')
                        }}
                      >
                        Chấp nhận
                      </Button>
                      <Button
                        size="sm"
                        variant="outline"
                        className="h-7 rounded-lg text-xs px-2"
                        onClick={async () => {
                          await declineRequest(req.id)
                          toast.success('Đã từ chối')
                        }}
                      >
                        Từ chối
                      </Button>
                    </div>
                  </div>
                ))
              ))}

            {/* ── Lời mời đã gửi ── */}
            {tab === 'sent' &&
              (sentRequests.length === 0 ? (
                <EmptyState icon="📤" text="Chưa gửi lời mời nào" />
              ) : (
                sentRequests.map((req) => (
                  <div
                    key={req.id}
                    className="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-muted/60 transition-colors"
                  >
                    <Avatar name={getDisplayName(req, 'to')} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-foreground">
                        {getDisplayName(req, 'to')}
                      </p>
                      <p className="truncate text-xs text-muted-foreground">Đang chờ phản hồi</p>
                    </div>
                    <Button
                      size="sm"
                      variant="outline"
                      className="h-7 rounded-lg text-xs px-2"
                      onClick={async () => {
                        await declineRequest(req.id)
                        toast.success('Đã thu hồi lời mời')
                      }}
                    >
                      Thu hồi
                    </Button>
                  </div>
                ))
              ))}

            {/* ── Kết quả tìm kiếm ── */}
            {tab === 'search' &&
              (searchResult.length === 0 ? (
                <EmptyState icon="🔍" text={keyword.length < 2 ? 'Nhập ít nhất 2 ký tự' : 'Không tìm thấy kết quả'} />
              ) : (
                searchResult.map((user) => (
                  <div
                    key={user.id}
                    className="flex items-center gap-3 rounded-xl px-3 py-2 hover:bg-muted/60 transition-colors"
                  >
                    <Avatar name={user.display_name} />
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-medium text-foreground">
                        {user.display_name}
                      </p>
                      <p className="truncate text-xs text-muted-foreground">@{user.username}</p>
                    </div>
                    <Button
                      size="sm"
                      className="h-7 rounded-lg text-xs px-2 gap-1"
                      onClick={async () => {
                        await sendRequest(user.id)
                        toast.success('Đã gửi lời mời kết bạn')
                      }}
                    >
                      <UserPlusIcon size={12} />
                      Kết bạn
                    </Button>
                  </div>
                ))
              ))}
          </div>
        </ScrollArea>
      </div>

      {/* ── Right: empty state on desktop ── */}
      <div className="hidden md:flex flex-1 flex-col items-center justify-center gap-3 text-muted-foreground bg-background">
        <div className="text-5xl">👥</div>
        <p className="text-sm">Chọn một người bạn để nhắn tin</p>
      </div>
    </div>
  )
}

function Avatar({ name }: { name: string }) {
  const initials = name
    .split(' ')
    .map((w) => w[0])
    .join('')
    .slice(0, 2)
    .toUpperCase()

  return (
    <div className="flex h-9 w-9 shrink-0 items-center justify-center rounded-full bg-primary/20 text-primary text-xs font-semibold">
      {initials}
    </div>
  )
}

function EmptyState({ icon, text }: { icon: string; text: string }) {
  return (
    <div className="flex flex-col items-center justify-center gap-2 py-12 text-muted-foreground">
      <span className="text-3xl">{icon}</span>
      <p className="text-xs">{text}</p>
    </div>
  )
}
