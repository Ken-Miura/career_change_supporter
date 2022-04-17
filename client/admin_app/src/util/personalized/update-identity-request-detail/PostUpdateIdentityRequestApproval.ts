import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostUpdateIdentityRequestApprovalResp } from './PostUpdateIdentityRequestApprovalResp'

export async function postUpdateIdentityRequestApproval (userAccountId: number): Promise<PostUpdateIdentityRequestApprovalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/update-identity-request-approval', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostUpdateIdentityRequestApprovalResp.create()
}
