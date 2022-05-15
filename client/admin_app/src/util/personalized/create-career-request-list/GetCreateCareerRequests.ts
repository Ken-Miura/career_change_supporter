import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { CreateCareerRequestItem } from './CreateCareerRequestItem'
import { GetCreateCareerRequestsResp } from './GetCreateCareerRequestsResp'

export async function getCreateCareerRequests (page: number, perPage: number): Promise<GetCreateCareerRequestsResp | ApiErrorResp> {
  const params = { page: page.toString(), per_page: perPage.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/create-career-requests?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const items = await response.json() as CreateCareerRequestItem[]
  items.forEach(e => {
    const utcTime = e.requested_at
    e.requested_at = new Date(utcTime.toLocaleString())
  })
  return GetCreateCareerRequestsResp.create(items)
}
