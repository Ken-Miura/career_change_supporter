import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetSettlementByConsultationIdResp } from './GetSettlementByConsultationIdResp'
import { SettlementResult } from './SettlementResult'

export async function getSettlementByConsultationId (consultationId: string): Promise<GetSettlementByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/settlement-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as SettlementResult
  return GetSettlementByConsultationIdResp.create(result)
}
