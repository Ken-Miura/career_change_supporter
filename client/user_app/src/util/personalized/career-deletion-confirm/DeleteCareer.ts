import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { DeleteCareerResp } from './DeleteCareerResp'

export async function deleteCareer (careerId: number): Promise<DeleteCareerResp | ApiErrorResp> {
  const params = { career_id: careerId.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/career?${query}`, {
    method: 'DELETE'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return DeleteCareerResp.create()
}
