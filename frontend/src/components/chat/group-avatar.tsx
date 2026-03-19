import type { Participant } from '@/types/chat'
import { cn } from '@/lib/utils'
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar'

interface GroupAvatarProps {
  avatarUrl?: string | null
  participants: Participant[]
  className?: string
  size?: 'sm' | 'md' | 'lg'
}

export function GroupAvatar({ avatarUrl, participants, className, size = 'md' }: GroupAvatarProps) {
  const sizeClasses = {
    sm: 'h-8 w-8',
    md: 'h-10 w-10',
    lg: 'h-12 w-12',
  }

  if (avatarUrl) {
    return (
      <Avatar className={cn(sizeClasses[size], className)}>
        <AvatarImage src={avatarUrl} />
        <AvatarFallback>{participants[0]?.display_name?.[0] || 'G'}</AvatarFallback>
      </Avatar>
    )
  }

  // Zalo-style composite avatar: max 4 members
  const displayParticipants = participants.slice(0, 4)
  const count = displayParticipants.length

  if (count === 0) {
    return (
      <div className={cn(sizeClasses[size], 'bg-muted rounded-full flex items-center justify-center', className)}>
        <span className="text-muted-foreground text-xs">G</span>
      </div>
    )
  }

  if (count === 1) {
    const p = displayParticipants[0]
    return (
      <Avatar className={cn(sizeClasses[size], className)}>
        <AvatarImage src={p.avatar_url || undefined} />
        <AvatarFallback>{p.display_name?.[0] || '?'}</AvatarFallback>
      </Avatar>
    )
  }

  return (
    <div
      className={cn(
        sizeClasses[size],
        'relative rounded-full overflow-hidden bg-muted flex flex-wrap gap-0.5 p-0.5',
        className
      )}
    >
      {displayParticipants.map((p, index) => {
        let width = 'w-full'
        let height = 'h-full'

        if (count === 2) {
          width = 'w-[calc(50%-1px)]'
          height = 'h-full'
        } else if (count === 3) {
          if (index === 0) {
            width = 'w-full'
            height = 'h-[calc(50%-1px)]'
          } else {
            width = 'w-[calc(50%-1px)]'
            height = 'h-[calc(50%-1px)]'
          }
        } else if (count === 4) {
          width = 'w-[calc(50%-1px)]'
          height = 'h-[calc(50%-1px)]'
        }

        return (
          <div
            key={p.user_id}
            className={cn(
              'flex items-center justify-center bg-background/50 overflow-hidden',
              width,
              height,
              count > 1 && 'rounded-sm'
            )}
          >
            {p.avatar_url ? (
              <img src={p.avatar_url} className="w-full h-full object-cover" alt="" />
            ) : (
              <span className="text-[8px] font-bold text-muted-foreground uppercase">
                {p.display_name?.[0]}
              </span>
            )}
          </div>
        )
      })}
    </div>
  )
}
