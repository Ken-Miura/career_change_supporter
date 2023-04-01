import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostEnableMfaReqResp } from './PostEnableMfaReqResp'

export async function postEnableMfaReq (passCode: string): Promise<PostEnableMfaReqResp | ApiErrorResp> {
  const data = { pass_code: passCode }
  const response = await fetch('/api/enable-mfa-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as { recovery_code: string }
  return PostEnableMfaReqResp.create(result.recovery_code)
}
