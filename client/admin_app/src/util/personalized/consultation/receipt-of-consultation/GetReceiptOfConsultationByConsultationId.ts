import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ReceiptOfConsultation } from '../../ReceiptOfConsultation'
import { GetReceiptOfConsultationByConsultationIdResp } from './GetReceiptOfConsultationByConsultationIdResp'

export async function getReceiptOfConsultationByConsultationId (consultationId: string): Promise<GetReceiptOfConsultationByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/receipt-of-consultation-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { receipt_of_consultation: ReceiptOfConsultation | null }
  return GetReceiptOfConsultationByConsultationIdResp.create(result.receipt_of_consultation)
}
