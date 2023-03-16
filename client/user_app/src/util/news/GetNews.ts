import { ApiError, ApiErrorResp } from '../ApiError'
import { GetNewResp } from './GetNewResp'
import { NewsResult } from './NewsResult'

export async function getNews (): Promise<GetNewResp | ApiErrorResp> {
  const response = await fetch('/api/news', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as NewsResult
  return GetNewResp.create(result)
}
