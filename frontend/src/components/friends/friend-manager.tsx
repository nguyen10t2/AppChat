import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { Input } from '@/components/ui/input'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Separator } from '@/components/ui/separator'
import { useFriends } from '@/hooks/use-friends'
import { conversationService } from '@/services/conversation.service'

function getInfoLabel(obj: { Id?: string; Info?: { display_name: string } }) {
  if (obj.Info) return obj.Info.display_name
  return obj.Id ? `ID: ${obj.Id.slice(0, 8)}` : 'Unknown'
}

export function FriendManager() {
  const navigate = useNavigate()
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
  const [keyword, setKeyword] = useState('')

  return (
    <div className="grid gap-4 lg:grid-cols-3">
      <Card>
        <CardHeader>
          <CardTitle>Tìm người dùng</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3">
          <Input
            value={keyword}
            onChange={(event) => {
              const value = event.target.value
              setKeyword(value)
              void searchUsers(value)
            }}
            placeholder="Nhập tên hoặc username..."
          />

          <ScrollArea className="h-80">
            <div className="space-y-2 pr-2">
              {searchResult.map((user) => (
                <div
                  key={user.id}
                  className="flex items-center justify-between border border-border/60 p-2"
                >
                  <div>
                    <p className="text-sm font-medium">{user.display_name}</p>
                    <p className="text-xs text-muted-foreground">@{user.username}</p>
                  </div>
                  <Button
                    size="sm"
                    onClick={async () => {
                      await sendRequest(user.id)
                      toast.success('Đã gửi lời mời kết bạn')
                    }}
                  >
                    Kết bạn
                  </Button>
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Lời mời kết bạn</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-80">
            <div className="space-y-3 pr-2">
              {requests.map((request) => (
                <div key={request.id} className="border border-border/60 p-2">
                  <p className="text-xs text-muted-foreground">
                    {getInfoLabel(request.from)} → {getInfoLabel(request.to)}
                  </p>
                  {request.message ? <p className="mt-1 text-sm">{request.message}</p> : null}

                  <div className="mt-2 flex gap-2">
                    <Button
                      size="sm"
                      onClick={async () => {
                        await acceptRequest(request.id)
                        toast.success('Đã chấp nhận lời mời')
                      }}
                    >
                      Chấp nhận
                    </Button>
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={async () => {
                        await declineRequest(request.id)
                        toast.success('Đã từ chối lời mời')
                      }}
                    >
                      Từ chối
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Danh sách bạn bè</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-80">
            <div className="space-y-2 pr-2">
              {friends.map((friend) => (
                <div key={friend.id} className="border border-border/60 p-2">
                  <p className="text-sm font-medium">{friend.display_name}</p>
                  <p className="text-xs text-muted-foreground">@{friend.username}</p>
                  <Separator className="my-2" />
                  <div className="flex gap-2">
                    <Button
                      size="sm"
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
                      variant="outline"
                      onClick={async () => {
                        await removeFriend(friend.id)
                        toast.success('Đã hủy kết bạn')
                      }}
                    >
                      Xóa
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>
    </div>
  )
}
