import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultationsByConsultantIdResp } from './GetConsultationsByConsultantIdResp'
import { ConsultationsResult } from './ConsultationsResult'

export async function getConsultationsByConsultantId (consultantId: string): Promise<GetConsultationsByConsultantIdResp | ApiErrorResp> {
  const params = { consultant_id: consultantId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/consultations-by-consultant-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationsResult = await response.json() as ConsultationsResult
  return GetConsultationsByConsultantIdResp.create(consultationsResult)
}
