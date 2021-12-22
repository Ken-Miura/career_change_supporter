import { ApiErrorResp, ApiError } from '../ApiError'
import { GetProfileResp } from './GetProfileResp'
import { Profile } from './Profile'

export async function getProfile (): Promise<GetProfileResp | ApiErrorResp> {
  const response = await fetch('/api/profile', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as Profile
  return GetProfileResp.create(result)
}
