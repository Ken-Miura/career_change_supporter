import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostLeftAwaitingWithdrawalResp } from './PostLeftAwaitingWithdrawalResp'

export async function postLeftAwaitingWithdrawal (consultationId: number): Promise<PostLeftAwaitingWithdrawalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { consultation_id: consultationId }
  const response = await fetch('/admin/api/receipt-of-consultation', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostLeftAwaitingWithdrawalResp.create()
}
