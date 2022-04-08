import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostCreateIdentityRequestRejectionResp } from './PostCreateIdentityRequestRejectionResp'

export async function postCreateIdentityRequestRejection (userAccountId: number, rejectionReason: string): Promise<PostCreateIdentityRequestRejectionResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId, rejection_reason: rejectionReason }
  const response = await fetch('/admin/api/create-identity-request-rejection', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostCreateIdentityRequestRejectionResp.create()
}
