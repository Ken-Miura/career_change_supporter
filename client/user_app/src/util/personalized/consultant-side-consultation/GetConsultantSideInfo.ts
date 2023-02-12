import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetConsultantSideInfoResp } from './GetConsultantSideInfoResp'
import { ConsultantSideInfo } from './ConsultantSideInfo'

export async function getConsultantSideInfo (consultationId: string, audioTestDone: boolean): Promise<GetConsultantSideInfoResp | ApiErrorResp> {
  const params = { consultation_id: consultationId, audio_test_done: audioTestDone.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/consultant-side-info?${query}`, {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as ConsultantSideInfo
  return GetConsultantSideInfoResp.create(result)
}
