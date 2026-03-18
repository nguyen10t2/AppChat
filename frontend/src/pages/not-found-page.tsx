import { Link } from 'react-router-dom'
import { Button } from '@/components/ui/button'

export function NotFoundPage() {
  return (
    <div className="grid min-h-screen place-items-center bg-background p-4">
      <div className="space-y-3 text-center">
        <h1 className="text-2xl font-semibold">404</h1>
        <p className="text-sm text-muted-foreground">Trang bạn tìm không tồn tại.</p>
        <Button asChild>
          <Link to="/chat">Về trang chat</Link>
        </Button>
      </div>
    </div>
  )
}
