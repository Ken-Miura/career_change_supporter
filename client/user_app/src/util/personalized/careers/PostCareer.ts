import { ApiErrorResp, ApiError } from '../../ApiError'
import { Career } from '../Career'
import { PostCareerResp } from './PostCareerResp'

export async function postCareer (career: Career, image1: File, image2: File | null): Promise<PostCareerResp | ApiErrorResp> {
  const formData = new FormData()
  formData.append('career', JSON.stringify(career))
  formData.append('career-image1', image1)
  if (image2 !== null) {
    formData.append('career-image2', image2)
  }
  const response = await fetch('/api/career', {
    method: 'POST',
    body: formData
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostCareerResp.create()
}
