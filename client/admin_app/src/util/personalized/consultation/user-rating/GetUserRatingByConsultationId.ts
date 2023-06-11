import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetUserRatingByConsultationIdResp } from './GetUserRatingByConsultationIdResp'
import { UserRatingResult } from './UserRatingResult'

export async function getUserRatingByConsultationId (consultationId: string): Promise<GetUserRatingByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/user-rating-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationsResult = await response.json() as UserRatingResult
  return GetUserRatingByConsultationIdResp.create(consultationsResult)
}
