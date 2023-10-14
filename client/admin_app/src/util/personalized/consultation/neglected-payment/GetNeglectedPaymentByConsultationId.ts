import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { NeglectedPayment } from '../../NeglectedPayment'
import { GetNeglectedPaymentByConsultationIdResp } from './GetNeglectedPaymentByConsultationIdResp'

export async function getNeglectedPaymentByConsultationId (consultationId: string): Promise<GetNeglectedPaymentByConsultationIdResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/neglected-payment-by-consultation-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as { neglected_payment: NeglectedPayment | null }
  return GetNeglectedPaymentByConsultationIdResp.create(result.neglected_payment)
}
