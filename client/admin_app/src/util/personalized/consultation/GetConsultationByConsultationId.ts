import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultationByConsultationIdResp } from './GetConsultationByConsultationIdResp'
import { ConsultationResult } from './ConsultationResult'

export async function getConsultationByConsultationId (consultationId: string): Promise<GetConsultationByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/consultation-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationsResult = await response.json() as ConsultationResult
  return GetConsultationByConsultationIdResp.create(consultationsResult)
}
