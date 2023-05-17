import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetTenantIdByUserAccountIdResp } from './GetTenantIdByUserAccountIdResp'
import { TenantIdResult } from './TenantIdResult'

export async function getTenantIdByUserAccountId (userAccountId: string): Promise<GetTenantIdByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/tenant-id-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const tenantIdResult = await response.json() as TenantIdResult
  return GetTenantIdByUserAccountIdResp.create(tenantIdResult)
}
