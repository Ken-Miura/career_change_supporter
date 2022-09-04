import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetFeePerHourInYenForApplicationResp } from './GetFeePerHourInYenForApplicationResp'

export async function getFeePerHourInYenForApplication (consultantId: string): Promise<GetFeePerHourInYenForApplicationResp | ApiErrorResp> {
  const params = { consultant_id: consultantId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/fee-per-hour-in-yen-for-application?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as { fee_per_hour_in_yen: number }
  return GetFeePerHourInYenForApplicationResp.create(result.fee_per_hour_in_yen)
}
