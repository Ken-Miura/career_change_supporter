import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetCareersByUserAccountIdResp } from './GetCareersByUserAccountIdResp'
import { CareersResult } from './CareersResult'

export async function getCareersByUserAccountId (userAccountId: string): Promise<GetCareersByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/careers-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const careersResult = await response.json() as CareersResult
  return GetCareersByUserAccountIdResp.create(careersResult)
}
