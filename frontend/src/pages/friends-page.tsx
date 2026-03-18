import { useEffect } from 'react'
import { FriendManager } from '@/components/friends/friend-manager'
import { useFriends } from '@/hooks/use-friends'

export function FriendsPage() {
  const { loadFriends, loadRequests } = useFriends()

  useEffect(() => {
    void Promise.all([loadFriends(), loadRequests()])
  }, [loadFriends, loadRequests])

  return (
    <div className="space-y-4">
      <h2 className="text-xl font-semibold">Quản lý bạn bè</h2>
      <FriendManager />
    </div>
  )
}
