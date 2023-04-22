import { ApiError, ApiErrorResp } from '@/util/ApiError'
import { DeleteAccountResp } from './DeleteAccountResp'

export async function deleteAccount (accountDeleteConfirmed: boolean): Promise<DeleteAccountResp | ApiErrorResp> {
  const params = { account_delete_confirmed: accountDeleteConfirmed.toString() }
  const query = new URLSearchParams(params)
  const response = await fetch(`/api/accounts?${query}`, {
    method: 'DELETE'
  })
  if (!response.ok) {
    const apiErr = await response.json() as { code: number }
    return ApiErrorResp.create(response.status, ApiError.create(apiErr.code))
  }
  return DeleteAccountResp.create()
}
