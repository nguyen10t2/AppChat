import { useFriendStore } from '@/stores/friend.store'
import { useShallow } from 'zustand/react/shallow'

export function useFriends() {
  return useFriendStore(
    useShallow((state) => ({
      friends: state.friends,
      requests: state.requests,
      searchResult: state.searchResult,
      loading: state.loading,
      loadFriends: state.loadFriends,
      loadRequests: state.loadRequests,
      searchUsers: state.searchUsers,
      sendRequest: state.sendRequest,
      acceptRequest: state.acceptRequest,
      declineRequest: state.declineRequest,
      removeFriend: state.removeFriend,
    }))
  )
}
