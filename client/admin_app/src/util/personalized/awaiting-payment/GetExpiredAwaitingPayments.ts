import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingPayment } from './AwaitingPayment'
import { GetExpiredAwaitingPaymentsResp } from './GetExpiredAwaitingPaymentsResp'

export async function getExpiredAwaitingPayments (page: number, perPage: number): Promise<GetExpiredAwaitingPaymentsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/expired-awaiting-payments?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { awaiting_payments: AwaitingPayment[] }
  return GetExpiredAwaitingPaymentsResp.create(result.awaiting_payments)
}
