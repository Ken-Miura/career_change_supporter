import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostConsultantRatingResp } from './PostConsultantRatingResp'

export async function postConsultantRating (consultantRatingId: number, rating: number): Promise<PostConsultantRatingResp | ApiErrorResp> {
  const data = { consultant_rating_id: consultantRatingId, rating }
  const response = await fetch('/api/consultant-rating', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostConsultantRatingResp.create()
}
