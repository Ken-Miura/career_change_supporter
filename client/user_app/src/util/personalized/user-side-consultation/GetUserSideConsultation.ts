import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UserSideConsultation } from './UserSideConsultation'
import { GetUserSideConsultationResp } from './GetUserSideConsultationResp'

export async function getUserSideConsultation (consultationId: string): Promise<GetUserSideConsultationResp | ApiErrorResp> {
  const params = { consultation_id: consultationId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/user-side-consultation?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as UserSideConsultation
  return GetUserSideConsultationResp.create(result)
}
