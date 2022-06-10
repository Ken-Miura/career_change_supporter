import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostFeePerHourInYenResp } from './PostFeePerHourInYenResp'

export async function postFeePerHourInYen (feePerHourInYen: number): Promise<PostFeePerHourInYenResp | ApiErrorResp> {
  const data = { 'fee-per-hour-in-yen': feePerHourInYen }
  const response = await fetch('/api/fee-per-hour-in-yen', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostFeePerHourInYenResp.create()
}
