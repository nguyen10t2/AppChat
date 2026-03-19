import { useState, useEffect } from 'react'
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { useFriendStore } from '@/stores/friend.store'
import { useChatStore } from '@/stores/chat.store'
import { conversationService } from '@/services/conversation.service'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { Check } from 'lucide-react'
import { cn } from '@/lib/utils'
import { toast } from 'sonner'
import { extractErrorMsg } from '@/lib/api'

interface Props {
  open: boolean
  onOpenChange: (open: boolean) => void
}

export function CreateGroupModal({ open, onOpenChange }: Props) {
  const { friends, loadFriends } = useFriendStore()
  const { loadConversations, openConversation } = useChatStore()
  
  const [name, setName] = useState('')
  const [selectedIds, setSelectedIds] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [search, setSearch] = useState('')

  useEffect(() => {
    if (open) {
      loadFriends()
      setName('')
      setSelectedIds([])
      setSearch('')
    }
  }, [open, loadFriends])

  const filteredFriends = friends.filter((f) =>
    f.display_name.toLowerCase().includes(search.toLowerCase())
  )

  const toggleFriend = (id: string) => {
    setSelectedIds((prev) =>
      prev.includes(id) ? prev.filter((i) => i !== id) : [...prev, id]
    )
  }

  const handleCreate = async () => {
    if (!name.trim()) {
      toast.error('Vui lòng nhập tên nhóm')
      return
    }
    if (selectedIds.length < 2) {
      toast.error('Chọn ít nhất 2 bạn bè để tạo nhóm')
      return
    }

    setLoading(true)
    try {
      const group = await conversationService.create({
        type: 'group',
        name: name.trim(),
        member_ids: selectedIds,
      })

      if (group) {
        toast.success('Tạo nhóm thành công')
        await loadConversations()
        await openConversation(group.conversation_id)
        onOpenChange(false)
      }
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[420px] p-0 gap-0 overflow-hidden">
        <DialogHeader className="p-4 border-b">
          <DialogTitle>Tạo nhóm mới</DialogTitle>
        </DialogHeader>

        <div className="p-4 space-y-4">
          <Input
            placeholder="Nhập tên nhóm..."
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="h-11"
          />

          <div className="space-y-2">
            <p className="text-xs font-medium text-muted-foreground uppercase tracking-wider">
              Chọn bạn bè ({selectedIds.length})
            </p>
            <Input
              placeholder="Tìm kiếm bạn bè..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="h-9 text-xs"
            />
            
            <div className="max-h-[300px] overflow-y-auto pr-1 space-y-1">
              {filteredFriends.length === 0 ? (
                <p className="text-center py-8 text-xs text-muted-foreground">
                  {friends.length === 0 ? 'Bạn chưa có bạn bè nào' : 'Không tìm thấy kết quả'}
                </p>
              ) : (
                filteredFriends.map((friend) => {
                  const isSelected = selectedIds.includes(friend.id)
                  return (
                    <button
                      key={friend.id}
                      onClick={() => toggleFriend(friend.id)}
                      className={cn(
                        "flex w-full items-center gap-3 p-2 rounded-lg transition-colors group",
                        isSelected ? "bg-primary/5" : "hover:bg-muted"
                      )}
                    >
                      <div className="relative shrink-0">
                        <Avatar className="h-9 w-9">
                          <AvatarImage src={friend.avatar_url || undefined} />
                          <AvatarFallback className="text-xs">{friend.display_name[0]}</AvatarFallback>
                        </Avatar>
                        {isSelected && (
                          <div className="absolute -right-1 -bottom-1 bg-primary text-primary-foreground rounded-full p-0.5 border-2 border-background">
                            <Check className="h-2 w-2" />
                          </div>
                        )}
                      </div>
                      <span className={cn(
                        "text-sm flex-1 text-left truncate",
                        isSelected ? "text-primary font-medium" : "text-foreground"
                      )}>
                        {friend.display_name}
                      </span>
                    </button>
                  )
                })
              )}
            </div>
          </div>
        </div>

        <DialogFooter className="p-4 bg-muted/30 border-t">
          <Button variant="ghost" onClick={() => onOpenChange(false)}>Hủy</Button>
          <Button 
            onClick={handleCreate} 
            loading={loading}
            disabled={!name.trim() || selectedIds.length < 2}
          >
            Tạo nhóm
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
