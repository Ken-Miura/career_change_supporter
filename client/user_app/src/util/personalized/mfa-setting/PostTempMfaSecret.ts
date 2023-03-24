import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostTempMfaSecretResp } from './PostTempMfaSecretResp'

export async function postTempMfaSecret (): Promise<PostTempMfaSecretResp | ApiErrorResp> {
  const response = await fetch('/api/temp-mfa-secret', {
    method: 'POST'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostTempMfaSecretResp.create()
}
