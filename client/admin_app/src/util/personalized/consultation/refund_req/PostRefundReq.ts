import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostRefundReqResp } from './PostRefundReqResp'

export async function postRefundReq (receiptId: number): Promise<PostRefundReqResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { receipt_id: receiptId }
  const response = await fetch('/admin/api/refund-req', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostRefundReqResp.create()
}
