import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UserAccountRetrievalResp } from './UserAccountRetrievalResp'
import { UserAccountRetrievalResult } from './UserAccountRetrievalResult'

export async function postUserAccountRetrievalByUserAccountId (userAccountId: number): Promise<UserAccountRetrievalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/user-account-retrieval-by-user-account-id', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as UserAccountRetrievalResult
  return UserAccountRetrievalResp.create(result)
}
