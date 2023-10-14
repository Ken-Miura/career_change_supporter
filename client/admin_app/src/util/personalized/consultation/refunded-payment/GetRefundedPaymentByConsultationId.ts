import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { RefundedPayment } from '../../RefundedPayment'
import { GetRefundedPaymentByConsultationIdResp } from './GetRefundedPaymentByConsultationIdResp'

export async function getRefundedPaymentByConsultationId (consultationId: string): Promise<GetRefundedPaymentByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/refunded-payment-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { refunded_payment: RefundedPayment | null }
  return GetRefundedPaymentByConsultationIdResp.create(result.refunded_payment)
}
