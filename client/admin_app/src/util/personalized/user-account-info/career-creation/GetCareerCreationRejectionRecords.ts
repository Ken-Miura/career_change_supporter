import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetCareerCreationRejectionRecordsResp } from './GetCareerCreationRejectionRecordsResp'
import { CareerCreationRejectionRecordsResult } from './CareerCreationRejectionRecordsResult'

export async function getCareerCreationRejectionRecords (userAccountId: string): Promise<GetCareerCreationRejectionRecordsResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/career-creation-rejection-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as CareerCreationRejectionRecordsResult
  return GetCareerCreationRejectionRecordsResp.create(result)
}
