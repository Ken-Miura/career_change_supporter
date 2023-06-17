import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostMakePaymentReqResp } from './PostMakePaymentReqResp'

export async function postMakePaymentReq (settlementId: number): Promise<PostMakePaymentReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { settlement_id: settlementId }
  const response = await fetch('/admin/api/make-payment-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostMakePaymentReqResp.create()
}
