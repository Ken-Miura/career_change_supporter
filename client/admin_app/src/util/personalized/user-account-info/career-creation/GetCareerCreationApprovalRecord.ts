import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetCareerCreationApprovalRecordResp } from './GetCareerCreationApprovalRecordResp'
import { CareerCreationApprovalRecordResult } from './CareerCreationApprovalRecordResult'

export async function getCareerCreationApprovalRecord (userAccountId: string): Promise<GetCareerCreationApprovalRecordResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/career-creation-approval-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as CareerCreationApprovalRecordResult
  return GetCareerCreationApprovalRecordResp.create(result)
}
