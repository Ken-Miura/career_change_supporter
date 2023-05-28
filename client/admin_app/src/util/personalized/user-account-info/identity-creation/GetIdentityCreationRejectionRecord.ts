import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityCreationRejectionRecordResp } from './GetIdentityCreationRejectionRecordResp'
import { IdentityCreationRejectionRecordResult } from './IdentityCreationRejectionRecordResult'

export async function getIdentityCreationRejectionRecord (userAccountId: string): Promise<GetIdentityCreationRejectionRecordResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-creation-rejection-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as IdentityCreationRejectionRecordResult
  return GetIdentityCreationRejectionRecordResp.create(result)
}
