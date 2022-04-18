import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostUpdateIdentityRequestRejectionResp } from './PostUpdateIdentityRequestRejectionResp'

export async function postUpdateIdentityRequestRejection (userAccountId: number, rejectionReason: string): Promise<PostUpdateIdentityRequestRejectionResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId, rejection_reason: rejectionReason }
  const response = await fetch('/admin/api/update-identity-request-rejection', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostUpdateIdentityRequestRejectionResp.create()
}
