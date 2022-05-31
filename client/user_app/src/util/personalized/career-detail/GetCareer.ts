import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { Career } from '../Career'
import { GetCareerResp } from './GetCareerResp'

export async function getCareer (careerId: number): Promise<GetCareerResp | ApiErrorResp> {
  const params = { career_id: careerId.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/career?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const career = await response.json() as Career
  return GetCareerResp.create(career)
}
