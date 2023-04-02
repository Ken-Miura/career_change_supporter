import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostRecoveryCodeResp } from './PostRecoveryCodeResp'

export async function postRecoveryCode (recoveryCode: string): Promise<PostRecoveryCodeResp | ApiErrorResp> {
  const data = { recovery_code: recoveryCode }
  const response = await fetch('/api/recovery-code', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostRecoveryCodeResp.create()
}
