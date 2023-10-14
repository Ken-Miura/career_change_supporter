import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { AwaitingPayment } from './AwaitingPayment'
import { GetAwaitingPaymentByConsultationIdResp } from './GetAwaitingPaymentByConsultationIdResp'

export async function getAwaitingPaymentByConsultationId (consultationId: string): Promise<GetAwaitingPaymentByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/awaiting-payment-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { awaiting_payment: AwaitingPayment | null }
  return GetAwaitingPaymentByConsultationIdResp.create(result.awaiting_payment)
}
