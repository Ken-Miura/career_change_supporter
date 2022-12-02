import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationRequestDetail } from './ConsultationRequestDetail'
import { GetConsultationRequestDetailResp } from './GetConsultationRequestDetailResp'

export async function getConsultationRequestDetail (consultationRequestId: string): Promise<GetConsultationRequestDetailResp | ApiErrorResp> {
  const params = { consultation_req_id: consultationRequestId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/consultation-request-detail?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as ConsultationRequestDetail
  return GetConsultationRequestDetailResp.create(result)
}
