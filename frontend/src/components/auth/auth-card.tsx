import { Link } from 'react-router-dom'
import type { ReactNode } from 'react'
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'

type AuthCardProps = {
  title: string
  subtitle: string
  footerText: string
  footerActionText: string
  footerActionTo: string
  children: ReactNode
}

export function AuthCard(props: AuthCardProps) {
  return (
    <div className="grid min-h-screen place-items-center bg-gradient-to-br from-background via-background to-primary/10 p-4">
      <Card className="w-full max-w-md border-border/60 bg-card/90 backdrop-blur">
        <CardHeader>
          <CardTitle className="text-lg">{props.title}</CardTitle>
          <p className="text-xs text-muted-foreground">{props.subtitle}</p>
        </CardHeader>
        <CardContent className="space-y-4">
          {props.children}
          <p className="text-xs text-muted-foreground">
            {props.footerText}{' '}
            <Link to={props.footerActionTo} className="text-primary hover:underline">
              {props.footerActionText}
            </Link>
          </p>
        </CardContent>
      </Card>
    </div>
  )
}
