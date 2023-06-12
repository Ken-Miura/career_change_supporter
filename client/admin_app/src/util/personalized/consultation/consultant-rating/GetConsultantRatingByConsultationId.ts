import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultantRatingByConsultationIdResp } from './GetConsultantRatingByConsultationIdResp'
import { ConsultantRatingResult } from './ConsultantRatingResult'

export async function getConsultantRatingByConsultationId (consultationId: string): Promise<GetConsultantRatingByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/consultant-rating-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as ConsultantRatingResult
  return GetConsultantRatingByConsultationIdResp.create(result)
}
