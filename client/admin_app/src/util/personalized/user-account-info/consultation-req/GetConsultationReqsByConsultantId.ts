import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultationReqsByConsultantIdResp } from './GetConsultationReqsByConsultantIdResp'
import { ConsultationReqsResult } from './ConsultationReqsResult'

export async function getConsultationReqsByConsultantId (consultantId: string): Promise<GetConsultationReqsByConsultantIdResp | ApiErrorResp> {
  const params = { consultant_id: consultantId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/consultation-reqs-by-consultant-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationReqsResult = await response.json() as ConsultationReqsResult
  return GetConsultationReqsByConsultantIdResp.create(consultationReqsResult)
}
