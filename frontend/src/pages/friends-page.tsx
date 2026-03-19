import { useEffect } from 'react'
import { FriendManager } from '@/components/friends/friend-manager'
import { useFriends } from '@/hooks/use-friends'

export function FriendsPage() {
  const { loadFriends, loadRequests } = useFriends()

  useEffect(() => {
    void Promise.all([loadFriends(), loadRequests()])
  }, [loadFriends, loadRequests])

  return (
    <div className="flex h-full w-full overflow-hidden">
      <FriendManager />
    </div>
  )
}
