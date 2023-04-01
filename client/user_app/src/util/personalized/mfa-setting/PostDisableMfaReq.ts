import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostDisableMfaReqResp } from './PostDisableMfaReqResp'

export async function postDisableMfaReq (): Promise<PostDisableMfaReqResp | ApiErrorResp> {
  const response = await fetch('/api/disable-mfa-req', {
    method: 'POST'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostDisableMfaReqResp.create()
}
