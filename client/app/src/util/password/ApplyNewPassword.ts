import { ApiErrorResp, ApiError } from '../ApiError'
import { ApplyNewPasswordResp } from './ApplyNewPasswordResp'

export async function applyNewPassword (jsonData: string): Promise<ApplyNewPasswordResp | ApiErrorResp> {
  const response = await fetch('/api/password-change', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: jsonData
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return ApplyNewPasswordResp.create()
}
