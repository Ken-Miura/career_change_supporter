import { ApiErrorResp, ApiError } from '../ApiError'
import { RefreshResp } from './RefreshResp'

export async function refresh (): Promise<RefreshResp | ApiErrorResp> {
  const response = await fetch('/api/refresh', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return RefreshResp.create()
}
