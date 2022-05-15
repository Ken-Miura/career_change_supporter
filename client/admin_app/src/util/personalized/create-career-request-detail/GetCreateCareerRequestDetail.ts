import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { CreateCareerRequestDetail } from './CreateCareerRequestDetail'
import { GetCreateCareerRequestDetailResp } from './GetCreateCareerRequestDetailResp'

export async function getCreateCareerRequestDetail (createCreerReqId: string): Promise<GetCreateCareerRequestDetailResp | ApiErrorResp> {
  const params = { create_career_req_id: createCreerReqId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/create-career-request-detail?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const detail = await response.json() as CreateCareerRequestDetail
  return GetCreateCareerRequestDetailResp.create(detail)
}
