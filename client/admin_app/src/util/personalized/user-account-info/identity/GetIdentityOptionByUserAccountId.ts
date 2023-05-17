import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityOptionByUserAccountIdResp } from './GetIdentityOptionByUserAccountIdResp'
import { IdentityResult } from './IdentityResult'

export async function getIdentityOptionByUserAccountId (userAccountId: string): Promise<GetIdentityOptionByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-option-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const identityResult = await response.json() as IdentityResult
  return GetIdentityOptionByUserAccountIdResp.create(identityResult)
}
