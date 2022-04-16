import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UpdateIdentityRequestItem } from './UpdateIdentityRequestItem'
import { GetUpdateIdentityRequestsResp } from './GetUpdateIdentityRequestsResp'

export async function getUpdateIdentityRequests (page: number, perPage: number): Promise<GetUpdateIdentityRequestsResp | ApiErrorResp> {
  const params = { page: page.toString(), per_page: perPage.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/update-identity-requests?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const items = await response.json() as UpdateIdentityRequestItem[]
  items.forEach(e => {
    const utcTime = e.requested_at
    e.requested_at = new Date(utcTime.toLocaleString())
  })
  return GetUpdateIdentityRequestsResp.create(items)
}
