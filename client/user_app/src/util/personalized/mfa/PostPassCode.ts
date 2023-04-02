import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostPassCodeResp } from './PostPassCodeResp'

export async function postPassCode (passCode: string): Promise<PostPassCodeResp | ApiErrorResp> {
  const data = { pass_code: passCode }
  const response = await fetch('/api/pass-code', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostPassCodeResp.create()
}
