import { type ChangeEvent, useEffect, useRef, useState } from 'react'
import { Camera, Check, Moon, Sun, User } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { useAuthStore } from '@/stores/auth.store'
import { useThemeStore } from '@/stores/theme.store'
import { fileUploadService } from '@/services/file-upload.service'
import { cn } from '@/lib/utils'
import { extractErrorMsg } from '@/lib/api'
import { toast } from 'sonner'

type Props = {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function SettingsPanel({ open, onOpenChange }: Props) {
  const user = useAuthStore((state) => state.user)
  const updateProfile = useAuthStore((state) => state.updateProfile)
  const { theme, setTheme } = useThemeStore()

  const [displayName, setDisplayName] = useState(user?.display_name ?? '')
  const [bio, setBio] = useState(user?.bio ?? '')
  const [phone, setPhone] = useState(user?.phone ?? '')
  const [loading, setLoading] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (!open) return
    setDisplayName(user?.display_name ?? '')
    setBio(user?.bio ?? '')
    setPhone(user?.phone ?? '')
  }, [open, user])

  const saveProfile = async () => {
    if (!displayName.trim()) return
    setLoading(true)
    try {
      await updateProfile({
        display_name: displayName.trim(),
        bio: bio.trim(),
        phone: phone.trim(),
      })
      toast.success('Cập nhật hồ sơ thành công')
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  const uploadAvatar = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    if (!file) return
    setLoading(true)
    try {
      const { url } = await fileUploadService.upload(file)
      await updateProfile({ avatar_url: url })
      toast.success('Cập nhật ảnh đại diện thành công')
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
      event.target.value = ''
    }
  }

  if (!open) return null

  return (
    <>
      <div
        className="fixed inset-0 z-40 bg-black/30 backdrop-blur-[1px]"
        onClick={() => onOpenChange(false)}
      />
      <aside className="fixed inset-y-0 right-0 z-50 w-full max-w-md border-l border-border/60 bg-card shadow-xl animate-in slide-in-from-right duration-200">
        <div className="flex h-full flex-col">
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-4">
            <h2 className="text-sm font-semibold">Cài đặt</h2>
            <Button variant="ghost" size="sm" onClick={() => onOpenChange(false)}>
              Đóng
            </Button>
          </div>

          <div className="flex-1 space-y-6 overflow-y-auto p-5">
            <section className="space-y-4">
              <h3 className="flex items-center gap-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                <User className="h-3.5 w-3.5" /> Hồ sơ
              </h3>

              <div className="flex flex-col items-center gap-3">
                <button
                  onClick={() => fileInputRef.current?.click()}
                  className="group relative"
                  disabled={loading}
                >
                  <Avatar className="h-20 w-20 border border-border/60">
                    <AvatarImage src={user?.avatar_url ?? undefined} />
                    <AvatarFallback>{user?.display_name?.[0] ?? '?'}</AvatarFallback>
                  </Avatar>
                  <span className="absolute inset-0 flex items-center justify-center rounded-full bg-black/40 opacity-0 transition-opacity group-hover:opacity-100">
                    <Camera className="h-5 w-5 text-white" />
                  </span>
                </button>
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  className="hidden"
                  onChange={uploadAvatar}
                />
                <p className="text-xs text-muted-foreground">@{user?.username}</p>
              </div>

              <div className="space-y-3">
                <Input
                  value={displayName}
                  onChange={(e) => setDisplayName(e.target.value)}
                  placeholder="Tên hiển thị"
                />
                <Input
                  value={phone}
                  onChange={(e) => setPhone(e.target.value)}
                  placeholder="Số điện thoại"
                />
                <Textarea
                  value={bio}
                  onChange={(e) => setBio(e.target.value)}
                  placeholder="Giới thiệu của bạn"
                  className="h-20 resize-none"
                />
                <Button onClick={saveProfile} disabled={loading || !displayName.trim()} className="w-full">
                  Lưu thay đổi
                </Button>
              </div>
            </section>

            <section className="space-y-4">
              <h3 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                Giao diện
              </h3>
              <div className="grid grid-cols-2 gap-3">
                <button
                  onClick={() => setTheme('light')}
                  className={cn(
                    'rounded-xl border p-4 text-left transition-colors',
                    theme === 'light'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:bg-muted/50',
                  )}
                >
                  <Sun className="mb-2 h-5 w-5 text-amber-500" />
                  <div className="flex items-center justify-between text-sm">
                    <span>Sáng</span>
                    {theme === 'light' && <Check className="h-4 w-4 text-primary" />}
                  </div>
                </button>
                <button
                  onClick={() => setTheme('dark')}
                  className={cn(
                    'rounded-xl border p-4 text-left transition-colors',
                    theme === 'dark'
                      ? 'border-primary bg-primary/5'
                      : 'border-border hover:bg-muted/50',
                  )}
                >
                  <Moon className="mb-2 h-5 w-5 text-blue-400" />
                  <div className="flex items-center justify-between text-sm">
                    <span>Tối</span>
                    {theme === 'dark' && <Check className="h-4 w-4 text-primary" />}
                  </div>
                </button>
              </div>
            </section>
          </div>
        </div>
      </aside>
    </>
  )
}
