import { ApiError, ApiErrorResp } from '../ApiError'
import { AgreeTermsOfUseResp } from './AgreeTermsOfUseResp'

export async function agreeTermsOfUse (): Promise<AgreeTermsOfUseResp | ApiErrorResp> {
  const response = await fetch('/api/agreement', {
    method: 'POST'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return AgreeTermsOfUseResp.create()
}
