import { ApiError, ApiErrorResp } from '../ApiError'
import { CheckAgreementStatusResp } from './CheckAgreementStatusResp'

export async function checkAgreementStatus (): Promise<CheckAgreementStatusResp | ApiErrorResp> {
  const response = await fetch('/api/agreement-status', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return CheckAgreementStatusResp.create()
}
