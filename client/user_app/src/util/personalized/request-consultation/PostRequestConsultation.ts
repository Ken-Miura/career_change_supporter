import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationRequest } from './ConsultationRequest'
import { PostRequestConsultationResp } from './PostRequestConsultationResp'

export async function postRequestConsultation (consultationRequest: ConsultationRequest): Promise<PostRequestConsultationResp | ApiErrorResp> {
  const response = await fetch('/api/request-consultation', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(consultationRequest)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as { charge_id: string }
  return PostRequestConsultationResp.create(result.charge_id)
}
