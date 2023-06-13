import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetReceiptByConsultationIdResp } from './GetReceiptByConsultationIdResp'
import { ReceiptResult } from './ReceiptResult'

export async function getReceiptByConsultationId (consultationId: string): Promise<GetReceiptByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/receipt-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as ReceiptResult
  return GetReceiptByConsultationIdResp.create(result)
}
