import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityUpdateApprovalRecordResp } from './GetIdentityUpdateApprovalRecordResp'
import { IdentityUpdateApprovalRecordResult } from './IdentityUpdateApprovalRecordResult'

export async function getIdentityUpdateApprovalRecord (userAccountId: string): Promise<GetIdentityUpdateApprovalRecordResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-update-approval-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as IdentityUpdateApprovalRecordResult
  return GetIdentityUpdateApprovalRecordResp.create(result)
}
