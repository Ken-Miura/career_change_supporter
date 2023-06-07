import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityUpdateApprovalRecordsResp } from './GetIdentityUpdateApprovalRecordsResp'
import { IdentityUpdateApprovalRecordsResult } from './IdentityUpdateApprovalRecordsResult'

export async function getIdentityUpdateApprovalRecords (userAccountId: string): Promise<GetIdentityUpdateApprovalRecordsResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-update-approval-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as IdentityUpdateApprovalRecordsResult
  return GetIdentityUpdateApprovalRecordsResp.create(result)
}
