import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetIdentityUpdateRejectionRecordsResp } from './GetIdentityUpdateRejectionRecordsResp'
import { IdentityUpdateRejectionRecordsResult } from './IdentityUpdateRejectionRecordsResult'

export async function getIdentityUpdateRejectionRecords (userAccountId: string): Promise<GetIdentityUpdateRejectionRecordsResp | ApiErrorResp> {
  const params = { user_account_id: userAccountId }
  const query = new URLSearchParams(params)
  const response = await fetch(`/admin/api/identity-update-rejection-records?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as IdentityUpdateRejectionRecordsResult
  return GetIdentityUpdateRejectionRecordsResp.create(result)
}
