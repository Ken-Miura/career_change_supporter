import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { PostCreateCareerRequestRejectionResp } from './PostCreateCareerRequestRejectionResp'

export async function postCreateCareerRequestRejection (createCareerReqId: number, rejectionReason: string): Promise<PostCreateCareerRequestRejectionResp | ApiErrorResp> {
  // eslint-disable-next-line
  const data = { create_career_req_id: createCareerReqId, rejection_reason: rejectionReason }
  const response = await fetch('/admin/api/create-career-request-rejection', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(data)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostCreateCareerRequestRejectionResp.create()
}
