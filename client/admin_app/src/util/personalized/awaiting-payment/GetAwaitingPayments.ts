import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingPayment } from './AwaitingPayment'
import { GetAwaitingPaymentsResp } from './GetAwaitingPaymentsResp'

export async function getAwaitingPayments (page: number, perPage: number): Promise<GetAwaitingPaymentsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/awaiting-payments?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { awaiting_payments: AwaitingPayment[] }
  return GetAwaitingPaymentsResp.create(result.awaiting_payments)
}
