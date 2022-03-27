import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { CreateIdentityRequestDetail } from './CreateIdentityRequestDetail'
import { GetCreateIdentityRequestDetailResp } from './GetCreateIdentityRequestDetailResp'

export async function getCreateIdentityRequestDetail (userAccountId: string): Promise<GetCreateIdentityRequestDetailResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/create-identity-request-detail?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const detail = await response.json() as CreateIdentityRequestDetail
  return GetCreateIdentityRequestDetailResp.create(detail)
}
