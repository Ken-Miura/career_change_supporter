import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { RefundedPayment } from '../RefundedPayment'
import { GetRefundedPaymentsResp } from './GetRefundedPaymentsResp'

export async function getRefundedPayments (page: number, perPage: number): Promise<GetRefundedPaymentsResp | ApiErrorResp> {
  /* eslint-disable camelcase */
  const params = { page: page.toString(), per_page: perPage.toString() }
  /* eslint-enable camelcase */
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/refunded-payments?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { refunded_payments: RefundedPayment[] }
  return GetRefundedPaymentsResp.create(result.refunded_payments)
}
