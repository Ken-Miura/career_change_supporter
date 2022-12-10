import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationRequestRejectionParam } from './ConsultationRequestRejectionParam'
import { PostConsultationRequestRejectionResp } from './PostConsultationRequestRejectionResp'

export async function postConsultationRequestRejection (param: ConsultationRequestRejectionParam): Promise<PostConsultationRequestRejectionResp | ApiErrorResp> {
  const response = await fetch('/api/consultation-request-rejection', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(param)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostConsultationRequestRejectionResp.create()
}
