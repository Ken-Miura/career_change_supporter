import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { UnratedItemsResult } from './UnratedItemsResult'
import { UnratedItemsResultResp } from './UnratedItemsResultResp'

export async function getUnratedItems (): Promise<UnratedItemsResultResp | ApiErrorResp> {
  const response = await fetch('/api/unrated-items', {
    method: 'GET'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  // eslint-disable-next-line
  const result = await response.json() as UnratedItemsResult
  return UnratedItemsResultResp.create(result)
}
