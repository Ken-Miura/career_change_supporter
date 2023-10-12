import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostRefundFromAwaitingWithdrawalResp } from './PostRefundFromAwaitingWithdrawalResp'

export async function postRefundFromAwaitingWithdrawal (consultationId: number): Promise<PostRefundFromAwaitingWithdrawalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { consultation_id: consultationId }
  const response = await fetch('/admin/api/left-awaiting-withdrawal', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostRefundFromAwaitingWithdrawalResp.create()
}
