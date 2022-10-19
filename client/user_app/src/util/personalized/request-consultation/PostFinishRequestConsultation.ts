import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { FinishRequestConsultation } from './FinishRequestConsultation'
import { PostFinishRequestConsultationResp } from './PostFinishRequestConsultationResp'

export async function postFinishRequestConsultation (finishRequestConsultation: FinishRequestConsultation): Promise<PostFinishRequestConsultationResp | ApiErrorResp> {
  const response = await fetch('/api/finish-request-consultation', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(finishRequestConsultation)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostFinishRequestConsultationResp.create()
}
