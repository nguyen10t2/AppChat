export function formatTime(input: string) {
  const date = new Date(input)
  return new Intl.DateTimeFormat('vi-VN', {
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}

export function formatDateTime(input: string | null) {
  if (!input) return 'Không xác định'
  const date = new Date(input)
  return new Intl.DateTimeFormat('vi-VN', {
    day: '2-digit',
    month: '2-digit',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  }).format(date)
}
