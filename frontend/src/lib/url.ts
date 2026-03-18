import { env } from '@/lib/env'

export function resolveAssetUrl(input: string) {
  if (!input) return input
  if (input.startsWith('http://') || input.startsWith('https://')) return input

  const api = new URL(env.apiBaseUrl)
  return `${api.origin}${input.startsWith('/') ? input : `/${input}`}`
}
