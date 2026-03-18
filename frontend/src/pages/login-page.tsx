import { z } from 'zod'
import { zodResolver } from '@hookform/resolvers/zod'
import { useForm } from 'react-hook-form'
import { useNavigate } from 'react-router-dom'
import { toast } from 'sonner'
import { AuthCard } from '@/components/auth/auth-card'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useAuth } from '@/hooks/use-auth'
import { extractErrorMsg } from '@/lib/api'

const schema = z.object({
  username: z.string().min(3, 'Tối thiểu 3 ký tự'),
  password: z.string().min(6, 'Tối thiểu 6 ký tự'),
})

type FormValue = z.infer<typeof schema>

export function LoginPage() {
  const navigate = useNavigate()
  const { signIn } = useAuth()

  const form = useForm<FormValue>({
    resolver: zodResolver(schema),
    defaultValues: {
      username: '',
      password: '',
    },
  })

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      await signIn(values)
      toast.success('Đăng nhập thành công')
      navigate('/chat', { replace: true })
    } catch (error) {
      toast.error(extractErrorMsg(error))
    }
  })

  return (
    <AuthCard
      title="Đăng nhập"
      subtitle="Chào mừng bạn quay lại AppChat"
      footerText="Chưa có tài khoản?"
      footerActionText="Đăng ký"
      footerActionTo="/register"
    >
      <form className="space-y-3" onSubmit={onSubmit}>
        <div className="space-y-1">
          <Label htmlFor="username">Username</Label>
          <Input id="username" {...form.register('username')} />
          <FormError message={form.formState.errors.username?.message} />
        </div>

        <div className="space-y-1">
          <Label htmlFor="password">Mật khẩu</Label>
          <Input id="password" type="password" {...form.register('password')} />
          <FormError message={form.formState.errors.password?.message} />
        </div>

        <Button type="submit" className="w-full" disabled={form.formState.isSubmitting}>
          {form.formState.isSubmitting ? 'Đang đăng nhập...' : 'Đăng nhập'}
        </Button>
      </form>
    </AuthCard>
  )
}

function FormError({ message }: { message?: string }) {
  if (!message) return null
  return <p className="text-xs text-destructive">{message}</p>
}
