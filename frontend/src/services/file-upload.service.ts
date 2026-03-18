import { http } from '@/lib/http'
import { unwrapData } from '@/lib/api'

export type UploadedFile = {
  id: string
  filename: string
  original_filename: string
  mime_type: string
  file_size: number
  url: string
  created_at: string
}

export const fileUploadService = {
  async upload(file: File): Promise<UploadedFile> {
    const formData = new FormData()
    formData.append('file', file)

    const response = await http.post('/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    })

    return unwrapData<UploadedFile>(response)
  },
}
