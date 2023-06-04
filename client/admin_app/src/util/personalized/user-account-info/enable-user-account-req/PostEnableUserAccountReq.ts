import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostEnableUserAccountReqResp } from './PostEnableUserAccountReqResp'
import { UserAccountRetrievalResult } from '../UserAccountRetrievalResult'

export async function postEnableUserAccountReq (userAccountId: number): Promise<PostEnableUserAccountReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/enable-user-account-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as UserAccountRetrievalResult
  return PostEnableUserAccountReqResp.create(result)
}
