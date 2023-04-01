import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetTempMfaSecretResp } from './GetTempMfaSecretResp'
import { TempMfaSecret } from './TempMfaSecret'

export async function getTempMfaSecret (): Promise<GetTempMfaSecretResp | ApiErrorResp> {
  const response = await fetch('/api/temp-mfa-secret', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as TempMfaSecret
  return GetTempMfaSecretResp.create(result)
}
