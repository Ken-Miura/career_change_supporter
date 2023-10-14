import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingWithdrawal } from './AwaitingWithdrawal'
import { GetAwaitingWithdrawalByConsultationIdResp } from './GetAwaitingWithdrawalByConsultationIdResp'

export async function getAwaitingWithdrawalByConsultationId (consultationId: string): Promise<GetAwaitingWithdrawalByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/awaiting-withdrawal-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { awaiting_withdrawal: AwaitingWithdrawal | null }
  return GetAwaitingWithdrawalByConsultationIdResp.create(result.awaiting_withdrawal)
}
