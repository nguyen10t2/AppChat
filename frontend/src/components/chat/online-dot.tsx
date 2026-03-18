import { cn } from '@/lib/utils'

export function OnlineDot({ online }: { online: boolean }) {
  return (
    <span
      className={cn(
        'inline-block h-2.5 w-2.5 rounded-full',
        online ? 'status-online' : 'status-offline',
      )}
    />
  )
}
