import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostDisableMfaReqResp } from './PostDisableMfaReqResp'
import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export async function postDisableMfaReq (userAccountId: string): Promise<PostDisableMfaReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/disable-mfa-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as UserAccountRetrievalResult
  return PostDisableMfaReqResp.create(result)
}
