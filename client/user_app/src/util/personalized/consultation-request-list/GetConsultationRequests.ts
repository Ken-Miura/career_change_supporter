import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationRequestsResult } from './ConsultationRequestsResult'
import { GetConsultationRequestsResp } from './GetConsultationRequestsResp'

export async function getConsultationRequests (): Promise<GetConsultationRequestsResp | ApiErrorResp> {
  const response = await fetch('/api/consultation-requests', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as ConsultationRequestsResult
  return GetConsultationRequestsResp.create(result)
}
