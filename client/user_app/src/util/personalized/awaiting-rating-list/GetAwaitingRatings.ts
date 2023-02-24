import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingRatings } from './AwaitingRatings'
import { AwaitingRatingsResp } from './AwaitingRatingsResp'

export async function getAwaitingRatings (): Promise<AwaitingRatingsResp | ApiErrorResp> {
  const response = await fetch('/api/awaiting-ratings', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as AwaitingRatings
  return AwaitingRatingsResp.create(result)
}
