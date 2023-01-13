import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultationsResult } from './ConsultationsResult'
import { GetConsultationsResp } from './GetConsultationsResp'

export async function getConsultations (): Promise<GetConsultationsResp | ApiErrorResp> {
  const response = await fetch('/api/consultations', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as ConsultationsResult
  return GetConsultationsResp.create(result)
}
