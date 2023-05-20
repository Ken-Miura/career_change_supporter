import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultationReqsByUserAccountIdResp } from './GetConsultationReqsByUserAccountIdResp'
import { ConsultationReqsResult } from './ConsultationReqsResult'

export async function getConsultationReqsByUserAccountId (userAccountId: string): Promise<GetConsultationReqsByUserAccountIdResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/get-consultation-reqs-by-user-account-id?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const consultationReqsResult = await response.json() as ConsultationReqsResult
  return GetConsultationReqsByUserAccountIdResp.create(consultationReqsResult)
}
