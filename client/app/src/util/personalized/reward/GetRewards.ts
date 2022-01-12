import { ApiErrorResp, ApiError } from '../../ApiError'
import { GetRewardsResp } from './GetRewardsResp'
import { Rewards } from './Rewards'

export async function getRewards (): Promise<GetRewardsResp | ApiErrorResp> {
  const response = await fetch('/api/rewards', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as Rewards
  return GetRewardsResp.create(result)
}
