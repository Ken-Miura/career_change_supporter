import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetUserSideInfoResp } from './GetUserSideInfoResp'
import { UserSideInfo } from './UserSideInfo'

export async function getUserSideInfo (consultationId: string, audioTestDone: boolean): Promise<GetUserSideInfoResp | ApiErrorResp> {
  const params = { consultation_id: consultationId, audio_test_done: audioTestDone.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/user-side-info?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as UserSideInfo
  return GetUserSideInfoResp.create(result)
}
