import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostAwaitingWithdrawalResp } from './PostAwaitingWithdrawalResp'

export async function postAwaitingWithdrawal (consultationId: number): Promise<PostAwaitingWithdrawalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { consultation_id: consultationId }
  const response = await fetch('/admin/api/awaiting-withdrawal', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostAwaitingWithdrawalResp.create()
}
