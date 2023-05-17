import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetFeePerHourInYenByUserAccountIdResp } from './GetFeePerHourInYenByUserAccountIdResp'
import { FeePerHourInYenResult } from './FeePerHourInYenResult'

export async function getFeePerHourInYenByUserAccountId (userAccountId: string): Promise<GetFeePerHourInYenByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/fee-per-hour-in-yen-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const feePerHourInYenResult = await response.json() as FeePerHourInYenResult
  return GetFeePerHourInYenByUserAccountIdResp.create(feePerHourInYenResult)
}
