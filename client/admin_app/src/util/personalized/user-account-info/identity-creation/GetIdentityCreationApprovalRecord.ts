import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityCreationApprovalRecordResp } from './GetIdentityCreationApprovalRecordResp'
import { IdentityCreationApprovalRecordResult } from './IdentityCreationApprovalRecordResult'

export async function getIdentityCreationApprovalRecord (userAccountId: string): Promise<GetIdentityCreationApprovalRecordResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-creation-approval-record?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const tenantIdResult = await response.json() as IdentityCreationApprovalRecordResult
  return GetIdentityCreationApprovalRecordResp.create(tenantIdResult)
}
