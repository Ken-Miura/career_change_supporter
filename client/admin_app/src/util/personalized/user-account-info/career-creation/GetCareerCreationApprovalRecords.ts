import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetCareerCreationApprovalRecordsResp } from './GetCareerCreationApprovalRecordsResp'
import { CareerCreationApprovalRecordsResult } from './CareerCreationApprovalRecordsResult'

export async function getCareerCreationApprovalRecords (userAccountId: string): Promise<GetCareerCreationApprovalRecordsResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/career-creation-approval-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as CareerCreationApprovalRecordsResult
  return GetCareerCreationApprovalRecordsResp.create(result)
}
