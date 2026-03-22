import axios, { type AxiosResponse, type AxiosError } from 'axios'

export type ApiSuccess<T> = {
  data?: T
  message?: string
}

export type ApiErrorResponse = {
  code: string
  message: string
}

export function unwrapData<T>(response: AxiosResponse<ApiSuccess<T>>): T {
  return response.data.data as T
}

export function extractErrorMsg(error: unknown): string {
  if (axios.isAxiosError(error)) {
    const axErr = error as AxiosError<ApiErrorResponse>
    const errData = axErr.response?.data

    if (errData?.message) {
      return errData.message
    }

    return axErr.message || 'Lỗi kết nối máy chủ'
  }


  if (error instanceof Error) {
    return error.message
  }

  return 'Lỗi không xác định'
}
