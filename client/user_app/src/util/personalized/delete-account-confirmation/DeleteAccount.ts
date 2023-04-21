import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { DeleteAccountResp } from './DeleteAccountResp'

export async function deleteAccount (): Promise<DeleteAccountResp | ApiErrorResp> {
  const response = await fetch('/api/accounts', {
    method: 'DELETE'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return DeleteAccountResp.create()
}
