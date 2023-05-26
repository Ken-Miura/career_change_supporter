import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultationsByUserAccountIdResp } from './GetConsultationsByUserAccountIdResp'
import { ConsultationsResult } from './ConsultationsResult'

export async function getConsultationsByUserAccountId (userAccountId: string): Promise<GetConsultationsByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/consultations-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationsResult = await response.json() as ConsultationsResult
  return GetConsultationsByUserAccountIdResp.create(consultationsResult)
}
