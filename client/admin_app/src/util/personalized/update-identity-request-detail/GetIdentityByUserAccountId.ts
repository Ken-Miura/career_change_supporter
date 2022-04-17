import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Identity } from './Identity'
import { GetIdentityByUserAccountIdResp } from './GetIdentityByUserAccountIdResp'

export async function getIdentityByUserAccountId (userAccountId: string): Promise<GetIdentityByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const detail = await response.json() as Identity
  return GetIdentityByUserAccountIdResp.create(detail)
}
