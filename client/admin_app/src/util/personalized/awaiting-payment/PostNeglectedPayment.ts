import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostNeglectedPaymentResp } from './PostNeglectedPaymentResp'

export async function postNeglectedPayment (consultationId: number): Promise<PostNeglectedPaymentResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { consultation_id: consultationId }
  const response = await fetch('/admin/api/neglected-payment', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostNeglectedPaymentResp.create()
}
