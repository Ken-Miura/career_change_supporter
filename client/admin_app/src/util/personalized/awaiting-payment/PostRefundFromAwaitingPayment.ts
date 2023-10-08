import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostRefundFromAwaitingPaymentResp } from './PostRefundFromAwaitingPaymentResp'

export async function postRefundFromAwaitingPayment (consultationId: number): Promise<PostRefundFromAwaitingPaymentResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { consultation_id: consultationId }
  const response = await fetch('/admin/api/refund-from-awaiting-payment', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostRefundFromAwaitingPaymentResp.create()
}
