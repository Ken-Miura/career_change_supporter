import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetRatingInfoByUserAccountIdResp } from './GetRatingInfoByUserAccountIdResp'
import { RatingInfoResult } from './RatingInfoResult'

export async function getRatingInfoByUserAccountId (userAccountId: string): Promise<GetRatingInfoByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/rating-info-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const ratingInfoResult = await response.json() as RatingInfoResult
  return GetRatingInfoByUserAccountIdResp.create(ratingInfoResult)
}
