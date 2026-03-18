import { z } from 'zod'
import type { ReactNode } from 'react'
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

const schema = z
  .object({
    username: z.string().min(3, 'Tối thiểu 3 ký tự'),
    display_name: z.string().min(1, 'Không được để trống'),
    email: z.email('Email không hợp lệ'),
    password: z.string().min(6, 'Tối thiểu 6 ký tự'),
    confirmPassword: z.string().min(6, 'Tối thiểu 6 ký tự'),
  })
  .refine((value) => value.password === value.confirmPassword, {
    path: ['confirmPassword'],
    message: 'Mật khẩu xác nhận không khớp',
  })

type FormValue = z.infer<typeof schema>

export function RegisterPage() {
  const navigate = useNavigate()
  const { signUp, signIn } = useAuth()

  const form = useForm<FormValue>({
    resolver: zodResolver(schema),
    defaultValues: {
      username: '',
      display_name: '',
      email: '',
      password: '',
      confirmPassword: '',
    },
  })

  const onSubmit = form.handleSubmit(async (values) => {
    try {
      await signUp({
        username: values.username,
        display_name: values.display_name,
        email: values.email,
        password: values.password,
      })

      await signIn({
        username: values.username,
        password: values.password,
      })

      toast.success('Đăng ký thành công')
      navigate('/chat', { replace: true })
    } catch (error) {
      toast.error(extractErrorMsg(error))
    }
  })

  return (
    <AuthCard
      title="Đăng ký"
      subtitle="Tạo tài khoản mới để bắt đầu chat"
      footerText="Đã có tài khoản?"
      footerActionText="Đăng nhập"
      footerActionTo="/login"
    >
      <form className="space-y-3" onSubmit={onSubmit}>
        <Field
          id="username"
          label="Username"
          message={form.formState.errors.username?.message}
        >
          <Input id="username" {...form.register('username')} />
        </Field>

        <Field
          id="display_name"
          label="Tên hiển thị"
          message={form.formState.errors.display_name?.message}
        >
          <Input id="display_name" {...form.register('display_name')} />
        </Field>

        <Field id="email" label="Email" message={form.formState.errors.email?.message}>
          <Input id="email" type="email" {...form.register('email')} />
        </Field>

        <Field
          id="password"
          label="Mật khẩu"
          message={form.formState.errors.password?.message}
        >
          <Input id="password" type="password" {...form.register('password')} />
        </Field>

        <Field
          id="confirmPassword"
          label="Xác nhận mật khẩu"
          message={form.formState.errors.confirmPassword?.message}
        >
          <Input id="confirmPassword" type="password" {...form.register('confirmPassword')} />
        </Field>

        <Button type="submit" className="w-full" disabled={form.formState.isSubmitting}>
          {form.formState.isSubmitting ? 'Đang xử lý...' : 'Tạo tài khoản'}
        </Button>
      </form>
    </AuthCard>
  )
}

function Field({
  id,
  label,
  message,
  children,
}: {
  id: string
  label: string
  message?: string
  children: ReactNode
}) {
  return (
    <div className="space-y-1">
      <Label htmlFor={id}>{label}</Label>
      {children}
      {message ? <p className="text-xs text-destructive">{message}</p> : null}
    </div>
  )
}
