import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { ConsultantSearchParam } from '../ConsultantSearchParam'
import { ConsultantsSearchResult } from './ConsultantsSearchResult'
import { PostConsultantsSearchResp } from './PostConsultantsSearchResp'

export async function postConsultantsSearch (consultantSearchParam: ConsultantSearchParam): Promise<PostConsultantsSearchResp | ApiErrorResp> {
  const response = await fetch('/api/consultants-search', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json; charset=utf-8' },
    body: JSON.stringify(consultantSearchParam)
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as ConsultantsSearchResult
  return PostConsultantsSearchResp.create(result)
}
