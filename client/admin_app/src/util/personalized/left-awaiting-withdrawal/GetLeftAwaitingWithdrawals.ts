import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { LeftAwaitingWithdrawal } from '../LeftAwaitingWithdrawal'
import { GetLeftAwaitingWithdrawalsResp } from './GetLeftAwaitingWithdrawalsResp'

export async function getLeftAwaitingWithdrawals (page: number, perPage: number): Promise<GetLeftAwaitingWithdrawalsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/left-awaiting-withdrawals?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { left_awaiting_withdrawals: LeftAwaitingWithdrawal[] }
  return GetLeftAwaitingWithdrawalsResp.create(result.left_awaiting_withdrawals)
}
