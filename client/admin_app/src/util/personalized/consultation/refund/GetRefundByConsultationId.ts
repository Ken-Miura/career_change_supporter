import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetRefundByConsultationIdResp } from './GetRefundByConsultationIdResp'
import { RefundResult } from './RefundResult'

export async function getRefundByConsultationId (consultationId: string): Promise<GetRefundByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/refund-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as RefundResult
  return GetRefundByConsultationIdResp.create(result)
}
