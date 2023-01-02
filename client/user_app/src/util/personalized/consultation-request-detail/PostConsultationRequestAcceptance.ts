import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationRequestAcceptanceParam } from './ConsultationRequestAcceptanceParam'
import { PostConsultationRequestAcceptanceResp } from './PostConsultationRequestAcceptanceResp'

export async function postConsultationRequestAcceptance (param: ConsultationRequestAcceptanceParam): Promise<PostConsultationRequestAcceptanceResp | ApiErrorResp> {
  const response = await fetch('/api/consultation-request-acceptance', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(param)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostConsultationRequestAcceptanceResp.create()
}
