import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostUserRatingResp } from './PostUserRatingResp'

export async function postUserRating (userRatingId: number, rating: number): Promise<PostUserRatingResp | ApiErrorResp> {
  const data = { 'user-rating-id': userRatingId, rating }
  const response = await fetch('/api/user-rating', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostUserRatingResp.create()
}
