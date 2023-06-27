import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { GetLatestNewsResp } from './GetLatestNewsResp'
import { LatestNewsResult } from './LatestNewsResult'

export async function getLatestNews (): Promise<GetLatestNewsResp | ApiErrorResp> {
  const response = await fetch('/admin/api/latest-news', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  const result = await response.json() as LatestNewsResult
  return GetLatestNewsResp.create(result)
}
