import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostDisableUserAccountReqResp } from './PostDisableUserAccountReqResp'
import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export async function postDisableUserAccountReq (userAccountId: number): Promise<PostDisableUserAccountReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/disable-user-account-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as UserAccountRetrievalResult
  return PostDisableUserAccountReqResp.create(result)
}
