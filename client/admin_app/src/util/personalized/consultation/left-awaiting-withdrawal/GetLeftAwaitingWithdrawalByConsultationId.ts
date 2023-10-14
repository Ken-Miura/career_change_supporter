import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { LeftAwaitingWithdrawal } from '../../LeftAwaitingWithdrawal'
import { GetLeftAwaitingWithdrawalByConsultationIdResp } from './GetLeftAwaitingWithdrawalByConsultationIdResp'

export async function getLeftAwaitingWithdrawalByConsultationId (consultationId: string): Promise<GetLeftAwaitingWithdrawalByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/left-awaiting-withdrawal-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { left_awaiting_withdrawal: LeftAwaitingWithdrawal | null }
  return GetLeftAwaitingWithdrawalByConsultationIdResp.create(result.left_awaiting_withdrawal)
}
