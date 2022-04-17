import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UpdateIdentityRequestDetail } from './UpdateIdentityRequestDetail'
import { GetUpdateIdentityRequestDetailResp } from './GetUpdateIdentityRequestDetailResp'

export async function getUpdateIdentityRequestDetail (userAccountId: string): Promise<GetUpdateIdentityRequestDetailResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/update-identity-request-detail?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const detail = await response.json() as UpdateIdentityRequestDetail
  return GetUpdateIdentityRequestDetailResp.create(detail)
}
