import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostCreateCareerRequestApprovalResp } from './PostCreateCareerRequestApprovalResp'

export async function postCreateCareerRequestApproval (createCareerReqId: number): Promise<PostCreateCareerRequestApprovalResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { create_career_req_id: createCareerReqId }
  const response = await fetch('/admin/api/create-career-request-approval', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostCreateCareerRequestApprovalResp.create()
}
