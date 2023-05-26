import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetRatingInfoByConsultantIdResp } from './GetRatingInfoByConsultantIdResp'
import { RatingInfoResult } from './RatingInfoResult'

export async function getRatingInfoByConsultantId (consultantId: string): Promise<GetRatingInfoByConsultantIdResp | ApiErrorResp> {
  const params = { consultant_id: consultantId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/rating-info-by-consultant-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const ratingInfoResult = await response.json() as RatingInfoResult
  return GetRatingInfoByConsultantIdResp.create(ratingInfoResult)
}
