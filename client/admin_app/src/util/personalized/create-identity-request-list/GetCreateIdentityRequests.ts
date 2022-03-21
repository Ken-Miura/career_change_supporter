import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { CreateIdentityRequestItem } from './CreateIdentityRequestItem'
import { GetCreateIdentityRequests } from './GetCreateIdentityRequestsResp'

export async function getCreateIdentityRequests (page: number, perPage: number): Promise<GetCreateIdentityRequests | ApiErrorResp> {
  const params = { page: page.toString(), per_page: perPage.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/create-identity-requests?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const items = await response.json() as CreateIdentityRequestItem[]
  items.forEach(e => {
    const utcTime = e.requested_at
    e.requested_at = new Date(utcTime.toLocaleString())
  })
  return GetCreateIdentityRequests.create(items)
}
