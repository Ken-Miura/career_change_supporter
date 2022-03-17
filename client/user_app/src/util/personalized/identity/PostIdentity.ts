import { ApiErrorResp, ApiError } from '../../ApiError'
import { Identity } from '../profile/Identity'
import { PostIdentityResp } from './PostIdentityResp'

export async function postIdentity (identity: Identity, image1: File, image2: File | null): Promise<PostIdentityResp | ApiErrorResp> {
  const formData = new FormData()
  formData.append('identity', JSON.stringify(identity))
  formData.append('identity-image1', image1)
  if (image2 !== null) {
    formData.append('identity-image2', image2)
  }
  const response = await fetch('/api/identity', {
    method: 'POST',
    body: formData
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return PostIdentityResp.create()
}
