import { ApiErrorResp, ApiError } from '../../ApiError'
import { LogoutResp } from './LogoutResp'

export async function logout (): Promise<LogoutResp | ApiErrorResp> {
  const response = await fetch('/api/logout', {
    method: 'POST'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return LogoutResp.create()
}
