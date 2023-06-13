import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetStoppedSettlementByConsultationIdResp } from './GetStoppeSettlementByConsultationIdResp'
import { StoppedSettlementResult } from './StoppedSettlementResult'

export async function getStoppedSettlementByConsultationId (consultationId: string): Promise<GetStoppedSettlementByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/stopped-settlement-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as StoppedSettlementResult
  return GetStoppedSettlementByConsultationIdResp.create(result)
}
