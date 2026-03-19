import { useState, useRef } from 'react'
import type { Conversation } from '@/types/chat'
import { useAuthStore } from '@/stores/auth.store'
import { useChatStore } from '@/stores/chat.store'
import { useFriendStore } from '@/stores/friend.store'
import { conversationService } from '@/services/conversation.service'
import { fileUploadService } from '@/services/file-upload.service'
import { GroupAvatar } from '@/components/chat/group-avatar'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'
import { 
  Pencil, 
  UserPlus, 
  UserMinus, 
  LogOut, 
  X,
  Camera,
  Plus
} from 'lucide-react'
import { toast } from 'sonner'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { extractErrorMsg } from '@/lib/api'

interface Props {
  conversation: Conversation
  onClose: () => void
}

export function GroupInfoPanel({ conversation, onClose }: Props) {
  const myUser = useAuthStore((state) => state.user)
  const { updateConversation, loadConversations, setActiveConversationId } = useChatStore()
  const { friends, loadFriends } = useFriendStore()

  const [isEditingName, setIsEditingName] = useState(false)
  const [newName, setNewName] = useState(conversation.group_info?.name || '')
  const [loading, setLoading] = useState(false)
  const [showAddMember, setShowAddMember] = useState(false)

  const fileInputRef = useRef<HTMLInputElement>(null)

  const isCreator = myUser?.id === conversation.group_info?.created_by

  const handleUpdateName = async () => {
    if (!newName.trim() || newName === conversation.group_info?.name) {
      setIsEditingName(false)
      return
    }

    setLoading(true)
    try {
      await conversationService.updateGroup(conversation.conversation_id, {
        name: newName.trim()
      })
      updateConversation(conversation.conversation_id, {
        group_info: { ...conversation.group_info!, name: newName.trim() }
      })
      toast.success('Đổi tên nhóm thành công')
      setIsEditingName(false)
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  const handleAvatarClick = () => {
    if (isCreator) {
      fileInputRef.current?.click()
    }
  }

  const handleFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    setLoading(true)
    try {
      const { url } = await fileUploadService.upload(file)
      await conversationService.updateGroup(conversation.conversation_id, {
        avatar_url: url
      })
      updateConversation(conversation.conversation_id, {
        group_info: { ...conversation.group_info!, avatar_url: url }
      })
      toast.success('Cập nhật ảnh đại diện nhóm thành công')
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  const handleKick = async (userId: string) => {
    if (!confirm('Bạn có chắc muốn xóa thành viên này khỏi nhóm?')) return
    
    setLoading(true)
    try {
      await conversationService.removeMember(conversation.conversation_id, userId)
      toast.success('Đã xóa thành viên')
      // Đợi WS update hoặc load lại
      await loadConversations()
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  const handleLeave = async () => {
    if (!confirm('Bạn có chắc muốn rời khỏi nhóm này?')) return

    setLoading(true)
    try {
      await conversationService.removeMember(conversation.conversation_id, myUser!.id)
      toast.success('Đã rời khỏi nhóm')
      setActiveConversationId(null)
      await loadConversations()
      onClose()
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  const handleAddMember = async (userId: string) => {
    setLoading(true)
    try {
      await conversationService.addMember(conversation.conversation_id, userId)
      toast.success('Đã thêm thành viên')
      await loadConversations()
    } catch (error) {
      toast.error(extractErrorMsg(error))
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex flex-col h-full w-full bg-card border-l border-border/60">
      <div className="flex items-center justify-between p-4 border-b border-border/60">
        <h3 className="font-semibold text-sm text-foreground">Thông tin nhóm</h3>
        <Button variant="ghost" size="icon-sm" onClick={onClose}>
          <X size={18} />
        </Button>
      </div>

      <ScrollArea className="flex-1">
        <div className="p-6 flex flex-col items-center gap-4">
          <div className="relative group cursor-pointer" onClick={handleAvatarClick}>
            <GroupAvatar 
              avatarUrl={conversation.group_info?.avatar_url}
              participants={conversation.participants}
              size="lg"
              className="h-20 w-20"
            />
            {isCreator && (
              <div className="absolute inset-0 bg-black/40 rounded-full flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity">
                <Camera className="text-white h-6 w-6" />
              </div>
            )}
            <input 
              type="file" 
              ref={fileInputRef} 
              className="hidden" 
              accept="image/*" 
              onChange={handleFileChange}
            />
          </div>

          <div className="w-full text-center space-y-2">
            {isEditingName ? (
              <div className="flex items-center gap-2">
                <Input 
                  value={newName} 
                  onChange={(e) => setNewName(e.target.value)}
                  className="h-9"
                  autoFocus
                />
                <Button size="sm" onClick={handleUpdateName} loading={loading}>Lưu</Button>
                <Button size="sm" variant="ghost" onClick={() => setIsEditingName(false)}>Hủy</Button>
              </div>
            ) : (
              <div className="flex items-center justify-center gap-2">
                <h4 className="text-lg font-bold">{conversation.group_info?.name}</h4>
                {isCreator && (
                  <Button variant="ghost" size="icon-xs" onClick={() => setIsEditingName(true)}>
                    <Pencil size={14} />
                  </Button>
                )}
              </div>
            )}
            <p className="text-xs text-muted-foreground">{conversation.participants.length} thành viên</p>
          </div>
        </div>

        <Separator />

        <div className="p-4 space-y-4">
          <div className="flex items-center justify-between">
            <h5 className="text-xs font-semibold uppercase tracking-wider text-muted-foreground">Thành viên</h5>
            {isCreator && (
              <Button 
                variant="ghost" 
                size="sm" 
                className="h-7 text-[10px] gap-1"
                onClick={() => {
                  loadFriends()
                  setShowAddMember(!showAddMember)
                }}
              >
                <UserPlus size={12} /> Thêm
              </Button>
            )}
          </div>

          {showAddMember && (
            <div className="space-y-2 p-2 bg-muted/30 rounded-lg border border-border/40 animate-in fade-in slide-in-from-top-1">
              <p className="text-[10px] font-medium text-muted-foreground uppercase px-1">Chọn bạn bè để thêm</p>
              <div className="max-h-[150px] overflow-y-auto space-y-1">
                {friends
                  .filter(f => !conversation.participants.some(p => p.user_id === f.id))
                  .map(friend => (
                    <button
                      key={friend.id}
                      onClick={() => handleAddMember(friend.id)}
                      className="flex w-full items-center gap-2 p-1.5 hover:bg-primary/5 rounded-md transition-colors text-left group"
                    >
                      <Avatar className="h-6 w-6">
                        <AvatarImage src={friend.avatar_url || undefined} />
                        <AvatarFallback className="text-[10px]">{friend.display_name[0]}</AvatarFallback>
                      </Avatar>
                      <span className="text-xs flex-1 truncate">{friend.display_name}</span>
                      <Plus className="h-3 w-3 text-muted-foreground opacity-0 group-hover:opacity-100" />
                    </button>
                  ))
                }
                {friends.filter(f => !conversation.participants.some(p => p.user_id === f.id)).length === 0 && (
                  <p className="text-[10px] text-center py-2 text-muted-foreground italic">Không có bạn bè mới để thêm</p>
                )}
              </div>
            </div>
          )}

          <div className="space-y-1">
            {conversation.participants.map((participant) => (
              <div key={participant.user_id} className="flex items-center gap-3 p-2 hover:bg-muted/40 rounded-lg group">
                <Avatar className="h-8 w-8">
                  <AvatarImage src={participant.avatar_url || undefined} />
                  <AvatarFallback className="text-xs">{participant.display_name?.[0]}</AvatarFallback>
                </Avatar>
                <div className="flex-1 min-w-0">
                  <p className="text-sm font-medium truncate">{participant.display_name}</p>
                  <p className="text-[10px] text-muted-foreground">
                    {participant.user_id === conversation.group_info?.created_by ? 'Trưởng nhóm' : 'Thành viên'}
                  </p>
                </div>
                {isCreator && participant.user_id !== myUser?.id && (
                  <Button 
                    variant="ghost" 
                    size="icon-xs" 
                    className="opacity-0 group-hover:opacity-100 text-destructive hover:bg-destructive/10"
                    onClick={() => handleKick(participant.user_id)}
                  >
                    <UserMinus size={14} />
                  </Button>
                )}
              </div>
            ))}
          </div>
        </div>
      </ScrollArea>

      <div className="p-4 border-t border-border/60">
        <Button 
          variant="destructive" 
          className="w-full h-9 text-xs gap-2" 
          onClick={handleLeave}
          loading={loading}
        >
          <LogOut size={14} /> Rời khỏi nhóm
        </Button>
      </div>
    </div>
  )
}
