import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostCreateIdentityRequestApprovalResp } from './PostCreateIdentityRequestApprovalResp'

export async function postCreateIdentityRequestApproval (userAccountId: number): Promise<PostCreateIdentityRequestApprovalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { user_account_id: userAccountId }
  const response = await fetch('/admin/api/create-identity-request-approval', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostCreateIdentityRequestApprovalResp.create()
}
