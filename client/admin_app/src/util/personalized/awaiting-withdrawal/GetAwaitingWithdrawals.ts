import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingWithdrawal } from './AwaitingWithdrawal'
import { GetAwaitingWithdrawalsResp } from './GetAwaitingWithdrawalsResp'

export async function getAwaitingWithdrawals (page: number, perPage: number): Promise<GetAwaitingWithdrawalsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/awaiting-withdrawals?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { awaiting_withdrawals: AwaitingWithdrawal[] }
  return GetAwaitingWithdrawalsResp.create(result.awaiting_withdrawals)
}
